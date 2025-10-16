//! Common types for Monero Marketplace

use crate::MONERO_RPC_URL;
use serde::{Deserialize, Serialize};

/// Monero address type
pub type MoneroAddress = String;

/// Monero transaction hash
pub type TxHash = String;

/// Monero amount in atomic units (1 XMR = 1e12 atomic units)
pub type Amount = u64;

/// Escrow ID type
pub type EscrowId = String;

/// User ID type
pub type UserId = String;

/// Monero RPC request
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Monero RPC response
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroRpcResponse<T = serde_json::Value> {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MoneroRpcError>,
}

/// Monero RPC error
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroRpcError {
    pub code: i32,
    pub message: String,
}

/// Multisig info for export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigInfo {
    pub multisig_info: String,
}

/// Wallet status
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStatus {
    pub is_multisig: bool,
    pub multisig_threshold: Option<u32>,
    pub multisig_total: Option<u32>,
    pub is_locked: bool,
    pub balance: Amount,
    pub unlocked_balance: Amount,
}

/// Complete wallet information
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub version: String,
    pub balance: Amount,
    pub unlocked_balance: Amount,
    pub is_multisig: bool,
    pub multisig_threshold: Option<u32>,
    pub multisig_total: Option<u32>,
    pub is_locked: bool,
    pub block_height: u64,
    pub daemon_block_height: u64,
}

/// Escrow transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowTransaction {
    pub escrow_id: EscrowId,
    pub buyer: UserId,
    pub seller: UserId,
    pub amount: Amount,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Escrow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EscrowStatus {
    Created,
    Funded,
    Disputed,
    Released,
    Refunded,
    Cancelled,
}

/// Tor status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorStatus {
    pub is_tor: bool,
    pub ip: String,
    pub exit_node: String,
}

/// RPC request structure
#[derive(Debug, Serialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl RpcRequest {
    pub fn new(method: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: "0".to_string(),
            method: method.to_string(),
            params: None,
        }
    }
}

/// RPC response structure
#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String, // Toujours "2.0"
    pub id: String,      // Match request ID
    pub result: Option<T>,
    pub error: Option<RpcErrorDetails>,
}

/// RPC error details
#[derive(Debug, Deserialize)]
pub struct RpcErrorDetails {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>, // Détails additionnels
}

/// Prepare multisig result
#[derive(Debug, Deserialize)]
pub struct PrepareMultisigResult {
    pub multisig_info: String,
}

/// Make multisig result (step 2/6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeMultisigResult {
    pub address: String,       // Multisig address (starts with "5" on testnet)
    pub multisig_info: String, // Info for next step (export/import)
}

/// Export multisig info result (step 3/6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMultisigInfoResult {
    pub info: String, // Multisig info to share with other participants
}

/// Import multisig info result (step 4/6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMultisigInfoResult {
    pub n_outputs: u64, // Number of outputs imported
}

/// Configuration for Monero RPC
#[derive(Debug, Clone)]
pub struct MoneroConfig {
    pub rpc_url: String,
    pub rpc_user: Option<String>,
    pub rpc_password: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for MoneroConfig {
    fn default() -> Self {
        Self {
            rpc_url: MONERO_RPC_URL.to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 30,
        }
    }
}
