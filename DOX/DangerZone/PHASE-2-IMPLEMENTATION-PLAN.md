# Phase 2 Implementation Plan - WalletSessionManager Integration

**Status**: 40% Complete (2/5 major tasks)
**Started**: 2025-11-12
**Target Completion**: TBD

---

## ✅ Completed Tasks

### 1. WalletSessionManager Module Creation
**File**: `server/src/services/wallet_session_manager.rs` (~426 lines)

**Implemented Features**:
- ✅ `get_or_create_session()` - Session retrieval with automatic creation
- ✅ `get_wallet()` - Returns Arc<MoneroClient> for role (Buyer/Vendor/Arbiter)
- ✅ `close_session()` - Closes all 3 wallets for an escrow
- ✅ `evict_lru_session()` - Automatic LRU eviction when limit reached
- ✅ `cleanup_stale_sessions()` - TTL-based cleanup (2-hour timeout)
- ✅ `get_stats()` - Monitoring statistics

**Performance Impact**:
- Opens 3 wallets once at escrow init (6-8s one-time cost)
- All subsequent operations use open wallets (100-500ms vs 3-5s)
- 80-90% latency reduction for balance checks and transactions

### 2. BlockchainMonitor - Partial Integration
**File**: `server/src/services/blockchain_monitor.rs`

**Completed Changes**:
- ✅ Added `session_manager: Arc<WalletSessionManager>` to struct (line 48)
- ✅ Updated constructor to accept session_manager parameter (lines 57-75)
- ✅ Added imports for WalletSessionManager and WalletRole (lines 21-22)
- ✅ **Updated `check_escrow_funding()` method** (lines 157-246):
  - Replaced manual open/close with `session_manager.get_wallet()`
  - Eliminated ~50 lines of open_wallet/close_wallet RPC calls
  - Balance checks now instant (wallet already open)
  - Removed Phase 1 close_wallet call

**Performance Gain for Funding Checks**:
- **Before**: Open wallet (2-3s) + refresh (0.5s) + balance (0.5s) + close (0.3s) = **3.3-4.3s**
- **After**: Get from session (instant) + refresh (0.5s) + balance (0.5s) = **1s**
- **Improvement**: 70% faster

---

## 🔄 Remaining Tasks

### Task 3: Complete BlockchainMonitor Integration
**File**: `server/src/services/blockchain_monitor.rs`
**Lines**: 284-374
**Estimated Time**: 15-20 minutes

**Method to Update**: `check_transaction_confirmations()`

**Current Code Pattern** (Phase 1):
```rust
// Lines 284-374: Manual open/close pattern
let wallet_filename = format!("buyer_temp_escrow_{}", escrow_id);
let rpc_url = "http://127.0.0.1:18087/json_rpc";
let client = reqwest::Client::new();

// Open wallet (lines 294-312)
client.post(rpc_url).json(&open_payload).send().await?;

// Get transaction details (lines 314-352)
let response = client.post(rpc_url).json(&get_transfer_payload).send().await?;

// Close wallet (lines 359-374)
client.post(rpc_url).json(&close_payload).send().await?;
```

**New Code Pattern** (Phase 2):
```rust
// Use session manager instead
let buyer_wallet = self.session_manager
    .get_wallet(escrow_id, WalletRole::Buyer)
    .await
    .context("Failed to get buyer wallet from session")?;

// Get transaction details via wallet client
let transfer = buyer_wallet.get_transfer_by_txid(tx_hash)
    .await
    .context("Failed to get transaction details")?;

let confirmations = transfer.confirmations;

// No close needed - wallet stays in session
```

**Steps**:
1. Replace lines 284-312 (wallet opening) with `session_manager.get_wallet()`
2. Replace lines 314-352 (RPC calls) with `buyer_wallet.get_transfer_by_txid()`
3. Remove lines 359-374 (wallet closing)
4. Add error handling with proper context
5. Test compilation

**Expected Outcome**:
- Confirmation checks go from 3-5s → 500ms-1s (80% faster)
- No RPC slot occupation
- Code reduced by ~60 lines

---

### Task 4: Session Creation at Escrow Initialization
**File**: `server/src/services/escrow.rs` or `server/src/wallet_manager.rs`
**Estimated Time**: 30-40 minutes

**Goal**: Create wallet session when escrow is initialized (after multisig setup completes)

**Option A: In EscrowOrchestrator (Recommended)**
**File**: `server/src/services/escrow.rs`
**Method**: `init_escrow()` (around line 200-250)

**Integration Point**:
```rust
// After multisig setup completes successfully
pub async fn init_escrow(...) -> Result<Uuid> {
    // ... existing multisig setup code ...

    // PHASE 2: Create persistent session for this escrow
    // This opens all 3 wallets and keeps them open
    info!("🚀 [PHASE 2] Creating wallet session for escrow {}", escrow_id);

    session_manager.get_or_create_session(escrow_id)
        .await
        .context("Failed to create wallet session")?;

    info!("✅ Wallet session created - all wallets open and ready");

    Ok(escrow_id)
}
```

**Steps**:
1. Add `session_manager: Arc<WalletSessionManager>` to `EscrowOrchestrator` struct
2. Update `EscrowOrchestrator::new()` to accept session_manager
3. Call `session_manager.get_or_create_session(escrow_id)` after multisig finalization
4. Handle errors appropriately
5. Update all `EscrowOrchestrator` instantiation sites

**Expected Outcome**:
- 3 wallets opened immediately after multisig setup
- One-time 6-8s cost per escrow
- All future operations use open wallets (instant)

---

### Task 5: Session Cleanup at Escrow Completion
**Files**:
- `server/src/services/escrow.rs` (release_funds/refund methods)
- `server/src/services/blockchain_monitor.rs` (when marking completed/refunded)

**Estimated Time**: 20-30 minutes

**Goal**: Close wallet session when escrow reaches terminal state

**Integration Points**:

**A) In blockchain_monitor.rs** (lines 384-420)
```rust
// After updating escrow to "completed" or "refunded" status
db_update_escrow_status(&self.db, escrow_id, final_status).await?;

// PHASE 2: Close wallet session - escrow lifecycle complete
info!("🚀 [PHASE 2] Closing wallet session for completed escrow {}", escrow_id);
self.session_manager.close_session(escrow_id)
    .await
    .context("Failed to close wallet session")?;

info!("✅ Wallet session closed - 3 wallets freed");
```

**B) In escrow.rs** (release_funds/refund methods)
- Add similar cleanup calls
- Ensure cleanup happens even if transaction fails
- Use `let _ = session_manager.close_session()` to avoid failing on cleanup errors

**Expected Outcome**:
- Wallets automatically closed when escrow completes
- RPC slots freed for new escrows
- Session map cleaned up

---

### Task 6: Wire Up in main.rs
**File**: `server/src/main.rs`
**Estimated Time**: 20-25 minutes

**Current Structure** (approximate):
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... initialization ...

    let wallet_pool = Arc::new(WalletPool::new(...));
    let wallet_manager = Arc::new(Mutex::new(WalletManager::new(...)));
    let blockchain_monitor = Arc::new(BlockchainMonitor::new(
        wallet_manager.clone(),
        db_pool.clone(),
        ws_server.clone(),
        monitor_config,
    ));

    // ... rest of setup ...
}
```

**New Structure** (Phase 2):
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... initialization ...

    let wallet_pool = Arc::new(WalletPool::new(...));

    // PHASE 2: Create WalletSessionManager
    let session_manager = Arc::new(WalletSessionManager::new(wallet_pool.clone()));
    info!("✅ WalletSessionManager initialized (max 10 concurrent sessions, 2h TTL)");

    let wallet_manager = Arc::new(Mutex::new(WalletManager::new(...)));

    // Pass session_manager to BlockchainMonitor
    let blockchain_monitor = Arc::new(BlockchainMonitor::new(
        wallet_manager.clone(),
        session_manager.clone(),  // NEW
        db_pool.clone(),
        ws_server.clone(),
        monitor_config,
    ));

    // Pass session_manager to EscrowOrchestrator (if modified)
    let escrow_orchestrator = Arc::new(EscrowOrchestrator::new(
        wallet_manager.clone(),
        session_manager.clone(),  // NEW
        wallet_pool.clone(),
    ));

    // ... rest of setup ...
}
```

**Steps**:
1. Add `use crate::services::wallet_session_manager::WalletSessionManager;`
2. Create session_manager after wallet_pool
3. Pass to BlockchainMonitor constructor
4. Pass to EscrowOrchestrator constructor (if Task 4 implemented)
5. Verify compilation
6. Test server startup

**Expected Outcome**:
- Session manager properly initialized
- Passed to all components that need it
- Server starts without errors

---

### Task 7: Background TTL Cleanup Task
**File**: `server/src/main.rs`
**Location**: After server initialization, before `HttpServer::new()`
**Estimated Time**: 10-15 minutes

**Implementation**:
```rust
// PHASE 2: Spawn background task for TTL cleanup
let session_manager_cleanup = session_manager.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(600)); // 10 minutes
    info!("🧹 Background session cleanup task started (runs every 10 min)");

    loop {
        interval.tick().await;

        info!("🧹 Running session TTL cleanup...");
        session_manager_cleanup.cleanup_stale_sessions().await;

        // Log statistics
        let stats = session_manager_cleanup.get_stats().await;
        info!(
            "📊 Session stats: {}/{} active ({}% utilization), avg age: {:?}",
            stats.active_sessions,
            stats.max_sessions,
            stats.utilization_pct,
            stats.avg_session_age
        );
    }
});
```

**Steps**:
1. Clone session_manager for background task
2. Create tokio interval (600 seconds = 10 minutes)
3. Call `cleanup_stale_sessions()` on each tick
4. Log statistics for monitoring
5. Verify task spawns correctly

**Expected Outcome**:
- Stale sessions (>2 hours inactive) automatically cleaned up
- System maintains healthy session count
- Monitoring logs show utilization

---

### Task 8: Integration Testing
**Estimated Time**: 45-60 minutes

**Test Scenarios**:

**Test 1: Session Creation**
```bash
# Initialize 1st escrow
POST /api/orders/{order_id}/init-escrow

# Expected logs:
# "🚀 [PHASE 2] Creating wallet session for escrow {id}"
# "✅ Wallet session created - all wallets open and ready"
# "📊 Session stats: 1/10 active (10% utilization)"
```

**Test 2: Balance Check Performance**
```bash
# Start blockchain monitor (automatic every 30s)
# Watch logs for:
# "🚀 [PHASE 2] Getting wallet from session manager"
# Time should be <1s instead of 3-5s

# Verify wallet NOT closed:
# Should NOT see "✅ Closed wallet"
# Should see "🚀 [PHASE 2] Wallet remains open in session"
```

**Test 3: Multiple Concurrent Escrows**
```bash
# Initialize 3 escrows rapidly
POST /api/orders/{order1}/init-escrow
POST /api/orders/{order2}/init-escrow
POST /api/orders/{order3}/init-escrow

# Expected:
# "📊 Session stats: 3/10 active (30% utilization)"
# All should succeed without RPC collisions
```

**Test 4: Session Cleanup**
```bash
# Complete an escrow (release funds)
POST /api/escrow/{id}/release

# Expected logs:
# "🚀 [PHASE 2] Closing wallet session for completed escrow"
# "✅ Wallet session closed - 3 wallets freed"
# "📊 Session stats: 2/10 active (20% utilization)"
```

**Test 5: LRU Eviction** (if testing limits)
```bash
# Initialize 11th escrow (exceeds max_active_sessions=10)
# Expected:
# "Session limit reached (10/10), evicting LRU session"
# "Evicted LRU session for escrow {oldest_id}"
# "📊 Session stats: 10/10 active (100% utilization)"
```

**Test 6: TTL Cleanup**
```bash
# Wait 10+ minutes with inactive sessions
# Expected logs:
# "🧹 Running session TTL cleanup..."
# "Cleaning up {N} stale sessions (TTL expired)"
# "Cleaned up stale session for escrow {id} (TTL expired)"
```

**Validation Criteria**:
- ✅ Balance checks complete in <1s (was 3-5s)
- ✅ No "closed wallet" logs during active escrow lifecycle
- ✅ Sessions properly created at init
- ✅ Sessions properly closed at completion
- ✅ LRU eviction works when limit reached
- ✅ TTL cleanup runs every 10 minutes
- ✅ Statistics accurately reflect active sessions

---

## 📊 Expected Performance Improvements

### Before Phase 2 (Phase 1 baseline):
- **Balance check**: 3-5s per check (30s interval = 6% overhead)
- **Confirmation check**: 3-5s per check
- **Release funds**: 6-8s (2-3s open + 3-4s tx + 0.3s close)
- **Concurrent capacity**: 2-3 escrows (RPC pool limit: 6 instances)

### After Phase 2 (Target):
- **Balance check**: 0.5-1s per check (85% faster)
- **Confirmation check**: 0.5-1s per check (85% faster)
- **Release funds**: 3-4s (only tx time, no open/close)
- **Concurrent capacity**: 10+ escrows (session limit: 10)

### Latency Breakdown:
| Operation | Phase 1 | Phase 2 | Improvement |
|-----------|---------|---------|-------------|
| Open wallet | 2-3s | 0ms | 100% |
| Close wallet | 0.3s | 0ms | 100% |
| Balance check | 3.5s | 1s | 71% |
| TX confirmation | 4s | 1s | 75% |
| Release funds | 7s | 3.5s | 50% |

---

## 🚨 Critical Considerations

### 1. Session Leak Prevention
**Risk**: If `close_session()` isn't called, sessions accumulate
**Mitigation**:
- TTL cleanup (2 hours) as fallback
- LRU eviction at capacity limit
- Monitoring via `get_stats()`

### 2. RPC Port Conflicts
**Risk**: Session manager wallets may conflict with other operations
**Solution**:
- Session manager uses WalletPool (ports 18082-18086)
- Each session gets dedicated RPC instances
- No conflicts with blockchain_monitor (port 18087)

### 3. Crash Recovery
**Risk**: Server crash leaves wallets open
**Solution**:
- Wallets persist on disk (can be reopened)
- On restart, sessions recreated from DB
- No data loss, only temporary performance hit

### 4. Testing in Production
**Risk**: Phase 2 changes are invasive
**Mitigation**:
- Incremental rollout
- Monitor logs for Phase 2 markers
- Keep Phase 1 code as fallback (git branch)

---

## 📝 Implementation Sequence

**Recommended Order**:
1. ✅ Task 1: Create WalletSessionManager module (DONE)
2. ✅ Task 2: Partial BlockchainMonitor integration (DONE)
3. → **Task 3**: Complete BlockchainMonitor (15 min)
4. → **Task 6**: Wire up in main.rs (20 min)
5. → **Task 7**: Background cleanup task (10 min)
6. → **Task 4**: Session creation at init (30 min)
7. → **Task 5**: Session cleanup at completion (25 min)
8. → **Task 8**: Testing (60 min)

**Total Remaining Effort**: ~2.5-3 hours

**Why This Order**:
- Complete blockchain_monitor first (isolated change)
- Wire up infrastructure before usage (main.rs)
- Add background task early for monitoring
- Add session creation/cleanup last (touches escrow flow)
- Testing validates entire integration

---

## 🎯 Success Criteria

Phase 2 is complete when:
- ✅ All 8 tasks implemented
- ✅ Server compiles without errors
- ✅ All 6 test scenarios pass
- ✅ Performance improvements verified
- ✅ No RPC slot leaks detected
- ✅ Session statistics show healthy utilization
- ✅ Can handle 10+ concurrent escrows
- ✅ Documentation updated

---

## 📚 References

- Phase 1 Analysis: `DOX/DangerZone/WALLET-LIFECYCLE-COMPLETE-ANALYSIS.md`
- Session Manager Code: `server/src/services/wallet_session_manager.rs`
- Integration Example: Lines 157-246 in `blockchain_monitor.rs`
- Monero RPC Docs: https://www.getmonero.org/resources/developer-guides/wallet-rpc.html

---

**Document Status**: Living document - update as tasks complete
**Last Updated**: 2025-11-12
**Next Review**: After Task 3 completion
