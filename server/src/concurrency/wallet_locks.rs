//! Per-wallet operation locking
//!
//! # Critical Purpose
//! Prevents concurrent multisig operations on the SAME wallet across different escrows.
//!
//! # The Race Scenario This Prevents
//! - Escrow A buyer wallet on RPC 18082
//! - Escrow B buyer wallet ALSO on RPC 18082 (round-robin reuse)
//! - Without this lock: Both can call prepare_multisig() concurrently
//! - Result: Monero RPC generates different crypto material (proven with test)
//!
//! # Architecture
//! - DashMap for thread-safe lock registry
//! - Key: (rpc_url, wallet_filename) - Unique per wallet
//! - Arc<Mutex<()>> per wallet
//! - Automatic lock creation on first access
//!
//! # Usage
//! ```rust
//! let wallet_lock = wallet_ops.get_wallet_lock(
//!     "http://127.0.0.1:18082/json_rpc",
//!     "buyer_temp_escrow_abc123"
//! );
//! let _guard = wallet_lock.lock().await;
//! // ENTIRE multisig sequence (open → prepare → make → finalize → close)
//! // _guard drops → lock released
//! ```

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Per-wallet operation lock registry
///
/// Ensures only ONE multisig operation sequence runs per wallet at a time,
/// even across different escrows using the same RPC instance.
pub struct WalletOperationLock {
    /// Locks per (rpc_url, wallet_filename)
    ///
    /// This prevents concurrent operations on the same wallet file,
    /// which Monero RPC does NOT protect against internally.
    locks: DashMap<(String, String), Arc<Mutex<()>>>,
}

impl WalletOperationLock {
    pub fn new() -> Self {
        info!("🔒 Initializing WalletOperationLock (per-wallet serialization)");
        Self {
            locks: DashMap::new(),
        }
    }

    /// Get exclusive lock for a specific wallet
    ///
    /// # Arguments
    /// * `rpc_url` - Full RPC URL (e.g., "http://127.0.0.1:18082/json_rpc")
    /// * `wallet_filename` - Wallet filename (e.g., "buyer_temp_escrow_abc123")
    ///
    /// # Returns
    /// Arc<Mutex<()>> that can be locked with `.lock().await`
    ///
    /// # Example
    /// ```rust
    /// let lock = wallet_ops.get_wallet_lock(
    ///     "http://127.0.0.1:18082/json_rpc",
    ///     "buyer_temp_escrow_abc123"
    /// );
    /// let _guard = lock.lock().await;
    /// // Entire operation sequence executes
    /// // _guard drops at end of scope
    /// ```
    pub fn get_wallet_lock(&self, rpc_url: &str, wallet_filename: &str) -> Arc<Mutex<()>> {
        let key = (rpc_url.to_string(), wallet_filename.to_string());

        self.locks
            .entry(key.clone())
            .or_insert_with(|| {
                debug!(
                    rpc_url = %rpc_url,
                    wallet = %wallet_filename,
                    "Creating new wallet operation lock"
                );
                Arc::new(Mutex::new(()))
            })
            .clone()
    }

    /// Cleanup locks for completed wallets (call periodically)
    ///
    /// # Arguments
    /// * `completed_wallets` - List of (rpc_url, wallet_filename) that are done
    ///
    /// # Usage
    /// Call every 10 minutes with list of escrows in terminal state
    pub fn cleanup_completed(&self, completed_wallets: &[(String, String)]) {
        for key in completed_wallets {
            if self.locks.remove(key).is_some() {
                debug!(
                    rpc_url = %key.0,
                    wallet = %key.1,
                    "Removed wallet operation lock"
                );
            }
        }
        info!("🧹 Cleaned up {} wallet operation locks", completed_wallets.len());
    }

    /// Get current number of active wallet locks (for monitoring)
    pub fn active_count(&self) -> usize {
        self.locks.len()
    }
}

impl Default for WalletOperationLock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_lock_exclusivity() {
        let registry = WalletOperationLock::new();
        let rpc_url = "http://127.0.0.1:18082/json_rpc";
        let wallet = "test_wallet";

        let lock = registry.get_wallet_lock(rpc_url, wallet);
        let guard1 = lock.lock().await;

        // Second lock attempt on SAME wallet should block
        let lock2 = registry.get_wallet_lock(rpc_url, wallet);
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(10),
            lock2.lock()
        )
        .await;

        assert!(
            result.is_err(),
            "Second lock should timeout while first held"
        );

        drop(guard1); // Release first lock

        // Now second lock should succeed
        let _guard2 = lock2.lock().await;
    }

    #[tokio::test]
    async fn test_different_wallets_no_conflict() {
        let registry = WalletOperationLock::new();
        let rpc_url = "http://127.0.0.1:18082/json_rpc";

        let lock1 = registry.get_wallet_lock(rpc_url, "wallet_A");
        let lock2 = registry.get_wallet_lock(rpc_url, "wallet_B");

        let _guard1 = lock1.lock().await;

        // Should NOT block - different wallet files
        let _guard2 = lock2.lock().await;
    }

    #[test]
    fn test_cleanup() {
        let registry = WalletOperationLock::new();

        registry.get_wallet_lock("http://127.0.0.1:18082", "wallet1");
        registry.get_wallet_lock("http://127.0.0.1:18083", "wallet2");
        assert_eq!(registry.active_count(), 2);

        registry.cleanup_completed(&[(
            "http://127.0.0.1:18082".to_string(),
            "wallet1".to_string()
        )]);
        assert_eq!(registry.active_count(), 1);
    }
}
