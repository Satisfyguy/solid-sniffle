//! Non-custodial wallet client for Phase 2 migration
//!
//! This module implements the client-side flow for non-custodial escrow:
//! 1. Client creates local wallet (NOT on server)
//! 2. Client runs local monero-wallet-rpc
//! 3. Client registers RPC URL with server coordinator
//! 4. Client participates in coordinated multisig setup
//! 5. Client finalizes multisig locally (server never touches keys)
//!
//! **Architecture:**
//! ```
//! Client (local wallet-rpc) → Server (coordinator only) ← Other clients
//! ```

use anyhow::{Context, Result};
use monero_marketplace_common::types::MoneroConfig;
use monero_marketplace_wallet::{multisig::MultisigManager, MoneroClient};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

// ============================================================================
// REQUEST/RESPONSE TYPES (match server/src/handlers/noncustodial.rs)
// ============================================================================

/// Request to register client wallet with coordinator
#[derive(Debug, Serialize)]
struct RegisterClientWalletRequest {
    pub escrow_id: String,
    pub role: String, // "buyer", "seller", or "arbiter"
    pub rpc_url: String,
}

/// Response from coordinator after registration
#[derive(Debug, Deserialize)]
struct RegisterClientWalletResponse {
    pub success: bool,
    pub message: String,
    pub escrow_id: String,
    pub role: String,
    pub coordination_state: String,
    pub awaiting: Vec<String>,
}

/// Request to coordinate multisig exchange
#[derive(Debug, Serialize)]
struct CoordinateExchangeRequest {
    pub escrow_id: String,
}

/// Response with exchanged multisig infos
#[derive(Debug, Deserialize)]
struct CoordinateExchangeResponse {
    pub success: bool,
    pub message: String,
    pub escrow_id: String,
    pub exchange_result: MultisigExchangeResult,
}

#[derive(Debug, Deserialize)]
struct MultisigExchangeResult {
    pub buyer_receives: Vec<String>,
    pub seller_receives: Vec<String>,
    pub arbiter_receives: Vec<String>,
}

/// Coordination status response
#[derive(Debug, Deserialize)]
struct GetCoordinationStatusResponse {
    pub success: bool,
    pub escrow_id: String,
    pub state: String,
    pub buyer_registered: bool,
    pub seller_registered: bool,
    pub arbiter_registered: bool,
    pub ready_for_exchange: bool,
}

// ============================================================================
// NON-CUSTODIAL CLIENT
// ============================================================================

/// Non-custodial escrow client
///
/// This client interacts with:
/// - Local monero-wallet-rpc (for actual wallet operations)
/// - Server coordinator API (for multisig info exchange only)
pub struct NonCustodialClient {
    /// Local wallet RPC client
    local_wallet: MoneroClient,
    /// HTTP client for server API calls
    http_client: HttpClient,
    /// Server coordinator URL (e.g., "http://localhost:8080")
    server_url: String,
    /// Client's role in escrow
    role: EscrowRole,
    /// Local RPC URL
    local_rpc_url: String,
}

/// Role in escrow (matches server enum)
#[derive(Debug, Clone)]
pub enum EscrowRole {
    Buyer,
    Seller,
    Arbiter,
}

impl EscrowRole {
    pub fn as_str(&self) -> &str {
        match self {
            EscrowRole::Buyer => "buyer",
            EscrowRole::Seller => "seller",
            EscrowRole::Arbiter => "arbiter",
        }
    }
}

impl NonCustodialClient {
    /// Create new non-custodial client
    ///
    /// **Parameters:**
    /// - `local_rpc_url`: Local monero-wallet-rpc URL (e.g., "http://127.0.0.1:18083")
    /// - `server_url`: Server coordinator URL (e.g., "http://localhost:8080")
    /// - `role`: Client's role in escrow
    pub fn new(local_rpc_url: String, server_url: String, role: EscrowRole) -> Result<Self> {
        let config = MoneroConfig {
            rpc_url: local_rpc_url.clone(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 30,
        };

        let local_wallet = MoneroClient::new(config)
            .context("Failed to create local wallet client")?;

        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            local_wallet,
            http_client,
            server_url,
            role,
            local_rpc_url,
        })
    }

    /// Initialize non-custodial escrow flow
    ///
    /// **Flow:**
    /// 1. Create local wallet (if not exists)
    /// 2. Prepare multisig locally
    /// 3. Register with server coordinator
    /// 4. Wait for other participants
    /// 5. Coordinate multisig info exchange
    /// 6. Finalize multisig locally
    ///
    /// **Returns:** Multisig address
    pub async fn init_escrow(&self, escrow_id: &str, wallet_name: &str) -> Result<String> {
        info!("🔐 Starting non-custodial escrow initialization for {}", self.role.as_str());
        info!("Escrow ID: {}", escrow_id);
        info!("Local wallet: {}", wallet_name);

        // Step 1: Create local wallet
        self.create_local_wallet(wallet_name).await?;

        // Step 2: Prepare multisig locally
        info!("📝 Preparing multisig locally...");
        let prepare_result = self.local_wallet
            .multisig()
            .prepare_multisig()
            .await
            .context("Failed to prepare multisig")?;

        info!("✅ Local multisig prepared");
        info!("Multisig info length: {} chars", prepare_result.multisig_info.len());

        // Step 3: Register with server coordinator
        info!("📡 Registering with server coordinator...");
        self.register_with_coordinator(escrow_id).await?;

        // Step 4: Wait for other participants
        info!("⏳ Waiting for other participants to register...");
        self.wait_for_all_participants(escrow_id).await?;

        // Step 5: Coordinate multisig exchange
        info!("🔄 Coordinating multisig info exchange...");
        let infos_to_use = self.coordinate_exchange(escrow_id).await?;

        info!("✅ Received {} multisig infos from coordinator", infos_to_use.len());

        // Step 6: Finalize multisig locally
        info!("🔧 Finalizing multisig locally (make_multisig with threshold=2)...");
        let make_result = self.local_wallet
            .multisig()
            .make_multisig(2, infos_to_use)
            .await
            .context("Failed to make multisig")?;

        info!("✅ Multisig wallet created locally!");
        info!("Multisig address: {}", make_result.address);

        // Step 7: Export for sync round 1
        info!("📤 Exporting multisig info for sync round 1...");
        let export_result = self.local_wallet
            .multisig()
            .export_multisig_info()
            .await
            .context("Failed to export multisig info")?;

        info!("✅ Export successful, info length: {} chars", export_result.info.len());
        info!("ℹ️  Next steps:");
        info!("  1. Share export info with other participants");
        info!("  2. Import their export infos (sync round 1)");
        info!("  3. Repeat export/import (sync round 2)");
        info!("  4. Wallet will be ready for transactions");

        Ok(make_result.address)
    }

    /// Create local wallet (skip if exists)
    async fn create_local_wallet(&self, wallet_name: &str) -> Result<()> {
        info!("📁 Creating local wallet '{}'...", wallet_name);

        // Try to create wallet (will fail if exists, which is ok)
        match self.local_wallet.rpc().create_wallet(wallet_name, "").await {
            Ok(_) => {
                info!("✅ Wallet '{}' created", wallet_name);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("already exists") || error_msg.contains("Cannot create wallet") {
                    warn!("Wallet '{}' already exists, will use existing", wallet_name);
                    Ok(())
                } else {
                    Err(e).context("Failed to create wallet")
                }
            }
        }
    }

    /// Register local wallet RPC URL with server coordinator
    async fn register_with_coordinator(&self, escrow_id: &str) -> Result<()> {
        let url = format!("{}/api/v2/escrow/register-wallet", self.server_url);

        let request = RegisterClientWalletRequest {
            escrow_id: escrow_id.to_string(),
            role: self.role.as_str().to_string(),
            rpc_url: self.local_rpc_url.clone(),
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send registration request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Registration failed: {}", error_text));
        }

        let reg_response: RegisterClientWalletResponse = response
            .json()
            .await
            .context("Failed to parse registration response")?;

        if !reg_response.success {
            return Err(anyhow::anyhow!("Registration failed: {}", reg_response.message));
        }

        info!("✅ Registered as {} for escrow {}", self.role.as_str(), escrow_id);
        info!("State: {}", reg_response.coordination_state);
        if !reg_response.awaiting.is_empty() {
            info!("Waiting for: {:?}", reg_response.awaiting);
        }

        Ok(())
    }

    /// Wait for all participants to register
    async fn wait_for_all_participants(&self, escrow_id: &str) -> Result<()> {
        let url = format!("{}/api/v2/escrow/coordination-status/{}", self.server_url, escrow_id);
        let max_attempts = 60; // 60 attempts * 2s = 2 minutes max
        let mut attempts = 0;

        loop {
            attempts += 1;
            if attempts > max_attempts {
                return Err(anyhow::anyhow!("Timeout waiting for participants after {} attempts", max_attempts));
            }

            let response = self.http_client
                .get(&url)
                .send()
                .await
                .context("Failed to get coordination status")?;

            if !response.status().is_success() {
                warn!("Status check failed (attempt {}), retrying...", attempts);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }

            let status: GetCoordinationStatusResponse = response
                .json()
                .await
                .context("Failed to parse status response")?;

            if status.buyer_registered && status.seller_registered && status.arbiter_registered {
                info!("✅ All participants registered!");
                return Ok(());
            }

            let missing: Vec<&str> = vec![
                (!status.buyer_registered).then_some("buyer"),
                (!status.seller_registered).then_some("seller"),
                (!status.arbiter_registered).then_some("arbiter"),
            ]
            .into_iter()
            .flatten()
            .collect();

            info!("Waiting for participants: {:?} (attempt {}/{})", missing, attempts, max_attempts);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    /// Coordinate multisig info exchange through server
    ///
    /// Server will:
    /// 1. Call prepare_multisig on all 3 wallets
    /// 2. Collect all multisig_info strings
    /// 3. Return the appropriate infos for this role
    async fn coordinate_exchange(&self, escrow_id: &str) -> Result<Vec<String>> {
        let url = format!("{}/api/v2/escrow/coordinate-exchange", self.server_url);

        let request = CoordinateExchangeRequest {
            escrow_id: escrow_id.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send coordinate request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Coordination failed: {}", error_text));
        }

        let coord_response: CoordinateExchangeResponse = response
            .json()
            .await
            .context("Failed to parse coordination response")?;

        if !coord_response.success {
            return Err(anyhow::anyhow!("Coordination failed: {}", coord_response.message));
        }

        // Extract the multisig infos for our role
        let infos = match self.role {
            EscrowRole::Buyer => coord_response.exchange_result.buyer_receives,
            EscrowRole::Seller => coord_response.exchange_result.seller_receives,
            EscrowRole::Arbiter => coord_response.exchange_result.arbiter_receives,
        };

        info!("✅ Coordination successful");
        info!("Received {} multisig infos from other participants", infos.len());

        if infos.len() != 2 {
            error!("❌ Expected 2 multisig infos but got {}", infos.len());
            return Err(anyhow::anyhow!("Invalid number of multisig infos: expected 2, got {}", infos.len()));
        }

        Ok(infos)
    }

    /// Get local wallet info for debugging
    pub async fn get_wallet_info(&self) -> Result<()> {
        info!("Getting local wallet information...");

        let wallet_info = self.local_wallet.get_wallet_info().await?;

        info!("📊 Wallet Information:");
        info!("  Multisig: {}", wallet_info.is_multisig);
        if let Some(threshold) = wallet_info.multisig_threshold {
            if let Some(total) = wallet_info.multisig_total {
                info!("  Threshold: {}/{}", threshold, total);
            }
        }
        info!("  Balance: {} XMR", wallet_info.balance as f64 / 1e12);
        info!("  Block Height: {}", wallet_info.block_height);

        Ok(())
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Parse role string to EscrowRole enum
pub fn parse_role(role_str: &str) -> Result<EscrowRole> {
    match role_str.to_lowercase().as_str() {
        "buyer" => Ok(EscrowRole::Buyer),
        "seller" => Ok(EscrowRole::Seller),
        "arbiter" => Ok(EscrowRole::Arbiter),
        _ => Err(anyhow::anyhow!("Invalid role: must be 'buyer', 'seller', or 'arbiter'")),
    }
}
