# Multisig Instrumentation Skill

**Purpose:** Debug race conditions, RPC cache pollution, and state corruption in concurrent multisig escrow operations.

**When to use:** User reports random failures, concurrency bugs, or "wallet already multisig" errors in escrow operations.

---

## Workflow

When the user asks you to debug multisig concurrency issues, follow this 5-step workflow:

### 1. ENABLE INSTRUMENTATION

```bash
export ENABLE_INSTRUMENTATION=1
cargo run --bin server
```

Tell the user: "Instrumentation enabled. JSON files will be created for each escrow in the current directory."

### 2. REPRODUCE THE BUG

**Single escrow (baseline):**
```bash
curl -X POST http://localhost:8080/api/escrow/init \
  -H "Content-Type: application/json" \
  -d '{"buyer_id": "buyer1", "vendor_id": "vendor1", "amount": 1000000}'
```

**Concurrent escrows (trigger race condition):**
```bash
for i in {1..3}; do
  curl -X POST http://localhost:8080/api/escrow/init \
    -H "Content-Type: application/json" \
    -d "{\"buyer_id\": \"buyer$i\", \"vendor_id\": \"vendor$i\", \"amount\": 1000000}" &
done
wait
```

### 3. ANALYZE RESULTS

```bash
# List generated files
ls -lh escrow_*.json

# Basic analysis
python3 tools/analyze_escrow_json.py escrow_abc123.json

# Compare successful vs failed
python3 tools/analyze_escrow_json.py --compare escrow_success.json escrow_failed.json

# Timeline view
python3 tools/analyze_escrow_json.py --timeline escrow_failed.json

# RPC statistics
python3 tools/analyze_escrow_json.py --rpc-only escrow_abc123.json

# Snapshot analysis
python3 tools/analyze_escrow_json.py --snapshots-only escrow_abc123.json
```

### 4. IDENTIFY ROOT CAUSE

Look for these patterns in the analysis output:

#### Pattern A: RPC Cache Pollution
**Symptom:**
```
[+0ms] SNAPSHOT_PRE_ROUND1 role=buyer multisig=true ❌
Error: Wallet already in multisig mode
```

**Root cause:** RPC cache retains state from previous operation

**Fix:**
- Increase delay between operations (10s → 15s)
- Add explicit RPC cache flush
- Use `close_wallet()` + `open_wallet()` cycle

#### Pattern B: Race Condition
**Symptom:**
```
COMPARING: escrow_1.json vs escrow_3.json
Divergence at event #15
File 1: [RPC_CALL_END] role=buyer
File 3: [ERROR_FINAL] role=buyer
```

**Root cause:** Concurrent access to shared RPC instance

**Fix:**
- Use `WALLET_CREATION_LOCK` global mutex
- Implement wallet pool with exclusive RPC instances
- Add file-level locking

#### Pattern C: State Divergence
**Symptom:**
```
buyer.address_hash:   abc123...
vendor.address_hash:  abc123...
arbiter.address_hash: def456... ❌
```

**Root cause:** Incorrect prepare_infos ordering or wrong infos sent

**Fix:**
- Sort prepare_infos alphabetically before make_multisig()
- Validate prepare_infos content with SHA256 hashes
- Ensure each wallet receives correct OTHER 2 infos

### 5. IMPLEMENT & VERIFY FIX

After identifying the root cause:

1. **Implement fix** in the relevant code
2. **Re-run test** with instrumentation enabled
3. **Compare before/after:**
   ```bash
   python3 tools/analyze_escrow_json.py --compare escrow_BEFORE.json escrow_AFTER.json
   ```
4. **Verify fix:** All 3 concurrent escrows should succeed

---

## Key Instrumentation Points

The instrumentation captures state at 7 critical points:

1. **SNAPSHOT_PRE_ROUND1** - Before prepare_multisig
2. **SNAPSHOT_POST_MAKE_MULTISIG** - After make_multisig (×3 wallets)
3. **SNAPSHOT_PRE_ROUND2** - Before first exchange_multisig_keys
4. **SNAPSHOT_POST_EXPORT_MULTISIG** - After export
5. **SNAPSHOT_PRE_ROUND3** - Before second exchange_multisig_keys
6. **SNAPSHOT_POST_IMPORT_MULTISIG** - After import
7. **SNAPSHOT_FINAL** - Final state

Plus RPC call timing (start, end, duration, success/failure) for every operation.

---

## Quick Commands Reference

```bash
# Enable instrumentation
export ENABLE_INSTRUMENTATION=1

# Run 3 concurrent escrows
for i in {1..3}; do curl -X POST http://localhost:8080/api/escrow/init \
  -H "Content-Type: application/json" \
  -d "{\"buyer_id\": \"buyer$i\", \"vendor_id\": \"vendor$i\", \"amount\": 1000000}" & done; wait

# Analyze all escrows
for file in escrow_*.json; do
  echo "=== $file ==="
  python3 tools/analyze_escrow_json.py --errors-only "$file"
done

# Compare two escrows
python3 tools/analyze_escrow_json.py --compare escrow_1.json escrow_3.json

# Full timeline of failed escrow
python3 tools/analyze_escrow_json.py --timeline escrow_failed.json > debug.txt
```

---

## Output Interpretation

When you analyze the JSON files, tell the user what you found:

**If RPC cache pollution detected:**
> "The instrumentation shows wallet was already in multisig mode before make_multisig() at timestamp X. This indicates RPC cache pollution from a previous operation. The fix is to increase the delay between operations or add explicit cache flushing."

**If race condition detected:**
> "The timeline comparison shows escrow_3 diverged from escrow_1 at event #15 during make_multisig(). This indicates a race condition on the shared RPC instance. The fix is to use the WALLET_CREATION_LOCK mutex or implement a wallet pool."

**If state divergence detected:**
> "The snapshots show buyer and vendor have address_hash 'abc123' but arbiter has 'def456'. This indicates incorrect prepare_infos were sent to the arbiter. The fix is to validate prepare_infos ordering and ensure alphabetical sorting."

---

## Performance Notes

- **Overhead when disabled:** Zero (default production mode)
- **Overhead when enabled:** <1% CPU, 10-50 KB memory, 1-5 MB disk per escrow
- **Recommendation:** Only enable for debugging, not in production

---

## Documentation References

- **Full skill description:** [DOX/skills/MULTISIG-INSTRUMENTATION-SKILL.md](../../DOX/skills/MULTISIG-INSTRUMENTATION-SKILL.md)
- **Complete guide:** [DOX/guides/INSTRUMENTATION-GUIDE.md](../../DOX/guides/INSTRUMENTATION-GUIDE.md)
- **Integration examples:** [DOX/guides/INSTRUMENTATION-INTEGRATION-EXAMPLE.md](../../DOX/guides/INSTRUMENTATION-INTEGRATION-EXAMPLE.md)
- **Quick start:** [INSTRUMENTATION-QUICK-START.md](../../INSTRUMENTATION-QUICK-START.md)

---

## Troubleshooting

**No JSON files generated?**
- Check `echo $ENABLE_INSTRUMENTATION` (should output "1")
- Verify server logs show "Instrumentation ENABLED for escrow..."

**Empty events array?**
- Check if escrow operation completed
- Look for error in server logs

**Python tool errors?**
- Use `python3` explicitly
- Check file exists: `ls escrow_*.json`
- Validate JSON: `cat escrow_abc.json | python3 -m json.tool`

---

## Success Criteria

After using this skill, the user should know:

✅ **Which wallet** failed (buyer, vendor, or arbiter)
✅ **At which round** (1, 2, or 3)
✅ **What the state was** before the error
✅ **Why it failed** (cache pollution, race condition, wrong inputs)
✅ **How to fix it** (specific code change with rationale)

**This transforms "it fails randomly" into "RPC cache on port 18082 retains multisig state from escrow_1, causing escrow_3 to fail at make_multisig() with 'already multisig' error."**

---

**Status:** ✅ Fully implemented and integrated
**Last updated:** 2025-11-13
