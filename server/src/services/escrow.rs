//! Escrow orchestration service for Monero Marketplace

use actix::Addr;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::crypto::encryption::encrypt_field;
use crate::db::{
    db_count_multisig_infos, db_insert_escrow, db_load_escrow, db_store_multisig_info,
    db_update_escrow_address, db_update_escrow_status, DbPool,
};
use crate::models::escrow::{Escrow, NewEscrow};
use crate::models::user::User;
use crate::wallet_manager::WalletManager;
use crate::websocket::{WebSocketServer, WsEvent};
use monero_marketplace_common::types::TransferDestination;
use tokio::sync::Mutex;

/// Manages escrow operations and state transitions
pub struct EscrowOrchestrator {
    /// Monero wallet manager for blockchain operations
    wallet_manager: Arc<Mutex<WalletManager>>,
    /// Database connection pool
    db: DbPool,
    /// WebSocket server actor address for real-time notifications
    websocket: Addr<WebSocketServer>,
    /// Encryption key for sensitive data
    encryption_key: Vec<u8>,
}

impl EscrowOrchestrator {
    /// Create a new EscrowOrchestrator
    pub fn new(
        wallet_manager: Arc<Mutex<WalletManager>>,
        db: DbPool,
        websocket: Addr<WebSocketServer>,
        encryption_key: Vec<u8>,
    ) -> Self {
        Self {
            wallet_manager,
            db,
            websocket,
            encryption_key,
        }
    }

    // ========================================================================
    // NON-CUSTODIAL: Client Wallet Registration
    // ========================================================================

    /// Register client's wallet RPC endpoint (NON-CUSTODIAL)
    ///
    /// This method allows buyers and vendors to provide their own wallet RPC URLs,
    /// ensuring the server never has access to their private keys.
    ///
    /// # Arguments
    /// * `user_id` - Authenticated user making the registration
    /// * `role` - Wallet role (Buyer or Vendor - Arbiter not allowed)
    /// * `rpc_url` - Client's wallet RPC endpoint
    /// * `rpc_user` - Optional RPC authentication username
    /// * `rpc_password` - Optional RPC authentication password
    ///
    /// # Returns
    /// Tuple of (wallet_id, wallet_address) on success
    ///
    /// # Errors
    /// - User not found in database
    /// - Role mismatch (user's role doesn't match provided role)
    /// - Wallet RPC connection failed
    /// - Non-custodial policy violation
    pub async fn register_client_wallet(
        &self,
        user_id: Uuid,
        role: crate::wallet_manager::WalletRole,
        rpc_url: String,
        rpc_user: Option<String>,
        rpc_password: Option<String>,
    ) -> Result<(Uuid, String)> {
        info!(
            "Registering client wallet RPC: user={}, role={:?}, url={}",
            user_id, role, rpc_url
        );

        // 1. Verify user exists and role matches
        let user_id_str = user_id.to_string();
        let db_clone = self.db.clone();
        let user = tokio::task::spawn_blocking(move || {
            let mut conn = db_clone.get().context("Failed to get DB connection")?;
            User::find_by_id(&mut conn, user_id_str)
        })
        .await
        .context("Database task panicked")??;

        let expected_role = match role {
            crate::wallet_manager::WalletRole::Buyer => "buyer",
            crate::wallet_manager::WalletRole::Vendor => "vendor",
            _ => {
                return Err(anyhow::anyhow!(
                    "Non-custodial policy: Cannot register arbiter wallet via this endpoint"
                ))
            }
        };

        if user.role != expected_role {
            return Err(anyhow::anyhow!(
                "Role mismatch: user role is '{}' but trying to register '{}' wallet",
                user.role,
                expected_role
            ));
        }

        // 2. Register wallet RPC via WalletManager
        // TODO: This should be called with actual escrow_id when wallet is registered for specific escrow
        // For now, using temporary escrow_id and manual recovery mode
        let mut wallet_manager = self.wallet_manager.lock().await;
        let wallet_id = wallet_manager
            .register_client_wallet_rpc(
                "temp-escrow-needs-refactor",  // TODO: Pass actual escrow_id
                role,
                rpc_url.clone(),
                rpc_user,
                rpc_password,
                "manual",  // Default to manual recovery for now
            )
            .await
            .context("Failed to register client wallet RPC")?;

        // 3. Get wallet address for response
        let wallet_instance = wallet_manager
            .wallets
            .get(&wallet_id)
            .ok_or_else(|| anyhow::anyhow!("Wallet instance not found after registration"))?;

        let wallet_address = wallet_instance.address.clone();

        info!(
            "✅ Client wallet registered: wallet_id={}, address={}, user={}",
            wallet_id, wallet_address, user_id
        );
        info!("🔒 NON-CUSTODIAL: Client controls private keys at {}", rpc_url);

        Ok((wallet_id, wallet_address))
    }

    // ========================================================================
    // Escrow Lifecycle
    // ========================================================================

    /// Initialize new escrow (step 1)
    pub async fn init_escrow(
        &self,
        order_id: Uuid,
        buyer_id: Uuid,
        vendor_id: Uuid,
        amount_atomic: i64,
    ) -> Result<Escrow> {
        info!(
            "Initializing new escrow: order_id={}, buyer={}, vendor={}, amount={}",
            order_id, buyer_id, vendor_id, amount_atomic
        );

        // 1. Assign arbiter using round-robin from available arbiters
        let arbiter_id = self.assign_arbiter().await?;
        info!("Assigned arbiter {} to escrow", arbiter_id);

        // 2. Create escrow in DB
        let new_escrow = NewEscrow {
            id: Uuid::new_v4().to_string(),
            order_id: order_id.to_string(),
            buyer_id: buyer_id.to_string(),
            vendor_id: vendor_id.to_string(),
            arbiter_id: arbiter_id.to_string(),
            amount: amount_atomic,
            status: "created".to_string(),
        };

        let escrow = db_insert_escrow(&self.db, new_escrow)
            .await
            .context("Failed to create escrow in database")?;

        // 3. Notify parties via WebSocket
        let escrow_uuid = escrow
            .id
            .parse()
            .context("Failed to parse escrow_id to Uuid")?;

        // Notify all parties of escrow initialization
        self.websocket.do_send(WsEvent::EscrowInit {
            escrow_id: escrow_uuid,
        });

        info!("Escrow {} initialized successfully", escrow.id);
        Ok(escrow)
    }

    /// Collect prepare_multisig from party (step 2)
    pub async fn collect_prepare_info(
        &self,
        escrow_id: Uuid,
        user_id: Uuid,
        multisig_info_str: String,
    ) -> Result<()> {
        info!(
            "Collecting prepare info for escrow {} from user {}",
            escrow_id, user_id
        );

        // Validate multisig info length
        if multisig_info_str.len() < 100 {
            return Err(anyhow::anyhow!("Multisig info too short (min 100 chars)"));
        }
        if multisig_info_str.len() > 5000 {
            return Err(anyhow::anyhow!("Multisig info too long (max 5000 chars)"));
        }

        // Encrypt multisig info
        let encrypted = encrypt_field(&multisig_info_str, &self.encryption_key)
            .context("Failed to encrypt multisig info")?;

        // Determine which party this user is
        let escrow = db_load_escrow(&self.db, escrow_id).await?;
        let party = if user_id.to_string() == escrow.buyer_id {
            "buyer"
        } else if user_id.to_string() == escrow.vendor_id {
            "vendor"
        } else if user_id.to_string() == escrow.arbiter_id {
            "arbiter"
        } else {
            return Err(anyhow::anyhow!(
                "User {} is not part of escrow {}",
                user_id,
                escrow_id
            ));
        };

        // Store in DB
        db_store_multisig_info(&self.db, escrow_id, party, encrypted)
            .await
            .context("Failed to store multisig info")?;

        // Check if all 3 received
        let count = db_count_multisig_infos(&self.db, escrow_id).await?;
        info!(
            "Collected {} of 3 multisig infos for escrow {}",
            count, escrow_id
        );

        if count == 3 {
            info!(
                "All multisig infos collected for escrow {}. Triggering make_multisig.",
                escrow_id
            );
            self.make_multisig(escrow_id).await?;
        }

        Ok(())
    }

    /// Make multisig for all 3 parties (step 3)
    async fn make_multisig(&self, escrow_id: Uuid) -> Result<()> {
        info!("Making multisig for escrow {}", escrow_id);

        // Load escrow with all wallet infos
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        let escrow_for_buyer = escrow.clone();
        let mut conn_buyer = self.db.get().context("Failed to get DB connection")?;
        let buyer = tokio::task::spawn_blocking(move || {
            User::find_by_id(&mut conn_buyer, escrow_for_buyer.buyer_id.clone())
        })
        .await??;

        let escrow_for_vendor = escrow.clone();
        let mut conn_vendor = self.db.get().context("Failed to get DB connection")?;
        let vendor = tokio::task::spawn_blocking(move || {
            User::find_by_id(&mut conn_vendor, escrow_for_vendor.vendor_id.clone())
        })
        .await??;

        let escrow_for_arbiter = escrow.clone();
        let mut conn_arbiter = self.db.get().context("Failed to get DB connection")?;
        let arbiter = tokio::task::spawn_blocking(move || {
            User::find_by_id(&mut conn_arbiter, escrow_for_arbiter.arbiter_id.clone())
        })
        .await??;

        let buyer_wallet_id = buyer
            .wallet_id
            .ok_or_else(|| anyhow::anyhow!("Buyer wallet ID not found"))?
            .parse::<Uuid>()?;
        let vendor_wallet_id = vendor
            .wallet_id
            .ok_or_else(|| anyhow::anyhow!("Vendor wallet ID not found"))?
            .parse::<Uuid>()?;
        let arbiter_wallet_id = arbiter
            .wallet_id
            .ok_or_else(|| anyhow::anyhow!("Arbiter wallet ID not found"))?
            .parse::<Uuid>()?;

        let mut wallet_manager = self.wallet_manager.lock().await;

        // 1. Prepare multisig for each participant
        let buyer_info = wallet_manager
            .make_multisig(&escrow_id.to_string(), buyer_wallet_id, vec![])
            .await?;
        let vendor_info = wallet_manager
            .make_multisig(&escrow_id.to_string(), vendor_wallet_id, vec![])
            .await?;
        let arbiter_info = wallet_manager
            .make_multisig(&escrow_id.to_string(), arbiter_wallet_id, vec![])
            .await?;

        // 2. Exchange multisig info
        wallet_manager
            .exchange_multisig_info(escrow_id, vec![buyer_info, vendor_info, arbiter_info])
            .await?;

        // 3. Finalize multisig
        let multisig_address = wallet_manager.finalize_multisig(escrow_id).await?;

        info!(
            "Multisig address created successfully: {}",
            multisig_address
        );

        // Store multisig address in DB
        db_update_escrow_address(&self.db, escrow_id, &multisig_address)
            .await
            .context("Failed to update escrow with multisig address")?;

        // Update status to 'funded' (ready to receive funds)
        db_update_escrow_status(&self.db, escrow_id, "funded")
            .await
            .context("Failed to update escrow status")?;

        // Notify all parties of status change
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "funded".to_string(),
        });

        info!("Multisig setup complete for escrow {}", escrow_id);
        Ok(())
    }

    /// Assign an arbiter using round-robin selection
    async fn assign_arbiter(&self) -> Result<String> {
        // Get connection from pool
        let mut conn = self.db.get().context("Failed to get DB connection")?;

        // Find all users with 'arbiter' role
        let arbiters =
            tokio::task::spawn_blocking(move || User::find_by_role(&mut conn, "arbiter"))
                .await
                .context("Task join error")??;

        if arbiters.is_empty() {
            return Err(anyhow::anyhow!("No arbiters available in the system"));
        }

        // Simple round-robin: pick first arbiter
        // In production, track arbiter workload and balance assignments
        let selected_arbiter = &arbiters[0];

        info!(
            "Selected arbiter: {} ({})",
            selected_arbiter.username, selected_arbiter.id
        );
        Ok(selected_arbiter.id.clone())
    }

    /// Release funds to vendor (buyer approves)
    ///
    /// # Flow
    /// 1. Validate buyer is requester
    /// 2. Create multisig transaction to send funds to vendor_address
    /// 3. Sign with buyer's wallet (first signature)
    /// 4. Get arbiter to sign (second signature - 2-of-3 threshold met)
    /// 5. Submit fully signed transaction to network
    /// 6. Update escrow status to "released"
    ///
    /// # Arguments
    /// * `escrow_id` - The escrow to release
    /// * `requester_id` - Must be buyer
    /// * `vendor_address` - Monero address to send funds to
    ///
    /// # Important
    /// This implementation requires all three wallets (buyer, vendor, arbiter)
    /// to be managed by the server. In a fully decentralized system, each party
    /// would sign independently via secure channels and signatures would be
    /// exchanged out-of-band (PGP, Tor messaging, etc.).
    pub async fn release_funds(
        &self,
        escrow_id: Uuid,
        requester_id: Uuid,
        vendor_address: String,
    ) -> Result<String> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Only buyer can release funds
        if requester_id.to_string() != escrow.buyer_id {
            return Err(anyhow::anyhow!("Only buyer can release funds"));
        }

        // Validate escrow is in funded state
        if escrow.status != "funded" {
            return Err(anyhow::anyhow!(
                "Escrow must be in 'funded' state to release funds (current: {})",
                escrow.status
            ));
        }

        // Validate vendor address format (basic check)
        if !vendor_address.starts_with('4') || vendor_address.len() != 95 {
            return Err(anyhow::anyhow!(
                "Invalid Monero address format (must start with 4 and be 95 chars)"
            ));
        }

        info!(
            "Releasing funds from escrow {} to vendor address {}",
            escrow_id,
            &vendor_address[..10]
        );

        // Create multisig transaction destinations
        // Validate amount is positive before casting i64 -> u64
        let amount_u64 = u64::try_from(escrow.amount).map_err(|_| {
            anyhow::anyhow!(
                "Invalid escrow amount: {}. Amount must be positive.",
                escrow.amount
            )
        })?;

        let destinations = vec![TransferDestination {
            address: vendor_address.clone(),
            amount: amount_u64,
        }];

        // Use WalletManager to release funds through multisig flow
        let mut wallet_manager = self.wallet_manager.lock().await;
        let tx_hash = wallet_manager
            .release_funds(escrow_id, destinations)
            .await?;

        info!("Transaction submitted to network: tx_hash={}", tx_hash);

        // Update escrow with transaction hash (for blockchain monitoring)
        crate::db::db_update_escrow_transaction_hash(&self.db, escrow_id, &tx_hash).await?;

        // Update escrow status to 'releasing' (will become 'completed' after confirmations)
        db_update_escrow_status(&self.db, escrow_id, "releasing").await?;

        // Notify parties via WebSocket
        self.websocket.do_send(WsEvent::TransactionConfirmed {
            tx_hash: tx_hash.clone(),
            confirmations: 0,
        });
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "releasing".to_string(),
        });

        info!(
            "Funds releasing for escrow {}: tx={} (awaiting confirmations)",
            escrow_id, tx_hash
        );
        Ok(tx_hash.clone())
    }

    /// Refund funds to buyer (vendor or arbiter approves)
    ///
    /// # Flow
    /// 1. Validate requester is vendor or arbiter
    /// 2. Create multisig transaction to send funds back to buyer_address
    /// 3. Sign with vendor's wallet (first signature)
    /// 4. Get arbiter to sign (second signature - 2-of-3 threshold met)
    /// 5. Submit fully signed transaction to network
    /// 6. Update escrow status to "refunded"
    ///
    /// # Arguments
    /// * `escrow_id` - The escrow to refund
    /// * `requester_id` - Must be vendor or arbiter
    /// * `buyer_address` - Monero address to refund to
    pub async fn refund_funds(
        &self,
        escrow_id: Uuid,
        requester_id: Uuid,
        buyer_address: String,
    ) -> Result<String> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Vendor or arbiter can initiate refund
        if requester_id.to_string() != escrow.vendor_id
            && requester_id.to_string() != escrow.arbiter_id
        {
            return Err(anyhow::anyhow!(
                "Only vendor or arbiter can initiate refund"
            ));
        }

        // Validate escrow is in funded state
        if escrow.status != "funded" {
            return Err(anyhow::anyhow!(
                "Escrow must be in 'funded' state to refund (current: {})",
                escrow.status
            ));
        }

        // Validate buyer address format (basic check)
        if !buyer_address.starts_with('4') || buyer_address.len() != 95 {
            return Err(anyhow::anyhow!(
                "Invalid Monero address format (must start with 4 and be 95 chars)"
            ));
        }

        info!(
            "Refunding funds from escrow {} to buyer address {}",
            escrow_id,
            &buyer_address[..10]
        );

        // Create multisig transaction destinations
        // Validate amount is positive before casting i64 -> u64
        let amount_u64 = u64::try_from(escrow.amount).map_err(|_| {
            anyhow::anyhow!(
                "Invalid escrow amount: {}. Amount must be positive.",
                escrow.amount
            )
        })?;

        let destinations = vec![TransferDestination {
            address: buyer_address.clone(),
            amount: amount_u64,
        }];

        // Use WalletManager to refund funds through multisig flow
        let mut wallet_manager = self.wallet_manager.lock().await;
        let tx_hash = wallet_manager.refund_funds(escrow_id, destinations).await?;

        info!(
            "Refund transaction submitted to network: tx_hash={}",
            tx_hash
        );

        // Update escrow with transaction hash (for blockchain monitoring)
        crate::db::db_update_escrow_transaction_hash(&self.db, escrow_id, &tx_hash).await?;

        // Update escrow status to 'refunding' (will become 'refunded' after confirmations)
        db_update_escrow_status(&self.db, escrow_id, "refunding").await?;

        // Notify parties via WebSocket
        self.websocket.do_send(WsEvent::TransactionConfirmed {
            tx_hash: tx_hash.clone(),
            confirmations: 0,
        });
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "refunding".to_string(),
        });

        info!(
            "Funds refunding for escrow {}: tx={} (awaiting confirmations)",
            escrow_id, tx_hash
        );
        Ok(tx_hash.clone())
    }

    /// Initiate dispute (either party can call)
    pub async fn initiate_dispute(
        &self,
        escrow_id: Uuid,
        requester_id: Uuid,
        reason: String,
    ) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Only buyer or vendor can initiate dispute
        if requester_id.to_string() != escrow.buyer_id
            && requester_id.to_string() != escrow.vendor_id
        {
            return Err(anyhow::anyhow!("Only buyer or vendor can initiate dispute"));
        }

        db_update_escrow_status(&self.db, escrow_id, "disputed").await?;

        // Notify arbiter
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "disputed".to_string(),
        });

        info!(
            "Dispute initiated for escrow {} by user {}: {}",
            escrow_id, requester_id, reason
        );

        Ok(())
    }

    /// Arbiter resolves dispute
    pub async fn resolve_dispute(
        &self,
        escrow_id: Uuid,
        arbiter_id: Uuid,
        resolution: &str,
        recipient_address: String,
    ) -> Result<String> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // 1. Verify escrow is in disputed state
        if escrow.status != "disputed" {
            return Err(anyhow::anyhow!(
                "Escrow not in disputed state (current: {})",
                escrow.status
            ));
        }

        // 2. Verify requester is the assigned arbiter
        if arbiter_id.to_string() != escrow.arbiter_id {
            return Err(anyhow::anyhow!("Only assigned arbiter can resolve dispute"));
        }

        // 3. Validate resolution
        if resolution != "buyer" && resolution != "vendor" {
            return Err(anyhow::anyhow!(
                "Invalid resolution: must be 'buyer' or 'vendor'"
            ));
        }

        // 4. Update escrow status based on resolution
        let new_status = match resolution {
            "buyer" => "resolved_buyer",
            "vendor" => "resolved_vendor",
            _ => anyhow::bail!("Invalid resolution after validation: {}", resolution),
        };
        db_update_escrow_status(&self.db, escrow_id, new_status)
            .await
            .context("Failed to update escrow status after resolution")?;

        // 5. Notify all parties via WebSocket
        self.websocket.do_send(WsEvent::DisputeResolved {
            escrow_id,
            resolution: resolution.to_string(),
            decided_by: arbiter_id,
        });

        info!(
            "Dispute resolved for escrow {} in favor of {} by arbiter {}",
            escrow_id, resolution, arbiter_id
        );

        // 6. Auto-trigger appropriate action based on resolution
        let tx_hash = match resolution {
            "buyer" => {
                info!("Auto-triggering refund to buyer for escrow {}", escrow_id);
                self.refund_funds(escrow_id, arbiter_id, recipient_address)
                    .await?
            }
            "vendor" => {
                info!("Auto-triggering release to vendor for escrow {}", escrow_id);
                self.release_funds(escrow_id, arbiter_id, recipient_address)
                    .await?
            }
            _ => anyhow::bail!("Invalid resolution after validation: {}", resolution),
        };
        info!(
            "Dispute resolution complete for escrow {}: {} via tx {}",
            escrow_id, resolution, tx_hash
        );

        Ok(tx_hash)
    }
}
