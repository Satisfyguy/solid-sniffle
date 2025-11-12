//! Timeout monitoring service for detecting and handling stuck escrows
//!
//! This service runs in the background and periodically checks for escrows
//! that have exceeded their timeout deadlines. It takes automatic actions
//! based on the escrow status and sends notifications via WebSocket.

use actix::Addr;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::time::interval;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::TimeoutConfig;
use crate::db::DbPool;
use crate::models::escrow::Escrow;
use crate::repositories::MultisigStateRepository;
use crate::websocket::{NotifyUser, WebSocketServer, WsEvent};

/// Timeout monitoring service
///
/// Runs in the background and polls the database for escrows that have
/// exceeded their configured timeouts. Takes automatic actions and sends
/// notifications to affected parties.
pub struct TimeoutMonitor {
    db: DbPool,
    websocket: Addr<WebSocketServer>,
    config: TimeoutConfig,
    multisig_repo: Option<Arc<MultisigStateRepository>>,
}

impl TimeoutMonitor {
    /// Create a new TimeoutMonitor
    ///
    /// # Arguments
    /// * `db` - Database connection pool
    /// * `websocket` - WebSocket server for sending notifications
    /// * `config` - Timeout configuration (deadlines and polling intervals)
    pub fn new(db: DbPool, websocket: Addr<WebSocketServer>, config: TimeoutConfig) -> Self {
        info!(
            "TimeoutMonitor initialized with poll_interval={}s",
            config.poll_interval_secs
        );
        Self {
            db,
            websocket,
            config,
            multisig_repo: None,
        }
    }

    /// Create a new TimeoutMonitor with multisig state persistence
    ///
    /// # Arguments
    /// * `db` - Database connection pool
    /// * `websocket` - WebSocket server for sending notifications
    /// * `config` - Timeout configuration (deadlines and polling intervals)
    /// * `encryption_key` - Encryption key for multisig state data
    pub fn new_with_persistence(
        db: DbPool,
        websocket: Addr<WebSocketServer>,
        config: TimeoutConfig,
        encryption_key: Vec<u8>,
    ) -> Self {
        let multisig_repo = MultisigStateRepository::new(db.clone(), encryption_key);
        info!(
            "TimeoutMonitor initialized with persistence and poll_interval={}s",
            config.poll_interval_secs
        );
        Self {
            db,
            websocket,
            config,
            multisig_repo: Some(Arc::new(multisig_repo)),
        }
    }

    /// Start monitoring in background
    ///
    /// This spawns a background task that periodically checks for:
    /// - Expired escrows (past deadline)
    /// - Escrows approaching expiration (warning notifications)
    /// - Stuck multisig setups (if persistence enabled)
    ///
    /// The task runs indefinitely until the server shuts down.
    pub async fn start_monitoring(self: Arc<Self>) {
        let mut poll_timer = interval(self.config.poll_interval());

        info!("Starting timeout monitoring loop");

        loop {
            poll_timer.tick().await;

            // Check for expired escrows first (highest priority)
            if let Err(e) = self.check_expired_escrows().await {
                error!("Error checking expired escrows: {}", e);
            }

            // Check for escrows approaching expiration (send warnings)
            if let Err(e) = self.check_expiring_escrows().await {
                error!("Error checking expiring escrows: {}", e);
            }

            // Check for stuck multisig setups (if persistence enabled)
            if self.multisig_repo.is_some() {
                if let Err(e) = self.check_stuck_multisig_setups().await {
                    error!("Error checking stuck multisig setups: {}", e);
                }
            }
        }
    }

    /// Check for and handle expired escrows
    ///
    /// Finds all escrows past their deadline and takes appropriate action:
    /// - "created" → Cancel (multisig setup incomplete)
    /// - "funded" → Cancel (buyer never deposited funds)
    /// - "releasing"/"refunding" → Alert admin (transaction stuck)
    /// - "disputed" → Escalate (arbiter timeout)
    async fn check_expired_escrows(&self) -> Result<()> {
        let mut conn = self.db.get().context("Failed to get DB connection")?;

        let expired_escrows = tokio::task::spawn_blocking(move || {
            Escrow::find_expired(&mut conn)
        })
        .await
        .context("Task join error")??;

        if expired_escrows.is_empty() {
            return Ok(());
        }

        info!("Found {} expired escrows", expired_escrows.len());

        for escrow in expired_escrows {
            let escrow_id = escrow
                .id
                .parse::<Uuid>()
                .context("Failed to parse escrow_id")?;

            info!(
                "Processing expired escrow: id={}, status={}, created={}",
                escrow_id, escrow.status, escrow.created_at
            );

            // Handle based on current status
            match escrow.status.as_str() {
                "created" => {
                    self.handle_multisig_setup_timeout(escrow_id, escrow).await?;
                }
                "funded" => {
                    self.handle_funding_timeout(escrow_id, escrow).await?;
                }
                "releasing" | "refunding" => {
                    self.handle_transaction_timeout(escrow_id, escrow).await?;
                }
                "disputed" => {
                    self.handle_dispute_timeout(escrow_id, escrow).await?;
                }
                _ => {
                    warn!(
                        "Unexpected expired escrow status: {} for escrow {}",
                        escrow.status, escrow_id
                    );
                }
            }
        }

        Ok(())
    }

    /// Check for escrows approaching expiration and send warnings
    async fn check_expiring_escrows(&self) -> Result<()> {
        let warning_threshold = self.config.warning_threshold_secs;
        let mut conn = self.db.get().context("Failed to get DB connection")?;

        let expiring_escrows = tokio::task::spawn_blocking(move || {
            Escrow::find_expiring_soon(&mut conn, warning_threshold as i64)
        })
        .await
        .context("Task join error")??;

        if expiring_escrows.is_empty() {
            return Ok(());
        }

        info!("Found {} escrows approaching expiration", expiring_escrows.len());

        for escrow in expiring_escrows {
            let escrow_id = escrow
                .id
                .parse::<Uuid>()
                .context("Failed to parse escrow_id")?;

            let expires_in_secs = escrow
                .seconds_until_expiration()
                .unwrap_or(0);

            info!(
                "Sending expiration warning for escrow {}: {}s remaining",
                escrow_id, expires_in_secs
            );

            // Send warning to all parties (buyer, vendor, arbiter)
            self.send_expiring_warning(escrow_id, &escrow, expires_in_secs as u64)
                .await?;
        }

        Ok(())
    }

    /// Handle timeout for multisig setup (status: "created")
    ///
    /// Action: Cancel the escrow (no funds at risk, setup incomplete)
    async fn handle_multisig_setup_timeout(&self, escrow_id: Uuid, _escrow: Escrow) -> Result<()> {
        info!(
            "Multisig setup timeout for escrow {}: cancelling",
            escrow_id
        );

        // Update status to cancelled
        let mut conn = self.db.get().context("Failed to get DB connection")?;
        let escrow_id_clone = escrow_id.to_string();
        tokio::task::spawn_blocking(move || {
            Escrow::update_status(&mut conn, escrow_id_clone, "cancelled")
        })
        .await
        .context("Task join error")??;

        // Notify all parties
        self.websocket.do_send(WsEvent::EscrowAutoCancelled {
            escrow_id,
            reason: "Multisig setup not completed within 1 hour".to_string(),
            cancelled_at_status: "created".to_string(),
        });

        info!("Escrow {} auto-cancelled due to setup timeout", escrow_id);
        Ok(())
    }

    /// Handle timeout for funding (status: "funded")
    ///
    /// Action: Cancel the escrow (multisig ready but buyer never deposited)
    async fn handle_funding_timeout(&self, escrow_id: Uuid, _escrow: Escrow) -> Result<()> {
        info!(
            "Funding timeout for escrow {}: cancelling",
            escrow_id
        );

        // Update status to cancelled
        let mut conn = self.db.get().context("Failed to get DB connection")?;
        let escrow_id_clone = escrow_id.to_string();
        tokio::task::spawn_blocking(move || {
            Escrow::update_status(&mut conn, escrow_id_clone, "cancelled")
        })
        .await
        .context("Task join error")??;

        // Notify all parties
        self.websocket.do_send(WsEvent::EscrowAutoCancelled {
            escrow_id,
            reason: "Buyer did not fund escrow within 24 hours".to_string(),
            cancelled_at_status: "funded".to_string(),
        });

        info!("Escrow {} auto-cancelled due to funding timeout", escrow_id);
        Ok(())
    }

    /// Handle timeout for transaction confirmation (status: "releasing"/"refunding")
    ///
    /// Action: Alert admin (transaction may be stuck in mempool)
    /// No auto-action taken as funds are already on blockchain
    async fn handle_transaction_timeout(&self, escrow_id: Uuid, escrow: Escrow) -> Result<()> {
        let tx_hash = escrow.transaction_hash.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Escrow {} in {} status but no tx_hash", escrow_id, escrow.status)
        })?;

        warn!(
            "Transaction timeout for escrow {}: tx {} stuck for >6h",
            escrow_id,
            &tx_hash[..10]
        );

        // Calculate hours pending
        let secs_since_activity = (chrono::Utc::now().naive_utc()
            - escrow.last_activity_at)
            .num_seconds();
        let hours_pending = (secs_since_activity / 3600) as u64;

        // Send stuck transaction alert
        self.websocket.do_send(WsEvent::TransactionStuck {
            escrow_id,
            tx_hash: tx_hash.clone(),
            hours_pending,
            suggested_action: "Check blockchain explorer for transaction status. \
                              May need to increase fee or wait for mempool clearance."
                .to_string(),
        });

        info!(
            "Sent stuck transaction alert for escrow {} (tx: {})",
            escrow_id,
            &tx_hash[..10]
        );

        Ok(())
    }

    /// Handle timeout for dispute resolution (status: "disputed")
    ///
    /// Action: Escalate to admin (arbiter failed to resolve in time)
    /// Optionally: Auto-refund to buyer after escalation
    async fn handle_dispute_timeout(&self, escrow_id: Uuid, escrow: Escrow) -> Result<()> {
        let arbiter_id = escrow
            .arbiter_id
            .parse::<Uuid>()
            .context("Failed to parse arbiter_id")?;

        // Calculate days in dispute
        let secs_in_dispute = (chrono::Utc::now().naive_utc()
            - escrow.last_activity_at)
            .num_seconds();
        let days_in_dispute = (secs_in_dispute / 86400) as u64;

        warn!(
            "Dispute timeout for escrow {}: arbiter {} did not resolve in {} days",
            escrow_id, arbiter_id, days_in_dispute
        );

        // Send escalation notification
        self.websocket.do_send(WsEvent::DisputeEscalated {
            escrow_id,
            arbiter_id,
            days_in_dispute,
            action_taken: "Escalated to admin. Automatic refund may be triggered.".to_string(),
        });

        // TODO: Implement auto-refund policy after escalation
        // For now, just escalate and require manual intervention

        info!(
            "Dispute escalated for escrow {}: admin intervention required",
            escrow_id
        );

        Ok(())
    }

    /// Send expiration warning to all parties
    async fn send_expiring_warning(
        &self,
        escrow_id: Uuid,
        escrow: &Escrow,
        expires_in_secs: u64,
    ) -> Result<()> {
        let action_required = match escrow.status.as_str() {
            "created" => "Complete multisig setup".to_string(),
            "funded" => "Buyer: deposit funds to escrow address".to_string(),
            "releasing" | "refunding" => "Wait for blockchain confirmation".to_string(),
            "disputed" => "Arbiter: resolve dispute".to_string(),
            _ => "No action required".to_string(),
        };

        // Parse party IDs
        let buyer_id = escrow
            .buyer_id
            .parse::<Uuid>()
            .context("Failed to parse buyer_id")?;
        let vendor_id = escrow
            .vendor_id
            .parse::<Uuid>()
            .context("Failed to parse vendor_id")?;
        let arbiter_id = escrow
            .arbiter_id
            .parse::<Uuid>()
            .context("Failed to parse arbiter_id")?;

        // Send to all parties
        for user_id in [buyer_id, vendor_id, arbiter_id] {
            self.websocket.do_send(NotifyUser {
                user_id,
                event: WsEvent::EscrowExpiring {
                    escrow_id,
                    status: escrow.status.clone(),
                    expires_in_secs,
                    action_required: action_required.clone(),
                },
            });
        }

        info!(
            "Sent expiration warning for escrow {}: {}s remaining",
            escrow_id, expires_in_secs
        );

        Ok(())
    }

    /// Check for stuck multisig setups using persisted state
    ///
    /// Identifies escrows where multisig setup has stalled:
    /// - Last state update > 15 minutes ago
    /// - Status is "created" (setup not completed)
    /// - Multisig state exists but setup incomplete
    ///
    /// Action: Send stuck setup notification to all parties
    async fn check_stuck_multisig_setups(&self) -> Result<()> {
        let repo = self
            .multisig_repo
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("MultisigStateRepository not initialized"))?;

        // Use repository's built-in method to find stuck escrows (15 minutes = 900 seconds)
        let stuck_escrow_ids = repo.find_stuck_escrows(900)?;

        if stuck_escrow_ids.is_empty() {
            return Ok(());
        }

        info!(
            "Found {} stuck multisig setup(s)",
            stuck_escrow_ids.len()
        );

        for escrow_id_str in stuck_escrow_ids {
            let escrow_id = escrow_id_str
                .parse::<Uuid>()
                .context("Failed to parse escrow_id")?;

            // Load both escrow and snapshot to get phase and timestamp
            let mut conn = self.db.get().context("Failed to get DB connection")?;
            let escrow_id_clone = escrow_id_str.clone();

            let escrow_result = tokio::task::spawn_blocking(move || {
                use diesel::prelude::*;
                use crate::schema::escrows::dsl::*;
                escrows.find(escrow_id_clone).first::<Escrow>(&mut conn).optional()
            })
            .await
            .context("Task join error")??;

            if let Some(escrow) = escrow_result {
                match repo.load_snapshot(&escrow_id_str) {
                    Ok(Some(snapshot)) => {
                        let minutes_stuck = (chrono::Utc::now().naive_utc().timestamp()
                            - escrow.multisig_updated_at as i64) / 60;

                        warn!(
                            "Stuck multisig setup detected for escrow {}: {} minutes with no progress",
                            escrow_id, minutes_stuck
                        );

                        // Send stuck setup notification
                        self.websocket.do_send(WsEvent::MultisigSetupStuck {
                            escrow_id,
                            minutes_stuck: minutes_stuck as u64,
                            last_step: snapshot.phase.status_description(),
                            suggested_action: "Check wallet RPC connectivity and retry multisig setup"
                                .to_string(),
                        });

                        info!(
                            "Sent stuck multisig setup notification for escrow {}",
                            escrow_id
                        );
                    }
                    Ok(None) => {
                        warn!(
                            "Stuck escrow {} has no snapshot (should not happen)",
                            escrow_id
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to load multisig snapshot for escrow {}: {}",
                            escrow_id, e
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_creation() {
        let config = TimeoutConfig::default();
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| ":memory:".to_string());

        // Just verify we can create the struct (actual testing requires DB setup)
        assert_eq!(config.poll_interval_secs, 60);
        assert_eq!(config.multisig_setup_timeout_secs, 3600);
    }
}
