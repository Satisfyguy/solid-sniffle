//! Escrow-level locking to prevent concurrent operations on same escrow
//!
//! # Architecture
//! - DashMap for thread-safe lock registry
//! - Arc<Mutex<()>> per escrow_id
//! - Automatic lock creation on first access
//! - Periodic cleanup for completed escrows
//!
//! # Example
//! ```rust
//! let registry = EscrowLockRegistry::new();
//! let lock = registry.get_lock(escrow_id);
//! let _guard = lock.lock().await;  // Exclusive access
//! // ... critical section (multisig, DB updates, RPC calls) ...
//! // Lock released when _guard drops
//! ```

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use uuid::Uuid;

/// Global registry of per-escrow locks
///
/// Prevents race conditions by ensuring only ONE operation
/// can modify an escrow at a time.
pub struct EscrowLockRegistry {
    locks: DashMap<Uuid, Arc<Mutex<()>>>,
}

impl EscrowLockRegistry {
    pub fn new() -> Self {
        info!("🔒 Initializing EscrowLockRegistry");
        Self {
            locks: DashMap::new(),
        }
    }

    /// Get exclusive lock for escrow (creates if doesn't exist)
    ///
    /// # Arguments
    /// * `escrow_id` - UUID of the escrow to lock
    ///
    /// # Returns
    /// Arc<Mutex<()>> that can be locked with `.lock().await`
    pub fn get_lock(&self, escrow_id: Uuid) -> Arc<Mutex<()>> {
        self.locks
            .entry(escrow_id)
            .or_insert_with(|| {
                debug!(escrow_id = %escrow_id, "Creating new escrow lock");
                Arc::new(Mutex::new(()))
            })
            .clone()
    }

    /// Cleanup locks for completed escrows (call periodically)
    ///
    /// # Arguments
    /// * `completed_ids` - List of escrow IDs that reached terminal state
    ///
    /// # Usage
    /// Call every 10 minutes from background task with list of
    /// escrows in status: completed, refunded, cancelled
    pub fn cleanup_completed(&self, completed_ids: &[Uuid]) {
        for id in completed_ids {
            if self.locks.remove(id).is_some() {
                debug!(escrow_id = %id, "Removed lock for completed escrow");
            }
        }
        info!("🧹 Cleaned up {} escrow locks", completed_ids.len());
    }

    /// Get current number of active locks (for monitoring)
    pub fn active_count(&self) -> usize {
        self.locks.len()
    }
}

impl Default for EscrowLockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lock_exclusivity() {
        let registry = EscrowLockRegistry::new();
        let escrow_id = Uuid::new_v4();

        let lock = registry.get_lock(escrow_id);
        let guard1 = lock.lock().await;

        // Second lock attempt should block (test with timeout)
        let lock2 = registry.get_lock(escrow_id);
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

    #[test]
    fn test_cleanup() {
        let registry = EscrowLockRegistry::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        registry.get_lock(id1);
        registry.get_lock(id2);
        assert_eq!(registry.active_count(), 2);

        registry.cleanup_completed(&[id1]);
        assert_eq!(registry.active_count(), 1);
    }
}
