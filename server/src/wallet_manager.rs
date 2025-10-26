//! Wallet manager for server-side Monero interactions

use anyhow::Result;
use monero_marketplace_common::{
    error::{Error as CommonError, MoneroError},
    types::{MoneroConfig, MultisigInfo},
};
use monero_marketplace_wallet::MoneroClient;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, error, debug, warn};
use uuid::Uuid;

use crate::db::DbPool;
use crate::repositories::MultisigStateRepository;
use crate::models::multisig_state::{MultisigPhase, MultisigSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub enum WalletRole {
    Buyer,
    Vendor,
    Arbiter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MultisigState {
    NotStarted,
    PreparedInfo(MultisigInfo),
    InfoExchanged {
        round: u8,
        participants: Vec<String>,
    },
    Ready {
        address: String,
    },
}

pub struct WalletInstance {
    pub id: Uuid,
    pub role: WalletRole,
    pub rpc_client: MoneroClient,
    pub address: String,
    pub multisig_state: MultisigState,
}

#[derive(Error, Debug)]
pub enum WalletManagerError {
    #[error("Monero RPC error: {0}")]
    RpcError(#[from] CommonError),

    #[error("Invalid multisig state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Wallet not found: {0}")]
    WalletNotFound(Uuid),

    #[error("All RPC endpoints unavailable")]
    NoAvailableRpc,

    #[error("Multisig address mismatch: {addresses:?}")]
    AddressMismatch { addresses: Vec<String> },

    #[error("Non-custodial policy violation: Server cannot create {0} wallets. Clients must provide their own wallet RPC URL.")]
    NonCustodialViolation(String),

    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
}

pub struct WalletManager {
    pub wallets: HashMap<Uuid, WalletInstance>,
    rpc_configs: Vec<MoneroConfig>,
    next_rpc_index: usize,
    // Multisig state persistence (Option for backward compatibility)
    multisig_repo: Option<Arc<MultisigStateRepository>>,
    db_pool: Option<DbPool>,
}

impl WalletManager {
    pub fn new(configs: Vec<MoneroConfig>) -> Result<Self> {
        if configs.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one Monero RPC config is required"
            ));
        }
        info!(
            "WalletManager initialized with {} RPC endpoints",
            configs.len()
        );
        Ok(Self {
            wallets: HashMap::new(),
            rpc_configs: configs,
            next_rpc_index: 0,
            multisig_repo: None,
            db_pool: None,
        })
    }

    /// Create WalletManager with multisig state persistence enabled
    ///
    /// This constructor enables automatic persistence of multisig wallet states
    /// to the database, allowing recovery after server restarts.
    ///
    /// # Arguments
    /// * `configs` - Monero RPC endpoint configurations
    /// * `db_pool` - Database connection pool
    /// * `encryption_key` - 32-byte key for AES-256-GCM field encryption
    ///
    /// # Returns
    /// WalletManager instance with persistence enabled
    ///
    /// # Example
    /// ```ignore
    /// let encryption_key = env::var("MULTISIG_ENCRYPTION_KEY")?.into_bytes();
    /// let wallet_manager = WalletManager::new_with_persistence(
    ///     rpc_configs,
    ///     db_pool.clone(),
    ///     encryption_key,
    /// )?;
    /// ```
    pub fn new_with_persistence(
        configs: Vec<MoneroConfig>,
        db_pool: DbPool,
        encryption_key: Vec<u8>,
    ) -> Result<Self> {
        if configs.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one Monero RPC config is required"
            ));
        }

        let multisig_repo = MultisigStateRepository::new(db_pool.clone(), encryption_key);

        info!(
            "WalletManager initialized with {} RPC endpoints and persistence enabled",
            configs.len()
        );

        Ok(Self {
            wallets: HashMap::new(),
            rpc_configs: configs,
            next_rpc_index: 0,
            multisig_repo: Some(Arc::new(multisig_repo)),
            db_pool: Some(db_pool),
        })
    }

    /// DEPRECATED: Use create_arbiter_wallet_instance() or register_client_wallet_rpc() instead.
    ///
    /// This method is kept for backward compatibility but will be removed in future versions.
    /// Server should ONLY create arbiter wallets to maintain non-custodial architecture.
    #[deprecated(
        since = "0.2.7",
        note = "Use create_arbiter_wallet_instance() for arbiter or register_client_wallet_rpc() for buyer/vendor"
    )]
    pub async fn create_wallet_instance(
        &mut self,
        role: WalletRole,
    ) -> Result<Uuid, WalletManagerError> {
        // NON-CUSTODIAL ENFORCEMENT: Server cannot create buyer/vendor wallets
        match role {
            WalletRole::Buyer => {
                return Err(WalletManagerError::NonCustodialViolation(
                    "Buyer".to_string(),
                ))
            }
            WalletRole::Vendor => {
                return Err(WalletManagerError::NonCustodialViolation(
                    "Vendor".to_string(),
                ))
            }
            WalletRole::Arbiter => {
                // Arbiter is OK - this is the marketplace's wallet
                info!("Creating arbiter wallet instance (legacy method - use create_arbiter_wallet_instance instead)");
            }
        }

        let config = self
            .rpc_configs
            .get(self.next_rpc_index)
            .ok_or(WalletManagerError::NoAvailableRpc)?;
        self.next_rpc_index = (self.next_rpc_index + 1) % self.rpc_configs.len();

        let rpc_client = MoneroClient::new(config.clone())?;
        let wallet_info = rpc_client.get_wallet_info().await?;

        let instance = WalletInstance {
            id: Uuid::new_v4(),
            role: role.clone(),
            rpc_client,
            address: wallet_info.address,
            multisig_state: MultisigState::NotStarted,
        };
        let id = instance.id;
        self.wallets.insert(id, instance);
        info!("Created wallet instance {} (role: {:?})", id, role);
        Ok(id)
    }

    /// Create arbiter wallet instance (server-controlled wallet for marketplace arbitration)
    ///
    /// This is the ONLY wallet type the server should create directly.
    /// Buyer and vendor wallets must be provided by clients via register_client_wallet_rpc().
    ///
    /// # Returns
    /// UUID of the created arbiter wallet instance
    ///
    /// # Errors
    /// - NoAvailableRpc - No RPC configs available
    /// - RpcError - Failed to connect to wallet RPC or get wallet info
    pub async fn create_arbiter_wallet_instance(&mut self) -> Result<Uuid, WalletManagerError> {
        let config = self
            .rpc_configs
            .get(self.next_rpc_index)
            .ok_or(WalletManagerError::NoAvailableRpc)?;
        self.next_rpc_index = (self.next_rpc_index + 1) % self.rpc_configs.len();

        let rpc_client = MoneroClient::new(config.clone())?;
        let wallet_info = rpc_client.get_wallet_info().await?;

        let wallet_address = wallet_info.address.clone();
        let instance = WalletInstance {
            id: Uuid::new_v4(),
            role: WalletRole::Arbiter,
            rpc_client,
            address: wallet_info.address,
            multisig_state: MultisigState::NotStarted,
        };
        let id = instance.id;
        self.wallets.insert(id, instance);
        info!("✅ Created arbiter wallet instance: {} (address: {})", id, wallet_address);
        Ok(id)
    }

    /// Register a client-controlled wallet RPC endpoint (NON-CUSTODIAL)
    ///
    /// This method allows buyers and vendors to provide their own wallet RPC URLs,
    /// ensuring the server never has access to their private keys.
    ///
    /// # Security Requirements
    /// - Client must run their own monero-wallet-rpc instance
    /// - Client controls their private keys (never shared with server)
    /// - RPC URL can be local (client's machine) or via Tor hidden service
    ///
    /// # Arguments
    /// * `role` - Must be Buyer or Vendor (Arbiter not allowed - use create_arbiter_wallet_instance)
    /// * `rpc_url` - Client's wallet RPC endpoint (e.g., "http://127.0.0.1:18082/json_rpc" or "http://xyz.onion:18082/json_rpc")
    /// * `rpc_user` - Optional RPC authentication username
    /// * `rpc_password` - Optional RPC authentication password
    ///
    /// # Returns
    /// UUID of the registered client wallet instance
    ///
    /// # Errors
    /// - NonCustodialViolation - Attempted to register Arbiter wallet (must use create_arbiter_wallet_instance)
    /// - InvalidRpcUrl - Invalid or unreachable RPC URL
    /// - RpcError - Failed to connect to client's wallet RPC
    ///
    /// # Example
    /// ```rust
    /// // Client provides their wallet RPC URL
    /// let wallet_id = wallet_manager.register_client_wallet_rpc(
    ///     WalletRole::Buyer,
    ///     "http://buyer-machine.local:18082/json_rpc".to_string(),
    ///     Some("buyer_user".to_string()),
    ///     Some("buyer_password".to_string()),
    /// ).await?;
    /// ```
    pub async fn register_client_wallet_rpc(
        &mut self,
        role: WalletRole,
        rpc_url: String,
        rpc_user: Option<String>,
        rpc_password: Option<String>,
    ) -> Result<Uuid, WalletManagerError> {
        // NON-CUSTODIAL ENFORCEMENT: Only allow Buyer/Vendor (clients control these)
        if role == WalletRole::Arbiter {
            return Err(WalletManagerError::NonCustodialViolation(
                "Arbiter (use create_arbiter_wallet_instance instead)".to_string(),
            ));
        }

        // Validate RPC URL format
        if !rpc_url.starts_with("http://") && !rpc_url.starts_with("https://") {
            return Err(WalletManagerError::InvalidRpcUrl(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        // Create config from client-provided details
        let config = MoneroConfig {
            rpc_url: rpc_url.clone(),
            rpc_user,
            rpc_password,
            timeout_seconds: 30,
        };

        // Connect to client's wallet RPC
        let rpc_client = MoneroClient::new(config)
            .map_err(|e| WalletManagerError::InvalidRpcUrl(format!("Failed to connect: {}", e)))?;

        // Verify wallet is accessible
        let wallet_info = rpc_client
            .get_wallet_info()
            .await
            .map_err(|e| WalletManagerError::InvalidRpcUrl(format!("Cannot access wallet: {}", e)))?;

        let instance = WalletInstance {
            id: Uuid::new_v4(),
            role: role.clone(),
            rpc_client,
            address: wallet_info.address.clone(),
            multisig_state: MultisigState::NotStarted,
        };
        let id = instance.id;
        self.wallets.insert(id, instance);

        info!(
            "✅ Registered client wallet: id={}, role={:?}, address={}",
            id, role, wallet_info.address
        );
        info!("🔒 NON-CUSTODIAL: Client controls private keys at {}", rpc_url);

        Ok(id)
    }

    // ========== MULTISIG STATE PERSISTENCE HELPERS ==========

    /// Helper: Persist multisig state to database
    ///
    /// Called after each successful multisig transition to ensure state is saved.
    /// If persistence is not enabled (multisig_repo is None), this is a no-op.
    ///
    /// # Arguments
    /// * `escrow_id` - Escrow identifier
    /// * `phase` - New multisig phase to persist
    ///
    /// # Returns
    /// Ok(()) on success, Err if persistence fails
    ///
    /// # Errors
    /// Returns WalletManagerError if repository save fails
    async fn persist_multisig_state(
        &self,
        escrow_id: &str,
        phase: MultisigPhase,
    ) -> Result<(), WalletManagerError> {
        // Skip if persistence not enabled
        let Some(ref repo) = self.multisig_repo else {
            debug!("Multisig persistence not enabled, skipping save for escrow {}", escrow_id);
            return Ok(());
        };

        // Build snapshot with current wallet states
        let mut wallet_ids = HashMap::new();
        let mut rpc_urls = HashMap::new();

        for (uuid, instance) in &self.wallets {
            let role_str = match instance.role {
                WalletRole::Buyer => "buyer",
                WalletRole::Vendor => "vendor",
                WalletRole::Arbiter => "arbiter",
            };
            wallet_ids.insert(role_str.to_string(), *uuid);
            // TODO: Store actual RPC URL when available from MoneroClient
            rpc_urls.insert(role_str.to_string(), "localhost:18082".to_string());
        }

        let snapshot = MultisigSnapshot::new(phase.clone(), wallet_ids, rpc_urls);

        // Persist to database
        repo.save_phase(escrow_id, &phase, &snapshot)
            .map_err(|e| {
                error!(escrow_id, error = %e, "Failed to persist multisig state");
                WalletManagerError::RpcError(CommonError::MoneroRpc(
                    format!("Persistence failed: {}", e),
                ))
            })?;

        debug!(escrow_id, phase = ?phase, "✅ Multisig state persisted");
        Ok(())
    }

    // ========== PUBLIC MULTISIG METHODS ==========

    pub async fn make_multisig(
        &mut self,
        escrow_id: &str,
        wallet_id: Uuid,
        _participants: Vec<String>,
    ) -> Result<MultisigInfo, WalletManagerError> {
        let wallet = self
            .wallets
            .get_mut(&wallet_id)
            .ok_or(WalletManagerError::WalletNotFound(wallet_id))?;

        let info = wallet.rpc_client.multisig().prepare_multisig().await?;
        wallet.multisig_state = MultisigState::PreparedInfo(info.clone());

        // Persist state: Preparing phase
        let phase = MultisigPhase::Preparing {
            completed: vec![wallet_id.to_string()],
        };
        self.persist_multisig_state(escrow_id, phase).await?;

        info!(escrow_id, wallet_id = %wallet_id, "Multisig preparation completed and persisted");
        Ok(info)
    }

    pub async fn exchange_multisig_info(
        &mut self,
        escrow_id: Uuid,
        info_from_all: Vec<MultisigInfo>,
    ) -> Result<(), WalletManagerError> {
        let escrow_id_str = escrow_id.to_string();
        info!("Exchanging multisig info for escrow {}", escrow_id);

        // This is a simplified implementation. A real one would be more complex.
        for wallet in self.wallets.values_mut() {
            let other_infos = info_from_all
                .iter()
                .filter(|i| i.multisig_info != wallet.address) // This is incorrect, just a placeholder
                .map(|i| i.multisig_info.clone())
                .collect();
            let result = wallet
                .rpc_client
                .multisig()
                .make_multisig(2, other_infos)
                .await?;
            wallet.multisig_state = MultisigState::Ready {
                address: result.address.clone(),
            };
        }

        // Persist state: Exchanging phase (round 1)
        let mut infos_map = HashMap::new();
        for (idx, info) in info_from_all.iter().enumerate() {
            infos_map.insert(format!("participant_{}", idx), info.multisig_info.clone());
        }
        let phase = MultisigPhase::Exchanging {
            round: 1,
            infos: infos_map,
        };
        self.persist_multisig_state(&escrow_id_str, phase).await?;

        info!(escrow_id = %escrow_id, "Multisig info exchange completed and persisted");
        Ok(())
    }

    pub async fn finalize_multisig(
        &mut self,
        escrow_id: Uuid,
    ) -> Result<String, WalletManagerError> {
        let escrow_id_str = escrow_id.to_string();
        info!("Finalizing multisig for escrow {}", escrow_id);

        let mut addresses = std::collections::HashSet::new();
        for wallet in self.wallets.values() {
            if let MultisigState::Ready { address } = &wallet.multisig_state {
                addresses.insert(address.clone());
            }
        }

        if addresses.len() != 1 {
            return Err(WalletManagerError::AddressMismatch {
                addresses: addresses.into_iter().collect(),
            });
        }

        let multisig_address = addresses
            .into_iter()
            .next()
            .ok_or(WalletManagerError::InvalidState {
                expected: "at least one wallet in Ready state".to_string(),
                actual: "none".to_string(),
            })?;

        // Persist state: Ready phase
        let phase = MultisigPhase::Ready {
            address: multisig_address.clone(),
            finalized_at: chrono::Utc::now().timestamp(),
        };
        self.persist_multisig_state(&escrow_id_str, phase).await?;

        info!(escrow_id = %escrow_id, address = %multisig_address, "Multisig finalized and persisted");
        Ok(multisig_address)
    }

    /// Release funds from escrow to vendor (requires 2-of-3 signatures)
    ///
    /// This implements the production multisig transaction flow:
    /// 1. Create unsigned transaction with buyer wallet
    /// 2. Sign with buyer wallet
    /// 3. Sign with arbiter wallet (2nd signature)
    /// 4. Submit fully-signed transaction to network
    ///
    /// # Arguments
    /// * `escrow_id` - Escrow identifier for tracking
    /// * `destinations` - List of destination addresses and amounts
    ///
    /// # Returns
    /// Transaction hash once successfully broadcast
    ///
    /// # Errors
    /// - WalletNotFound - Required wallet not found
    /// - InvalidState - Wallet not in Ready multisig state
    /// - RpcError - Monero RPC error during transaction creation/signing/submission
    pub async fn release_funds(
        &mut self,
        escrow_id: Uuid,
        destinations: Vec<monero_marketplace_common::types::TransferDestination>,
    ) -> Result<String, WalletManagerError> {
        info!("release_funds called for escrow {}", escrow_id);

        // 1. Find buyer and arbiter wallets for this escrow
        let (buyer_id, arbiter_id) =
            self.find_wallets_for_escrow(WalletRole::Buyer, WalletRole::Arbiter)?;

        // 2. Validate both wallets are in Ready state
        self.validate_wallet_ready(buyer_id)?;
        self.validate_wallet_ready(arbiter_id)?;

        // 3. Create unsigned transaction using buyer wallet
        info!("Creating unsigned transaction with buyer wallet");
        let buyer_wallet = self
            .wallets
            .get(&buyer_id)
            .ok_or(WalletManagerError::WalletNotFound(buyer_id))?;

        let create_result = buyer_wallet
            .rpc_client
            .rpc()
            .transfer_multisig(destinations.clone())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        info!(
            "Transaction created: hash={}, fee={} atomic units",
            create_result.tx_hash, create_result.fee
        );

        // 4. Sign with buyer wallet (1st signature)
        info!("Signing transaction with buyer wallet (1/2)");
        let buyer_wallet = self
            .wallets
            .get(&buyer_id)
            .ok_or(WalletManagerError::WalletNotFound(buyer_id))?;

        let buyer_signed = buyer_wallet
            .rpc_client
            .rpc()
            .sign_multisig(create_result.multisig_txset.clone())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        // 5. Sign with arbiter wallet (2nd signature - completes 2-of-3)
        info!("Signing transaction with arbiter wallet (2/2)");
        let arbiter_wallet = self
            .wallets
            .get(&arbiter_id)
            .ok_or(WalletManagerError::WalletNotFound(arbiter_id))?;

        let arbiter_signed = arbiter_wallet
            .rpc_client
            .rpc()
            .sign_multisig(buyer_signed.tx_data_hex.clone())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        // 6. Submit fully-signed transaction to network
        info!("Submitting fully-signed transaction to network");
        let buyer_wallet = self
            .wallets
            .get(&buyer_id)
            .ok_or(WalletManagerError::WalletNotFound(buyer_id))?;

        let submit_result = buyer_wallet
            .rpc_client
            .rpc()
            .submit_multisig(arbiter_signed.tx_data_hex)
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        let tx_hash = submit_result
            .tx_hash_list
            .first()
            .ok_or_else(|| WalletManagerError::InvalidState {
                expected: "at least one tx_hash".to_string(),
                actual: "empty tx_hash_list".to_string(),
            })?
            .clone();

        info!(
            "Transaction successfully broadcast: tx_hash={}, escrow={}",
            tx_hash, escrow_id
        );

        Ok(tx_hash)
    }

    /// Refund funds from escrow to buyer (requires 2-of-3 signatures)
    ///
    /// Similar to release_funds but returns funds to buyer instead of vendor.
    /// Used when vendor cannot fulfill order or buyer disputes are upheld.
    ///
    /// # Arguments
    /// * `escrow_id` - Escrow identifier for tracking
    /// * `destinations` - List of destination addresses and amounts (typically buyer's refund address)
    ///
    /// # Returns
    /// Transaction hash once successfully broadcast
    ///
    /// # Errors
    /// - WalletNotFound - Required wallet not found
    /// - InvalidState - Wallet not in Ready multisig state
    /// - RpcError - Monero RPC error during transaction creation/signing/submission
    pub async fn refund_funds(
        &mut self,
        escrow_id: Uuid,
        destinations: Vec<monero_marketplace_common::types::TransferDestination>,
    ) -> Result<String, WalletManagerError> {
        info!("refund_funds called for escrow {}", escrow_id);

        // For refunds, we use vendor and arbiter signatures (buyer doesn't need to approve their own refund)
        // This allows arbiter to force refund even if buyer is unresponsive
        let (vendor_id, arbiter_id) =
            self.find_wallets_for_escrow(WalletRole::Vendor, WalletRole::Arbiter)?;

        // Validate both wallets are in Ready state
        self.validate_wallet_ready(vendor_id)?;
        self.validate_wallet_ready(arbiter_id)?;

        // Create unsigned transaction using vendor wallet
        info!("Creating unsigned refund transaction with vendor wallet");
        let vendor_wallet = self
            .wallets
            .get(&vendor_id)
            .ok_or(WalletManagerError::WalletNotFound(vendor_id))?;

        let create_result = vendor_wallet
            .rpc_client
            .rpc()
            .transfer_multisig(destinations.clone())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        info!(
            "Refund transaction created: hash={}, fee={} atomic units",
            create_result.tx_hash, create_result.fee
        );

        // Sign with vendor wallet (1st signature)
        info!("Signing refund transaction with vendor wallet (1/2)");
        let vendor_wallet = self
            .wallets
            .get(&vendor_id)
            .ok_or(WalletManagerError::WalletNotFound(vendor_id))?;

        let vendor_signed = vendor_wallet
            .rpc_client
            .rpc()
            .sign_multisig(create_result.multisig_txset.clone())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        // Sign with arbiter wallet (2nd signature - completes 2-of-3)
        info!("Signing refund transaction with arbiter wallet (2/2)");
        let arbiter_wallet = self
            .wallets
            .get(&arbiter_id)
            .ok_or(WalletManagerError::WalletNotFound(arbiter_id))?;

        let arbiter_signed = arbiter_wallet
            .rpc_client
            .rpc()
            .sign_multisig(vendor_signed.tx_data_hex.clone())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        // Submit fully-signed transaction to network
        info!("Submitting fully-signed refund transaction to network");
        let vendor_wallet = self
            .wallets
            .get(&vendor_id)
            .ok_or(WalletManagerError::WalletNotFound(vendor_id))?;

        let submit_result = vendor_wallet
            .rpc_client
            .rpc()
            .submit_multisig(arbiter_signed.tx_data_hex)
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        let tx_hash = submit_result
            .tx_hash_list
            .first()
            .ok_or_else(|| WalletManagerError::InvalidState {
                expected: "at least one tx_hash".to_string(),
                actual: "empty tx_hash_list".to_string(),
            })?
            .clone();

        info!(
            "Refund transaction successfully broadcast: tx_hash={}, escrow={}",
            tx_hash, escrow_id
        );

        Ok(tx_hash)
    }

    /// Find two wallets by their roles
    fn find_wallets_for_escrow(
        &self,
        role1: WalletRole,
        role2: WalletRole,
    ) -> Result<(Uuid, Uuid), WalletManagerError> {
        let wallet1 = self
            .wallets
            .iter()
            .find(|(_, w)| w.role == role1)
            .map(|(id, _)| *id)
            .ok_or_else(|| WalletManagerError::InvalidState {
                expected: format!("{:?} wallet", role1),
                actual: "not found".to_string(),
            })?;

        let wallet2 = self
            .wallets
            .iter()
            .find(|(_, w)| w.role == role2)
            .map(|(id, _)| *id)
            .ok_or_else(|| WalletManagerError::InvalidState {
                expected: format!("{:?} wallet", role2),
                actual: "not found".to_string(),
            })?;

        Ok((wallet1, wallet2))
    }

    /// Validate that a wallet is in Ready multisig state
    fn validate_wallet_ready(&self, wallet_id: Uuid) -> Result<(), WalletManagerError> {
        let wallet = self
            .wallets
            .get(&wallet_id)
            .ok_or(WalletManagerError::WalletNotFound(wallet_id))?;

        match &wallet.multisig_state {
            MultisigState::Ready { .. } => Ok(()),
            state => Err(WalletManagerError::InvalidState {
                expected: "Ready".to_string(),
                actual: format!("{:?}", state),
            }),
        }
    }

    /// Get wallet balance (total and unlocked) for any wallet
    ///
    /// # Arguments
    /// * `wallet_id` - The wallet to query
    ///
    /// # Returns
    /// Tuple of (total_balance, unlocked_balance) in atomic units
    ///
    /// # Errors
    /// - WalletNotFound - Wallet ID not found
    /// - RpcError - Monero RPC error during balance query
    pub async fn get_balance(&self, wallet_id: Uuid) -> Result<(u64, u64), WalletManagerError> {
        let wallet = self
            .wallets
            .get(&wallet_id)
            .ok_or(WalletManagerError::WalletNotFound(wallet_id))?;

        let (total, unlocked) = wallet
            .rpc_client
            .rpc()
            .get_balance()
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        info!(
            "Wallet {} balance: total={} unlocked={} atomic units",
            wallet_id, total, unlocked
        );

        Ok((total, unlocked))
    }

    /// Get transaction details by transaction hash
    ///
    /// # Arguments
    /// * `wallet_id` - Wallet that owns/knows about this transaction
    /// * `tx_hash` - Transaction hash to query
    ///
    /// # Returns
    /// Transaction details including confirmations, amount, block height
    ///
    /// # Errors
    /// - WalletNotFound - Wallet ID not found
    /// - RpcError - Monero RPC error (transaction not found, RPC unreachable, etc.)
    pub async fn get_transfer_by_txid(
        &self,
        wallet_id: Uuid,
        tx_hash: &str,
    ) -> Result<TransferInfo, WalletManagerError> {
        let wallet = self
            .wallets
            .get(&wallet_id)
            .ok_or(WalletManagerError::WalletNotFound(wallet_id))?;

        let transfer = wallet
            .rpc_client
            .rpc()
            .get_transfer_by_txid(tx_hash.to_string())
            .await
            .map_err(|e| WalletManagerError::RpcError(convert_monero_error(e)))?;

        info!(
            "Transaction {} details: confirmations={}, amount={} atomic units",
            &tx_hash[..10],
            transfer.confirmations,
            transfer.amount
        );

        Ok(TransferInfo {
            tx_hash: transfer.tx_hash,
            confirmations: transfer.confirmations as u32,
            amount: transfer.amount,
            block_height: Some(transfer.block_height),
        })
    }

    /// Recover active escrows from database after server restart
    ///
    /// Queries the repository for all active escrows and reconstructs
    /// WalletInstance objects in memory. Uses Log + Continue policy
    /// so individual escrow failures don't prevent server startup.
    ///
    /// # Returns
    /// Vec of successfully recovered escrow IDs
    ///
    /// # Errors
    /// Only fatal errors that prevent any recovery. Individual escrow
    /// failures are logged but don't cause method to fail.
    pub async fn recover_active_escrows(&mut self) -> Result<Vec<String>, WalletManagerError> {
        // Skip if persistence not enabled
        let Some(ref repo) = self.multisig_repo else {
            debug!("Multisig persistence not enabled, skipping recovery");
            return Ok(vec![]);
        };

        info!("Starting multisig wallet recovery from database");

        // Query all active escrows
        let snapshots = repo
            .find_active_escrows()
            .map_err(|e| {
                error!(error = %e, "Failed to query active escrows from database");
                WalletManagerError::RpcError(CommonError::MoneroRpc(
                    format!("Recovery query failed: {}", e),
                ))
            })?;

        if snapshots.is_empty() {
            info!("No active escrows found to recover");
            return Ok(vec![]);
        }

        info!("Found {} active escrows to recover", snapshots.len());

        let mut recovered_escrow_ids = Vec::new();

        // Recover each escrow (Log + Continue policy)
        for (escrow_id, snapshot) in &snapshots {
            match self.recover_single_escrow(escrow_id, snapshot).await {
                Ok(_) => {
                    info!(escrow_id, "✅ Escrow recovered successfully");
                    recovered_escrow_ids.push(escrow_id.clone());
                }
                Err(e) => {
                    // Log but don't fail - continue with other escrows
                    warn!(
                        escrow_id,
                        error = %e,
                        "Failed to recover escrow, skipping (Log + Continue policy)"
                    );
                }
            }
        }

        info!(
            "Recovery complete: {}/{} escrows recovered successfully",
            recovered_escrow_ids.len(),
            snapshots.len()
        );

        Ok(recovered_escrow_ids)
    }

    /// Recover a single escrow from snapshot
    ///
    /// Helper method for recover_active_escrows(). Reconstructs wallet
    /// instances for buyer/vendor/arbiter based on persisted state.
    ///
    /// # Arguments
    /// * `escrow_id` - Escrow identifier
    /// * `snapshot` - Persisted multisig snapshot
    ///
    /// # Returns
    /// Ok(()) if all wallets reconstructed successfully
    ///
    /// # Errors
    /// Returns error if wallet reconstruction fails (missing RPC URL, connection failed, etc.)
    async fn recover_single_escrow(
        &mut self,
        escrow_id: &str,
        snapshot: &MultisigSnapshot,
    ) -> Result<(), WalletManagerError> {
        debug!(escrow_id, phase = ?snapshot.phase, "Recovering escrow");

        // Reconstruct wallet instances for each role
        for (role_str, wallet_uuid) in &snapshot.wallet_ids {
            let role = match role_str.as_str() {
                "buyer" => WalletRole::Buyer,
                "vendor" => WalletRole::Vendor,
                "arbiter" => WalletRole::Arbiter,
                _ => {
                    warn!(escrow_id, role = role_str, "Unknown role, skipping");
                    continue;
                }
            };

            // Get RPC URL from snapshot
            let rpc_url = snapshot
                .rpc_urls
                .get(role_str)
                .ok_or_else(|| {
                    WalletManagerError::InvalidState {
                        expected: format!("RPC URL for {}", role_str),
                        actual: "not found in snapshot".to_string(),
                    }
                })?;

            // Reconstruct MoneroClient connection
            let config = MoneroConfig {
                rpc_url: format!("http://{}/json_rpc", rpc_url),
                rpc_user: None, // TODO: Store auth in snapshot if needed
                rpc_password: None,
                timeout_seconds: 30,
            };

            let rpc_client = MoneroClient::new(config)
                .map_err(|e| {
                    WalletManagerError::RpcError(CommonError::MoneroRpc(
                        format!("Failed to reconnect to {} wallet RPC: {}", role_str, e),
                    ))
                })?;

            // Get current wallet address
            let wallet_info = rpc_client
                .get_wallet_info()
                .await
                .map_err(|e| {
                    WalletManagerError::RpcError(CommonError::MoneroRpc(
                        format!("Failed to get wallet info during recovery: {}", e),
                    ))
                })?;

            // Determine multisig state from phase
            let multisig_state = match &snapshot.phase {
                MultisigPhase::NotStarted => MultisigState::NotStarted,
                MultisigPhase::Preparing { .. } => MultisigState::NotStarted,
                MultisigPhase::Exchanging { round, .. } => MultisigState::InfoExchanged {
                    round: *round,
                    participants: vec![], // TODO: Reconstruct from snapshot if needed
                },
                MultisigPhase::Ready { address, .. } => MultisigState::Ready {
                    address: address.clone(),
                },
                MultisigPhase::Failed { .. } => {
                    warn!(escrow_id, "Escrow in Failed state, not recovering");
                    return Err(WalletManagerError::InvalidState {
                        expected: "Active phase".to_string(),
                        actual: "Failed".to_string(),
                    });
                }
            };

            // Reconstruct WalletInstance
            let instance = WalletInstance {
                id: *wallet_uuid,
                role: role.clone(),
                rpc_client,
                address: wallet_info.address.clone(),
                multisig_state,
            };

            // Insert into in-memory wallet map
            self.wallets.insert(*wallet_uuid, instance);

            debug!(
                escrow_id,
                role = ?role,
                wallet_id = %wallet_uuid,
                address = %wallet_info.address,
                "Wallet instance reconstructed"
            );
        }

        Ok(())
    }
}

/// Transfer information returned from blockchain queries
#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub tx_hash: String,
    pub confirmations: u32,
    pub amount: u64,
    pub block_height: Option<u64>,
}

/// Convert MoneroError to CommonError
fn convert_monero_error(e: MoneroError) -> CommonError {
    match e {
        MoneroError::RpcUnreachable => CommonError::MoneroRpc("RPC unreachable".to_string()),
        MoneroError::AlreadyMultisig => {
            CommonError::Multisig("Already in multisig mode".to_string())
        }
        MoneroError::NotMultisig => CommonError::Multisig("Not in multisig mode".to_string()),
        MoneroError::WalletLocked => CommonError::Wallet("Wallet locked".to_string()),
        MoneroError::WalletBusy => CommonError::Wallet("Wallet busy".to_string()),
        MoneroError::ValidationError(msg) => CommonError::InvalidInput(msg),
        MoneroError::InvalidResponse(msg) => {
            CommonError::MoneroRpc(format!("Invalid response: {}", msg))
        }
        MoneroError::NetworkError(msg) => CommonError::Internal(format!("Network error: {}", msg)),
        MoneroError::RpcError(msg) => CommonError::MoneroRpc(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test convert_monero_error covers all variants
    #[test]
    fn test_convert_monero_error_all_variants() {
        // RpcUnreachable
        let err = convert_monero_error(MoneroError::RpcUnreachable);
        assert!(matches!(err, CommonError::MoneroRpc(_)));

        // AlreadyMultisig
        let err = convert_monero_error(MoneroError::AlreadyMultisig);
        assert!(matches!(err, CommonError::Multisig(_)));

        // NotMultisig
        let err = convert_monero_error(MoneroError::NotMultisig);
        assert!(matches!(err, CommonError::Multisig(_)));

        // WalletLocked
        let err = convert_monero_error(MoneroError::WalletLocked);
        assert!(matches!(err, CommonError::Wallet(_)));

        // WalletBusy
        let err = convert_monero_error(MoneroError::WalletBusy);
        assert!(matches!(err, CommonError::Wallet(_)));

        // ValidationError
        let err = convert_monero_error(MoneroError::ValidationError("test error".to_string()));
        assert!(matches!(err, CommonError::InvalidInput(_)));

        // InvalidResponse
        let err = convert_monero_error(MoneroError::InvalidResponse("bad response".to_string()));
        assert!(matches!(err, CommonError::MoneroRpc(_)));

        // NetworkError
        let err = convert_monero_error(MoneroError::NetworkError("connection failed".to_string()));
        assert!(matches!(err, CommonError::Internal(_)));

        // RpcError
        let err = convert_monero_error(MoneroError::RpcError("rpc failed".to_string()));
        assert!(matches!(err, CommonError::MoneroRpc(_)));
    }

    /// Test WalletManager creation
    #[test]
    fn test_wallet_manager_new() {
        let config = MoneroConfig::default();
        let manager = WalletManager::new(vec![config]);
        assert!(manager.is_ok());
    }

    /// Test WalletManager rejects empty config list
    #[test]
    fn test_wallet_manager_new_empty_configs() {
        let result = WalletManager::new(vec![]);
        assert!(result.is_err());
    }

    /// Test MultisigState equality
    #[test]
    fn test_multisig_state_equality() {
        let state1 = MultisigState::NotStarted;
        let state2 = MultisigState::NotStarted;
        assert_eq!(state1, state2);

        let ready1 = MultisigState::Ready {
            address: "test_address".to_string(),
        };
        let ready2 = MultisigState::Ready {
            address: "test_address".to_string(),
        };
        assert_eq!(ready1, ready2);
    }

    /// Test WalletRole equality
    #[test]
    fn test_wallet_role_equality() {
        assert_eq!(WalletRole::Buyer, WalletRole::Buyer);
        assert_eq!(WalletRole::Vendor, WalletRole::Vendor);
        assert_eq!(WalletRole::Arbiter, WalletRole::Arbiter);
        assert_ne!(WalletRole::Buyer, WalletRole::Vendor);
    }

    /// Test WalletManagerError display messages
    #[test]
    fn test_wallet_manager_error_display() {
        let wallet_id = Uuid::new_v4();
        let err = WalletManagerError::WalletNotFound(wallet_id);
        let msg = format!("{}", err);
        assert!(msg.contains("Wallet not found"));

        let err = WalletManagerError::NoAvailableRpc;
        let msg = format!("{}", err);
        assert!(msg.contains("All RPC endpoints unavailable"));

        let err = WalletManagerError::InvalidState {
            expected: "Ready".to_string(),
            actual: "NotStarted".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid multisig state"));
        assert!(msg.contains("Ready"));
        assert!(msg.contains("NotStarted"));
    }

    /// Test RPC config round-robin
    #[test]
    fn test_rpc_round_robin() {
        let config1 = MoneroConfig {
            rpc_url: "http://127.0.0.1:18081".to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 60,
        };
        let config2 = MoneroConfig {
            rpc_url: "http://127.0.0.1:18082".to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 60,
        };

        let manager =
            WalletManager::new(vec![config1, config2]).expect("Failed to create WalletManager");

        assert_eq!(manager.next_rpc_index, 0);
        assert_eq!(manager.rpc_configs.len(), 2);
    }
}
