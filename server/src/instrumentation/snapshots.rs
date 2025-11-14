// server/src/instrumentation/snapshots.rs
//! Wallet state snapshot functionality for multisig instrumentation
//!
//! Captures the complete state of a wallet before/after critical operations
//! to enable differential analysis and root cause identification.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;
use tokio::fs;
use uuid::Uuid;
use monero_marketplace_wallet::MoneroClient;

/// Complete snapshot of a wallet's state at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSnapshot {
    /// Timestamp when snapshot was taken (milliseconds since Unix epoch)
    pub timestamp: u64,

    /// Wallet UUID
    pub wallet_id: String,

    /// Role in the escrow (buyer, vendor, arbiter)
    pub role: String,

    /// Is the wallet in multisig mode?
    pub is_multisig: bool,

    /// Balance (total, unlocked) in atomic units
    pub balance: (u64, u64),

    /// Primary address
    pub address: String,

    /// SHA256 hash of the address (for safe comparison in logs)
    pub address_hash: String,

    /// File permissions of wallet file (e.g. "rw-------")
    pub file_perms: Option<String>,

    /// Does the wallet file exist on disk?
    pub file_exists: bool,

    /// RPC port used for this wallet
    pub rpc_port: Option<u16>,

    /// Time taken to collect this snapshot (milliseconds)
    pub collection_time_ms: u64,

    /// Number of currently open wallets in RPC (from list_wallets if available)
    pub open_wallets_count: Option<usize>,
}

impl WalletSnapshot {
    /// Take a complete snapshot of a wallet's state
    ///
    /// # Arguments
    /// * `wallet_id` - UUID of the wallet
    /// * `role` - Role in escrow (buyer, vendor, arbiter)
    /// * `rpc_client` - MoneroClient for RPC calls
    /// * `wallet_path` - Optional path to wallet file on disk
    /// * `rpc_port` - Optional RPC port
    pub async fn capture(
        wallet_id: Uuid,
        role: &str,
        rpc_client: &MoneroClient,
        wallet_path: Option<&str>,
        rpc_port: Option<u16>,
    ) -> Result<Self> {
        let start = Instant::now();

        // Collect RPC state
        let is_multisig = rpc_client
            .rpc()
            .is_multisig()
            .await
            .context("Failed to check multisig status")?;

        let (total, unlocked) = rpc_client
            .rpc()
            .get_balance()
            .await
            .context("Failed to get balance")?;

        let address = rpc_client
            .get_address()
            .await
            .context("Failed to get address")?;

        // Hash the address for safe logging
        let address_hash = {
            let mut hasher = Sha256::new();
            hasher.update(address.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Check file system state if path provided
        let (file_exists, file_perms) = if let Some(path) = wallet_path {
            match fs::metadata(path).await {
                Ok(metadata) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let perms = metadata.permissions();
                        let mode = perms.mode();
                        let perms_str = format!(
                            "{}{}{}",
                            if mode & 0o400 != 0 { "r" } else { "-" },
                            if mode & 0o200 != 0 { "w" } else { "-" },
                            if mode & 0o100 != 0 { "x" } else { "-" }
                        );
                        (true, Some(perms_str))
                    }
                    #[cfg(not(unix))]
                    {
                        (true, Some("unknown".to_string()))
                    }
                }
                Err(_) => (false, None),
            }
        } else {
            (false, None)
        };

        let collection_time_ms = start.elapsed().as_millis() as u64;

        Ok(Self {
            timestamp: super::events::now_ms(),
            wallet_id: wallet_id.to_string(),
            role: role.to_string(),
            is_multisig,
            balance: (total, unlocked),
            address,
            address_hash,
            file_perms,
            file_exists,
            rpc_port,
            collection_time_ms,
            open_wallets_count: None, // Could be populated with list_wallets() call
        })
    }

    /// Compare two snapshots and return differences
    pub fn diff(&self, other: &WalletSnapshot) -> Vec<String> {
        let mut diffs = Vec::new();

        if self.is_multisig != other.is_multisig {
            diffs.push(format!(
                "is_multisig changed: {} -> {}",
                self.is_multisig, other.is_multisig
            ));
        }

        if self.balance != other.balance {
            diffs.push(format!(
                "balance changed: {:?} -> {:?}",
                self.balance, other.balance
            ));
        }

        if self.address_hash != other.address_hash {
            diffs.push(format!(
                "address changed: {} -> {}",
                self.address_hash, other.address_hash
            ));
        }

        if self.file_exists != other.file_exists {
            diffs.push(format!(
                "file_exists changed: {} -> {}",
                self.file_exists, other.file_exists
            ));
        }

        if self.file_perms != other.file_perms {
            diffs.push(format!(
                "file_perms changed: {:?} -> {:?}",
                self.file_perms, other.file_perms
            ));
        }

        diffs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_diff_identical() {
        let snap1 = WalletSnapshot {
            timestamp: 1000,
            wallet_id: "abc".to_string(),
            role: "buyer".to_string(),
            is_multisig: false,
            balance: (0, 0),
            address: "test_addr".to_string(),
            address_hash: "hash1".to_string(),
            file_perms: Some("rw-".to_string()),
            file_exists: true,
            rpc_port: Some(18082),
            collection_time_ms: 50,
            open_wallets_count: None,
        };

        let snap2 = snap1.clone();
        let diffs = snap1.diff(&snap2);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_snapshot_diff_multisig_changed() {
        let mut snap1 = WalletSnapshot {
            timestamp: 1000,
            wallet_id: "abc".to_string(),
            role: "buyer".to_string(),
            is_multisig: false,
            balance: (0, 0),
            address: "test_addr".to_string(),
            address_hash: "hash1".to_string(),
            file_perms: Some("rw-".to_string()),
            file_exists: true,
            rpc_port: Some(18082),
            collection_time_ms: 50,
            open_wallets_count: None,
        };

        let mut snap2 = snap1.clone();
        snap2.is_multisig = true;
        snap2.address_hash = "hash2".to_string();

        let diffs = snap1.diff(&snap2);
        assert_eq!(diffs.len(), 2);
        assert!(diffs[0].contains("is_multisig"));
        assert!(diffs[1].contains("address"));
    }
}
