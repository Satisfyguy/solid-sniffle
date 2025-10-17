//! Common types for Monero Marketplace

use crate::MONERO_RPC_URL;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Represents the state of an escrow process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EscrowState {
    /// The escrow has been created but not yet funded.
    Created,
    /// The buyer has funded the escrow.
    Funded,
    /// The seller has released the funds to the buyer.
    Released,
    /// The funds have been refunded to the buyer.
    Refunded,
    /// The escrow is in a disputed state, requiring arbiter intervention.
    Disputed,
}

/// Contains all the necessary data for an escrow agreement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowData {
    pub buyer: UserId,
    pub seller: UserId,
    pub arbiter: UserId,
    pub amount: Amount,
    pub multisig_address: MoneroAddress,
}

/// Event triggered when a new escrow is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowCreated {
    pub escrow_id: EscrowId,
    pub data: EscrowData,
}

/// Event triggered when an escrow is funded by the buyer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowFunded {
    pub escrow_id: EscrowId,
    pub tx_hash: TxHash,
}

/// Event triggered when funds are released to the seller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowReleased {
    pub escrow_id: EscrowId,
    pub tx_hash: TxHash,
}

/// Event triggered when funds are refunded to the buyer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowRefunded {
    pub escrow_id: EscrowId,
    pub tx_hash: TxHash,
}

/// Event triggered when an escrow enters dispute state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowDisputed {
    pub escrow_id: EscrowId,
    pub reason: String,
    pub disputed_by: UserId,
}

/// Complete escrow information with state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub id: EscrowId,
    pub data: EscrowData,
    pub state: EscrowState,
    pub created_at: u64,
    pub updated_at: u64,
    pub funding_tx_hash: Option<TxHash>,
    pub release_tx_hash: Option<TxHash>,
    pub refund_tx_hash: Option<TxHash>,
    pub dispute_reason: Option<String>,
    pub disputed_by: Option<UserId>,
}

/// Escrow operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowResult {
    Created(Box<Escrow>),
    Funded {
        escrow_id: EscrowId,
        tx_hash: TxHash,
    },
    Released {
        escrow_id: EscrowId,
        tx_hash: TxHash,
    },
    Refunded {
        escrow_id: EscrowId,
        tx_hash: TxHash,
    },
    Disputed {
        escrow_id: EscrowId,
        reason: String,
    },
}

/// Monero RPC request#[derive(Debug, Serialize, Deserialize)]
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

/// Transaction destination (recipient address and amount)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDestination {
    pub address: MoneroAddress,
    pub amount: Amount,
}

/// Result from creating a multisig transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionResult {
    pub tx_data_hex: String,    // Unsigned transaction data (hex)
    pub tx_hash: TxHash,        // Transaction hash
    pub tx_key: String,         // Transaction key
    pub amount: Amount,         // Total amount being sent
    pub fee: Amount,            // Transaction fee
    pub multisig_txset: String, // Multisig transaction set for signing
}

/// Result from signing a multisig transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignMultisigResult {
    pub tx_data_hex: String,       // Partially signed transaction data
    pub tx_hash_list: Vec<TxHash>, // List of transaction hashes
}

/// Result from submitting (finalizing) a multisig transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitMultisigResult {
    pub tx_hash_list: Vec<TxHash>, // List of submitted transaction hashes
}

/// Transaction status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub tx_hash: TxHash,
    pub confirmations: u64,
    pub block_height: u64,
    pub timestamp: u64,
    pub amount: Amount,
    pub fee: Amount,
}

// ============================================================================
// ESCROW IMPLEMENTATIONS
// ============================================================================

impl EscrowState {
    /// Check if a state transition is valid
    pub fn can_transition_to(&self, new_state: &EscrowState) -> bool {
        use EscrowState::*;
        matches!(
            (self, new_state),
            (Created, Funded)
                | (Funded, Released)
                | (Funded, Refunded)
                | (Funded, Disputed)
                | (Disputed, Released)
                | (Disputed, Refunded)
        )
    }

    /// Get the next possible states from current state
    pub fn next_possible_states(&self) -> Vec<EscrowState> {
        use EscrowState::*;
        match self {
            Created => vec![Funded],
            Funded => vec![Released, Refunded, Disputed],
            Disputed => vec![Released, Refunded],
            Released | Refunded => vec![], // Terminal states
        }
    }
}

impl Escrow {
    /// Create a new escrow
    pub fn new(id: EscrowId, data: EscrowData) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            data,
            state: EscrowState::Created,
            created_at: now,
            updated_at: now,
            funding_tx_hash: None,
            release_tx_hash: None,
            refund_tx_hash: None,
            dispute_reason: None,
            disputed_by: None,
        }
    }

    /// Transition to a new state if valid
    pub fn transition_to(&mut self, new_state: EscrowState) -> Result<(), String> {
        if !self.state.can_transition_to(&new_state) {
            return Err(format!(
                "Invalid state transition from {:?} to {:?}",
                self.state, new_state
            ));
        }

        self.state = new_state;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(())
    }

    /// Check if escrow is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, EscrowState::Released | EscrowState::Refunded)
    }

    /// Check if escrow can be funded
    pub fn can_be_funded(&self) -> bool {
        matches!(self.state, EscrowState::Created)
    }

    /// Check if escrow can be released
    pub fn can_be_released(&self) -> bool {
        matches!(self.state, EscrowState::Funded | EscrowState::Disputed)
    }

    /// Check if escrow can be refunded
    pub fn can_be_refunded(&self) -> bool {
        matches!(self.state, EscrowState::Funded | EscrowState::Disputed)
    }

    /// Check if escrow can be disputed
    pub fn can_be_disputed(&self) -> bool {
        matches!(self.state, EscrowState::Funded)
    }
}

// ============================================================================
// CHECKPOINT IMPLEMENTATIONS
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WorkflowStep {
    Initiated,
    Prepared,
    Made,
    SyncedRound1,
    SyncedRound2,
    Ready,
    // Transaction-related steps
    TxCreationStarted,
    TxCreated,
    TxSigned,
    TxFinalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checkpoint {
    pub session_id: String,
    pub current_step: WorkflowStep,
    pub last_updated: String, // ISO 8601 timestamp
    pub multisig_address: Option<String>,
    pub required_signatures: Option<u32>,
    // Stores this wallet's own generated multisig info/keys
    pub local_multisig_info: Option<String>,
    // Stores multisig info received from other participants
    pub remote_multisig_infos: Vec<String>,
    // Stores data related to a transaction being created/signed
    pub transaction_data: Option<TransactionCheckpointData>,
    // Generic key-value store for future use or user notes
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionCheckpointData {
    pub unsigned_tx_set: Option<String>,
    pub collected_signatures: Vec<String>,
    pub tx_hash: Option<String>,
}

impl Checkpoint {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_step: WorkflowStep::Initiated,
            last_updated: chrono::Utc::now().to_rfc3339(),
            multisig_address: None,
            required_signatures: None,
            local_multisig_info: None,
            remote_multisig_infos: Vec::new(),
            transaction_data: None,
            metadata: HashMap::new(),
        }
    }
}
