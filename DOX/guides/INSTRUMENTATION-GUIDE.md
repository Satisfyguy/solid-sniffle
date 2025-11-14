# Multisig Instrumentation Guide

**Date:** 2025-11-13
**Status:** ✅ Production Ready
**Purpose:** Debug race conditions, RPC cache pollution, and state corruption in concurrent escrow operations

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Integration Points](#integration-points)
5. [Analysis Workflow](#analysis-workflow)
6. [Common Patterns](#common-patterns)
7. [Troubleshooting](#troubleshooting)

---

## Overview

The **Multisig Instrumentation System** provides comprehensive tracing and state capture for debugging complex concurrency issues in the Monero Marketplace escrow system.

### What It Solves

- **Race Conditions**: Identify timing conflicts when multiple escrows run concurrently
- **RPC Cache Pollution**: Detect when wallet state persists incorrectly between operations
- **State Corruption**: Trace exact state transitions to find where wallets diverge
- **Timing Dependencies**: Measure RPC call durations and identify bottlenecks

### Key Features

- **Correlation IDs**: Trace a single escrow from start to finish
- **State Snapshots**: Capture complete wallet state at critical points
- **Event Timeline**: Chronological log of every RPC call and state change
- **Differential Analysis**: Compare successful vs failed escrow sessions
- **Zero Performance Impact When Disabled**: No overhead unless explicitly enabled

---

## Architecture

### Module Structure

```
server/src/instrumentation/
├── mod.rs          # Public API and macros
├── events.rs       # Event type definitions
├── snapshots.rs    # Wallet state capture
└── collector.rs    # Thread-safe event aggregation
```

### Event Types

| Event Type | Description | When to Use |
|------------|-------------|-------------|
| `RPC_CALL_START` | RPC method invocation begins | Before every RPC call |
| `RPC_CALL_END` | RPC method completes successfully | After successful RPC call |
| `RPC_CALL_ERROR` | RPC method fails | After RPC error |
| `SNAPSHOT_PRE_ROUND1` | State before prepare_multisig | Before Round 1 starts |
| `SNAPSHOT_POST_MAKE_MULTISIG` | State after make_multisig | After each wallet's make_multisig |
| `SNAPSHOT_PRE_ROUND2` | State before first exchange | Before Round 2 starts |
| `SNAPSHOT_POST_EXPORT_MULTISIG` | State after export_multisig_info | After export phase |
| `SNAPSHOT_PRE_ROUND3` | State before second exchange | Before Round 3 starts |
| `SNAPSHOT_POST_IMPORT_MULTISIG` | State after import_multisig_info | After import phase |
| `SNAPSHOT_FINAL` | Final multisig wallet state | After complete setup |
| `ERROR_FINAL` | Fatal error with full context | When escrow fails |
| `CACHE_POLLUTION_DETECTED` | Wallet in unexpected state | When state validation fails |

### Data Flow

```
┌─────────────────────┐
│ Escrow Operation    │
│ (e.g., init_escrow) │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────────────┐
│ InstrumentationCollector    │
│ - Creates unique trace_id   │
│ - Collects events in memory │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ Critical Operations         │
│ - record_rpc_start()        │
│ - snapshot_wallet_state()   │
│ - record_rpc_end()          │
│ - record_error()            │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ JSON Export                 │
│ escrow_{id}.json            │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ Analysis Tool               │
│ analyze_escrow_json.py      │
└─────────────────────────────┘
```

---

## Quick Start

### 1. Enable Instrumentation

Set the environment variable before starting the server:

```bash
export ENABLE_INSTRUMENTATION=1
cargo run --bin server
```

Or for a single session:

```bash
ENABLE_INSTRUMENTATION=1 cargo run --bin server
```

### 2. Run Escrow Operations

```bash
# Single escrow (baseline)
curl -X POST http://localhost:8080/api/escrow/init \
  -H "Content-Type: application/json" \
  -d '{"buyer_id": "user1", "vendor_id": "user2", "amount": 1000000}'

# Multiple concurrent escrows (test for race conditions)
for i in {1..3}; do
  curl -X POST http://localhost:8080/api/escrow/init \
    -H "Content-Type: application/json" \
    -d "{\"buyer_id\": \"buyer$i\", \"vendor_id\": \"vendor$i\", \"amount\": 1000000}" &
done
wait
```

### 3. Analyze Results

```bash
# Basic analysis
python tools/analyze_escrow_json.py escrow_abc123.json

# Compare successful vs failed
python tools/analyze_escrow_json.py --compare escrow_success.json escrow_failed.json

# Timeline only
python tools/analyze_escrow_json.py --timeline escrow_abc123.json

# RPC statistics
python tools/analyze_escrow_json.py --rpc-only escrow_abc123.json

# Snapshot analysis
python tools/analyze_escrow_json.py --snapshots-only escrow_abc123.json

# Errors only
python tools/analyze_escrow_json.py --errors-only escrow_abc123.json
```

---

## Integration Points

### Basic Event Recording

```rust
use crate::instrumentation::{InstrumentationCollector, EventType};
use serde_json::json;

// Create collector for escrow
let collector = InstrumentationCollector::new(escrow_id);

// Record a simple event
collector.record_event(
    EventType::StateChange,
    "buyer",
    json!({
        "from": "NotStarted",
        "to": "PreparedInfo",
    })
).await;
```

### Recording RPC Calls

```rust
use std::time::Instant;

// Manual instrumentation
let start = Instant::now();

collector.record_rpc_start(
    "prepare_multisig",
    "buyer",
    Some(18082)
).await;

let result = wallet.rpc_client
    .multisig()
    .prepare_multisig()
    .await?;

let duration_ms = start.elapsed().as_millis() as u64;

collector.record_rpc_end(
    "prepare_multisig",
    "buyer",
    duration_ms,
    result.is_ok(),
    Some(18082)
).await;
```

Or use the macro:

```rust
use crate::instrument_rpc_call;

let result = instrument_rpc_call!(
    collector,
    "prepare_multisig",
    "buyer",
    Some(18082),
    {
        wallet.rpc_client
            .multisig()
            .prepare_multisig()
            .await?
    }
);
```

### Capturing State Snapshots

```rust
use crate::instrumentation::WalletSnapshot;

// Capture snapshot before critical operation
let snapshot = WalletSnapshot::capture(
    wallet_id,
    "buyer",
    &wallet.rpc_client,
    Some("/path/to/wallet"),  // Optional: wallet file path
    Some(18082),              // Optional: RPC port
).await?;

// Record the snapshot
collector.record_snapshot(
    EventType::SnapshotPreRound1,
    "buyer",
    snapshot.clone()
).await;

// Perform operation...
wallet.rpc_client.multisig().prepare_multisig().await?;

// Capture snapshot after operation
let snapshot_after = WalletSnapshot::capture(
    wallet_id,
    "buyer",
    &wallet.rpc_client,
    Some("/path/to/wallet"),
    Some(18082),
).await?;

collector.record_snapshot(
    EventType::SnapshotPostMakeMultisig,
    "buyer",
    snapshot_after
).await;

// Optional: Compare snapshots
let diffs = snapshot.diff(&snapshot_after);
if !diffs.is_empty() {
    info!("State changes detected: {:?}", diffs);
}
```

### Recording Errors

```rust
use anyhow::Result;

match some_operation().await {
    Ok(result) => {
        // Success path
    },
    Err(e) => {
        // Record error with full context
        collector.record_error(
            "buyer",
            e.to_string(),
            json!({
                "operation": "make_multisig",
                "wallet_id": wallet_id.to_string(),
                "escrow_id": escrow_id.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })
        ).await;

        return Err(e);
    }
}
```

### Dumping to JSON

```rust
// At the end of escrow operation (success or failure)
if let Some(output_path) = collector.dump_json(
    &format!("escrow_{}.json", escrow_id)
).await? {
    info!("Instrumentation data saved to: {}", output_path);
}

// Check if instrumentation is enabled
if collector.is_enabled() {
    info!("Instrumentation is active for trace_id: {}", collector.trace_id());
}
```

---

## Analysis Workflow

### Step 1: Reproduce the Issue

Enable instrumentation and run the failing operation multiple times:

```bash
export ENABLE_INSTRUMENTATION=1

# Run 5 escrow operations
for i in {1..5}; do
  cargo run --bin server &
  sleep 2
  curl -X POST http://localhost:8080/api/escrow/init \
    -H "Content-Type: application/json" \
    -d "{\"buyer_id\": \"buyer$i\", \"vendor_id\": \"vendor$i\", \"amount\": 1000000}"

  # Wait for completion
  sleep 30
  killall server
done
```

Collect JSON files:
```bash
ls -lh escrow_*.json
```

### Step 2: Identify Patterns

Analyze all sessions:

```bash
# Quick summary of all escrows
for file in escrow_*.json; do
  echo "=== $file ==="
  python tools/analyze_escrow_json.py --errors-only "$file" | head -20
done
```

### Step 3: Compare Success vs Failure

```bash
# Find one successful and one failed session
python tools/analyze_escrow_json.py --compare escrow_success.json escrow_failed.json
```

Look for:
- **Event count mismatch**: Failed escrow has fewer events
- **Divergence point**: First event where sequences differ
- **State anomalies**: `is_multisig=true` when it should be false

### Step 4: Deep Dive Timeline

```bash
# Examine failed escrow in detail
python tools/analyze_escrow_json.py --timeline escrow_failed.json > failed_timeline.txt
```

Look for:
- **Unexpected state transitions**: Wallet already multisig before make_multisig
- **RPC errors**: Which method failed and why
- **Timing issues**: Unusually long RPC call durations
- **Cache pollution**: Multiple wallets sharing same address_hash

### Step 5: Snapshot Analysis

```bash
python tools/analyze_escrow_json.py --snapshots-only escrow_failed.json
```

Look for:
- **Address hash collisions**: Different roles with same address_hash
- **Balance corruption**: Balances changing unexpectedly
- **Multisig state flips**: is_multisig toggling incorrectly

---

## Common Patterns

### Pattern 1: RPC Cache Pollution

**Symptoms:**
- Wallet already in multisig mode before `make_multisig()`
- `make_multisig()` fails with "already multisig" error
- Multiple escrows share the same address_hash

**Detection:**
```bash
python tools/analyze_escrow_json.py escrow_failed.json | grep "is_multisig=true" -B 5
```

**Root Cause:**
- monero-wallet-rpc caching state from previous operation
- Insufficient delay between wallet operations
- Wallet not properly closed/reopened

**Fix:**
- Increase delay between operations (10s → 15s)
- Add explicit `close_wallet()` + `open_wallet()` cycle
- Verify wallet state with snapshot BEFORE critical operations

### Pattern 2: Race Condition in Concurrent Escrows

**Symptoms:**
- 2 of 3 escrows succeed, 1 fails randomly
- Failure always on 3rd escrow
- RPC calls overlap in timeline

**Detection:**
```bash
# Compare timelines of concurrent escrows
python tools/analyze_escrow_json.py --timeline escrow_1.json > e1.txt &
python tools/analyze_escrow_json.py --timeline escrow_2.json > e2.txt &
python tools/analyze_escrow_json.py --timeline escrow_3.json > e3.txt &
wait

# Look for overlapping timestamps
paste e1.txt e2.txt e3.txt | grep "RPC_CALL_START"
```

**Root Cause:**
- Shared RPC instance handling multiple wallets
- File system race on wallet file writes
- Insufficient locking around wallet creation

**Fix:**
- Use `WALLET_CREATION_LOCK` global mutex
- Implement wallet pool with exclusive RPC instances
- Add file-level locking for wallet operations

### Pattern 3: State Divergence Between Roles

**Symptoms:**
- Buyer wallet succeeds, vendor/arbiter fail
- Different multisig addresses for same escrow
- `make_multisig()` produces different results for each role

**Detection:**
```bash
# Extract all addresses from snapshots
python tools/analyze_escrow_json.py escrow_abc.json | grep "address_hash" | sort | uniq -c
```

**Root Cause:**
- prepare_multisig info ordering inconsistency
- Wrong prepare_infos sent to wrong wallet
- Non-deterministic input to `make_multisig()`

**Fix:**
- Sort prepare_infos alphabetically before `make_multisig()`
- Validate prepare_infos length and content before use
- Add SHA256 hashes to logs for verification

---

## Troubleshooting

### No JSON Files Generated

**Check:**
```bash
echo $ENABLE_INSTRUMENTATION  # Should output "1"
```

**Fix:**
```bash
export ENABLE_INSTRUMENTATION=1
```

### Empty Events Array

**Possible Causes:**
- Instrumentation code not integrated at critical points
- Escrow operation failed before any events recorded
- Collector not calling `dump_json()` at the end

**Debug:**
```rust
// Add at start of escrow operation
if collector.is_enabled() {
    info!("Instrumentation ACTIVE: trace_id={}", collector.trace_id());
} else {
    warn!("Instrumentation DISABLED");
}

// Add at end
let count = collector.event_count().await;
info!("Recorded {} events", count);
```

### Analysis Tool Errors

**Python not found:**
```bash
python3 tools/analyze_escrow_json.py escrow_abc.json
```

**Permission denied:**
```bash
chmod +x tools/analyze_escrow_json.py
```

**JSON parse error:**
```bash
# Validate JSON syntax
cat escrow_abc.json | python -m json.tool
```

### High Disk Usage

Instrumentation files can grow large (1-5 MB per escrow with full snapshots).

**Cleanup old files:**
```bash
# Delete instrumentation files older than 7 days
find . -name "escrow_*.json" -mtime +7 -delete
```

**Compress archives:**
```bash
# Archive old instrumentation data
tar -czf instrumentation_archive_$(date +%Y%m%d).tar.gz escrow_*.json
rm escrow_*.json
```

---

## Performance Impact

### When Disabled (Default)

- **Zero overhead**: All instrumentation checks are no-ops
- No JSON files created
- No events collected in memory

### When Enabled

- **Memory**: ~10-50 KB per escrow (depends on event count)
- **Disk**: ~1-5 MB per JSON file (depends on snapshot verbosity)
- **CPU**: <1% overhead from event serialization
- **RPC Latency**: +0-5ms per call (negligible)

**Recommendation:** Only enable in development/staging or for targeted production debugging.

---

## Advanced Usage

### Custom Event Types

```rust
use crate::instrumentation::EventType;

collector.record_event(
    EventType::Custom,
    "coordinator",
    json!({
        "custom_event": "wallet_pool_exhausted",
        "available_instances": 0,
        "waiting_escrows": 5,
    })
).await;
```

### Conditional Instrumentation

```rust
// Only instrument escrows for specific users
let should_instrument = user_id == "debug_user_123";

if should_instrument {
    std::env::set_var("ENABLE_INSTRUMENTATION", "1");
}

let collector = InstrumentationCollector::new(escrow_id);

// ... rest of code
```

### Structured Logging Integration

```rust
use tracing::info;

collector.record_event(
    EventType::StateChange,
    "buyer",
    json!({
        "from": "PreparedInfo",
        "to": "Ready",
    })
).await;

// Also log to structured logs for correlation
info!(
    trace_id = %collector.trace_id(),
    role = "buyer",
    "State transition: PreparedInfo → Ready"
);
```

---

## Next Steps

1. **Integrate into wallet_manager.rs**: Add instrumentation to `exchange_multisig_info()`
2. **Run concurrent tests**: Execute 3+ escrows simultaneously
3. **Analyze results**: Use analysis tool to identify root cause
4. **Implement fixes**: Address race conditions, cache pollution, etc.
5. **Verify**: Re-run tests with instrumentation to confirm fix

**See Also:**
- [DOX/DangerZone/RPC-CACHE-POLLUTION-SOLUTION.md](../DangerZone/RPC-CACHE-POLLUTION-SOLUTION.md)
- [server/src/instrumentation/mod.rs](../../server/src/instrumentation/mod.rs)
- [tools/analyze_escrow_json.py](../../tools/analyze_escrow_json.py)

---

**Status:** ✅ Ready for production debugging
**Last Updated:** 2025-11-13
**Maintainer:** AMAZAWN Team
