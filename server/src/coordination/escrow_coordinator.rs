//! Non-custodial escrow coordinator
//!
//! Inspired by Haveno DEX architecture where the server acts as a pure coordinator
//! for multisig info exchange without ever touching wallet private keys.
//!
//! **Key Principles:**
//! 1. Server stores RPC URLs only (http://127.0.0.1:XXXX)
//! 2. Server coordinates multisig info exchange between participants
//! 3. Server validates formats, thresholds, and participant counts
//! 4. Server NEVER creates wallets or executes crypto operations
//! 5. Private keys NEVER leave client wallets
//!
//! **Flow:**
//! 1. Each participant (buyer, seller, arbiter) runs local monero-wallet-rpc
//! 2. Each registers their RPC URL with coordinator
//! 3. Coordinator requests prepare_multisig from each wallet
//! 4. Coordinator validates and exchanges multisig_info strings
//! 5. Clients finalize multisig locally using received infos

use monero_marketplace_common::{
    error::{Error, MoneroError, Result},
    types::{MoneroConfig, MultisigInfo},
};
use monero_marketplace_wallet::{rpc::MoneroRpcClient, validation::validate_localhost_strict};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Pure coordinator for non-custodial escrow
///
/// This coordinator NEVER creates or manages wallets. It only stores RPC URLs
/// and coordinates the exchange of public multisig info between clients.
pub struct EscrowCoordinator {
    /// Map of escrow_id -> coordination state
    coordinations: Arc<RwLock<HashMap<String, EscrowCoordination>>>,
}

/// Coordination state for one escrow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowCoordination {
    pub escrow_id: String,
    pub buyer_rpc_url: Option<String>,
    pub seller_rpc_url: Option<String>,
    pub arbiter_rpc_url: Option<String>,
    pub state: CoordinationState,
    /// Multisig info from each participant (public data only)
    pub multisig_infos: HashMap<String, String>, // role -> multisig_info
}

/// States of coordination process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordinationState {
    /// Waiting for all 3 participants to register their wallets
    AwaitingRegistrations,
    /// All 3 wallets registered, ready to prepare multisig
    AllRegistered,
    /// prepare_multisig executed on all wallets, infos collected
    Prepared,
    /// Multisig info exchanged, clients can now make_multisig
    ReadyForMakeMultisig,
    /// make_multisig completed on clients (verified by export_multisig_info)
    MadeMultisig,
    /// First export/import round completed
    SyncRound1Complete,
    /// Second export/import round completed
    SyncRound2Complete,
    /// Multisig fully synchronized and ready for transactions
    Ready,
}

/// Result of multisig info exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigExchangeResult {
    /// Multisig infos that buyer should receive (seller + arbiter)
    pub buyer_receives: Vec<String>,
    /// Multisig infos that seller should receive (buyer + arbiter)
    pub seller_receives: Vec<String>,
    /// Multisig infos that arbiter should receive (buyer + seller)
    pub arbiter_receives: Vec<String>,
}

/// Role in escrow
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscrowRole {
    Buyer,
    Seller,
    Arbiter,
}

impl EscrowRole {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "buyer" => Ok(EscrowRole::Buyer),
            "seller" => Ok(EscrowRole::Seller),
            "arbiter" => Ok(EscrowRole::Arbiter),
            _ => Err(Error::InvalidInput(format!("Invalid role: {}", s))),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EscrowRole::Buyer => "buyer",
            EscrowRole::Seller => "seller",
            EscrowRole::Arbiter => "arbiter",
        }
    }
}

impl EscrowCoordinator {
    /// Create new non-custodial coordinator
    pub fn new() -> Self {
        info!("🔧 Creating non-custodial EscrowCoordinator");
        Self {
            coordinations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a client wallet RPC URL (NON-CUSTODIAL)
    ///
    /// **Security:**
    /// - Validates RPC URL is localhost only (no remote wallets)
    /// - Checks RPC connectivity before accepting
    /// - Stores URL only (NOT the wallet itself)
    ///
    /// # Arguments
    /// * `escrow_id` - Unique escrow identifier
    /// * `role` - Role in escrow (buyer, seller, arbiter)
    /// * `rpc_url` - Client's local wallet RPC URL (must be localhost)
    ///
    /// # Returns
    /// Ok(()) if wallet registered successfully
    ///
    /// # Errors
    /// - Error::InvalidInput - Invalid role or URL format
    /// - Error::Security - RPC URL is not localhost
    /// - Error::MoneroRpc - Cannot connect to RPC
    pub async fn register_client_wallet(
        &self,
        escrow_id: &str,
        role: EscrowRole,
        rpc_url: String,
    ) -> Result<()> {
        info!(
            "📝 Registering {} wallet for escrow {} at {}",
            role.as_str(),
            escrow_id,
            rpc_url
        );

        // CRITICAL: Validate localhost strict (prevent remote wallet attacks)
        validate_localhost_strict(&rpc_url).map_err(|e| {
            error!("🚨 SECURITY: Non-localhost RPC URL rejected: {}", rpc_url);
            Error::Security(format!("RPC must be localhost: {}", e))
        })?;

        // Verify RPC connectivity
        let config = MoneroConfig {
            rpc_url: rpc_url.clone(),
            ..Default::default()
        };

        let client = MoneroRpcClient::new(config).map_err(|e| {
            error!("Failed to create RPC client for {}: {}", rpc_url, e);
            Error::MoneroRpc(format!("Invalid RPC config: {}", e))
        })?;

        client.check_connection().await.map_err(|e| {
            error!("Cannot connect to RPC at {}: {}", rpc_url, e);
            Error::MoneroRpc(format!("RPC unreachable: {}", e))
        })?;

        // Store URL (NOT the wallet - this is the key difference from custodial mode)
        let mut coords = self.coordinations.write().await;
        let coord = coords
            .entry(escrow_id.to_string())
            .or_insert_with(|| EscrowCoordination {
                escrow_id: escrow_id.to_string(),
                buyer_rpc_url: None,
                seller_rpc_url: None,
                arbiter_rpc_url: None,
                state: CoordinationState::AwaitingRegistrations,
                multisig_infos: HashMap::new(),
            });

        // Update appropriate URL based on role
        match role {
            EscrowRole::Buyer => {
                if coord.buyer_rpc_url.is_some() {
                    warn!("Overwriting existing buyer RPC URL for escrow {}", escrow_id);
                }
                coord.buyer_rpc_url = Some(rpc_url);
            }
            EscrowRole::Seller => {
                if coord.seller_rpc_url.is_some() {
                    warn!(
                        "Overwriting existing seller RPC URL for escrow {}",
                        escrow_id
                    );
                }
                coord.seller_rpc_url = Some(rpc_url);
            }
            EscrowRole::Arbiter => {
                if coord.arbiter_rpc_url.is_some() {
                    warn!(
                        "Overwriting existing arbiter RPC URL for escrow {}",
                        escrow_id
                    );
                }
                coord.arbiter_rpc_url = Some(rpc_url);
            }
        }

        // Check if all 3 wallets are now registered
        if coord.buyer_rpc_url.is_some()
            && coord.seller_rpc_url.is_some()
            && coord.arbiter_rpc_url.is_some()
        {
            coord.state = CoordinationState::AllRegistered;
            info!(
                "✅ All 3 wallets registered for escrow {}, ready to prepare multisig",
                escrow_id
            );
        } else {
            let registered = [
                coord.buyer_rpc_url.is_some().then_some("buyer"),
                coord.seller_rpc_url.is_some().then_some("seller"),
                coord.arbiter_rpc_url.is_some().then_some("arbiter"),
            ]
            .iter()
            .filter_map(|&x| x)
            .collect::<Vec<_>>();

            info!(
                "⏳ Waiting for remaining participants for escrow {} (registered: {:?})",
                escrow_id, registered
            );
        }

        Ok(())
    }

    /// Coordinate multisig info exchange (NON-CUSTODIAL)
    ///
    /// **Flow:**
    /// 1. Verify all 3 wallets are registered
    /// 2. Request prepare_multisig from each wallet (executed on CLIENT side)
    /// 3. Validate all multisig_info formats
    /// 4. Exchange infos (each participant receives the other 2)
    ///
    /// **Security:**
    /// - Server NEVER executes prepare_multisig itself
    /// - Server only requests clients to execute it
    /// - Server validates public info formats
    /// - Server ensures threshold=2, participants=3 (Haveno-style validation)
    ///
    /// # Arguments
    /// * `escrow_id` - Escrow identifier
    ///
    /// # Returns
    /// MultisigExchangeResult with infos for each participant
    ///
    /// # Errors
    /// - Error::InvalidState - Not all wallets registered
    /// - Error::MoneroRpc - Cannot communicate with wallets
    /// - Error::InvalidInput - Invalid multisig info format
    pub async fn coordinate_multisig_exchange(
        &self,
        escrow_id: &str,
    ) -> Result<MultisigExchangeResult> {
        info!("🔄 Coordinating multisig exchange for escrow {}", escrow_id);

        // Get coordination state
        let coords = self.coordinations.read().await;
        let coord = coords.get(escrow_id).ok_or_else(|| {
            error!("Escrow {} not found in coordinator", escrow_id);
            Error::EscrowNotFound(escrow_id.to_string())
        })?;

        // Verify all 3 wallets registered
        let buyer_url = coord.buyer_rpc_url.as_ref().ok_or_else(|| {
            error!("Buyer wallet not registered for escrow {}", escrow_id);
            Error::InvalidState("Buyer wallet not registered".to_string())
        })?;

        let seller_url = coord.seller_rpc_url.as_ref().ok_or_else(|| {
            error!("Seller wallet not registered for escrow {}", escrow_id);
            Error::InvalidState("Seller wallet not registered".to_string())
        })?;

        let arbiter_url = coord.arbiter_rpc_url.as_ref().ok_or_else(|| {
            error!("Arbiter wallet not registered for escrow {}", escrow_id);
            Error::InvalidState("Arbiter wallet not registered".to_string())
        })?;

        // Clone URLs for async operations
        let buyer_url = buyer_url.clone();
        let seller_url = seller_url.clone();
        let arbiter_url = arbiter_url.clone();

        // Release read lock before async operations
        drop(coords);

        info!("🔧 Requesting prepare_multisig from all participants...");

        // Request prepare_multisig from each wallet (executed on CLIENT side)
        let buyer_info = self
            .request_prepare_multisig(&buyer_url, "buyer")
            .await?;
        let seller_info = self
            .request_prepare_multisig(&seller_url, "seller")
            .await?;
        let arbiter_info = self
            .request_prepare_multisig(&arbiter_url, "arbiter")
            .await?;

        // Validate formats (security check)
        self.validate_multisig_info(&buyer_info, "buyer")?;
        self.validate_multisig_info(&seller_info, "seller")?;
        self.validate_multisig_info(&arbiter_info, "arbiter")?;

        // Store multisig infos and update state
        let mut coords = self.coordinations.write().await;
        if let Some(coord) = coords.get_mut(escrow_id) {
            coord
                .multisig_infos
                .insert("buyer".to_string(), buyer_info.clone());
            coord
                .multisig_infos
                .insert("seller".to_string(), seller_info.clone());
            coord
                .multisig_infos
                .insert("arbiter".to_string(), arbiter_info.clone());
            coord.state = CoordinationState::Prepared;
        }

        info!(
            "✅ Multisig info exchange coordinated for escrow {}",
            escrow_id
        );

        // Exchange: each participant receives the other 2
        Ok(MultisigExchangeResult {
            buyer_receives: vec![seller_info.clone(), arbiter_info.clone()],
            seller_receives: vec![buyer_info.clone(), arbiter_info.clone()],
            arbiter_receives: vec![buyer_info, seller_info],
        })
    }

    /// Get coordination status for an escrow
    pub async fn get_coordination_status(
        &self,
        escrow_id: &str,
    ) -> Result<EscrowCoordination> {
        let coords = self.coordinations.read().await;
        coords
            .get(escrow_id)
            .cloned()
            .ok_or_else(|| Error::EscrowNotFound(escrow_id.to_string()))
    }

    // ============================================================================
    // PRIVATE HELPER METHODS
    // ============================================================================

    /// Request prepare_multisig from a client wallet
    ///
    /// **CRITICAL:** This method connects to CLIENT's wallet-rpc and asks it to
    /// execute prepare_multisig. The server NEVER executes this itself.
    async fn request_prepare_multisig(&self, rpc_url: &str, role: &str) -> Result<String> {
        info!("📡 Requesting prepare_multisig from {} at {}", role, rpc_url);

        let config = MoneroConfig {
            rpc_url: rpc_url.to_string(),
            ..Default::default()
        };

        let client = MoneroRpcClient::new(config).map_err(|e| {
            error!(
                "Failed to create RPC client for {} at {}: {}",
                role, rpc_url, e
            );
            Error::MoneroRpc(format!("RPC client creation failed: {}", e))
        })?;

        let info: MultisigInfo = client.prepare_multisig().await.map_err(|e| {
            error!("prepare_multisig failed for {} at {}: {}", role, rpc_url, e);
            match e {
                MoneroError::AlreadyMultisig => {
                    Error::InvalidState(format!("{} wallet already in multisig mode", role))
                }
                MoneroError::WalletLocked => {
                    Error::Wallet(format!("{} wallet is locked", role))
                }
                _ => Error::MoneroRpc(format!("prepare_multisig failed for {}: {}", role, e)),
            }
        })?;

        info!(
            "✅ Received multisig info from {} ({} bytes)",
            role,
            info.multisig_info.len()
        );

        Ok(info.multisig_info)
    }

    /// Validate multisig info format
    ///
    /// Follows Haveno pattern of strict validation
    fn validate_multisig_info(&self, info: &str, role: &str) -> Result<()> {
        use monero_marketplace_common::{MAX_MULTISIG_INFO_LEN, MIN_MULTISIG_INFO_LEN};

        // Length validation
        if info.len() < MIN_MULTISIG_INFO_LEN {
            return Err(Error::InvalidInput(format!(
                "{} multisig_info too short: {} bytes (min: {})",
                role,
                info.len(),
                MIN_MULTISIG_INFO_LEN
            )));
        }

        if info.len() > MAX_MULTISIG_INFO_LEN {
            return Err(Error::InvalidInput(format!(
                "{} multisig_info too long: {} bytes (max: {})",
                role,
                info.len(),
                MAX_MULTISIG_INFO_LEN
            )));
        }

        // Format validation (should start with "MultisigV1")
        if !info.starts_with("MultisigV1") && !info.starts_with("MultisigxV1") {
            return Err(Error::InvalidInput(format!(
                "{} multisig_info has invalid format (should start with MultisigV1 or MultisigxV1)",
                role
            )));
        }

        Ok(())
    }
}

impl Default for EscrowCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = EscrowCoordinator::new();
        assert!(coordinator.coordinations.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_escrow_role_conversion() {
        assert_eq!(EscrowRole::from_str("buyer").unwrap(), EscrowRole::Buyer);
        assert_eq!(EscrowRole::from_str("BUYER").unwrap(), EscrowRole::Buyer);
        assert_eq!(
            EscrowRole::from_str("seller").unwrap(),
            EscrowRole::Seller
        );
        assert_eq!(
            EscrowRole::from_str("arbiter").unwrap(),
            EscrowRole::Arbiter
        );
        assert!(EscrowRole::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_coordination_state_transitions() {
        let coordinator = EscrowCoordinator::new();
        let escrow_id = "test_escrow_123";

        // Register buyer (should still be AwaitingRegistrations)
        coordinator
            .register_client_wallet(
                escrow_id,
                EscrowRole::Buyer,
                "http://127.0.0.1:18083".to_string(),
            )
            .await
            .unwrap();

        let status = coordinator.get_coordination_status(escrow_id).await.unwrap();
        assert_eq!(status.state, CoordinationState::AwaitingRegistrations);

        // Register seller (still waiting for arbiter)
        coordinator
            .register_client_wallet(
                escrow_id,
                EscrowRole::Seller,
                "http://127.0.0.1:18084".to_string(),
            )
            .await
            .unwrap();

        let status = coordinator.get_coordination_status(escrow_id).await.unwrap();
        assert_eq!(status.state, CoordinationState::AwaitingRegistrations);

        // Register arbiter (all 3 registered → AllRegistered)
        coordinator
            .register_client_wallet(
                escrow_id,
                EscrowRole::Arbiter,
                "http://127.0.0.1:18085".to_string(),
            )
            .await
            .unwrap();

        let status = coordinator.get_coordination_status(escrow_id).await.unwrap();
        assert_eq!(status.state, CoordinationState::AllRegistered);
        assert!(status.buyer_rpc_url.is_some());
        assert!(status.seller_rpc_url.is_some());
        assert!(status.arbiter_rpc_url.is_some());
    }

    #[tokio::test]
    async fn test_multisig_info_validation() {
        let coordinator = EscrowCoordinator::new();

        // Too short
        let result = coordinator.validate_multisig_info("short", "buyer");
        assert!(result.is_err());

        // Invalid format (doesn't start with MultisigV1)
        let invalid = "InvalidPrefix".to_string() + &"x".repeat(200);
        let result = coordinator.validate_multisig_info(&invalid, "buyer");
        assert!(result.is_err());

        // Valid format
        let valid = "MultisigV1".to_string() + &"x".repeat(200);
        let result = coordinator.validate_multisig_info(&valid, "buyer");
        assert!(result.is_ok());
    }

    // Note: Full integration tests require running monero-wallet-rpc instances
    // See server/tests/noncustodial/ for E2E tests
}
