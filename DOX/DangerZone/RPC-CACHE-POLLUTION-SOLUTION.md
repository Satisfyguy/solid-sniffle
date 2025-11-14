# RPC Cache Pollution Solution - Phase 3.5

**Date:** 2025-11-13  
**Status:** Implemented + Testing Required  
**Issue:** "pubkey recommended" errors persist despite 3s cache purge delay

---

## Problem Analysis

### Test Results (15:50:37-40)
User tested 3 concurrent escrows after implementing Phase 3 (3s delay):
- `7e8dc04a-c8dd-473e-a71e-7a74da3a4682` → HTTP 500 (Failed)
- `20671d82-8fc9-40f5-abd8-f2ec481301e5` → HTTP 500 (Failed)
- `dbb2b706-7f0f-4d93-88d4-37dd40f4a079` → HTTP 500 (Failed)

**Result:** 0/3 success rate

### Root Cause (15:50:24)
```
ERROR: Buyer wallet round 2 FAILED: A pubkey recommended by multisig kex messages
       had an unexpected number of recommendations.
WARN:  RPC CACHE POLLUTION DETECTED for wallet 00c4c52f-06ac-4942-9eba-3bba4050ed04
```

**Diagnosis:** The 3-second delay is **insufficient** for monero-wallet-rpc daemon to fully purge its internal multisig state cache.

---

## Solution: Phase 3.5 - Extended Cache Purge Delay

### Implementation
Increased RPC cache purge delay from **3 seconds → 10 seconds** in two critical locations:

#### 1. `server/src/wallet_manager.rs:1538`
**Context:** RPC cache pollution detection/recovery
```rust
let _ = wallet.rpc_client.close_wallet().await;
tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;  // Was 3s
```

#### 2. `server/src/services/escrow.rs:614`
**Context:** Post-cleanup delay in Phase 3 flow
```rust
// [PHASE 3] Post-cleanup delay: Let RPC daemon purge internal cache
tokio::time::sleep(Duration::from_secs(10)).await;  // Was 3s
```

### Rationale
monero-wallet-rpc daemon's internal cache management is slower than anticipated:
- **3s delay:** Insufficient (still seeing "pubkey recommended" errors)
- **10s delay:** Conservative estimate to ensure complete cache purge
- **Trade-off:** +7s latency per escrow vs eliminating RPC cache pollution

---

## Testing Protocol

### Test Scenario
Create **3 concurrent escrows** via UI at http://localhost:8080

### Expected Results (90%+ Success Rate)
- **Phase 1 (25s global stagger):** Prevents concurrent multisig operations
- **Phase 2 (Guaranteed cleanup):** Ensures all wallet files deleted
- **Phase 3.5 (10s RPC cache purge):** Eliminates "pubkey recommended" errors

**Target:** 2/3 or 3/3 successful escrow creations

---

## Deployment

**Binary:** `target/release/server` (built 2025-11-13 17:16:48)  
**Server:** Running on http://localhost:8080 (PID 748779)  
**Ready for testing**
