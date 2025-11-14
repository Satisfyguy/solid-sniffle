# How to Use the Instrumentation Skill

## Quick Invocation

When you need to debug multisig concurrency issues, you can now invoke the instrumentation skill directly:

### Using Claude Code Skill Tool

```
Use the instrumentation skill to debug multisig concurrency issues
```

Or simply:

```
Debug multisig race conditions with instrumentation
```

### Manual Activation

If you prefer to activate manually:

```bash
export ENABLE_INSTRUMENTATION=1
cargo run --bin server
```

## What the Skill Does

When invoked, Claude will:

1. ✅ Enable instrumentation (`ENABLE_INSTRUMENTATION=1`)
2. ✅ Guide you to reproduce the bug (single + concurrent escrows)
3. ✅ Analyze generated JSON files with `tools/analyze_escrow_json.py`
4. ✅ Identify root cause (cache pollution, race condition, state divergence)
5. ✅ Provide specific fix with code examples
6. ✅ Verify fix with before/after comparison

## Example Workflow

**You:** "My escrows fail randomly when I run 3 at the same time"

**Claude (with skill):**
```
I'll use the instrumentation skill to debug this.

Step 1: Enabling instrumentation...
[Enables ENABLE_INSTRUMENTATION=1]

Step 2: Reproduce the issue...
[Runs 3 concurrent escrows]

Step 3: Analyzing results...
[Runs analyze_escrow_json.py on generated files]

Step 4: Root cause identified:
The instrumentation shows RPC cache pollution. Wallet was already
in multisig mode before make_multisig() at event #5. This happened
because the RPC instance retained state from the first escrow.

Step 5: Implementing fix...
[Shows code to increase delay or add cache flush]

Step 6: Verifying fix...
[Re-runs test, compares before/after]

✓ All 3 concurrent escrows now succeed!
```

## Key Commands

The skill knows these analysis commands:

```bash
# Basic analysis
python3 tools/analyze_escrow_json.py escrow_abc123.json

# Compare success vs failure
python3 tools/analyze_escrow_json.py --compare escrow_1.json escrow_3.json

# Timeline view
python3 tools/analyze_escrow_json.py --timeline escrow_failed.json

# RPC stats
python3 tools/analyze_escrow_json.py --rpc-only escrow_abc123.json

# Snapshots only
python3 tools/analyze_escrow_json.py --snapshots-only escrow_abc123.json
```

## Patterns it Recognizes

The skill automatically identifies these bug patterns:

### 1. RPC Cache Pollution
```
Symptom: multisig=true before make_multisig()
Fix: Increase delay or flush cache
```

### 2. Race Condition
```
Symptom: 3rd escrow fails at same point every time
Fix: Use WALLET_CREATION_LOCK mutex
```

### 3. State Divergence
```
Symptom: Different address_hash per wallet
Fix: Sort prepare_infos alphabetically
```

## Files Created

When you use this skill, it creates:

- `escrow_<uuid>.json` - Full instrumentation data (1-5 MB each)
- If escrow fails: `escrow_<uuid>_FAILED_ROUND<N>.json` - Partial data for debugging

## Cleanup

After debugging:

```bash
# Archive instrumentation files
tar -czf instrumentation_$(date +%Y%m%d).tar.gz escrow_*.json
rm escrow_*.json

# Or delete old files (7+ days)
find . -name "escrow_*.json" -mtime +7 -delete
```

## Documentation

- **Skill file:** [.claude/skills/instrumentation.md](.claude/skills/instrumentation.md)
- **Complete guide:** [DOX/guides/INSTRUMENTATION-GUIDE.md](../../DOX/guides/INSTRUMENTATION-GUIDE.md)
- **Full skill spec:** [DOX/skills/MULTISIG-INSTRUMENTATION-SKILL.md](../../DOX/skills/MULTISIG-INSTRUMENTATION-SKILL.md)
- **Quick start:** [INSTRUMENTATION-QUICK-START.md](../../INSTRUMENTATION-QUICK-START.md)

## Performance

- **Disabled (default):** 0% overhead
- **Enabled:** <1% CPU, 10-50 KB RAM, 1-5 MB disk per escrow
- **Recommendation:** Only enable for debugging

---

**Status:** ✅ Ready to use
**Invocation:** Just ask Claude to debug multisig concurrency issues!
