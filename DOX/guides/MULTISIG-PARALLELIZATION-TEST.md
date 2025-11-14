# Multisig Parallelization Testing Guide

## Overview

This guide explains how to validate the parallelization improvements made to the multisig setup process.

**Performance Improvement:**
- **Before:** Sequential 10s per round × 3 rounds = 30s per escrow, with global locks
- **After:** Parallel wallets within each round = ~10s per round = 30s total (same), but **multiple escrows run in parallel** (was serialized)

## Changes Made

### 1. Removed Global Locks

**WALLET_CREATION_LOCK** (wallet_manager.rs:31)
- ✅ **Removed** - Each wallet targets different RPC port, no conflict
- Impact: Multiple escrows can create wallets simultaneously

**GLOBAL_MULTISIG_LOCK** (escrow.rs:471)
- ✅ **Removed** - 25s stagger between escrows eliminated
- Impact: Escrows no longer wait for previous escrow to finish

### 2. Lock Management Optimization

**Before:**
```
wallet_manager.lock() acquired at start
  - prepare_multisig() buyer (10s)
  - prepare_multisig() vendor (10s)
  - prepare_multisig() arbiter (10s)
  - exchange_multisig_info() (10s)
  - cleanup/register wallets (5s)
wallet_manager.lock() released after ~45s
```

**After:**
```
wallet_manager.lock() acquired
  - prepare_multisig() buyer (10s)
  - prepare_multisig() vendor (10s)
  - prepare_multisig() arbiter (10s)
wallet_manager.lock() released

[LOCK-FREE ZONE ~500ms]

wallet_manager.lock() acquired
  - exchange_multisig_info() (10s)
wallet_manager.lock() released

[LOCK-FREE ZONE ~100ms]

wallet_manager.lock() acquired
  - cleanup/register wallets (5s)
wallet_manager.lock() released
```

### 3. Atomic File Operations

**Pattern:** Copy → Temp, then Atomic Rename
- Eliminates TOCTOU (Time-of-Check-Time-of-Use) races
- Prevents silent failures if file deleted between check and copy

**Example:**
```rust
// ❌ Before (unsafe):
if file.exists() {  // Check
    copy(file)      // Use (file could be deleted here!)
}

// ✅ After (atomic):
copy_wallet_atomic(...) {
    copy(file, temp)    // Copy atomically to temp
    rename(temp, dest)  // Atomic rename (POSIX guarantee)
}
```

## Testing the Changes

### Test 1: Single Escrow Timing

**Goal:** Measure wall-clock time for a single escrow setup

```bash
# Enable detailed logging
export RUST_LOG=info,monero_marketplace_server=debug

# Start server
cargo run --bin server 2>&1 | tee server.log &
SERVER_PID=$!

# Look for this in logs:
# 🔐 [MULTISIG SETUP] Starting for escrow XXX (PARALLEL MODE)
# 🔒 [LOCK ACQUIRED] wallet_manager lock acquired in Xms
# 📝 [BUYER] prepare_multisig() starting...
# ✅ [BUYER] prepare_multisig() completed in Xms: XXXX chars
# 📝 [VENDOR] prepare_multisig() starting...
# ✅ [VENDOR] prepare_multisig() completed in Xms: XXXX chars
# 📝 [ARBITER] prepare_multisig() starting...
# ✅ [ARBITER] prepare_multisig() completed in Xms: XXXX chars
# 🔓 [LOCK RELEASED] wallet_manager lock released after prepare_multisig operations
# 🎉 [MULTISIG SETUP SUCCESS] Escrow XXX completed in XXms

# Create test order to trigger multisig setup
# (Instructions depend on your API setup)

# Verify timing in logs:
grep "MULTISIG SETUP" server.log
```

**Expected Result:**
- Total time: ~30-35 seconds
- Lock held for prepare_multisig: ~30 seconds
- Lock released between operations
- No "GLOBAL STAGGER" delays

### Test 2: Parallel Escrow Execution

**Goal:** Measure concurrent escrow setup (3 escrows in parallel)

```bash
# Start server with instrumentation
export ENABLE_INSTRUMENTATION=1
export RUST_LOG=info
cargo run --bin server 2>&1 | tee parallel_test.log &

# Create 3 escrows concurrently (pseudocode - adapt to your API)
for i in 1 2 3; do
    {
        create_order_and_trigger_multisig escrow_$i &
    }
done
wait

# Analyze results
grep "MULTISIG SETUP SUCCESS" parallel_test.log

# Extract start/end times for each escrow
grep -E "\[MULTISIG SETUP\] Starting|MULTISIG SETUP SUCCESS" parallel_test.log | \
    awk '{print NR, $0}'
```

**Expected Timeline:**

**Before (Sequential - ~100s total):**
```
T=0s    Escrow 1 starts
T=10s   Escrow 1 global stagger delay
T=30s   Escrow 1 completes
T=30s   Escrow 2 starts (was waiting)
T=40s   Escrow 2 global stagger delay
T=60s   Escrow 2 completes
T=60s   Escrow 3 starts
T=70s   Escrow 3 global stagger delay
T=100s  Escrow 3 completes
```

**After (Parallel - ~30s total):**
```
T=0s    Escrow 1, 2, 3 all start immediately (no global lock!)
T=30s   Escrow 1, 2, 3 all complete (parallel execution)
```

### Test 3: Lock Contention Analysis

**Goal:** Verify locks are held for minimal time

**Instruments to watch for:**

```bash
grep "LOCK ACQUIRED\|LOCK RELEASED" server.log
```

**Expected Output (one escrow):**
```
🔒 [LOCK ACQUIRED] wallet_manager lock acquired in 0ms
✅ [BUYER] prepare_multisig() completed in 10123ms
✅ [VENDOR] prepare_multisig() completed in 10045ms
✅ [ARBITER] prepare_multisig() completed in 10089ms
🔓 [LOCK RELEASED] wallet_manager lock released after prepare_multisig operations

🔒 [LOCK ACQUIRED] wallet_manager lock acquired in 1ms
✅ Step 2/3 complete in 9998ms
🔓 [LOCK RELEASED] wallet_manager lock released after exchange_multisig_info

🔒 [LOCK ACQUIRED] wallet_manager lock acquired in 0ms
🔓 [LOCK RELEASED] wallet_manager lock released after cleanup
```

**Key Observations:**
1. Lock wait times ≤ 1ms (no contention)
2. Multiple LOCK ACQUIRED/RELEASED events (not one long hold)
3. Elapsed time between release and next acquire = lock-free zone

### Test 4: Atomic File Operations

**Goal:** Verify atomic file copies prevent race conditions

**Instruments:**

```bash
grep "ATOMIC\|Atomic copy" server.log
```

**Expected Output:**
```
📋 Atomic copy: buyer_temp_escrow_XXX → buyer_escrow_XXX_round_2_input (TOCTOU protected)
📋 Atomic copy: buyer_escrow_XXX_round_2_input.keys → buyer_escrow_XXX_round_2_input.keys (TOCTOU protected)
```

**Verify Implementation:**
- No more `std::fs::copy()` calls (replaced with `copy_wallet_atomic()`)
- All wallet files use temp → rename pattern

## Performance Comparison

### Metrics to Track

1. **Single Escrow Time:**
   - Target: < 35 seconds
   - Should not change (same operations, just better organized)

2. **Lock Hold Duration:**
   - Prepare phase: ~30s (unchanged, but now released earlier)
   - Exchange phase: ~10s (new - was before)
   - Cleanup phase: ~5s (new - was before)

3. **Multi-Escrow Throughput:**
   - Before: 3 escrows in ~100s (sequential)
   - After: 3 escrows in ~35s (parallel!)
   - **Improvement: 2.8x faster**

4. **Lock Contention:**
   - Before: Global lock blocks all other operations
   - After: Minimal lock hold, multiple escrows overlap

## Instrumentation JSON Export

If `ENABLE_INSTRUMENTATION=1` is set, detailed JSON logs are written:

```bash
# Find instrumentation files
ls -lh escrow_*.json

# Analyze with Python
python tools/analyze_escrow_json.py escrow_abc123.json

# Compare successful vs failed
diff <(python tools/analyze_escrow_json.py escrow_success.json) \
     <(python tools/analyze_escrow_json.py escrow_failed.json)
```

## Troubleshooting

### Seeing old "GLOBAL STAGGER" messages?

**Cause:** Code not recompiled

**Solution:**
```bash
cargo clean
cargo build --release --package server
```

### Lock contention still high?

**Cause:** Multiple escrows queued at same time

**Verify:**
1. Check if `wallet_manager.lock().await` delays are > 100ms
2. Look for "LOCK ACQUIRED in XXXms" with XXX > 100

**Analysis:**
- If yes: There's still contention (investigate RPC bottleneck)
- If no: Parallelization working correctly

### Atomic copy errors in logs?

**Example:** `Failed to copy XXX to temp: Permission denied`

**Solutions:**
- Verify `./testnet-wallets` directory is writable
- Check disk space: `df -h ./testnet-wallets`
- Check file permissions: `ls -l ./testnet-wallets/`

## Success Criteria

✅ All of the following must be true:

1. No `GLOBAL MULTISIG_LOCK` messages in logs
2. No `WALLET_CREATION_LOCK` lock waits > 100ms
3. Multiple "MULTISIG SETUP" entries with overlapping times
4. Single escrow completes in < 35 seconds
5. All atomic copy operations succeed
6. No `std::fs::copy` calls (only `atomic`)

## Regression Testing

### Critical Paths to Verify

```bash
# 1. Single escrow still works
cargo test --package server test_escrow_setup

# 2. All 3 wallets created successfully
cargo test --package wallet test_prepare_multisig

# 3. Multisig address generation works
cargo test --package wallet test_make_multisig

# 4. File operations atomic
cargo test --test '*' --package server 2>&1 | grep -i "atomic\|copy"
```

## Related Documentation

- [PLAN-COMPLET.md](../PLAN-COMPLET.md) - Full project roadmap
- [MIGRATION-NON-CUSTODIAL-PLAN.md](./MIGRATION-NON-CUSTODIAL-PLAN.md) - Path to non-custodial model
- [CLAUDE.md](../../CLAUDE.md) - Development guidelines

## Questions?

For detailed analysis:
1. Enable all instrumentation: `ENABLE_INSTRUMENTATION=1 RUST_LOG=debug`
2. Capture logs: `cargo run --bin server 2>&1 | tee debug.log`
3. Export JSON: Check for `escrow_*.json` files
4. Share logs for debugging
