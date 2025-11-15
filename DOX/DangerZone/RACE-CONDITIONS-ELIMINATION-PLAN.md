# Plan d'Élimination des Race Conditions Multisig

**Date:** 2025-11-14
**Status:** 🚀 READY TO IMPLEMENT
**Baseline:** Commit 60e582a (1 escrow ✅, 2 escrows ⚠️ 50% succès)
**Objectif:** 100% succès pour 2-10 escrows concurrents

---

## Contexte

### Situation Actuelle
- **Commit 60e582a**: Code fonctionnel pour 1 escrow isolé
- **Race conditions**: 2 escrows concurrents = 1/2 succès (50%)
- **Cause racine**: Opérations concurrentes sur même escrow + collisions RPC

### Documentation Analysée
- `ROUND-ROBIN-FIX-FINAL.md`: Fix get_rpc_for_role() vs get_healthy_rpc_for_role()
- `COPY-BASED-STATE-PERSISTENCE.md`: File copying entre rounds
- `RPC-CACHE-POLLUTION-SOLUTION.md`: 10s cache purge delay
- `MULTISIG-RACE-CONDITIONS-AND-INSTRUMENTATION-PLAN.md`: Global lock issues
- `PHASE-2-HANDOFF.md`: WalletSessionManager (non appliqué sur 60e582a)

### Stratégie
Plan en 3 axes avec implémentation incrémentale (2.5 jours → production-ready).

---

## PHASE 1: Lock Par-Escrow (3 heures) - PRIORITÉ CRITIQUE

**Objectif**: Sérialiser strictement toutes les opérations sur un même escrow.

### 1.1 Créer le Registre de Verrous

**Nouveau fichier**: `server/src/concurrency/locks.rs`

```rust
//! Escrow-level locking to prevent concurrent operations on same escrow
//!
//! # Architecture
//! - DashMap for thread-safe lock registry
//! - Arc<Mutex<()>> per escrow_id
//! - Automatic lock creation on first access
//! - Periodic cleanup for completed escrows

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tracing::{debug, info};

/// Global registry of per-escrow locks
///
/// Prevents race conditions by ensuring only ONE operation
/// can modify an escrow at a time.
///
/// # Example
/// ```rust
/// let registry = EscrowLockRegistry::new();
/// let lock = registry.get_lock(escrow_id);
/// let _guard = lock.lock().await;  // Exclusive access
/// // ... critical section (multisig, DB updates, RPC calls) ...
/// // Lock released when _guard drops
/// ```
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
        ).await;

        assert!(result.is_err(), "Second lock should timeout while first held");

        drop(guard1);  // Release first lock

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
```

### 1.2 Déclarer Module

**Modifier**: `server/src/lib.rs`

Ajouter:
```rust
pub mod concurrency;
```

**Créer**: `server/src/concurrency/mod.rs`

```rust
pub mod locks;

pub use locks::EscrowLockRegistry;
```

### 1.3 Intégrer dans Main.rs

**Modifier**: `server/src/main.rs`

```rust
use crate::concurrency::EscrowLockRegistry;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... existing initialization ...

    // PHASE 1: Create global escrow lock registry
    let escrow_locks = Arc::new(EscrowLockRegistry::new());
    info!("✅ EscrowLockRegistry initialized");

    // Pass to components that need it
    let wallet_manager = Arc::new(Mutex::new(WalletManager::new(
        rpc_configs,
        wallet_dir,
        Some(db_pool.clone()),
        Some(encryption_key.clone()),
        escrow_locks.clone(),  // NEW
    )));

    let escrow_orchestrator = Arc::new(EscrowOrchestrator::new(
        wallet_manager.clone(),
        wallet_pool.clone(),
        db_pool.clone(),
        escrow_locks.clone(),  // NEW
    ));

    let blockchain_monitor = Arc::new(BlockchainMonitor::new(
        wallet_manager.clone(),
        db_pool.clone(),
        ws_server.clone(),
        monitor_config,
        escrow_locks.clone(),  // NEW
    ));

    // Background cleanup task
    let escrow_locks_cleanup = escrow_locks.clone();
    let db_cleanup = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(600)); // 10 min
        loop {
            interval.tick().await;

            // Get completed escrow IDs from DB
            let completed = get_completed_escrow_ids(&db_cleanup).await;
            escrow_locks_cleanup.cleanup_completed(&completed);
        }
    });

    // ... rest of main.rs ...
}

async fn get_completed_escrow_ids(pool: &DbPool) -> Vec<Uuid> {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    tokio::task::spawn_blocking(move || {
        use crate::schema::escrows::dsl::*;
        use diesel::prelude::*;

        escrows
            .filter(status.eq_any(&["completed", "refunded", "cancelled"]))
            .select(id)
            .load::<String>(&mut conn)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect()
    })
    .await
    .unwrap_or_default()
}
```

### 1.4 Intégrer dans WalletManager

**Modifier**: `server/src/wallet_manager.rs`

```rust
use crate::concurrency::EscrowLockRegistry;

pub struct WalletManager {
    // ... existing fields ...

    /// Per-escrow locks to prevent concurrent operations
    escrow_locks: Option<Arc<EscrowLockRegistry>>,
}

impl WalletManager {
    pub fn new(
        rpc_configs: Vec<MoneroRpcConfig>,
        wallet_dir: PathBuf,
        db_pool: Option<DbPool>,
        encryption_key: Option<Vec<u8>>,
        escrow_locks: Option<Arc<EscrowLockRegistry>>,  // NEW
    ) -> Self {
        Self {
            // ... existing initialization ...
            escrow_locks,
        }
    }

    /// Exchange multisig info (Round 1, 2, 3) with per-escrow locking
    pub async fn exchange_multisig_info(
        &mut self,
        escrow_id: Uuid,
        roles: Vec<(String, Uuid, String)>,
    ) -> Result<String, WalletManagerError> {
        // PHASE 1: Acquire escrow lock BEFORE any operations
        let _guard = if let Some(ref locks) = self.escrow_locks {
            let lock = locks.get_lock(escrow_id);
            Some(lock.lock().await)
        } else {
            None
        };

        info!(
            escrow_id = %escrow_id,
            "🔒 Escrow lock acquired - starting multisig exchange"
        );

        // ... existing exchange_multisig_info logic ...
        // All rounds (1, 2, 3) now protected by lock

        info!(
            escrow_id = %escrow_id,
            "✅ Multisig exchange complete - releasing lock"
        );

        Ok(multisig_address)
    }

    /// Reopen wallet for signing with lock protection
    pub async fn reopen_wallet_for_signing(
        &mut self,
        escrow_id: Uuid,
        role: &str,
    ) -> Result<(), WalletManagerError> {
        // PHASE 1: Acquire lock
        let _guard = if let Some(ref locks) = self.escrow_locks {
            let lock = locks.get_lock(escrow_id);
            Some(lock.lock().await)
        } else {
            None
        };

        info!(
            escrow_id = %escrow_id,
            role = role,
            "🔒 Lock acquired for wallet reopen"
        );

        // ... existing reopen logic ...

        Ok(())
    }
}
```

### 1.5 Intégrer dans EscrowOrchestrator

**Modifier**: `server/src/services/escrow.rs`

```rust
use crate::concurrency::EscrowLockRegistry;

pub struct EscrowOrchestrator {
    // ... existing fields ...
    escrow_locks: Arc<EscrowLockRegistry>,
}

impl EscrowOrchestrator {
    pub fn new(
        wallet_manager: Arc<Mutex<WalletManager>>,
        wallet_pool: Arc<WalletPool>,
        db_pool: DbPool,
        escrow_locks: Arc<EscrowLockRegistry>,  // NEW
    ) -> Self {
        Self {
            // ... existing fields ...
            escrow_locks,
        }
    }

    /// Initialize escrow with lock protection
    pub async fn init_escrow(
        &self,
        order_uuid: Uuid,
        buyer_uuid: Uuid,
        vendor_uuid: Uuid,
        total_xmr: f64,
    ) -> Result<Uuid> {
        // PHASE 1: Acquire escrow lock FIRST
        let lock = self.escrow_locks.get_lock(order_uuid);
        let _guard = lock.lock().await;

        info!(
            escrow_id = %order_uuid,
            "🔒 Escrow lock acquired - starting initialization"
        );

        // ... existing init_escrow logic (multisig setup, etc.) ...

        info!(
            escrow_id = %order_uuid,
            "✅ Escrow initialized - releasing lock"
        );

        Ok(order_uuid)
    }

    /// Release funds with lock protection
    pub async fn release_funds(
        &self,
        escrow_id: Uuid,
        destination_address: String,
    ) -> Result<String> {
        // PHASE 1: Acquire lock
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        info!(
            escrow_id = %escrow_id,
            "🔒 Lock acquired for release_funds"
        );

        // ... existing release_funds logic ...

        Ok(tx_hash)
    }

    /// Refund with lock protection
    pub async fn refund_funds(
        &self,
        escrow_id: Uuid,
        destination_address: String,
    ) -> Result<String> {
        // PHASE 1: Acquire lock
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        info!(
            escrow_id = %escrow_id,
            "🔒 Lock acquired for refund_funds"
        );

        // ... existing refund logic ...

        Ok(tx_hash)
    }
}
```

### 1.6 Intégrer dans BlockchainMonitor

**Modifier**: `server/src/services/blockchain_monitor.rs`

```rust
use crate::concurrency::EscrowLockRegistry;

pub struct BlockchainMonitor {
    // ... existing fields ...
    escrow_locks: Arc<EscrowLockRegistry>,
}

impl BlockchainMonitor {
    pub fn new(
        wallet_manager: Arc<Mutex<WalletManager>>,
        db: DbPool,
        websocket: Addr<WsServer>,
        config: MonitorConfig,
        escrow_locks: Arc<EscrowLockRegistry>,  // NEW
    ) -> Self {
        Self {
            // ... existing fields ...
            escrow_locks,
        }
    }

    async fn check_escrow_funding(&self, escrow_id: Uuid, escrow: Escrow) -> Result<()> {
        // PHASE 1: Acquire lock
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        // ... existing funding check logic ...

        Ok(())
    }

    async fn check_transaction_confirmations(&self, escrow_id: Uuid, escrow: Escrow) -> Result<()> {
        // PHASE 1: Acquire lock
        let lock = self.escrow_locks.get_lock(escrow_id);
        let _guard = lock.lock().await;

        // ... existing confirmation check logic ...

        Ok(())
    }
}
```

### Résultat Attendu Phase 1

✅ **Une seule opération à la fois par escrow** → élimine 80% des race conditions
✅ **Différents escrows peuvent s'exécuter en parallèle** → scalabilité préservée
✅ **Deadlocks impossibles** (un seul lock par escrow, pas de lock ordering)
✅ **Tests**: 2 escrows concurrents passent de 50% → 90%+ succès

---

## PHASE 2: CAS Database + State Machine (4 heures)

**Objectif**: Transitions d'état atomiques, empêcher double-release/refund.

### 2.1 Migration: Ajouter Version Column

**Nouveau fichier**: `migrations/{timestamp}_add_escrow_version/up.sql`

```sql
-- Add version column for optimistic locking (CAS)
ALTER TABLE escrows ADD COLUMN escrow_version INTEGER NOT NULL DEFAULT 1;

-- Index for fast CAS queries
CREATE INDEX idx_escrow_version ON escrows(id, escrow_version);

-- Audit log: track all state transitions
CREATE TABLE escrow_state_transitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    escrow_id TEXT NOT NULL,
    from_status TEXT NOT NULL,
    to_status TEXT NOT NULL,
    escrow_version INTEGER NOT NULL,
    actor TEXT,  -- 'buyer', 'vendor', 'system', 'monitor'
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE
);

CREATE INDEX idx_transitions_escrow ON escrow_state_transitions(escrow_id, created_at);
```

**Fichier**: `migrations/{timestamp}_add_escrow_version/down.sql`

```sql
DROP INDEX IF EXISTS idx_escrow_version;
ALTER TABLE escrows DROP COLUMN escrow_version;

DROP TABLE IF EXISTS escrow_state_transitions;
```

### 2.2 Repository: CAS Methods

**Modifier**: `server/src/repositories/escrow.rs`

```rust
use diesel::prelude::*;
use uuid::Uuid;

/// Update escrow status with optimistic locking (CAS)
///
/// # Arguments
/// * `conn` - Database connection
/// * `escrow_id` - Escrow UUID
/// * `expected_version` - Version we read (for CAS check)
/// * `new_status` - Target status
///
/// # Returns
/// Ok(true) if update succeeded, Ok(false) if version conflict
///
/// # Example
/// ```rust
/// let escrow = load_escrow(conn, escrow_id)?;
/// let success = update_status_cas(
///     conn,
///     escrow_id,
///     escrow.escrow_version,
///     "releasing"
/// )?;
/// if !success {
///     // Another operation changed escrow concurrently - retry
/// }
/// ```
pub fn update_status_cas(
    conn: &mut SqliteConnection,
    escrow_id_str: String,
    expected_version: i32,
    new_status: &str,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::escrows::dsl::*;

    let rows = diesel::update(escrows)
        .filter(id.eq(&escrow_id_str))
        .filter(escrow_version.eq(expected_version))
        .set((
            status.eq(new_status),
            escrow_version.eq(expected_version + 1),
            updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;

    Ok(rows > 0)
}

/// Update transaction hash with CAS (prevents double-write)
pub fn update_tx_hash_cas(
    conn: &mut SqliteConnection,
    escrow_id_str: String,
    expected_version: i32,
    tx_hash: &str,
    new_status: &str,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::escrows::dsl::*;

    let rows = diesel::update(escrows)
        .filter(id.eq(&escrow_id_str))
        .filter(escrow_version.eq(expected_version))
        .filter(transaction_hash.is_null())  // CRITICAL: Only if not already set
        .set((
            transaction_hash.eq(tx_hash),
            status.eq(new_status),
            escrow_version.eq(expected_version + 1),
            updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;

    Ok(rows > 0)
}

/// Log state transition to audit table
pub fn log_state_transition(
    conn: &mut SqliteConnection,
    escrow_id_str: String,
    from_status: &str,
    to_status: &str,
    version: i32,
    actor: &str,
    reason: Option<&str>,
) -> Result<(), diesel::result::Error> {
    use crate::schema::escrow_state_transitions;

    diesel::insert_into(escrow_state_transitions::table)
        .values((
            escrow_state_transitions::escrow_id.eq(escrow_id_str),
            escrow_state_transitions::from_status.eq(from_status),
            escrow_state_transitions::to_status.eq(to_status),
            escrow_state_transitions::escrow_version.eq(version),
            escrow_state_transitions::actor.eq(actor),
            escrow_state_transitions::reason.eq(reason.unwrap_or("")),
        ))
        .execute(conn)?;

    Ok(())
}
```

### 2.3 State Machine Validation

**Nouveau fichier**: `server/src/models/escrow_state.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Initialized,
    Active,
    Releasing,
    Completed,
    Refunding,
    Refunded,
    Disputed,
    Resolved,
    Cancelled,
    Failed,
}

impl EscrowStatus {
    /// Parse from database string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "initialized" => Ok(Self::Initialized),
            "active" => Ok(Self::Active),
            "releasing" => Ok(Self::Releasing),
            "completed" => Ok(Self::Completed),
            "refunding" => Ok(Self::Refunding),
            "refunded" => Ok(Self::Refunded),
            "disputed" => Ok(Self::Disputed),
            "resolved" => Ok(Self::Resolved),
            "cancelled" => Ok(Self::Cancelled),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Unknown escrow status: {}", s)),
        }
    }

    /// Convert to database string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initialized => "initialized",
            Self::Active => "active",
            Self::Releasing => "releasing",
            Self::Completed => "completed",
            Self::Refunding => "refunding",
            Self::Refunded => "refunded",
            Self::Disputed => "disputed",
            Self::Resolved => "resolved",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }

    /// Check if transition from current to new status is valid
    ///
    /// # Critical Rules
    /// - Cannot release AND refund (mutually exclusive)
    /// - Cannot modify after terminal state (completed/refunded/cancelled)
    /// - Dispute can happen from active or releasing/refunding
    pub fn can_transition_to(&self, new: &EscrowStatus) -> bool {
        use EscrowStatus::*;

        match (self, new) {
            // Happy paths
            (Initialized, Active) => true,
            (Active, Releasing) => true,
            (Active, Refunding) => true,
            (Releasing, Completed) => true,
            (Refunding, Refunded) => true,

            // Dispute flows
            (Active, Disputed) => true,
            (Releasing, Disputed) => true,
            (Refunding, Disputed) => true,
            (Disputed, Resolved) => true,
            (Resolved, Releasing) => true,
            (Resolved, Refunding) => true,

            // Cancellation
            (Initialized, Cancelled) => true,
            (Active, Cancelled) => true,

            // Failure
            (_, Failed) => true,  // Can fail from any state

            // CRITICAL: Prevent double-spend scenarios
            (Releasing, Refunding) => false,  // Cannot refund while releasing
            (Refunding, Releasing) => false,  // Cannot release while refunding
            (Completed, _) => false,  // Terminal state
            (Refunded, _) => false,   // Terminal state
            (Cancelled, _) => false,  // Terminal state

            // All other transitions invalid
            _ => false,
        }
    }

    /// Check if status is terminal (no further transitions allowed)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            EscrowStatus::Completed
                | EscrowStatus::Refunded
                | EscrowStatus::Cancelled
                | EscrowStatus::Failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prevent_double_spend() {
        let releasing = EscrowStatus::Releasing;
        let refunding = EscrowStatus::Refunding;

        // CRITICAL: Cannot transition from releasing to refunding
        assert!(!releasing.can_transition_to(&refunding));
        assert!(!refunding.can_transition_to(&releasing));
    }

    #[test]
    fn test_terminal_states() {
        assert!(EscrowStatus::Completed.is_terminal());
        assert!(EscrowStatus::Refunded.is_terminal());
        assert!(!EscrowStatus::Active.is_terminal());
    }
}
```

### 2.4 Usage Pattern with Retry

**Exemple dans `escrow.rs::release_funds()`**:

```rust
pub async fn release_funds(
    &self,
    escrow_id: Uuid,
    destination_address: String,
) -> Result<String> {
    // Lock acquired (Phase 1)
    let lock = self.escrow_locks.get_lock(escrow_id);
    let _guard = lock.lock().await;

    // PHASE 2: CAS retry loop (max 3 attempts)
    for attempt in 1..=3 {
        // 1. Read current state
        let mut conn = self.db_pool.get().context("Failed to get DB connection")?;
        let escrow_id_str = escrow_id.to_string();

        let escrow = tokio::task::spawn_blocking({
            let escrow_id_str = escrow_id_str.clone();
            move || {
                use crate::models::escrow::Escrow;
                Escrow::find_by_id(&mut conn, escrow_id_str)
            }
        })
        .await
        .context("Task join error")?
        .context("Escrow not found")?;

        let current_version = escrow.escrow_version;
        let current_status = EscrowStatus::from_str(&escrow.status)?;

        // 2. Validate transition
        if !current_status.can_transition_to(&EscrowStatus::Releasing) {
            return Err(anyhow::anyhow!(
                "Invalid state transition: {} -> releasing",
                escrow.status
            ));
        }

        // 3. Check not already processing
        if escrow.transaction_hash.is_some() {
            return Err(anyhow::anyhow!("Transaction already exists for this escrow"));
        }

        // 4. Create and sign transaction
        let tx_hash = self.create_and_sign_tx(escrow_id, destination_address.clone()).await?;

        // 5. Attempt CAS update
        let mut conn = self.db_pool.get().context("Failed to get DB connection")?;
        let success = tokio::task::spawn_blocking({
            let escrow_id_str = escrow_id_str.clone();
            let tx_hash_clone = tx_hash.clone();
            move || {
                use crate::repositories::escrow::{update_tx_hash_cas, log_state_transition};

                // Atomic: set tx_hash + status=releasing + increment version
                let success = update_tx_hash_cas(
                    &mut conn,
                    escrow_id_str.clone(),
                    current_version,
                    &tx_hash_clone,
                    "releasing",
                )?;

                if success {
                    // Log transition for audit
                    log_state_transition(
                        &mut conn,
                        escrow_id_str,
                        &escrow.status,
                        "releasing",
                        current_version + 1,
                        "system",
                        Some("release_funds initiated"),
                    )?;
                }

                Ok::<_, anyhow::Error>(success)
            }
        })
        .await
        .context("Task join error")??;

        if success {
            info!(
                escrow_id = %escrow_id,
                tx_hash = %tx_hash,
                attempt = attempt,
                "✅ Transaction persisted with CAS"
            );
            return Ok(tx_hash);
        }

        // CAS failed - version conflict
        warn!(
            escrow_id = %escrow_id,
            attempt = attempt,
            "⚠️ CAS version conflict - retrying"
        );

        if attempt == 3 {
            return Err(anyhow::anyhow!(
                "Failed to update escrow after 3 CAS attempts - concurrent modification"
            ));
        }

        // Small delay before retry
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    unreachable!()
}
```

### Résultat Attendu Phase 2

✅ **Double-release/refund impossible** (CAS + state machine)
✅ **Version conflicts détectés et gérés** (retry logic)
✅ **Audit trail complet** (escrow_state_transitions table)
✅ **Tests**: Concurrent release on same escrow → 1 succès, autres rejettés

---

## PHASE 3: Idempotency Keys (2 heures)

**Objectif**: Empêcher double-exécution lors de retry réseau.

### 3.1 Migration: Table Idempotency

**Nouveau fichier**: `migrations/{timestamp}_create_idempotency_requests/up.sql`

```sql
-- Idempotency table for release_funds/refund_funds operations
CREATE TABLE idempotency_requests (
    idempotency_key TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL,
    action TEXT NOT NULL,  -- 'release_funds', 'refund_funds', 'dispute_init'
    request_hash TEXT,     -- Hash of request parameters (for validation)
    result_hash TEXT,      -- Hash of result (tx_hash, status, etc.)
    result_json TEXT,      -- Full result JSON
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL DEFAULT (datetime('now', '+24 hours')),
    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE
);

CREATE INDEX idx_idempotency_escrow ON idempotency_requests(escrow_id, action);
CREATE INDEX idx_idempotency_expires ON idempotency_requests(expires_at);
```

**Fichier**: `migrations/{timestamp}_create_idempotency_requests/down.sql`

```sql
DROP TABLE IF EXISTS idempotency_requests;
```

### 3.2 Repository Methods

**Nouveau fichier**: `server/src/repositories/idempotency.rs`

```rust
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub idempotency_key: String,
    pub escrow_id: String,
    pub action: String,
    pub result_json: String,
}

/// Check if request was already processed
pub fn check_idempotency(
    conn: &mut SqliteConnection,
    key: &str,
    escrow_id: &str,
    action: &str,
) -> Result<Option<String>, diesel::result::Error> {
    use crate::schema::idempotency_requests::dsl::*;

    idempotency_requests
        .filter(idempotency_key.eq(key))
        .filter(escrow_id.eq(escrow_id))
        .filter(action.eq(action))
        .select(result_json)
        .first::<String>(conn)
        .optional()
}

/// Store idempotent result
pub fn store_idempotency(
    conn: &mut SqliteConnection,
    key: &str,
    escrow_id_str: &str,
    action_str: &str,
    result: &str,
) -> Result<(), diesel::result::Error> {
    use crate::schema::idempotency_requests;

    diesel::insert_into(idempotency_requests::table)
        .values((
            idempotency_requests::idempotency_key.eq(key),
            idempotency_requests::escrow_id.eq(escrow_id_str),
            idempotency_requests::action.eq(action_str),
            idempotency_requests::result_json.eq(result),
        ))
        .execute(conn)?;

    Ok(())
}

/// Cleanup expired idempotency records (call from background task)
pub fn cleanup_expired_idempotency(
    conn: &mut SqliteConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::idempotency_requests::dsl::*;

    diesel::delete(idempotency_requests)
        .filter(expires_at.lt(diesel::dsl::now))
        .execute(conn)
}
```

### 3.3 Handler Integration

**Modifier**: `server/src/handlers/escrow.rs`

```rust
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ReleaseFundsRequest {
    pub destination_address: String,
}

#[derive(Serialize)]
pub struct ReleaseFundsResponse {
    pub success: bool,
    pub tx_hash: String,
    pub status: String,
}

/// Release funds with idempotency protection
///
/// # Headers
/// - Idempotency-Key: UUID (required)
///
/// # Example
/// ```bash
/// curl -X POST http://localhost:8080/api/escrow/{id}/release \
///   -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000" \
///   -H "Content-Type: application/json" \
///   -d '{"destination_address": "..."}'
/// ```
pub async fn release_funds(
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<ReleaseFundsRequest>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    db_pool: web::Data<DbPool>,
) -> impl Responder {
    // PHASE 3: Extract idempotency key
    let idempotency_key = match req.headers().get("Idempotency-Key") {
        Some(header) => match header.to_str() {
            Ok(key) => key.to_string(),
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid Idempotency-Key header"
                }));
            }
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing Idempotency-Key header (required for safety)"
            }));
        }
    };

    // Parse escrow_id
    let escrow_id_str = path.into_inner();
    let escrow_id = match Uuid::parse_str(&escrow_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // PHASE 3: Check if already processed
    let mut conn = match db_pool.get() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection failed: {}", e)
            }));
        }
    };

    let previous_result = match tokio::task::spawn_blocking({
        let key = idempotency_key.clone();
        let escrow_id_str = escrow_id_str.clone();
        move || {
            use crate::repositories::idempotency::check_idempotency;
            check_idempotency(&mut conn, &key, &escrow_id_str, "release_funds")
        }
    })
    .await
    {
        Ok(Ok(Some(result_json))) => {
            // Request already processed - return cached result
            let response: ReleaseFundsResponse = serde_json::from_str(&result_json).unwrap();
            return HttpResponse::Ok().json(response);
        }
        Ok(Ok(None)) => None,  // New request
        Ok(Err(e)) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Idempotency check failed: {}", e)
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Task error: {}", e)
            }));
        }
    };

    // Execute release_funds
    let tx_hash = match escrow_orchestrator
        .release_funds(escrow_id, payload.destination_address.clone())
        .await
    {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Release funds failed: {}", e)
            }));
        }
    };

    let response = ReleaseFundsResponse {
        success: true,
        tx_hash: tx_hash.clone(),
        status: "releasing".to_string(),
    };

    // PHASE 3: Store result for idempotency
    let mut conn = match db_pool.get() {
        Ok(c) => c,
        Err(_) => {
            // Log error but don't fail request (already executed)
            return HttpResponse::Ok().json(response);
        }
    };

    let response_json = serde_json::to_string(&response).unwrap();
    let _ = tokio::task::spawn_blocking({
        let key = idempotency_key.clone();
        let escrow_id_str = escrow_id_str.clone();
        let result = response_json.clone();
        move || {
            use crate::repositories::idempotency::store_idempotency;
            store_idempotency(&mut conn, &key, &escrow_id_str, "release_funds", &result)
        }
    })
    .await;

    HttpResponse::Ok().json(response)
}
```

### Résultat Attendu Phase 3

✅ **Client peut retry sans risque** (même Idempotency-Key → même résultat)
✅ **Pas de double transaction** même si réseau retry
✅ **TTL 24h** (cleanup automatique)
✅ **Tests**: Replay request with same key → cached response

---

## PHASE 4: Contrôle RPC Pool (2 heures)

**Objectif**: Rate-limiting des opérations lourdes (sync, balance checks).

### 4.1 Sémaphore pour Sync Multisig

**Modifier**: `server/src/wallet_manager.rs`

```rust
use tokio::sync::Semaphore;

pub struct WalletManager {
    // ... existing fields ...

    /// Semaphore to limit concurrent sync operations (prevents RPC overload)
    sync_semaphore: Arc<Semaphore>,
}

impl WalletManager {
    pub fn new(/* ... */) -> Self {
        Self {
            // ... existing initialization ...

            // Limit to 4 concurrent sync operations
            sync_semaphore: Arc::new(Semaphore::new(4)),
        }
    }

    /// Sync multisig wallets with rate limiting
    pub async fn sync_multisig_wallets(
        &mut self,
        escrow_id: Uuid,
    ) -> Result<(), WalletManagerError> {
        // PHASE 4: Acquire semaphore permit (blocks if 4 already running)
        let _permit = self.sync_semaphore
            .acquire()
            .await
            .map_err(|_| WalletManagerError::InvalidState {
                expected: "semaphore available".to_string(),
                actual: "semaphore closed".to_string(),
            })?;

        info!(
            escrow_id = %escrow_id,
            "🔄 Sync permit acquired (max 4 concurrent)"
        );

        // ... existing sync logic ...
        // Permit released when _permit drops

        Ok(())
    }
}
```

### 4.2 WalletPool: Vérifier Allocation Exclusive

**Vérifier**: `server/src/wallet_pool.rs`

```rust
pub struct WalletPool {
    // ... existing fields ...

    /// Track allocated ports (prevents double allocation)
    allocated_ports: Arc<Mutex<HashSet<u16>>>,
}

impl WalletPool {
    pub fn new(/* ... */) -> Self {
        Self {
            // ... existing initialization ...
            allocated_ports: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Load wallet for signing with exclusive port allocation
    pub async fn load_wallet_for_signing(
        &self,
        escrow_id: Uuid,
        role: WalletRole,
    ) -> Result<(Arc<MoneroClient>, u16), WalletPoolError> {
        // Find available RPC
        let config = self.get_rpc_for_role(&role)?;
        let port = config.port;

        // PHASE 4: Check port not already allocated
        let mut allocated = self.allocated_ports.lock().await;
        if allocated.contains(&port) {
            return Err(WalletPoolError::PortAlreadyInUse(port));
        }

        // Reserve port
        allocated.insert(port);
        drop(allocated);  // Release lock

        // Open wallet
        let client = match self.open_wallet_on_rpc(escrow_id, role, &config).await {
            Ok(c) => c,
            Err(e) => {
                // Failed to open - release port
                self.allocated_ports.lock().await.remove(&port);
                return Err(e);
            }
        };

        Ok((Arc::new(client), port))
    }

    /// Close wallet and release port
    pub async fn close_wallet(&self, port: u16) -> Result<(), WalletPoolError> {
        // ... existing close logic ...

        // PHASE 4: Release port back to pool
        self.allocated_ports.lock().await.remove(&port);

        info!(port = port, "✅ Port released back to pool");

        Ok(())
    }
}
```

### Résultat Attendu Phase 4

✅ **Max 4 sync operations concurrentes** (évite surcharge RPC)
✅ **Allocation exclusive des ports** (pas de collisions)
✅ **Graceful degradation** (requests queued si pool saturé)

---

## PHASE 5: Tests de Concurrence (3 heures)

### 5.1 Test: Concurrent Release on Same Escrow

**Nouveau fichier**: `server/tests/concurrent_operations_test.rs`

```rust
use uuid::Uuid;
use std::sync::Arc;

#[tokio::test]
#[ignore]  // Run with --ignored
async fn test_concurrent_release_same_escrow() {
    // Setup test escrow
    let (orchestrator, db_pool) = setup_test_environment().await;
    let escrow_id = create_test_escrow(&db_pool, 1.0).await;
    fund_escrow(&orchestrator, escrow_id).await;

    let destination = "valid_monero_address_here".to_string();

    // Launch 2 concurrent release attempts
    let handle1 = {
        let orch = orchestrator.clone();
        let dest = destination.clone();
        tokio::spawn(async move {
            orch.release_funds(escrow_id, dest).await
        })
    };

    let handle2 = {
        let orch = orchestrator.clone();
        let dest = destination.clone();
        tokio::spawn(async move {
            orch.release_funds(escrow_id, dest).await
        })
    };

    let (result1, result2) = tokio::join!(handle1, handle2);

    // Exactly ONE should succeed
    let successes = [result1.unwrap(), result2.unwrap()]
        .iter()
        .filter(|r| r.is_ok())
        .count();

    assert_eq!(successes, 1, "Exactly one release should succeed");

    // Verify DB state
    let escrow = load_escrow(&db_pool, escrow_id).await;
    assert_eq!(escrow.status, "releasing");
    assert!(escrow.transaction_hash.is_some());
    assert_eq!(escrow.escrow_version, 2);  // Version incremented once
}
```

### 5.2 Test: 10 Concurrent Escrows

```rust
#[tokio::test]
#[ignore]
async fn test_10_concurrent_escrows() {
    let (orchestrator, db_pool) = setup_test_environment().await;

    let mut handles = vec![];

    for i in 0..10 {
        let orch = orchestrator.clone();
        let handle = tokio::spawn(async move {
            let escrow_id = Uuid::new_v4();
            orch.init_escrow(escrow_id, buyer_id, vendor_id, 0.5).await
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // All should succeed (no race conditions)
    let successes = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    assert_eq!(successes, 10, "All 10 escrows should initialize successfully");
}
```

### 5.3 Test: Idempotency

```rust
#[tokio::test]
#[ignore]
async fn test_idempotent_release() {
    let (orchestrator, db_pool) = setup_test_environment().await;
    let escrow_id = create_funded_escrow(&db_pool).await;

    let idempotency_key = "test-idempotency-key-12345";
    let destination = "valid_address".to_string();

    // First call
    let result1 = release_funds_with_idempotency(
        &orchestrator,
        &db_pool,
        escrow_id,
        destination.clone(),
        idempotency_key,
    ).await.unwrap();

    // Second call (replay)
    let result2 = release_funds_with_idempotency(
        &orchestrator,
        &db_pool,
        escrow_id,
        destination.clone(),
        idempotency_key,
    ).await.unwrap();

    // Same result
    assert_eq!(result1.tx_hash, result2.tx_hash);

    // Only ONE transaction in DB
    let tx_count = count_transactions(&db_pool, escrow_id).await;
    assert_eq!(tx_count, 1);
}
```

### Résultat Attendu Phase 5

✅ **Tests concurrent release**: 1/2 succès, état DB cohérent
✅ **Tests 10 escrows**: 100% succès
✅ **Tests idempotency**: Replay safe, pas de side-effects

---

## PHASE 6: Observabilité (1 heure)

**Ajouter métriques Prometheus**:

**Modifier**: `server/src/monitoring/metrics.rs`

```rust
use prometheus::{IntCounter, IntGauge};
use once_cell::sync::Lazy;

// Lock metrics
pub static ESCROW_LOCKS_ACTIVE: Lazy<IntGauge> = Lazy::new(|| {
    IntGauge::new("escrow_locks_active", "Number of escrow locks currently held")
});

pub static ESCROW_LOCKS_WAIT_TIME_MS: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "escrow_locks_wait_time_ms",
        "Time spent waiting for escrow lock (milliseconds)"
    ).buckets(vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0]);
    Histogram::with_opts(opts).unwrap()
});

// CAS metrics
pub static CAS_ATTEMPTS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    IntCounter::new("cas_attempts_total", "Total CAS update attempts")
});

pub static CAS_FAILURES_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    IntCounter::new("cas_failures_total", "Total CAS version conflicts")
});

// Idempotency metrics
pub static IDEMPOTENCY_HITS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    IntCounter::new("idempotency_hits_total", "Requests served from idempotency cache")
});

// RPC pool metrics
pub static RPC_POOL_FREE_PORTS: Lazy<IntGauge> = Lazy::new(|| {
    IntGauge::new("rpc_pool_free_ports", "Number of free RPC ports")
});

pub static SYNC_OPERATIONS_CONCURRENT: Lazy<IntGauge> = Lazy::new(|| {
    IntGauge::new("sync_operations_concurrent", "Number of concurrent sync operations")
});
```

**Usage dans code**:

```rust
// Before acquiring lock
ESCROW_LOCKS_ACTIVE.inc();
let start = std::time::Instant::now();
let _guard = lock.lock().await;
ESCROW_LOCKS_WAIT_TIME_MS.observe(start.elapsed().as_millis() as f64);

// After CAS attempt
CAS_ATTEMPTS_TOTAL.inc();
if !success {
    CAS_FAILURES_TOTAL.inc();
}

// On idempotency hit
IDEMPOTENCY_HITS_TOTAL.inc();
```

### Résultat Attendu Phase 6

✅ **Dashboard Grafana**: Locks actifs, CAS conflicts, RPC utilization
✅ **Alertes**: CAS failure rate >5% → investigate
✅ **Debugging**: Correlation entre locks, CAS, et succès escrow

---

## Timeline d'Implémentation

### Sprint 1 (Jour 1): Quick Wins
- ⏱️ **09h00-12h00**: Phase 1 (Lock par-escrow) - 3h
- ⏱️ **13h00-15h00**: Phase 2.1-2.2 (CAS database) - 2h
- ⏱️ **15h00-16h00**: Phase 5 tests basiques - 1h
- ✅ **Validation**: 2-3 escrows concurrents → 90%+ succès

### Sprint 2 (Jour 2): Robustesse
- ⏱️ **09h00-11h00**: Phase 2.3-2.4 (State machine) - 2h
- ⏱️ **11h00-13h00**: Phase 3 (Idempotency) - 2h
- ⏱️ **14h00-15h00**: Phase 4 (RPC control) - 1h
- ⏱️ **15h00-16h00**: Phase 5 tests avancés - 1h
- ✅ **Validation**: 10 escrows concurrents, idempotency OK

### Sprint 3 (Jour 3, matin): Production-Ready
- ⏱️ **09h00-10h00**: Phase 6 (Observabilité) - 1h
- ⏱️ **10h00-12h00**: Load testing (50 escrows) - 2h
- ⏱️ **13h00-14h00**: Documentation - 1h
- ✅ **Validation**: Prêt pour beta (5 XMR limit)

**Temps total: 2.5 jours**

---

## Critères de Succès

### Sprint 1 (Critical):
- ✅ 1 escrow: 100% succès (baseline préservée)
- ✅ 2 escrows concurrents: 90%+ succès (était 50%)
- ✅ 3 escrows concurrents: 90%+ succès
- ✅ Aucun double-release détecté dans tests

### Sprint 2 (Important):
- ✅ 10 escrows concurrents: 95%+ succès
- ✅ State machine rejette transitions invalides
- ✅ Idempotency tests passent (replay safe)

### Sprint 3 (Production):
- ✅ 50 escrows en load test: 90%+ succès
- ✅ Métriques: locks OK, CAS <1% conflicts, RPC pool sain
- ✅ Audit score 100/100

---

## Checklist Complète

### Créer (Nouveaux fichiers):
- [ ] `server/src/concurrency/locks.rs`
- [ ] `server/src/concurrency/mod.rs`
- [ ] `server/src/models/escrow_state.rs`
- [ ] `server/src/repositories/idempotency.rs`
- [ ] `migrations/{timestamp}_add_escrow_version/up.sql`
- [ ] `migrations/{timestamp}_add_escrow_version/down.sql`
- [ ] `migrations/{timestamp}_create_idempotency_requests/up.sql`
- [ ] `migrations/{timestamp}_create_idempotency_requests/down.sql`
- [ ] `server/tests/concurrent_operations_test.rs`

### Modifier (Fichiers existants):
- [ ] `server/src/lib.rs` (add concurrency module)
- [ ] `server/src/main.rs` (initialize locks, cleanup task)
- [ ] `server/src/wallet_manager.rs` (locks, semaphore)
- [ ] `server/src/services/escrow.rs` (locks, CAS, idempotency)
- [ ] `server/src/services/blockchain_monitor.rs` (locks)
- [ ] `server/src/handlers/escrow.rs` (idempotency headers)
- [ ] `server/src/repositories/escrow.rs` (CAS methods)
- [ ] `server/src/wallet_pool.rs` (exclusive port allocation)
- [ ] `server/src/monitoring/metrics.rs` (new metrics)

### Appliquer (Migrations):
- [ ] `diesel migration run`
- [ ] `diesel print-schema > server/src/schema.rs`

### Tester:
- [ ] Unit tests (repositories CAS)
- [ ] Integration tests (concurrent operations)
- [ ] Load tests (50 escrows)
- [ ] Manual testing (UI 2-3 escrows)

---

## Notes de Sécurité

### Lock Ordering
- **Single lock per escrow**: Deadlocks impossibles
- **Timeout**: 60s max hold time
- **Cleanup**: Périodique (completed escrows)

### CAS Retry Limits
- **Max 3 attempts**: Fail-fast après version conflicts répétés
- **Exponential backoff**: 50ms, 100ms, 200ms

### Idempotency
- **TTL 24h**: Automatic cleanup
- **Validation**: Request hash matching

### RPC Pool
- **Never exceed**: Allocated port count (6 instances = 6 ports max)
- **Semaphore limit**: CPU/2 concurrent syncs

---

## Prochaine Action Immédiate

**COMMENCER PHASE 1, ÉTAPE 1.1**:

Créer le fichier `server/src/concurrency/locks.rs` avec le registre de verrous.

**Commande**:
```bash
mkdir -p server/src/concurrency
# Créer locks.rs avec le code fourni
```

---

**FIN DU PLAN - READY TO IMPLEMENT** 🚀
