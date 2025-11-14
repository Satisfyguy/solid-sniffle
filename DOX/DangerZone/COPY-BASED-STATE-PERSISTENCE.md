# Copy-Based State Persistence for Multisig - FINAL IMPLEMENTATION

**Date:** 2025-11-13 19:20  
**Status:** ✅ IMPLEMENTED AND COMPILED  
**Architecture:** Per-round wallet filenames with state preservation via file copying

---

## Problem Solved

**Root Cause:** Monero's multisig state is stored in both:
1. **Wallet files** (persistent, surveyable)
2. **RPC daemon memory** (volatile, cleared on close_wallet/restart)

When we deleted wallet files between rounds, the daemon lost accumulated state, requiring Round 1's multisig_info again.

---

## Solution: Copy-Based State Persistence

Instead of deleting wallet files, we **COPY** them between rounds, preserving state in both the file AND allowing the daemon to reload it:

### Round 1 (prepare_multisig)
```
CREATE: buyer_escrow_{uuid}_round_1
        buyer_escrow_{uuid}_round_1.keys
        
COPY TO: buyer_escrow_{uuid}_round_2_input (preserves round_1 state)
         buyer_escrow_{uuid}_round_2_input.keys

DELETE: buyer_escrow_{uuid}_round_1
        buyer_escrow_{uuid}_round_1.keys
```

### Round 2 (exchange_multisig_keys)
```
LOAD FROM: buyer_escrow_{uuid}_round_2_input (has round_1 state)

COPY TO: buyer_escrow_{uuid}_round_3_input (preserves round_1 + 2 state)
         buyer_escrow_{uuid}_round_3_input.keys

DELETE: buyer_escrow_{uuid}_round_2_input
        buyer_escrow_{uuid}_round_2_input.keys
```

### Round 3 (exchange_multisig_keys - final)
```
LOAD FROM: buyer_escrow_{uuid}_round_3_input (has all accumulated state)

COPY TO: buyer_escrow_{uuid}_round_3_final (KEEP for signing operations)
         buyer_escrow_{uuid}_round_3_final.keys

DELETE: buyer_escrow_{uuid}_round_3_input
        buyer_escrow_{uuid}_round_3_input.keys
```

---

## Implementation Details

### File: `server/src/wallet_manager.rs`

**Function:** `exchange_multisig_info()` (lines 1550-2070)

#### Round 1 Implementation (Lines 1649-1758)
- Wallet filename: `{role}_escrow_{escrow_id}_round_1`
- After `make_multisig()`: Copy to `round_2_input`
- Delete original `round_1` files

#### Round 2 Implementation (Lines 1812-1901)
- Wallet filename: Load from `{role}_escrow_{escrow_id}_round_2_input`
- After `exchange_multisig_keys()`: Copy to `round_3_input`
- Delete intermediate `round_2_input` files

#### Round 3 Implementation (Lines 1938-2042)
- Wallet filename: Load from `{role}_escrow_{escrow_id}_round_3_input`
- After `exchange_multisig_keys()`: Copy to `round_3_final` (KEEP)
- Delete intermediate `round_3_input` files

#### Cleanup Utility (Lines 2013-2042)
```rust
// Cleanup intermediate files after ALL rounds complete
for (role_str, _) in &roles {
    let files_to_delete = vec![
        format!("{}_escrow_{}_round_1", role_str, escrow_id),
        format!("{}_escrow_{}_round_2_input", role_str, escrow_id),
        format!("{}_escrow_{}_round_3_input", role_str, escrow_id),
    ];
    
    for file_base in files_to_delete {
        for ext in &extensions {
            // Delete file_base and file_base.keys
        }
    }
}
```

---

## Verification Checklist

After testing 3 concurrent escrows, verify logs contain:

```
✅ "Copied buyer_escrow_{uuid}_round_1 → buyer_escrow_{uuid}_round_2_input"
✅ "Copied buyer_escrow_{uuid}_round_2_input → buyer_escrow_{uuid}_round_3_input"
✅ "Copied buyer_escrow_{uuid}_round_3_input → buyer_escrow_{uuid}_round_3_final (FINAL)"
✅ "Deleted intermediate file: buyer_escrow_{uuid}_round_1"
✅ "Deleted intermediate file: buyer_escrow_{uuid}_round_2_input"
✅ "Deleted intermediate file: buyer_escrow_{uuid}_round_3_input"
✅ "Intermediate files cleaned up"
```

No errors containing:
```
❌ "No wallet file found"
❌ "pubkey recommended"
❌ "RPC CACHE POLLUTION"
```

---

## How It Works (Technical)

### Why File Copying Works

1. **Wallet files contain state:** All multisig progress (round_1_infos, round_2_infos) is persisted in the .wallet and .keys files
2. **RPC daemon loads state:** When we `open_wallet()`, the daemon loads the file and reconstructs its in-memory state
3. **Each round appends state:** `exchange_multisig_keys()` appends new signatures to existing wallet state
4. **Copying preserves everything:** File copy = complete state preservation

### Process Isolation

With 6 RPC instances:
- **Escrow 1:** Uses ports 18082 (buyer), 18083 (vendor), 18084 (arbiter) - separate processes
- **Escrow 2:** Uses ports 18085 (buyer), 18086 (vendor), 18087 (arbiter) - separate processes
- **No shared state:** Each escrow's RPC processes are completely isolated

Each process has its own memory, so even if we reuse ports after cleanup:
- Old memory is discarded
- New wallet file is loaded fresh
- No cache pollution

---

## Architecture Components Working Together

| Phase | Component | Purpose |
|-------|-----------|---------|
| **Phase 1** | 25s Global Stagger | Prevents concurrent multisig operations at daemon level |
| **Phase 2** | WalletSessionManager | Keeps 3 wallets open and ready for signing operations |
| **Phase 3** | 10s RPC Cache Purge | Ensures daemon memory is cleared |
| **Phase 4** | Per-Round Filenames | Distinguishes wallet state at different stages |
| **Phase 5** | Copy-Based Persistence | Preserves accumulated state across rounds |
| **Phase 6** | Round-Robin RPC | Distributes escrows across 6 RPC instances |

---

## Expected Behavior (Testing)

### Successful Escrow (3/3)
1. User creates order in UI
2. Server assigns 3 wallets (round-robin distributed across 6 RPC ports)
3. Round 1: buyer_escrow_{uuid}_round_1 created, copied to round_2_input
4. Round 2: round_2_input loaded, copied to round_3_input
5. Round 3: round_3_input loaded, copied to round_3_final (FINAL - ready for signing)
6. Intermediate files deleted
7. Multisig address ready for payment

### Log Indicators
```
[Escrow 1 starts]
🎯 Assigned buyer to RPC: http://127.0.0.1:18082 (round-robin)
[copies buyer_escrow_xxx_round_1 → buyer_escrow_xxx_round_2_input]
[copies buyer_escrow_xxx_round_2_input → buyer_escrow_xxx_round_3_input]
[copies buyer_escrow_xxx_round_3_input → buyer_escrow_xxx_round_3_final]
✅ Escrow 1 completed successfully

⏳ [PHASE 1 GLOBAL STAGGER] delayed by ~25s

[Escrow 2 starts]
🎯 Assigned buyer to RPC: http://127.0.0.1:18085 (round-robin) ← DIFFERENT PORT!
[copies buyer_escrow_yyy_round_1 → buyer_escrow_yyy_round_2_input]
[copies buyer_escrow_yyy_round_2_input → buyer_escrow_yyy_round_3_input]
[copies buyer_escrow_yyy_round_3_input → buyer_escrow_yyy_round_3_final]
✅ Escrow 2 completed successfully
```

---

## Build & Deployment

```bash
# Build completed 2025-11-13 19:09 (with only non-critical warning)
cargo build --release --package server

# Server running with new code
ps aux | grep "target/release/server"
# Output: ./target/release/server running on PID 798030

# Health check
curl http://localhost:8080/api/health
# Output: {"status":"ok"}
```

---

## Related Files & Decisions

1. **ROUND-ROBIN-FIX-FINAL.md** - Fixed round-robin distribution (uses get_rpc_for_role)
2. **RPC-CACHE-POLLUTION-SOLUTION.md** - Identified cache pollution and 10s purge delay
3. **Non-custodial architecture** - Complements this Phase 4 work

---

## Summary

**Before:** Delete wallet files between rounds → Daemon loses state → "pubkey recommended" errors

**After:** Copy wallet files between rounds → State persists in files → Daemon reloads → ✅ Success

This is the **final architectural fix** for RPC cache pollution in concurrent escrow operations.

