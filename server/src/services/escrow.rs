//! Escrow orchestration service for Monero Marketplace

use actix::Addr;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::db::{DbPool, db_insert_escrow, db_load_escrow, db_update_escrow_address, db_update_escrow_status, db_store_multisig_info, db_count_multisig_infos, db_load_multisig_infos};
use crate::crypto::encryption::{encrypt_field, decrypt_field};
use crate::models::escrow::{Escrow, NewEscrow};
use crate::models::user::User;
use crate::wallet_manager::WalletManager;
use monero_marketplace_common::types::TransferDestination;
use tokio::sync::Mutex;
use crate::websocket::{WebSocketServer, WsEvent};

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

        let escrow = db_insert_escrow(&self.db, new_escrow).await
            .context("Failed to create escrow in database")?;

        // 3. Notify parties via WebSocket
        let escrow_uuid = escrow.id.parse().context("Failed to parse escrow_id to Uuid")?;
        self.websocket.do_send(WsEvent::EscrowInit { escrow_id: escrow_uuid });
        self.websocket.do_send(WsEvent::EscrowInit { escrow_id: escrow_uuid });
        self.websocket.do_send(WsEvent::EscrowAssigned { escrow_id: escrow_uuid });

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
        info!("Collecting prepare info for escrow {} from user {}", escrow_id, user_id);

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
            return Err(anyhow::anyhow!("User {} is not part of escrow {}", user_id, escrow_id));
        };

        // Store in DB
        db_store_multisig_info(&self.db, escrow_id, party, encrypted).await
            .context("Failed to store multisig info")?;

        // Check if all 3 received
        let count = db_count_multisig_infos(&self.db, escrow_id).await?;
        info!("Collected {} of 3 multisig infos for escrow {}", count, escrow_id);

        if count == 3 {
            info!("All multisig infos collected for escrow {}. Triggering make_multisig.", escrow_id);
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
        let buyer = tokio::task::spawn_blocking(move || User::find_by_id(&mut conn_buyer, escrow_for_buyer.buyer_id.clone()))
            .await??;

        let escrow_for_vendor = escrow.clone();
        let mut conn_vendor = self.db.get().context("Failed to get DB connection")?;
        let vendor = tokio::task::spawn_blocking(move || User::find_by_id(&mut conn_vendor, escrow_for_vendor.vendor_id.clone()))
            .await??;

        let escrow_for_arbiter = escrow.clone();
        let mut conn_arbiter = self.db.get().context("Failed to get DB connection")?;
        let arbiter = tokio::task::spawn_blocking(move || User::find_by_id(&mut conn_arbiter, escrow_for_arbiter.arbiter_id.clone()))
            .await??;

        let buyer_wallet_id = buyer.wallet_id.ok_or_else(|| anyhow::anyhow!("Buyer wallet ID not found"))?.parse::<Uuid>()?;
        let vendor_wallet_id = vendor.wallet_id.ok_or_else(|| anyhow::anyhow!("Vendor wallet ID not found"))?.parse::<Uuid>()?;
        let arbiter_wallet_id = arbiter.wallet_id.ok_or_else(|| anyhow::anyhow!("Arbiter wallet ID not found"))?.parse::<Uuid>()?;

        let mut wallet_manager = self.wallet_manager.lock().await;

        // 1. Prepare multisig for each participant
        let buyer_info = wallet_manager.make_multisig(buyer_wallet_id, vec![]).await?;
        let vendor_info = wallet_manager.make_multisig(vendor_wallet_id, vec![]).await?;
        let arbiter_info = wallet_manager.make_multisig(arbiter_wallet_id, vec![]).await?;

        // 2. Exchange multisig info
        wallet_manager.exchange_multisig_info(escrow_id, vec![buyer_info, vendor_info, arbiter_info]).await?;

        // 3. Finalize multisig
        let multisig_address = wallet_manager.finalize_multisig(escrow_id).await?;

        info!("Multisig address created successfully: {}", multisig_address);

        // Store multisig address in DB
        db_update_escrow_address(&self.db, escrow_id, &multisig_address).await
            .context("Failed to update escrow with multisig address")?;

        // Update status to 'funded' (ready to receive funds)
        db_update_escrow_status(&self.db, escrow_id, "funded").await
            .context("Failed to update escrow status")?;

        // Notify parties
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "funded".to_string()
        });
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "funded".to_string()
        });
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "funded".to_string()
        });

        info!("Multisig setup complete for escrow {}", escrow_id);
        Ok(())
    }

    /// Assign an arbiter using round-robin selection
    async fn assign_arbiter(&self) -> Result<String> {
        // Get connection from pool
        let mut conn = self.db.get().context("Failed to get DB connection")?;

        // Find all users with 'arbiter' role
        let arbiters = tokio::task::spawn_blocking(move || {
            User::find_by_role(&mut conn, "arbiter")
        })
        .await
        .context("Task join error")??;

        if arbiters.is_empty() {
            return Err(anyhow::anyhow!("No arbiters available in the system"));
        }

        // Simple round-robin: pick first arbiter
        // In production, track arbiter workload and balance assignments
        let selected_arbiter = &arbiters[0];

        info!("Selected arbiter: {} ({})", selected_arbiter.username, selected_arbiter.id);
        Ok(selected_arbiter.id.clone())
    }

    /// Release funds to vendor (buyer approves)
    pub async fn release_funds(&self, escrow_id: Uuid, requester_id: Uuid) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Only buyer can release funds
        if requester_id.to_string() != escrow.buyer_id {
            return Err(anyhow::anyhow!("Only buyer can release funds"));
        }

        let mut conn = self.db.get().context("Failed to get DB connection")?;
        let vendor = tokio::task::spawn_blocking(move || User::find_by_id(&mut conn, escrow.vendor_id.to_string()))
            .await??;

        let destination = TransferDestination {
            address: vendor.wallet_address.ok_or_else(|| anyhow::anyhow!("Vendor wallet address not found"))?,
            amount: escrow.amount as u64,
        };

        let mut wallet_manager = self.wallet_manager.lock().await;
        wallet_manager.release_funds(escrow_id, vec![destination]).await?;

        db_update_escrow_status(&self.db, escrow_id, "released").await?;
        info!("Funds released for escrow {}", escrow_id);

        Ok(())
    }

    /// Initiate dispute (either party can call)
    pub async fn initiate_dispute(&self, escrow_id: Uuid, requester_id: Uuid, reason: String) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Only buyer or vendor can initiate dispute
        if requester_id.to_string() != escrow.buyer_id && requester_id.to_string() != escrow.vendor_id {
            return Err(anyhow::anyhow!("Only buyer or vendor can initiate dispute"));
        }

        db_update_escrow_status(&self.db, escrow_id, "disputed").await?;

        // Notify arbiter
        self.websocket.do_send(WsEvent::EscrowStatusChanged {
            escrow_id,
            new_status: "disputed".to_string()
        });

        info!("Dispute initiated for escrow {} by user {}: {}", escrow_id, requester_id, reason);

        Ok(())
    }

    /// Arbiter resolves dispute
    pub async fn resolve_dispute(&self, escrow_id: Uuid, arbiter_id: Uuid, resolution: &str) -> Result<()> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Only assigned arbiter can resolve
        if arbiter_id.to_string() != escrow.arbiter_id {
            return Err(anyhow::anyhow!("Only assigned arbiter can resolve dispute"));
        }

        // Resolution can be "buyer" or "vendor"
        let new_status = match resolution {
            "buyer" => "resolved_buyer",
            "vendor" => "resolved_vendor",
            _ => return Err(anyhow::anyhow!("Invalid resolution: must be 'buyer' or 'vendor'")),
        };

        db_update_escrow_status(&self.db, escrow_id, new_status).await?;

        info!("Dispute resolved for escrow {} in favor of {}", escrow_id, resolution);

        Ok(())
    }
}
