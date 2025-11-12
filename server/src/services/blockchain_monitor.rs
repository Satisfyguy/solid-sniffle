//! Blockchain monitoring service for tracking Monero transactions
//!
//! This service monitors the Monero blockchain for:
//! - Transaction confirmations
//! - Escrow funding status
//! - Transaction completion

use actix::Addr;
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::{db_load_escrow, db_update_escrow_status, DbPool};
use crate::models::order::{Order, OrderStatus};
use crate::wallet_manager::WalletManager;
use crate::websocket::WebSocketServer;
use crate::services::wallet_session_manager::WalletSessionManager;
use crate::wallet_pool::WalletRole;

/// Configuration for blockchain monitoring
#[derive(Clone, Debug)]
pub struct MonitorConfig {
    /// How often to check for transaction updates (in seconds)
    pub poll_interval_secs: u64,
    /// Number of confirmations required to consider a transaction settled
    pub required_confirmations: u32,
    /// Maximum number of blocks to scan in a single poll
    pub max_blocks_per_poll: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 30,
            required_confirmations: 10,
            max_blocks_per_poll: 100,
        }
    }
}

/// Blockchain monitoring service
pub struct BlockchainMonitor {
    wallet_manager: Arc<Mutex<WalletManager>>,
    session_manager: Arc<WalletSessionManager>,
    db: DbPool,
    #[allow(dead_code)]
    websocket: Addr<WebSocketServer>,
    config: MonitorConfig,
}

impl BlockchainMonitor {
    /// Create a new blockchain monitor
    pub fn new(
        wallet_manager: Arc<Mutex<WalletManager>>,
        session_manager: Arc<WalletSessionManager>,
        db: DbPool,
        websocket: Addr<WebSocketServer>,
        config: MonitorConfig,
    ) -> Self {
        info!(
            "BlockchainMonitor initialized with poll_interval={}s, required_confirmations={}",
            config.poll_interval_secs, config.required_confirmations
        );
        Self {
            wallet_manager,
            session_manager,
            db,
            websocket,
            config,
        }
    }

    /// Start monitoring in background
    ///
    /// This spawns a background task that periodically checks for:
    /// - New transactions to escrow addresses
    /// - Confirmation updates for pending transactions
    /// - Transaction completions
    pub async fn start_monitoring(self: Arc<Self>) {
        let mut poll_timer = interval(Duration::from_secs(self.config.poll_interval_secs));

        info!("Starting blockchain monitoring loop");

        loop {
            poll_timer.tick().await;

            if let Err(e) = self.poll_escrows().await {
                error!("Error polling escrows: {}", e);
            }
        }
    }

    /// Poll all active escrows for transaction updates
    async fn poll_escrows(&self) -> Result<()> {
        // Get all escrows in 'funded' state (waiting for buyer to deposit)
        let funded_escrows = self.get_funded_escrows().await?;

        info!(
            "Polling {} funded escrows for updates",
            funded_escrows.len()
        );

        for escrow_id_str in funded_escrows {
            let escrow_id = escrow_id_str
                .parse::<Uuid>()
                .context("Failed to parse escrow_id")?;

            if let Err(e) = self.check_escrow_funding(escrow_id).await {
                warn!("Error checking escrow {}: {}", escrow_id, e);
            }
        }

        // Get all escrows in 'releasing' or 'refunding' state (transactions in flight)
        let pending_tx_escrows = self.get_pending_transaction_escrows().await?;

        info!(
            "Polling {} escrows with pending transactions",
            pending_tx_escrows.len()
        );

        for escrow_id_str in pending_tx_escrows {
            let escrow_id = escrow_id_str
                .parse::<Uuid>()
                .context("Failed to parse escrow_id")?;

            if let Err(e) = self.check_transaction_confirmations(escrow_id).await {
                warn!("Error checking transaction for escrow {}: {}", escrow_id, e);
            }
        }

        Ok(())
    }

    /// Check if an escrow multisig address has received funding
    ///
    /// This monitors the multisig wallet balance and updates escrow status
    /// when funds are detected. The escrow must be in 'funded' status
    /// (multisig setup complete) and waiting for buyer deposit.
    async fn check_escrow_funding(&self, escrow_id: Uuid) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Escrow must have a multisig address
        let multisig_address = escrow
            .multisig_address
            .ok_or_else(|| anyhow::anyhow!("Escrow {} has no multisig address", escrow_id))?;

        info!(
            "Checking funding for escrow {} at address {}",
            escrow_id,
            &multisig_address[..10]
        );

        // PHASE 2: Use WalletSessionManager for persistent wallet sessions
        // Get the already-open buyer wallet from session (no open/close overhead!)
        info!(
            "🚀 [PHASE 2] Getting wallet from session manager for escrow {}",
            escrow_id
        );

        let buyer_wallet = self.session_manager
            .get_wallet(escrow_id, WalletRole::Buyer)
            .await
            .context("Failed to get buyer wallet from session")?;

        // Note: No explicit refresh() needed - Monero RPC auto-refreshes on balance queries
        // The wallet is persistent in the session, so it stays synced with blockchain

        // Get balance (instant - wallet already open!)
        let (total_balance, unlocked_balance) = buyer_wallet.rpc().get_balance().await
            .map_err(|e| anyhow::anyhow!("Failed to get wallet balance: {:?}", e))?;

        info!(
            "Escrow {} wallet balance: total={}, unlocked={}, expected={}",
            escrow_id, total_balance, unlocked_balance, escrow.amount
        );

        // Check if funds have arrived (use unlocked balance for safety)
        if unlocked_balance >= escrow.amount as u64 {
            info!(
                "Escrow {} is now funded! Updating status to 'active'",
                escrow_id
            );

            // Update escrow status to "active" (funds received, ready for transaction)
            db_update_escrow_status(&self.db, escrow_id, "active")
                .await
                .context("Failed to update escrow status to active")?;

            // Update associated order status to "funded"
            let order_id = escrow.order_id.clone();
            let order_id_for_log = order_id.clone();
            let db_pool = self.db.clone();
            match tokio::task::spawn_blocking(move || {
                let mut conn = db_pool.get().context("Failed to get DB connection")?;
                Order::update_status(&mut conn, order_id.clone(), OrderStatus::Funded)
                    .context("Failed to update order status to funded")
            })
            .await
            {
                Ok(Ok(_)) => {
                    info!("Order {} status updated to 'funded'", order_id_for_log);

                    // Notify vendor that order is now funded
                    if let Ok(_vendor_uuid) = Uuid::parse_str(&escrow.vendor_id) {
                        if let Ok(order_uuid) = Uuid::parse_str(&order_id_for_log) {
                            use crate::websocket::WsEvent;
                            self.websocket.do_send(WsEvent::OrderStatusChanged {
                                order_id: order_uuid,
                                new_status: "funded".to_string(),
                            });
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Failed to update order {} status: {}", order_id_for_log, e);
                    // Don't fail the escrow update, just log error
                }
                Err(e) => {
                    error!("Task join error updating order {}: {}", order_id_for_log, e);
                }
            }

            // Notify all parties via WebSocket about escrow status
            use crate::websocket::WsEvent;
            self.websocket.do_send(WsEvent::EscrowStatusChanged {
                escrow_id,
                new_status: "active".to_string(),
            });

            info!("Escrow {} funding complete and parties notified", escrow_id);
        } else {
            info!(
                "Escrow {} still waiting for funds: {}/{} atomic units",
                escrow_id, unlocked_balance, escrow.amount
            );
        }

        // PHASE 2: No need to close wallet - session manager keeps it open for entire escrow lifecycle
        info!("🚀 [PHASE 2] Wallet remains open in session for future operations (zero overhead!)");

        Ok(())
    }

    /// Check confirmation status of a transaction
    ///
    /// Monitors transactions in 'releasing' or 'refunding' status to track
    /// blockchain confirmations. When threshold is reached, finalizes the escrow.
    async fn check_transaction_confirmations(&self, escrow_id: Uuid) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        info!(
            "Checking transaction confirmations for escrow {} (status: {})",
            escrow_id, escrow.status
        );

        // Escrow must have a transaction hash (release or refund tx)
        let tx_hash = match &escrow.transaction_hash {
            Some(hash) => hash,
            None => {
                warn!(
                    "Escrow {} in {} status but has no transaction_hash",
                    escrow_id, escrow.status
                );
                return Ok(());
            }
        };

        // Only monitor transactions in releasing or refunding status
        if !matches!(escrow.status.as_str(), "releasing" | "refunding") {
            return Ok(());
        }

        info!(
            "Checking confirmations for transaction {} (escrow {})",
            &tx_hash[..10],
            escrow_id
        );

        // 🚀 [PHASE 2] Get wallet from session manager (wallet stays open)
        info!(
            "🚀 [PHASE 2] Getting buyer wallet from session for confirmation check - escrow {}",
            escrow_id
        );

        let buyer_wallet = match self.session_manager
            .get_wallet(escrow_id, WalletRole::Buyer)
            .await
        {
            Ok(wallet) => wallet,
            Err(e) => {
                warn!(
                    "Failed to get buyer wallet from session for escrow {}: {}. \
                    This is expected if escrow session was already cleaned up.",
                    escrow_id, e
                );
                return Ok(());
            }
        };

        // Get transaction details (instant - wallet already open!)
        let transfer_info = match buyer_wallet.rpc().get_transfer_by_txid(tx_hash.clone()).await {
            Ok(info) => info,
            Err(e) => {
                warn!("Failed to get transaction details for {}: {:?}", &tx_hash[..10], e);
                return Ok(());
            }
        };

        let confirmations = transfer_info.confirmations as u32;

        info!(
            "🚀 [PHASE 2] Transaction query completed instantly (wallet persistent in session)"
        );

        info!(
            "Transaction {} has {} confirmations (required: {})",
            &tx_hash[..10],
            confirmations,
            self.config.required_confirmations
        );

        // Check if transaction has enough confirmations
        if confirmations >= self.config.required_confirmations {
            // Determine final status based on current status
            let final_status = match escrow.status.as_str() {
                "releasing" => {
                    // Transaction completed successfully → Trigger review invitation
                    self.trigger_review_invitation(escrow_id, tx_hash)
                        .await
                        .context("Failed to trigger review invitation")?;
                    "completed"
                }
                "refunding" => "refunded",
                _ => {
                    warn!(
                        "Unexpected escrow status {} for confirmation check",
                        escrow.status
                    );
                    return Ok(());
                }
            };

            info!(
                "Transaction {} confirmed! Updating escrow {} to status '{}'",
                &tx_hash[..10],
                escrow_id,
                final_status
            );

            // Update escrow to final status
            db_update_escrow_status(&self.db, escrow_id, final_status)
                .await
                .context("Failed to update escrow to final status")?;

            // Notify all parties via WebSocket
            use crate::websocket::WsEvent;
            self.websocket.do_send(WsEvent::TransactionConfirmed {
                tx_hash: tx_hash.clone(),
                confirmations,
            });

            info!(
                "Escrow {} finalized with status '{}' (tx: {})",
                escrow_id,
                final_status,
                &tx_hash[..10]
            );

            // 🚀 [PHASE 2] Close wallet session - escrow lifecycle complete
            info!("🚀 [PHASE 2] Closing wallet session for completed escrow {}", escrow_id);
            if let Err(e) = self.session_manager.close_session(escrow_id).await {
                warn!(
                    "⚠️ [PHASE 2] Failed to close wallet session for escrow {}: {}. \
                    Session will be cleaned up by TTL (2h timeout)",
                    escrow_id, e
                );
            } else {
                info!("✅ [PHASE 2] Wallet session closed - 3 wallets freed");
            }
        }

        Ok(())
    }

    /// Trigger review invitation to buyer after escrow transaction completion
    ///
    /// This method is automatically called when a transaction reaches the required
    /// number of confirmations. It sends a WebSocket notification to the buyer,
    /// inviting them to submit a review for the completed transaction.
    ///
    /// # Arguments
    /// * `escrow_id` - The UUID of the escrow that was completed
    /// * `tx_hash` - The transaction hash on the blockchain
    ///
    /// # Production-Ready Features
    /// - Proper error handling with context
    /// - Secure logging (only first 8 chars of tx_hash)
    /// - UUID parsing validation
    /// - Database access error handling
    async fn trigger_review_invitation(&self, escrow_id: Uuid, tx_hash: &str) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id)
            .await
            .context("Failed to load escrow for review invitation")?;

        let buyer_id = escrow
            .buyer_id
            .parse::<Uuid>()
            .context("Failed to parse buyer_id as Uuid")?;

        let vendor_id = escrow
            .vendor_id
            .parse::<Uuid>()
            .context("Failed to parse vendor_id as Uuid")?;

        // Send WebSocket notification to buyer
        use crate::websocket::WsEvent;
        self.websocket.do_send(WsEvent::ReviewInvitation {
            escrow_id,
            tx_hash: tx_hash.to_string(),
            buyer_id,
            vendor_id,
        });

        info!(
            "Review invitation sent to buyer {} for completed transaction {} (vendor: {})",
            buyer_id,
            &tx_hash[..8],  // Only log first 8 chars for privacy
            vendor_id
        );

        Ok(())
    }

    /// Get all escrows in 'funded' state
    async fn get_funded_escrows(&self) -> Result<Vec<String>> {
        let mut conn = self.db.get().context("Failed to get DB connection")?;

        let escrow_ids = tokio::task::spawn_blocking(move || {
            use crate::schema::escrows::dsl::*;
            use diesel::prelude::*;

            // Monitor escrows in "created" or "funded" status that have a multisig address
            // "created" = multisig setup complete, waiting for payment
            // "funded" = payment detected but not yet confirmed (legacy status)
            escrows
                .filter(
                    status.eq("created")
                    .or(status.eq("funded"))
                )
                .filter(multisig_address.is_not_null())
                .select(id)
                .load::<String>(&mut conn)
        })
        .await
        .context("Task join error")??;

        Ok(escrow_ids)
    }

    /// Get all escrows with pending transactions
    async fn get_pending_transaction_escrows(&self) -> Result<Vec<String>> {
        let mut conn = self.db.get().context("Failed to get DB connection")?;

        let escrow_ids = tokio::task::spawn_blocking(move || {
            use crate::schema::escrows::dsl::*;
            use diesel::prelude::*;

            escrows
                .filter(
                    status
                        .eq("releasing")
                        .or(status.eq("refunding"))
                        .or(status.eq("active")),
                )
                .select(id)
                .load::<String>(&mut conn)
        })
        .await
        .context("Task join error")??;

        Ok(escrow_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.poll_interval_secs, 30);
        assert_eq!(config.required_confirmations, 10);
        assert_eq!(config.max_blocks_per_poll, 100);
    }

    #[test]
    fn test_monitor_config_custom() {
        let config = MonitorConfig {
            poll_interval_secs: 60,
            required_confirmations: 20,
            max_blocks_per_poll: 200,
        };
        assert_eq!(config.poll_interval_secs, 60);
        assert_eq!(config.required_confirmations, 20);
        assert_eq!(config.max_blocks_per_poll, 200);
    }
}
