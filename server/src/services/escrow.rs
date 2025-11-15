//! Escrow orchestration service for Monero Marketplace

use actix::Addr;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{error, info, warn};
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
    /// PHASE 1: Per-escrow locks to prevent race conditions
    escrow_locks: Arc<crate::concurrency::EscrowLockRegistry>,
}

impl EscrowOrchestrator {
    /// Create a new EscrowOrchestrator
    pub fn new(
        wallet_manager: Arc<Mutex<WalletManager>>,
        db: DbPool,
        websocket: Addr<WebSocketServer>,
        encryption_key: Vec<u8>,
        escrow_locks: Arc<crate::concurrency::EscrowLockRegistry>,
    ) -> Self {
        Self {
            wallet_manager,
            db,
            websocket,
            encryption_key,
            escrow_locks,
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

    /// Initialize new escrow (step 1) - NON-CUSTODIAL ARCHITECTURE
    ///
    /// **100% NON-CUSTODIAL FLOW:**
    /// 1. Assign arbiter
    /// 2. Create escrow record in database
    /// 3. Create 3 EMPTY temporary wallets (buyer_temp, vendor_temp, arbiter_temp)
    /// 4. Store temp wallet IDs in escrow record
    /// 5. Setup multisig to generate shared address
    /// 6. Buyer pays directly from external wallet → multisig address
    ///
    /// **CRITICAL:** Server creates EMPTY wallets only for multisig coordination.
    /// These wallets never hold funds - they only generate the multisig address.
    ///
    /// # ⚠️ DEPRECATED - Phase 3 Non-Custodial Migration
    ///
    /// **This function is CUSTODIAL despite the NON-CUSTODIAL label in comments.**
    ///
    /// **Why deprecated:**
    /// - Server creates temporary wallets on server (lines 202-214)
    /// - Uses `WalletManager::create_temporary_wallet()` which is custodial
    /// - Server has access to wallet files (even if empty)
    /// - Violates true non-custodial architecture (Haveno-style)
    ///
    /// **Use instead:**
    /// - `EscrowCoordinator::register_client_wallet()` - Clients provide RPC URLs
    /// - `EscrowCoordinator::coordinate_multisig_exchange()` - Server coordinates only
    /// - See: `DOX/guides/NON-CUSTODIAL-USER-GUIDE.md`
    /// - See: `DOX/guides/MIGRATION-TO-NONCUSTODIAL.md`
    ///
    /// **True non-custodial flow:**
    /// 1. Each client runs local `monero-wallet-rpc`
    /// 2. Clients register RPC URLs with server (no wallet creation)
    /// 3. Server coordinates multisig_info exchange
    /// 4. Clients finalize multisig locally
    ///
    /// This function will be removed in **v0.4.0** (estimated 2-3 weeks).
    #[deprecated(
        since = "0.3.0",
        note = "Server-side wallet creation is custodial. Use EscrowCoordinator with client wallets instead. Will be removed in v0.4.0. See DOX/guides/MIGRATION-TO-NONCUSTODIAL.md"
    )]
    pub async fn init_escrow(
        &self,
        order_id: Uuid,
        buyer_id: Uuid,
        vendor_id: Uuid,
        amount_atomic: i64,
    ) -> Result<Escrow> {
        // ⚠️ DEPRECATION WARNING
        warn!(
            "⚠️  DEPRECATED: EscrowOrchestrator::init_escrow() uses server-side wallet creation (CUSTODIAL). \
            Migrate to EscrowCoordinator for true non-custodial escrow. See DOX/guides/MIGRATION-TO-NONCUSTODIAL.md"
        );

        info!(
            "🔄 [CUSTODIAL-DEPRECATED] Initializing escrow: order={}, buyer={}, vendor={}, amount={}",
            order_id, buyer_id, vendor_id, amount_atomic
        );

        // 1. Assign arbiter using round-robin from available arbiters
        let arbiter_id = self.assign_arbiter().await?;
        info!("✅ Assigned arbiter {} to escrow", arbiter_id);

        // 2. Create escrow in DB
        let escrow_id = Uuid::new_v4();
        let new_escrow = NewEscrow {
            id: escrow_id.to_string(),
            order_id: order_id.to_string(),
            buyer_id: buyer_id.to_string(),
            vendor_id: vendor_id.to_string(),
            arbiter_id: arbiter_id.to_string(),
            amount: amount_atomic,
            status: "created".to_string(),
        };

        let mut escrow = db_insert_escrow(&self.db, new_escrow)
            .await
            .context("Failed to create escrow in database")?;

        info!("✅ Escrow record created: {}", escrow.id);

        // PHASE 1: Acquire per-escrow lock to prevent concurrent operations
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        // 3. Create 3 EMPTY temporary wallets for multisig coordination
        info!("🔒 [NON-CUSTODIAL] Creating 3 EMPTY temporary wallets (0 XMR balance)...");
        let mut wallet_manager = self.wallet_manager.lock().await;

        let buyer_temp_wallet_id = wallet_manager
            .create_temporary_wallet(escrow_id, "buyer")
            .await
            .context("Failed to create buyer temp wallet")?;

        let vendor_temp_wallet_id = wallet_manager
            .create_temporary_wallet(escrow_id, "vendor")
            .await
            .context("Failed to create vendor temp wallet")?;

        let arbiter_temp_wallet_id = wallet_manager
            .create_temporary_wallet(escrow_id, "arbiter")
            .await
            .context("Failed to create arbiter temp wallet")?;

        info!(
            "✅ Temporary wallets created: buyer={}, vendor={}, arbiter={}",
            buyer_temp_wallet_id, vendor_temp_wallet_id, arbiter_temp_wallet_id
        );

        // Release the wallet_manager lock before calling setup_multisig_non_custodial
        // (which will need to acquire the lock again)
        drop(wallet_manager);

        // 4. Wallet IDs kept in memory only (no DB persistence needed for deprecated feature)
        info!("✅ Temporary wallets ready: buyer={}, vendor={}, arbiter={}", buyer_temp_wallet_id, vendor_temp_wallet_id, arbiter_temp_wallet_id);

        // 5. Setup multisig to generate the shared address (Phase 2 - NON-CUSTODIAL)
        info!("🔐 [PHASE 2] Starting real multisig setup to generate shared address");

        let multisig_result = self
            .setup_multisig_non_custodial(
                escrow_id,
                buyer_temp_wallet_id,
                vendor_temp_wallet_id,
                arbiter_temp_wallet_id,
            )
            .await;

        let multisig_address = match multisig_result {
            Ok(address) => {
                info!(
                    "✅ [PHASE 2] Multisig address generated: {}",
                    &address[..10]
                );
                address
            }
            Err(e) => {
                // CRITICAL: Cleanup on failure to free RPC slots
                error!("❌ Multisig setup failed: {}. Cleaning up wallets to free RPC slots...", e);

                let mut wallet_manager = self.wallet_manager.lock().await;
                Self::cleanup_escrow_wallets(
                    &mut wallet_manager,
                    buyer_temp_wallet_id,
                    vendor_temp_wallet_id,
                    arbiter_temp_wallet_id,
                )
                .await;

                return Err(e.context("Failed to setup multisig"));
            }
        };

        // 6. Reload escrow again to get final state
        escrow = db_load_escrow(&self.db, escrow_id).await?;

        // 7. Notify parties via WebSocket
        self.websocket.do_send(WsEvent::EscrowInit {
            escrow_id,
        });

        info!(
            "✅ [NON-CUSTODIAL] Escrow {} initialized successfully! Buyer can now pay from ANY external wallet → {}",
            escrow.id, multisig_address
        );

        Ok(escrow)
    }

    /// Close all 3 temporary escrow wallets to free RPC slots (DRY helper)
    ///
    /// This method is called in multiple scenarios:
    /// 1. After successful multisig setup
    /// 2. After multisig setup failure (cleanup on error)
    /// 3. On timeout or any other error during escrow initialization
    ///
    /// # Arguments
    /// * `wallet_manager` - Locked WalletManager reference
    /// * `buyer_id` - Buyer wallet UUID
    /// * `vendor_id` - Vendor wallet UUID
    /// * `arbiter_id` - Arbiter wallet UUID
    ///
    /// # Returns
    /// Number of wallets successfully closed
    async fn cleanup_escrow_wallets(
        wallet_manager: &mut WalletManager,
        buyer_id: Uuid,
        vendor_id: Uuid,
        arbiter_id: Uuid,
    ) -> usize {
        info!("🔓 Closing 3 temporary wallets to free RPC instances...");

        let mut cleanup_count = 0;

        if let Some(pool) = wallet_manager.wallet_pool() {
            // Get pool stats BEFORE closing
            let stats_before = pool.stats().await;
            info!(
                "🔍 WalletPool stats BEFORE cleanup: {}/{} RPC slots free",
                stats_before.free, stats_before.total
            );

            // Collect wallet IDs
            let wallet_ids = [buyer_id, vendor_id, arbiter_id];

            // Close each wallet
            for wallet_id in &wallet_ids {
                if let Some(wallet) = wallet_manager.wallets.get(wallet_id) {
                    if let Some(port) = wallet.rpc_port {
                        match pool.close_wallet(port).await {
                            Ok(_) => {
                                info!("✅ Cleanup: Closed wallet {} on port {}", wallet_id, port);
                                cleanup_count += 1;
                            }
                            Err(e) => {
                                error!(
                                    "❌ Cleanup failed for wallet {} on port {}: {}",
                                    wallet_id, port, e
                                );
                            }
                        }
                    } else {
                        warn!("⚠️ Wallet {} has no rpc_port tracked", wallet_id);
                    }
                } else {
                    warn!("⚠️ Wallet {} not found in WalletManager", wallet_id);
                }
            }

            // Get pool stats AFTER closing
            let stats_after = pool.stats().await;
            info!(
                "🔍 WalletPool stats AFTER cleanup: {}/{} RPC slots free (freed {} slots)",
                stats_after.free, stats_after.total, cleanup_count
            );
        } else {
            warn!("⚠️ WalletPool not enabled - wallets will remain open (legacy mode)");
        }

        info!("✅ Cleanup complete: closed {} wallets", cleanup_count);
        cleanup_count
    }

    /// Setup multisig for non-custodial architecture (private method)
    ///
    /// **CRITICAL MULTISIG FLOW:**
    /// 1. prepare_multisig() - Each wallet generates multisig info
    /// 2. make_multisig() - Exchange infos to create multisig wallet
    /// 3. finalize_multisig() - Verify address consistency (must be 95 chars)
    ///
    /// **REMINDER:** All 3 wallets are EMPTY (0 XMR balance).
    /// The generated multisig address receives payment directly from buyer's external wallet.
    ///
    /// # Arguments
    /// * `escrow_id` - Escrow UUID
    /// * `buyer_temp_wallet_id` - Buyer temporary wallet UUID
    /// * `vendor_temp_wallet_id` - Vendor temporary wallet UUID
    /// * `arbiter_temp_wallet_id` - Arbiter temporary wallet UUID
    ///
    /// # Returns
    /// The generated multisig address (95 characters)
    async fn setup_multisig_non_custodial(
        &self,
        escrow_id: Uuid,
        buyer_temp_wallet_id: Uuid,
        vendor_temp_wallet_id: Uuid,
        arbiter_temp_wallet_id: Uuid,
    ) -> Result<String> {
        info!(
            "🔐 [MULTISIG SETUP] Starting for escrow {}",
            escrow_id
        );

        let mut wallet_manager = self.wallet_manager.lock().await;

        // Step 1: prepare_multisig() - Each wallet generates multisig info
        info!("📝 Step 1/3: Calling prepare_multisig() on all 3 wallets...");

        info!("📝 Calling prepare_multisig() for BUYER wallet {}", buyer_temp_wallet_id);
        let buyer_info = match wallet_manager
            .make_multisig(&escrow_id.to_string(), buyer_temp_wallet_id, vec![])
            .await
        {
            Ok(info) => {
                info!("✅ Buyer prepare_multisig() success: {} chars", info.multisig_info.len());
                info
            }
            Err(e) => {
                error!("❌ Buyer prepare_multisig() FAILED: {:?}", e);
                return Err(anyhow::anyhow!("Failed to prepare multisig for buyer: {:?}", e));
            }
        };

        let vendor_info = wallet_manager
            .make_multisig(&escrow_id.to_string(), vendor_temp_wallet_id, vec![])
            .await
            .context("Failed to prepare multisig for vendor temp wallet")?;

        let arbiter_info = wallet_manager
            .make_multisig(&escrow_id.to_string(), arbiter_temp_wallet_id, vec![])
            .await
            .context("Failed to prepare multisig for arbiter temp wallet")?;

        info!(
            "✅ Step 1/3 complete: All 3 wallets prepared (info lengths: buyer={}, vendor={}, arbiter={})",
            buyer_info.multisig_info.len(),
            vendor_info.multisig_info.len(),
            arbiter_info.multisig_info.len()
        );

        // Persist state after all 3 wallets are prepared
        use crate::models::multisig_state::MultisigPhase;
        let phase = MultisigPhase::Preparing {
            completed: vec![
                "buyer".to_string(),
                "vendor".to_string(),
                "arbiter".to_string(),
            ],
        };
        wallet_manager
            .persist_multisig_state(&escrow_id.to_string(), phase)
            .await
            .context("Failed to persist Preparing phase")?;

        info!("💾 Multisig Preparing phase persisted to database");

        // Step 2: make_multisig() - Exchange infos to create 2-of-3 multisig
        info!("🔄 Step 2/3: Exchanging multisig info (make_multisig)...");

        wallet_manager
            .exchange_multisig_info(
                escrow_id,
                vec![buyer_info, vendor_info, arbiter_info],
            )
            .await
            .context("Failed to exchange multisig info")?;

        info!("✅ Step 2/3 complete: Multisig info exchanged");

        // Step 3: finalize_multisig() - Generate final multisig address
        info!("🎯 Step 3/3: Finalizing multisig to generate address...");

        let multisig_address = wallet_manager
            .finalize_multisig(escrow_id)
            .await
            .context("Failed to finalize multisig")?;

        // Validate address is 95 characters (Monero mainnet multisig address standard)
        if multisig_address.len() != 95 {
            return Err(anyhow::anyhow!(
                "CRITICAL: Multisig address length is {} (expected 95 chars). Address: {}",
                multisig_address.len(),
                multisig_address
            ));
        }

        info!(
            "✅ Step 3/3 complete: Multisig address finalized (95 chars verified)"
        );

        // Store multisig address in database
        db_update_escrow_address(&self.db, escrow_id, &multisig_address)
            .await
            .context("Failed to update escrow with multisig address")?;

        info!(
            "💾 Multisig address stored in database: {}",
            &multisig_address[..10]
        );

        info!(
            "🎉 [MULTISIG SETUP] Complete! Address: {} (Wallets: EMPTY, ready for direct payment)",
            &multisig_address[..15]
        );

        // Register escrow wallet filenames for future reopening
        if let Some(pool) = wallet_manager.wallet_pool() {
            pool.register_escrow_wallets(escrow_id).await;
        }

        // Close all 3 temporary wallets to free RPC slots (DRY)
        Self::cleanup_escrow_wallets(
            &mut wallet_manager,
            buyer_temp_wallet_id,
            vendor_temp_wallet_id,
            arbiter_temp_wallet_id,
        )
        .await;

        Ok(multisig_address)
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
        // PHASE 1: Acquire per-escrow lock to prevent concurrent operations
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Only buyer can release funds
        if requester_id.to_string() != escrow.buyer_id {
            return Err(anyhow::anyhow!("Only buyer can release funds"));
        }

        // Validate escrow is in funded/active state (active = funds received and confirmed)
        if escrow.status != "funded" && escrow.status != "active" {
            return Err(anyhow::anyhow!(
                "Escrow must be in 'funded' or 'active' state to release funds (current: {})",
                escrow.status
            ));
        }

        // Validate vendor address format (mainnet: 4/8, testnet: 9/A/B, 95-106 chars)
        let first_char = vendor_address.chars().next().unwrap_or('0');
        let len = vendor_address.len();
        if len < 95 || len > 106 {
            return Err(anyhow::anyhow!(
                "Invalid Monero address length: {} (must be 95-106 characters)", len
            ));
        }
        if first_char != '4' && first_char != '8' && first_char != '9' && first_char != 'A' && first_char != 'B' {
            return Err(anyhow::anyhow!(
                "Invalid Monero address format: must start with 4/8 (mainnet) or 9/A/B (testnet)"
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
        // PHASE 1: Acquire per-escrow lock to prevent concurrent operations
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // Vendor or arbiter can initiate refund
        if requester_id.to_string() != escrow.vendor_id
            && requester_id.to_string() != escrow.arbiter_id
        {
            return Err(anyhow::anyhow!(
                "Only vendor or arbiter can initiate refund"
            ));
        }

        // Validate escrow is in funded/active state (active = funds received and confirmed)
        if escrow.status != "funded" && escrow.status != "active" {
            return Err(anyhow::anyhow!(
                "Escrow must be in 'funded' or 'active' state to refund (current: {})",
                escrow.status
            ));
        }

        // Validate buyer address format (mainnet: 4/8, testnet: 9/A/B, 95-106 chars)
        let first_char = buyer_address.chars().next().unwrap_or('0');
        let len = buyer_address.len();
        if len < 95 || len > 106 {
            return Err(anyhow::anyhow!(
                "Invalid Monero address length: {} (must be 95-106 characters)", len
            ));
        }
        if first_char != '4' && first_char != '8' && first_char != '9' && first_char != 'A' && first_char != 'B' {
            return Err(anyhow::anyhow!(
                "Invalid Monero address format: must start with 4/8 (mainnet) or 9/A/B (testnet)"
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

    /// Sync multisig wallets and get current balance (LAZY SYNC PATTERN)
    ///
    /// This method implements the lazy sync pattern to check escrow balance
    /// while maintaining RPC rotation architecture. It reopens all 3 wallets,
    /// performs multisig info exchange, checks balance, then closes wallets.
    ///
    /// # Arguments
    /// * `escrow_id` - UUID of the escrow to check balance for
    ///
    /// # Returns
    /// Tuple of (balance_atomic, unlocked_balance_atomic) in piconeros
    ///
    /// # Errors
    /// - Escrow not found
    /// - Wallet synchronization failed
    /// - Balance check failed
    ///
    /// # Performance
    /// Expected latency: 3-5 seconds (acceptable for manual balance checks)
    ///
    /// # Example
    /// ```rust
    /// let (balance, unlocked) = orchestrator.sync_and_get_balance(escrow_id).await?;
    /// info!("Escrow has {} XMR ({} unlocked)", balance / 1e12, unlocked / 1e12);
    /// ```
    pub async fn sync_and_get_balance(&self, escrow_id: Uuid) -> Result<(u64, u64)> {
        info!("🔄 Syncing multisig wallets for escrow: {}", escrow_id);

        // Verify escrow exists
        let escrow = db_load_escrow(&self.db, escrow_id)
            .await
            .context("Failed to load escrow")?;

        let address_preview = escrow.multisig_address
            .as_ref()
            .map(|addr| &addr[..10.min(addr.len())])
            .unwrap_or("(none)");

        info!(
            "Loaded escrow {}: status={}, multisig_address={}",
            escrow_id,
            escrow.status,
            address_preview
        );

        // Call WalletManager's sync method
        let mut wallet_manager = self.wallet_manager.lock().await;
        let (balance, unlocked_balance) = wallet_manager
            .sync_multisig_wallets(escrow_id)
            .await
            .context("Failed to sync multisig wallets")?;

        info!(
            "✅ Balance sync complete for escrow {}: {} atomic units ({} XMR)",
            escrow_id,
            balance,
            (balance as f64) / 1_000_000_000_000.0
        );

        Ok((balance, unlocked_balance))
    }

    /// DEV ONLY: Initialize mock multisig wallets for testing
    ///
    /// This method creates mock wallets in the WalletManager to allow
    /// testing the release/refund flow without real multisig setup.
    pub async fn dev_initialize_mock_wallets(&self, escrow_id: Uuid) -> Result<()> {
        info!("DEV: Initializing mock multisig wallets for escrow {}", escrow_id);

        let mut wallet_manager = self.wallet_manager.lock().await;

        // Call the dev method on wallet_manager to create mock wallets
        wallet_manager.dev_create_mock_multisig(escrow_id).await?;

        info!("DEV: Mock wallets created for escrow {}", escrow_id);
        Ok(())
    }
}
