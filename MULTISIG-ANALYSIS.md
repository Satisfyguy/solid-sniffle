# Monero Marketplace: Multisig Implementation Analysis

**Status:** Complete analysis of multisig state management, instrumentation framework, and race conditions  
**Date:** 2025-11-13  
**Scope:** server/src/wallet_manager.rs, services/wallet_session_manager.rs, instrumentation/ modules

---

## Executive Summary

The Monero Marketplace implements a sophisticated server-side multisig escrow system with three distinct architectural layers:

1. **WalletManager** - Core multisig orchestration with state persistence
2. **WalletSessionManager** - Persistent wallet sessions for performance (90% faster)
3. **WalletPool** - RPC rotation system for scalability (supports 10+ concurrent escrows)
4. **InstrumentationCollector** - Comprehensive event tracing for debugging

Recent commit (16e121e) introduced copy-based state persistence for multisig Rounds 1→2→3, replacing deletion-based approaches to prevent RPC cache pollution.

**Critical Finding:** 3 HIGH-severity race conditions identified + 1 architectural bottleneck (10s serialization).

---

## 1. Multisig State Management

### 1.1 MultisigPhase State Machine

**File:** `/home/malix/Desktop/monero.marketplace/server/src/models/multisig_state.rs`

The multisig setup follows a strict 5-state machine (lines 27-75):

```rust
pub enum MultisigPhase {
    NotStarted,                          // Initial state
    Preparing { completed: Vec<String> }, // prepare_multisig() in progress
    Exchanging { round: u8, infos: HashMap<String, String> }, // make/export/import rounds
    Ready { address: String, finalized_at: i64 }, // Terminal: Ready for transactions
    Failed { reason: String, failed_at: i64 }, // Terminal: Error occurred
}
```

**Key Methods:**
- `is_terminal()` (line 108) - Returns true for Ready/Failed states (blocks state transitions)
- `status_description()` (lines 113-131) - Human-readable phase status with participant count
- `as_db_string()` (line 81) - Database string representation for indexing
- `validate()` (lines 199-239) - Integrity checking (ensures 3 roles present, valid transitions)

**Storage:** Persisted as JSON in `escrows.multisig_state_json` column with version=1 schema

---

### 1.2 MultisigSnapshot - Complete State Recovery

**File:** `/home/malix/Desktop/monero.marketplace/server/src/models/multisig_state.rs` (lines 134-240)

Snapshot contains everything needed to reconstruct state after server restart:

```rust
pub struct MultisigSnapshot {
    pub phase: MultisigPhase,                    // Current phase
    pub wallet_ids: HashMap<String, Uuid>,       // Role → wallet UUID mapping
    pub rpc_urls: HashMap<String, String>,       // Role → RPC URL mapping
    pub multisig_infos: Option<HashMap<String, Vec<u8>>>, // Encrypted sensitive data
    pub version: u8,                             // Schema version
}
```

**Critical:** `multisig_infos` field is encrypted before DB storage (AES-256-GCM)

---

### 1.3 WalletManager Structure

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs` (lines 87-102)

```rust
pub struct WalletManager {
    pub wallets: HashMap<Uuid, WalletInstance>,           // In-memory wallet cache ⚠️ NO LOCK
    rpc_configs: Vec<MoneroConfig>,                       // RPC endpoint configurations
    
    // Round-robin counters (AtomicUsize for lock-free distribution)
    buyer_rpc_index: std::sync::atomic::AtomicUsize,      // Buyer RPC assignment
    vendor_rpc_index: std::sync::atomic::AtomicUsize,     // Vendor RPC assignment
    arbiter_rpc_index: std::sync::atomic::AtomicUsize,    // Arbiter RPC assignment
    
    // Persistence layer
    multisig_repo: Option<Arc<MultisigStateRepository>>,  // DB persistence
    db_pool: Option<DbPool>,                              // Database connection pool
    encryption_key: Option<Vec<u8>>,                      // 32-byte AES-256-GCM key
    
    // Production scalability
    wallet_pool: Option<Arc<WalletPool>>,                 // RPC rotation system
}
```

**Problem:** `wallets` HashMap is PUBLIC and unprotected. Concurrent access will cause corruption.

---

## 2. Synchronization Primitives (Critical for Race Conditions)

### 2.1 Global Wallet Creation Lock (CRITICAL BOTTLENECK)

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs`  
**Lines:** 26-31 (definition), 821 (usage)

```rust
static WALLET_CREATION_LOCK: Lazy<TokioMutex<()>> = Lazy::new(|| TokioMutex::new(()));

// Usage (line 821):
let _lock = WALLET_CREATION_LOCK.lock().await;
```

**Purpose:** Ensures only ONE wallet creation happens across entire server at a time  
**Rationale:** monero-wallet-rpc daemon can only handle one wallet-at-a-time operations

**CRITICAL ISSUES:**
- 🔴 Lock is BLOCKING. Held for entire wallet creation (multiple RPC calls)
- 🔴 No timeout on lock acquisition
- 🔴 No instrumentation to detect lock contention
- 🔴 Serializes concurrent escrow initialization
- 🔴 3 concurrent escrows = only 1 proceeding, others blocked indefinitely

---

### 2.2 AtomicUsize for Round-Robin Distribution

**Lines:** 92-94, 119-121, 448 (usage with Ordering::SeqCst)

```rust
buyer_rpc_index: std::sync::atomic::AtomicUsize::new(0),
vendor_rpc_index: std::sync::atomic::AtomicUsize::new(0),
arbiter_rpc_index: std::sync::atomic::AtomicUsize::new(0),

// Usage:
let index = self.buyer_rpc_index.fetch_add(1, Ordering::SeqCst) % buyer_rpcs.len();
Ok(buyer_rpcs[index].clone())
```

**Type:** Lock-free atomic for RPC selection (no mutex contention)  
**Distribution:** Each role gets dedicated pool (buyer: 0,3,6; vendor: 1,4,7; arbiter: 2,5,8)  
**Status:** ✅ Well-protected

---

### 2.3 WalletSessionManager Lock (RACE CONDITION RISK)

**File:** `/home/malix/Desktop/monero.marketplace/server/src/services/wallet_session_manager.rs`  
**Lines:** 29-31 (definition), 115-154 (get_or_create_session)

```rust
pub struct WalletSessionManager {
    active_sessions: Arc<Mutex<HashMap<Uuid, EscrowSession>>>,  // MUTEX
    wallet_pool: Arc<WalletPool>,
    config: SessionConfig,
}

// Lock usage:
pub async fn get_or_create_session(&self, escrow_id: Uuid) -> Result<Uuid> {
    let mut sessions = self.active_sessions.lock().await;  // LINE 116
    
    if let Some(session) = sessions.get_mut(&escrow_id) {
        session.last_activity = Instant::now();
        return Ok(escrow_id);
    }
    
    // ... limit check ...
    
    drop(sessions);  // LINE 136 - RELEASE LOCK
    
    // CREATE SESSION - NO LOCK HERE ⚠️
    let session = self.create_session(escrow_id).await?;
    
    let mut sessions = self.active_sessions.lock().await;  // LINE 143
    sessions.insert(escrow_id, session);
    Ok(escrow_id)
}
```

**Lock Scope:** Coarse-grained - entire HashMap of sessions  
**Race Condition Window:** Lines 136-143 (open wallet creation unprotected)

---

### 2.4 EscrowCoordinator Lock (Non-Custodial)

**File:** `/home/malix/Desktop/monero.marketplace/server/src/coordination/escrow_coordinator.rs`  
**Lines:** 35-37

```rust
pub struct EscrowCoordinator {
    coordinations: Arc<RwLock<HashMap<String, EscrowCoordination>>>,  // RW-LOCK
}
```

**Type:** RwLock (reader-writer lock) allows concurrent reads  
**Protected:** Multisig info exchange coordination state  
**Status:** ✅ Appropriate for read-heavy workload

---

## 3. Multisig Operations and Race Conditions

### 3.1 Complete Multisig Flow

```
PHASE 1: Preparation (Lines 1513-1583)
├─ prepare_multisig() on each wallet
│  └─ Location: WalletManager::make_multisig() line 1575
│     Returns: MultisigInfo { multisig_info: String }

PHASE 2: Round 1 - make_multisig (Lines 1616-1910)
├─ Copy temp wallet → round_1 file (lines 1704-1725)
├─ Open round_1 wallet
├─ Call make_multisig(2, [vendor_info, arbiter_info]) on buyer
├─ Instrumentation snapshots recorded (lines 1620-1656) ✅
├─ Close wallet after operation (line 1867)
├─ Wait 10s for RPC cache purge (line 1903) ⚠️ SERIALIZATION
├─ Copy round_1 → round_2_input
└─ Next wallet's make_multisig

PHASE 3: Round 2 - export/import (Lines 1918-1933)
├─ Open round_2_input wallet
├─ Call export_multisig_info()
├─ Call import_multisig_info(other_wallets)
├─ Close wallet
└─ Copy → round_3_input

PHASE 4: Round 3 - finalize (Lines 1938-2042)
├─ Open round_3_input wallet
├─ Call import_multisig_info() (final)
├─ Finalize multisig
├─ Copy → round_3_final (PERMANENT)
└─ Delete intermediate files
```

**Duration:** ~30 seconds per escrow (3 × 10s waits)

---

### 3.2 Copy-Based State Persistence (Commit 16e121e)

**Previous Approach (BROKEN):**
- Delete wallet file between rounds
- Lose accumulated multisig state
- RPC daemon reload from empty file = "pubkey recommended" errors

**Current Approach:**

```
Round 1:
├─ temp_escrow_uuid (created at init)
├─ Copy to {buyer,vendor,arbiter}_escrow_uuid_round_1 (preserve state)
├─ Perform make_multisig()
└─ Copy round_1 → round_2_input

Round 2:
├─ Load round_2_input (contains accumulated state from round_1)
├─ export_multisig_info()
├─ import_multisig_info()
└─ Copy → round_3_input

Round 3:
├─ Load round_3_input (final state)
├─ import_multisig_info() (final)
├─ Finalize
├─ Copy → round_3_final (PERMANENT)
└─ Clean up round_1, round_2_input, round_3_input
```

**Files Modified in Commit 16e121e:**
- `server/src/wallet_manager.rs` lines 1704-1725 (Round 1 copy)
- `server/src/wallet_manager.rs` lines 1704-1725 (Round 1 copy logic)
- `server/src/wallet_manager.rs` lines 1812-1901 (Round 2 copy logic)
- `server/src/wallet_manager.rs` lines 1938-2042 (Round 3 copy + cleanup)

---

## 4. Existing Instrumentation

### 4.1 InstrumentationCollector

**File:** `/home/malix/Desktop/monero.marketplace/server/src/instrumentation/collector.rs` (320 lines)

**Thread-Safety:** `Arc<Mutex<Vec<MultisigEvent>>>`

**Key Methods:**
- `new(escrow_id)` (line 33) - Create collector
- `is_enabled()` (line 249) - Check ENABLE_INSTRUMENTATION env var
- `record_event()` (line 61) - Generic event recording
- `record_rpc_start()` (line 84) - RPC call began
- `record_rpc_end()` (line 115) - RPC call completed
- `record_snapshot()` (line 149) - Wallet state snapshot
- `record_error()` (line 167) - Error with context
- `dump_json()` (line 205) - Export all events to JSON file
- `summary()` (line 230) - Event count by type

**Integration Status:** Only partially integrated (PRE-ROUND1 snapshots added in commit 16e121e)

---

### 4.2 WalletSnapshot

**File:** `/home/malix/Desktop/monero.marketplace/server/src/instrumentation/snapshots.rs` (240 lines)

**Captured State:**
```rust
pub struct WalletSnapshot {
    pub timestamp: u64,              // When snapshot taken (ms since epoch)
    pub wallet_id: String,           // UUID
    pub role: String,                // buyer/vendor/arbiter
    pub is_multisig: bool,           // ✅ Multisig mode status
    pub balance: (u64, u64),         // (total, unlocked) in atomic units
    pub address: String,             // Monero address
    pub address_hash: String,        // SHA256 hash (safe for logs)
    pub file_perms: Option<String>,  // rw------- permissions
    pub file_exists: bool,           // Disk file exists?
    pub rpc_port: Option<u16>,       // Which RPC port
    pub collection_time_ms: u64,     // Time to collect snapshot
    pub open_wallets_count: Option<usize>, // Count of open wallets
}
```

**Critical Feature:** `diff()` method (line 145) compares snapshots to detect state changes

---

### 4.3 EventType Enumeration

**File:** `/home/malix/Desktop/monero.marketplace/server/src/instrumentation/events.rs`

```rust
pub enum EventType {
    RpcCallStart,                    // RPC started
    RpcCallEnd,                      // RPC completed
    RpcCallError,                    // RPC failed
    SnapshotPreRound1,               // Before Round 1
    SnapshotPostMakeMultisig,        // After make_multisig
    SnapshotPreRound2,               // Before Round 2
    SnapshotPostExportMultisig,      // After export
    SnapshotPreRound3,               // Before Round 3
    SnapshotPostImportMultisig,      // After import
    SnapshotFinal,                   // Final state
    StateChange,                     // State transition
    FileOperation,                   // Copy/delete/chmod
    CachePollutionDetected,          // Wallet in multisig before operation
    ErrorFinal,                      // Final error
    Custom,                          // Custom event
}
```

---

### 4.4 Instrumentation Macro

**File:** `/home/malix/Desktop/monero.marketplace/server/src/instrumentation/mod.rs` (lines 116-136)

```rust
macro_rules! instrument_rpc_call {
    ($collector:expr, $method:expr, $role:expr, $port:expr, $call:block) => {{
        let start = std::time::Instant::now();
        $collector.record_rpc_start($method, $role, $port).await;
        let result = $call;
        let duration_ms = start.elapsed().as_millis() as u64;
        $collector.record_rpc_end($method, $role, duration_ms, result.is_ok(), $port).await;
        result
    }};
}
```

**Status:** Defined but NOT USED in wallet_manager.rs operations (should be used for all RPC calls)

---

## 5. Race Conditions (CRITICAL)

### 5.1 🔴 CRITICAL: Global Lock Serialization

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs` (line 821)

**Code:**
```rust
let _lock = WALLET_CREATION_LOCK.lock().await;  // Holds lock during create_temporary_wallet
```

**Scope:** Entire wallet creation (multiple RPC calls, could be 5-10 seconds)

**Risk:** 
- If 3 concurrent escrows start simultaneously → only 1 proceeds, others blocked
- No timeout on lock acquisition
- No instrumentation to detect lock contention
- Could lead to cascading failures in testnet

**Impact:** 3 concurrent escrows = 15-30 seconds before any complete

**Severity:** 🔴 HIGH - System cannot scale

---

### 5.2 🔴 CRITICAL: WalletSessionManager Session Creation Race

**File:** `/home/malix/Desktop/monero.marketplace/server/src/services/wallet_session_manager.rs` (lines 115-154)

**Race Condition Window:**
```rust
let mut sessions = self.active_sessions.lock().await;  // Line 116

if let Some(session) = sessions.get_mut(&escrow_id) {
    return Ok(escrow_id);
}

drop(sessions);  // LINE 136 - LOCK RELEASED ⚠️

// RACE WINDOW: Both threads can reach here
let session = self.create_session(escrow_id).await?;  // Creates 3 wallets

let mut sessions = self.active_sessions.lock().await;  // LINE 143
sessions.insert(escrow_id, session);  // First one wins, overwrites session
```

**Race Scenario:**
1. Thread A drops lock at line 136
2. Thread B acquires lock, sees no session, drops lock at 136
3. Both threads call `create_session()` concurrently
4. Both create 3 wallets for SAME escrow_id
5. First to insert at line 144 wins, second overwrites
6. Second wallet set is dropped without cleanup → resource leak

**Consequences:**
- Duplicate wallet creation
- Resource leaks (wallets opened but not closed)
- Potential multisig state corruption

**Severity:** 🔴 HIGH - Resource exhaustion + state corruption

**Fix:** Use double-checked locking or atomic compare-and-swap

---

### 5.3 🟠 MEDIUM: WalletManager HashMap State Corruption

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs` (line 88)

**Code:**
```rust
pub struct WalletManager {
    pub wallets: HashMap<Uuid, WalletInstance>,  // PUBLIC, NO LOCK
}
```

**Issues:**
- In-memory HashMap accessed by multiple methods without synchronization
- `exchange_multisig_info()` mutates wallets (lines 1634-1867)
- `prepare_multisig()` accesses wallets (line 1575)
- No lock protecting concurrent access

**Race Scenario:**
1. Thread A iterates wallets in `exchange_multisig_info()` loop
2. Thread B modifies wallets HashMap (inserts/removes)
3. Iterator invalidation or data corruption

**Severity:** 🟠 MEDIUM - Depends on timing, but guaranteed with concurrent escrows

---

### 5.4 🟡 MEDIUM: File Copy Race Between Rounds

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs` (lines 1704-1725)

**Code:**
```rust
let temp_path = std::path::PathBuf::from(format!("./testnet-wallets/{}", temp_wallet_name));
let round1_path = std::path::PathBuf::from(format!("./testnet-wallets/{}", wallet_filename));

if temp_path.exists() {  // ⚠️ NOT ATOMIC WITH COPY
    match std::fs::copy(&temp_path, &round1_path) {
        Ok(_) => { /* ... */ }
        Err(e) => { /* ... */ }
    }
}
```

**Race Condition:** 
- `exists()` check is not atomic with `copy()`
- If temp wallet deleted by cleanup thread, copy fails silently
- No explicit chmod after copy (assumes umask inherited)

**Severity:** 🟡 MEDIUM - Unlikely with round-robin, but possible with high concurrency

---

### 5.5 🔴 CRITICAL: 10-Second RPC Cache Purge (SERIALIZATION BOTTLENECK)

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs` (line 1903)

**Code:**
```rust
info!("⏳ Waiting 10 seconds before next make_multisig call (reset RPC cache)...");
tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
```

**Context:** Sequential loop processes 3 wallets
```
for role in &[WalletRole::Buyer, WalletRole::Vendor, WalletRole::Arbiter] {
    // ... make_multisig operation (1-2 seconds)
    tokio::time::sleep(Duration::from_secs(10)).await;  // BLOCKING!
}
```

**Total Duration:** 3 × 10 = 30 seconds per escrow setup (NOT parallelizable)

**Consequence:**
- System throughput: 1 escrow per 30 seconds
- 3 concurrent escrows = 90 seconds of serial waiting
- No instrumentation of actual wait time vs expected

**Severity:** 🔴 HIGH - Architectural bottleneck (not a race condition, but kills scalability)

---

### 5.6 🟢 LOW: Round-Robin Index Wraparound

**File:** `/home/malix/Desktop/monero.marketplace/server/src/wallet_manager.rs` (lines 489-495)

**Code:**
```rust
let index = self.arbiter_rpc_index.fetch_add(1, Ordering::SeqCst) % arbiter_rpcs.len();
Ok(arbiter_rpcs[index].clone())
```

**Issue:** 
- `fetch_add()` returns value BEFORE increment
- Under extreme load, index wraps around `usize::MAX`
- Modulo operation ensures index in range

**Severity:** 🟢 LOW - Well-protected by atomic operation, modulo is safe

---

### 5.7 🟠 MEDIUM: MultisigStateRepository Persistence Race

**Issue:**
- Database read-modify-write is not atomic
- `exchange_multisig_info()` reads phase, modifies, persists
- No transaction wrapping state transitions

**Race Scenario:**
1. Thread A: Reads phase=Preparing, updates to Exchanging
2. Thread B: Reads phase=Preparing (doesn't see update yet)
3. Both try to execute Preparing→Exchanging transition
4. Duplicate prepare_multisig() calls

**Severity:** 🟠 MEDIUM - Depends on database isolation level

---

## 6. Instrumentation Gaps

### 6.1 Critical Operations Not Instrumented

- [ ] `create_temporary_wallet()` - Holds global lock, no timing info
- [ ] `exchange_multisig_info()` - Main orchestrator, partial instrumentation only
- [ ] `prepare_multisig()` - Initial setup, only partial snapshots
- [ ] Lock acquisition/release on `WALLET_CREATION_LOCK`
- [ ] RPC failover attempts in `get_healthy_rpc_for_role()`
- [ ] WalletPool slot allocation/deallocation
- [ ] Session creation race window (lines 136-143)

### 6.2 Missing Metrics

- Lock contention count (how long do we wait for WALLET_CREATION_LOCK?)
- RPC cache purge wait time (just logging, not measured)
- Wallet file copy duration (bytes/second throughput)
- Session creation duration (how long does create_session() take?)
- Total escrow setup duration end-to-end
- Actual elapsed time vs 30 second theoretical

### 6.3 Missing Snapshots

- POST-PREPARE multisig (only PRE-ROUND1 implemented)
- POST-EXPORT multisig info
- POST-IMPORT multisig info
- POST-ROUND2 state
- POST-ROUND3 state

### 6.4 Tracing Coverage

**Verified:**
- ✅ Generic `info!()`, `warn!()`, `error!()` macros used
- ✅ Emoji prefixes for visual scanning
- ✅ Instrumentation collector framework complete

**Missing:**
- ❌ Structured spans for multi-step operations
- ❌ Context propagation (trace IDs not used in logs)
- ❌ Metrics (gauge, counter, histogram)
- ❌ Error details beyond message
- ❌ Performance profiling

---

## 7. Code Location Summary

| Component | File | Lines | Status |
|-----------|------|-------|--------|
| **MultisigPhase** | `models/multisig_state.rs` | 27-75 | ✅ Complete |
| **MultisigSnapshot** | `models/multisig_state.rs` | 134-240 | ✅ Complete |
| **WalletManager** | `wallet_manager.rs` | 87-102 | ✅ Core complete |
| **WALLET_CREATION_LOCK** | `wallet_manager.rs` | 31 | ⚠️ No instrumentation |
| **exchange_multisig_info** | `wallet_manager.rs` | 1586-2100+ | ⚠️ Partial |
| **prepare_multisig** | `wallet_manager.rs` | 1513-1583 | ⚠️ Partial |
| **RPC distribution** | `wallet_manager.rs` | 248-281 | ✅ Atomic ops |
| **WalletSessionManager** | `services/wallet_session_manager.rs` | 29-408 | ⚠️ Race condition |
| **EscrowCoordinator** | `coordination/escrow_coordinator.rs` | 35-546 | ✅ RwLock protected |
| **InstrumentationCollector** | `instrumentation/collector.rs` | 1-320 | ✅ Framework OK |
| **WalletSnapshot** | `instrumentation/snapshots.rs` | 1-240 | ✅ Framework OK |
| **EventType** | `instrumentation/events.rs` | 14-39 | ✅ Complete |
| **WalletPool** | `wallet_pool.rs` | 1-200+ | ✅ RPC rotation |

---

## 8. Summary Table

| Issue | Severity | Type | Mitigation |
|-------|----------|------|-----------|
| Global lock serializes wallet creation | 🔴 HIGH | Lock contention | Per-escrow locks or CAS |
| Session creation race | 🔴 HIGH | Race condition | Double-checked locking |
| HashMap state corruption | 🟠 MEDIUM | Concurrency | Wrap in Mutex/RwLock |
| File copy race | 🟡 MEDIUM | TOCTOU | Atomic rename or temp pattern |
| 10s RPC cache wait | 🔴 HIGH | Bottleneck | Parallel ops or RPC pooling |
| Lock contention not tracked | 🟠 MEDIUM | Observability | Add lock timing |
| Session creation not instrumented | 🟠 MEDIUM | Observability | Add logging + snapshots |
| Round timing not tracked | 🟡 MEDIUM | Observability | Custom events for milestones |

---

## Appendix: Commit 16e121e Details

**Author:** satisfyguy <satisfyguy31@gmail.com>  
**Date:** Nov 13, 2025, 19:21:47 UTC  
**Message:** "feat: Implement copy-based state persistence for multisig Round 1→2→3"

**Impact:**
- ✅ Preserves wallet state between rounds
- ✅ Reduces RPC daemon confusion
- ✅ Per-round filenames prevent conflicts
- ⚠️ Increases I/O operations
- ⚠️ Still has 10s serialization gap

