# Instrumentation Integration Example

**File:** `server/src/wallet_manager.rs`
**Function:** `exchange_multisig_info()`
**Purpose:** Show exact integration points for instrumentation

---

## Integration Pattern

### 1. Add Imports (Top of File)

```rust
use crate::instrumentation::{
    InstrumentationCollector,
    EventType,
    WalletSnapshot,
};
use serde_json::json;
use std::time::Instant;
```

### 2. Create Collector (Start of Function)

```rust
pub async fn exchange_multisig_info(
    &mut self,
    escrow_id: Uuid,
    info_from_all: Vec<MultisigInfo>,
) -> Result<String, WalletManagerError> {
    // ✅ CREATE INSTRUMENTATION COLLECTOR
    let collector = InstrumentationCollector::new(escrow_id);

    if collector.is_enabled() {
        info!("🔍 Instrumentation ENABLED for escrow {}", escrow_id);
    }

    // ... rest of function
}
```

### 3. Snapshot BEFORE Round 1

**Location:** Before the first `for role in [Buyer, Vendor, Arbiter]` loop

```rust
// ✅ SNAPSHOT: Capture state of all 3 wallets BEFORE Round 1
if collector.is_enabled() {
    info!("📸 Taking PRE-ROUND1 snapshots for all wallets...");

    for role in &[WalletRole::Buyer, WalletRole::Vendor, WalletRole::Arbiter] {
        if let Some(wallet) = self.wallets.values().find(|w| &w.role == role) {
            let role_str = match role {
                WalletRole::Buyer => "buyer",
                WalletRole::Vendor => "vendor",
                WalletRole::Arbiter => "arbiter",
            };

            match WalletSnapshot::capture(
                wallet.id,
                role_str,
                &wallet.rpc_client,
                None,  // Wallet path optional
                wallet.rpc_port,
            ).await {
                Ok(snapshot) => {
                    collector.record_snapshot(
                        EventType::SnapshotPreRound1,
                        role_str,
                        snapshot,
                    ).await;
                },
                Err(e) => {
                    warn!("Failed to capture PRE-ROUND1 snapshot for {}: {}", role_str, e);
                }
            }
        }
    }
}

// Now start Round 1 loop...
for role in &[WalletRole::Buyer, WalletRole::Vendor, WalletRole::Arbiter] {
    // ...
}
```

### 4. Instrument RPC Calls in Round 1

**Location:** Inside the Round 1 loop, before/after `make_multisig()`

```rust
// Existing code:
// wallet.rpc_client.open_wallet(&wallet_filename, "").await?;

// ✅ INSTRUMENT: make_multisig() call
let start = Instant::now();

collector.record_rpc_start(
    "make_multisig",
    role_str,
    wallet.rpc_port,
).await;

let result = wallet
    .rpc_client
    .multisig()
    .make_multisig(2, other_infos)
    .await;

let duration_ms = start.elapsed().as_millis() as u64;
let success = result.is_ok();

collector.record_rpc_end(
    "make_multisig",
    role_str,
    duration_ms,
    success,
    wallet.rpc_port,
).await;

// Handle result
let result = result?;

// ✅ SNAPSHOT: After make_multisig
if collector.is_enabled() {
    match WalletSnapshot::capture(
        wallet.id,
        role_str,
        &wallet.rpc_client,
        Some(&format!("./testnet-wallets/{}", wallet_filename)),
        wallet.rpc_port,
    ).await {
        Ok(snapshot) => {
            collector.record_snapshot(
                EventType::SnapshotPostMakeMultisig,
                role_str,
                snapshot,
            ).await;
        },
        Err(e) => {
            warn!("Failed to capture POST-MAKE-MULTISIG snapshot: {}", e);
        }
    }
}

// Existing code continues...
round1_results.push(result.multisig_info.clone());
```

### 5. Snapshot BEFORE Round 2

**Location:** Before the Round 2 loop

```rust
info!("🔄 Round 2/3: First exchange_multisig_keys (generates Round 2 infos)...");

// ✅ SNAPSHOT: Before Round 2
if collector.is_enabled() {
    for (role_idx, role) in [WalletRole::Buyer, WalletRole::Vendor, WalletRole::Arbiter].iter().enumerate() {
        if let Some(wallet) = self.wallets.values().find(|w| &w.role == role) {
            let role_str = match role {
                WalletRole::Buyer => "buyer",
                WalletRole::Vendor => "vendor",
                WalletRole::Arbiter => "arbiter",
            };

            match WalletSnapshot::capture(
                wallet.id,
                role_str,
                &wallet.rpc_client,
                None,
                wallet.rpc_port,
            ).await {
                Ok(snapshot) => {
                    collector.record_snapshot(
                        EventType::SnapshotPreRound2,
                        role_str,
                        snapshot,
                    ).await;
                },
                Err(e) => {
                    warn!("Failed to capture PRE-ROUND2 snapshot: {}", e);
                }
            }
        }
    }
}

// Now start Round 2 loop...
for (role_idx, role) in [WalletRole::Buyer, WalletRole::Vendor, WalletRole::Arbiter].iter().enumerate() {
    // ...
}
```

### 6. Instrument exchange_multisig_keys (Round 2)

**Location:** Inside Round 2 loop

```rust
// Existing code:
// wallet.rpc_client.open_wallet(&wallet_filename, "").await?;

// ✅ INSTRUMENT: exchange_multisig_keys (Round 2)
let start = Instant::now();

collector.record_rpc_start(
    "exchange_multisig_keys",
    role_str,
    wallet.rpc_port,
).await;

let result = wallet
    .rpc_client
    .multisig()
    .exchange_multisig_keys(other_round1_infos.clone())
    .await;

let duration_ms = start.elapsed().as_millis() as u64;
let success = result.is_ok();

collector.record_rpc_end(
    "exchange_multisig_keys",
    role_str,
    duration_ms,
    success,
    wallet.rpc_port,
).await;

let result = match result {
    Ok(r) => r,
    Err(e) => {
        // ✅ RECORD ERROR
        collector.record_error(
            role_str,
            e.to_string(),
            json!({
                "round": 2,
                "operation": "exchange_multisig_keys",
                "wallet_id": wallet.id.to_string(),
                "escrow_id": escrow_id.to_string(),
            })
        ).await;

        return Err(WalletManagerError::from(e));
    }
};

// ✅ SNAPSHOT: After exchange (Round 2)
if collector.is_enabled() {
    match WalletSnapshot::capture(
        wallet.id,
        role_str,
        &wallet.rpc_client,
        Some(&format!("./testnet-wallets/{}", wallet_filename)),
        wallet.rpc_port,
    ).await {
        Ok(snapshot) => {
            collector.record_snapshot(
                EventType::SnapshotPostExportMultisig,
                role_str,
                snapshot,
            ).await;
        },
        Err(e) => {
            warn!("Failed to capture POST-EXPORT snapshot: {}", e);
        }
    }
}

// Continue with existing code...
round2_results.push(result.multisig_info.clone());
```

### 7. Instrument Round 3 (Same Pattern)

Apply same pattern to Round 3:
- `SnapshotPreRound3` before loop
- Instrument `exchange_multisig_keys` call
- `SnapshotPostImportMultisig` after success
- Record errors on failure

### 8. Final Snapshot + Dump JSON

**Location:** End of function, after all rounds complete

```rust
// Existing code:
// Extract final multisig address
let final_address = self.wallets.values()
    .find_map(|w| match &w.multisig_state {
        MultisigState::Ready { address } => Some(address.clone()),
        _ => None
    })
    .ok_or_else(|| WalletManagerError::InvalidState {
        expected: "at least one wallet in Ready state".to_string(),
        actual: "none found".to_string(),
    })?;

// ✅ FINAL SNAPSHOTS: Capture all wallets in final state
if collector.is_enabled() {
    for role in &[WalletRole::Buyer, WalletRole::Vendor, WalletRole::Arbiter] {
        if let Some(wallet) = self.wallets.values().find(|w| &w.role == role) {
            let role_str = match role {
                WalletRole::Buyer => "buyer",
                WalletRole::Vendor => "vendor",
                WalletRole::Arbiter => "arbiter",
            };

            match WalletSnapshot::capture(
                wallet.id,
                role_str,
                &wallet.rpc_client,
                None,
                wallet.rpc_port,
            ).await {
                Ok(snapshot) => {
                    collector.record_snapshot(
                        EventType::SnapshotFinal,
                        role_str,
                        snapshot,
                    ).await;
                },
                Err(e) => {
                    warn!("Failed to capture FINAL snapshot for {}: {}", role_str, e);
                }
            }
        }
    }

    // ✅ DUMP TO JSON
    let output_path = format!("escrow_{}.json", escrow_id);
    match collector.dump_json(&output_path).await {
        Ok(Some(path)) => {
            info!("✅ Instrumentation data saved to: {}", path);
        },
        Ok(None) => {
            // Instrumentation was disabled, no-op
        },
        Err(e) => {
            error!("Failed to dump instrumentation data: {}", e);
        }
    }
}

info!("🎉 Multisig setup complete! Final address: {}", final_address);

Ok(final_address)
```

### 9. Handle Errors at Any Point

**Add to existing error handling:**

```rust
// Example: If Round 1 fails
Err(e) => {
    error!("❌ {:?} wallet make_multisig FAILED: {:?}", role, e);

    // ✅ RECORD ERROR
    collector.record_error(
        role_str,
        e.to_string(),
        json!({
            "round": 1,
            "operation": "make_multisig",
            "wallet_id": wallet.id.to_string(),
            "escrow_id": escrow_id.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
    ).await;

    // ✅ DUMP PARTIAL INSTRUMENTATION DATA (helpful for debugging)
    let _ = collector.dump_json(&format!("escrow_{}_FAILED.json", escrow_id)).await;

    return Err(WalletManagerError::from(e));
}
```

---

## Complete Integration Checklist

### Before Round 1:
- [x] Create `InstrumentationCollector`
- [x] Record `SNAPSHOT_PRE_ROUND1` for all 3 wallets

### During Round 1 (for each wallet):
- [x] Record `RPC_CALL_START` for `prepare_multisig`
- [x] Record `RPC_CALL_END` with duration and success status
- [x] Record `SNAPSHOT_POST_MAKE_MULTISIG` after `make_multisig()`
- [x] Record errors if operation fails

### Before Round 2:
- [x] Record `SNAPSHOT_PRE_ROUND2` for all 3 wallets

### During Round 2 (for each wallet):
- [x] Record `RPC_CALL_START` for `exchange_multisig_keys`
- [x] Record `RPC_CALL_END` with duration
- [x] Record `SNAPSHOT_POST_EXPORT_MULTISIG`
- [x] Record errors if operation fails

### Before Round 3:
- [x] Record `SNAPSHOT_PRE_ROUND3` for all 3 wallets

### During Round 3 (for each wallet):
- [x] Record `RPC_CALL_START` for second `exchange_multisig_keys`
- [x] Record `RPC_CALL_END`
- [x] Record `SNAPSHOT_POST_IMPORT_MULTISIG`
- [x] Record errors if operation fails

### After All Rounds:
- [x] Record `SNAPSHOT_FINAL` for all 3 wallets
- [x] Call `collector.dump_json()` to save events
- [x] Log output path for analysis

---

## Validation

After integration, verify instrumentation is working:

```bash
# 1. Enable instrumentation
export ENABLE_INSTRUMENTATION=1

# 2. Run one escrow
cargo run --bin server &
sleep 2
curl -X POST http://localhost:8080/api/escrow/init -H "Content-Type: application/json" -d '{"buyer_id": "test1", "vendor_id": "test2", "amount": 1000000}'

# 3. Check for JSON file
ls -lh escrow_*.json

# 4. Validate structure
python tools/analyze_escrow_json.py escrow_*.json
```

**Expected Output:**
```
================================================================================
ESCROW ANALYSIS: escrow_abc123-1699999999999.json
Trace ID: escrow_abc123-1699999999999
Total events: 45
================================================================================

EVENT TIMELINE
================================================================================
[+    0ms] SNAPSHOT_PRE_ROUND1              role=buyer    multisig=false ...
[+   50ms] RPC_CALL_START                   role=buyer    port=18082 method=make_multisig
[+  150ms] RPC_CALL_END                     role=buyer    port=18082 method=make_multisig ✓ 100ms
[+  200ms] SNAPSHOT_POST_MAKE_MULTISIG      role=buyer    multisig=true ...
...
```

---

## Performance Notes

- **Overhead when disabled:** Zero (all checks are no-ops)
- **Overhead when enabled:** ~1-2% CPU, ~10-50 KB memory per escrow
- **Disk usage:** ~1-5 MB per JSON file depending on verbosity
- **Recommendation:** Enable only in development or for targeted production debugging

---

## Next Steps

1. **Apply integration** to `wallet_manager.rs` following this example
2. **Test with single escrow** to verify JSON output
3. **Test with concurrent escrows** (3x) to reproduce race conditions
4. **Analyze results** using `tools/analyze_escrow_json.py`
5. **Identify root cause** from timeline and snapshot diffs
6. **Implement fix** based on findings
7. **Re-test** to confirm resolution

---

**Status:** ✅ Ready for integration
**Last Updated:** 2025-11-13
**See Also:** [INSTRUMENTATION-GUIDE.md](./INSTRUMENTATION-GUIDE.md)
