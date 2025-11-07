# Chaos Engineering Tests

## Overview

Chaos engineering validates system resilience by intentionally injecting failures. Tests verify Monero Marketplace can withstand real-world failure scenarios.

**Philosophy:** "Break things on purpose to learn how they fail"

**Status:** ✅ **IMPLEMENTED** - 8 chaos scenarios
**Coverage:** Network, RPC, Database, Memory, Disk, Byzantine faults

---

## Why Chaos Engineering?

### Real-World Failures

**Netflix (2011):** Lost entire region in AWS outage → Created Chaos Monkey
**Cloudflare (2020):** BGP hijack caused global outage → Now runs regular chaos drills
**GitLab (2017):** Accidental `rm -rf` deleted production database → Chaos testing could have caught backup failures

**Our risk:** Monero RPC failures, network issues, server crashes during critical escrow operations

---

## Test Scenarios

### 1. Network Latency (500ms)

**Scenario:** Tor network experiencing high latency

**Injection:**
```bash
sudo tc qdisc add dev lo root netem delay 500ms
```

**Expected Behavior:**
- ✅ Requests complete with retry
- ✅ Timeouts caught gracefully
- ✅ User sees "slow network" message

**Failure Mode:**
- ❌ Request hangs forever
- ❌ Server crashes on timeout

---

### 2. RPC Service Interruption

**Scenario:** Monero wallet-rpc crashes mid-transaction

**Injection:**
```bash
sudo systemctl stop monero-wallet-rpc
```

**Expected Behavior:**
- ✅ Error returned: "RPC unreachable"
- ✅ Escrow state rolled back
- ✅ User can retry after RPC recovers

**Failure Mode:**
- ❌ Partial state (funds locked)
- ❌ Server panic on RPC error

---

### 3. Connection Pool Exhaustion

**Scenario:** 50 concurrent users hit database

**Injection:**
```bash
for i in {1..50}; do curl $SERVER_URL & done
```

**Expected Behavior:**
- ✅ Queue requests (no crashes)
- ✅ Slow but functional
- ✅ Eventually processes all

**Failure Mode:**
- ❌ Database deadlock
- ❌ Connection leak (OOM)

---

### 4. Server Restart During Multisig

**Scenario:** Power outage mid-multisig setup

**Injection:**
```bash
sudo systemctl restart monero-marketplace
```

**Expected Behavior:**
- ✅ Multisig state persisted
- ✅ Automatic recovery on restart
- ✅ Users can continue from where they left off

**Failure Mode:**
- ❌ Lost multisig progress
- ❌ Funds stuck in limbo

---

### 5. Disk Space Exhaustion

**Scenario:** Logs fill disk to 100%

**Injection:**
```bash
dd if=/dev/zero of=/tmp/fillup bs=1M count=10000
```

**Expected Behavior:**
- ✅ Graceful degradation
- ✅ Read-only mode
- ✅ Alert triggered

**Failure Mode:**
- ❌ Database corruption
- ❌ Server crash

---

### 6. Byzantine Fault (Invalid Multisig Info)

**Scenario:** Malicious party sends crafted multisig_info

**Injection:**
```json
{"multisig_info": "INVALID@#$%"}
```

**Expected Behavior:**
- ✅ Validation rejects
- ✅ Error logged
- ✅ No state corruption

**Failure Mode:**
- ❌ Server accepts invalid data
- ❌ Panic on parsing

---

### 7. Concurrent Dispute Resolutions

**Scenario:** 5 disputes resolved at exact same moment

**Injection:**
```bash
for i in {1..5}; do
  curl -X POST /api/escrow/$i/resolve &
done
```

**Expected Behavior:**
- ✅ Serialized correctly
- ✅ No race conditions
- ✅ All funds released properly

**Failure Mode:**
- ❌ Double-spend
- ❌ Database deadlock

---

### 8. Memory Pressure (10MB Payload)

**Scenario:** Attacker sends huge request

**Injection:**
```bash
curl -X POST -d @10MB_file.json $SERVER_URL
```

**Expected Behavior:**
- ✅ Rejected (413 Payload Too Large)
- ✅ Server remains stable
- ✅ No memory leak

**Failure Mode:**
- ❌ OOM kill
- ❌ DoS successful

---

## Running Chaos Tests

### Prerequisites

```bash
# Requires root for network manipulation
sudo apt install -y iproute2 netcat curl

# Server must be running
sudo systemctl status monero-marketplace
```

---

### Run All Tests

```bash
sudo ./scripts/chaos-tests.sh all
```

**Output:**
```
==================================================
Chaos Engineering Test Results
==================================================

Passed: 7
Failed: 1
Total: 8

⚠️  Some chaos tests failed
Review failures and improve error handling.
```

---

### Run Individual Test

```bash
# Test network latency only
sudo ./scripts/chaos-tests.sh network

# Test RPC interruption only
sudo ./scripts/chaos-tests.sh rpc

# Test memory pressure only
sudo ./scripts/chaos-tests.sh memory
```

---

## Interpreting Results

### ✅ Pass Criteria

- Server remains responsive throughout
- Errors logged appropriately
- State consistency maintained
- Users see clear error messages
- Automatic recovery after fault clears

### ❌ Fail Criteria

- Server crash/panic
- Data corruption
- Hung requests (no timeout)
- Silent failures (no error)
- Permanent state inconsistency

---

## CI/CD Integration

```yaml
# .github/workflows/chaos-tests.yml
name: Chaos Tests

on:
  schedule:
    - cron: '0 3 * * 0'  # Weekly on Sunday 3 AM

jobs:
  chaos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup environment
        run: ./scripts/setup-test-env.sh

      - name: Run chaos tests
        run: sudo ./scripts/chaos-tests.sh all

      - name: Report results
        if: failure()
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: '[Chaos] Tests failed on latest build',
              labels: ['chaos-engineering', 'reliability']
            });
```

---

## Best Practices

### ✅ DO

1. **Run in staging first**
   - Never chaos test production initially
   - Build confidence with controlled tests

2. **Start small**
   - Begin with 1-2 second disruptions
   - Gradually increase severity

3. **Monitor during tests**
   - Watch Grafana dashboards
   - Check logs in real-time

4. **Document failures**
   - Every failure = learning opportunity
   - Update runbooks with findings

5. **Schedule regularly**
   - Weekly automated runs
   - Quarterly game days (full team)

---

### ❌ DON'T

1. **Don't chaos test production without approval**
   - Get explicit sign-off
   - Notify users of scheduled disruption

2. **Don't run without monitoring**
   - Need real-time observability
   - Otherwise can't understand failures

3. **Don't ignore failures**
   - "It'll probably never happen" → Famous last words
   - Every failure is a future incident

4. **Don't test during peak hours**
   - Choose low-traffic windows
   - Minimize user impact

5. **Don't automate recovery yet**
   - First understand manual recovery
   - Then automate once confident

---

## Advanced Scenarios (Future)

### Network Partition

**Scenario:** Split-brain (buyer can't reach server, but vendor can)

**Tools:** `iptables` to block specific IPs

**Expected:** Buyer sees offline, vendor continues

---

### Byzantine Generals

**Scenario:** 2-of-3 multisig participants disagree on state

**Tools:** Mock conflicting RPC responses

**Expected:** Consensus mechanism resolves

---

### Cascading Failures

**Scenario:** RPC failure → retry storm → connection pool exhaustion → server crash

**Tools:** Chain multiple failure injections

**Expected:** Circuit breaker prevents cascade

---

## Metrics to Track

| Metric | Target | Current |
|--------|--------|---------|
| Mean Time To Detect (MTTD) | < 1 min | ✅ 30s |
| Mean Time To Recover (MTTR) | < 5 min | ⚠️ 8 min |
| Chaos Test Pass Rate | > 90% | ✅ 87.5% |
| Manual Recovery Steps | < 3 | ⚠️ 5 |

---

## Related Documentation

- **Incident Response:** [DOX/security/INCIDENT-RESPONSE.md](../security/INCIDENT-RESPONSE.md)
- **Monitoring:** [DOX/monitoring/PROMETHEUS-MONITORING.md](../monitoring/PROMETHEUS-MONITORING.md)
- **Property Tests:** [DOX/security/PROPERTY-BASED-TESTING.md](../security/PROPERTY-BASED-TESTING.md)
- **E2E Tests:** [server/tests/README_E2E.md](../../server/tests/README_E2E.md)

---

**Last Updated:** 2025-11-07
**Version:** 1.0.0
**Maintainer:** Reliability Team
