# Incident Response Playbook - Monero Marketplace

**Version:** 1.0
**Last Updated:** 2025-10-22
**Owner:** Operations & Security Team

---

## 🚨 Table of Contents

1. [Incident Classification](#incident-classification)
2. [Response Procedures](#response-procedures)
3. [Incident Roles](#incident-roles)
4. [Communication Protocols](#communication-protocols)
5. [Technical Playbooks](#technical-playbooks)
6. [Post-Incident Process](#post-incident-process)

---

## 📋 Incident Classification

### Severity Levels

| Severity | Impact | Response Time | Examples |
|----------|--------|---------------|----------|
| **P1 - Critical** | Complete outage, data loss, security breach | 15 minutes | Database down, escrow funds inaccessible |
| **P2 - High** | Degraded service, partial outage | 1 hour | Single RPC unavailable, >5% error rate |
| **P3 - Medium** | Minor issues, no customer impact | 4 hours | Dashboard issues, log volume spikes |
| **P4 - Low** | Cosmetic issues, feature requests | 1 week | Documentation updates, UI tweaks |

---

## 🎯 Response Procedures

### P1 - Critical Incident Response

**Timeline: 0-15 minutes (Detection → Acknowledgment)**

1. **Alert Fires** (PagerDuty)
   - On-call engineer paged
   - Slack #incidents auto-notified
   - Status page auto-updated (Investigating)

2. **Initial Response** (0-5 min)
   ```bash
   # On-call engineer acknowledges alert
   # Quick triage - is this real?

   # Check service health
   cd 4.5/docker
   sudo docker compose ps

   # Check recent deployments
   git log --oneline -10

   # Check Prometheus alerts
   open http://localhost:9090/alerts
   ```

3. **Escalation Decision** (5-10 min)
   - Can you fix this alone? **NO** → Page Incident Commander
   - Is this a security breach? **YES** → Page Security Team
   - Is data loss involved? **YES** → Page Database Team

4. **War Room Setup** (10-15 min)
   - Create Slack channel `#incident-YYYY-MM-DD-HH-MM`
   - Invite Incident Commander, on-call engineer, stakeholders
   - Pin status updates to channel
   - Update status page: "Identified - Investigating root cause"

---

**Timeline: 15-60 minutes (Investigation → Mitigation)**

5. **Investigation** (15-30 min)
   ```bash
   # Gather logs
   sudo docker compose logs --tail=500 <service_name>

   # Check Grafana dashboards
   open http://localhost:3000/d/system-overview

   # Query Prometheus
   # CPU spike?
   rate(process_cpu_seconds_total[5m])

   # Memory leak?
   process_resident_memory_bytes

   # Error rate?
   rate(http_requests_total{status=~"5.."}[5m])
   ```

6. **Mitigation** (30-45 min)
   **Rollback Deployment:**
   ```bash
   # Rollback to last known good version
   cd 4.5/docker
   git log --oneline  # Find last stable commit
   git checkout <commit_hash>
   sudo docker compose up -d --build server
   ```

   **Restart Service:**
   ```bash
   sudo docker compose restart <service_name>
   ```

   **Scaling:**
   ```bash
   # Temporary increase resource limits
   sudo docker compose up -d --scale server=3
   ```

7. **Verification** (45-60 min)
   ```bash
   # Test critical endpoints
   curl -I http://localhost:8080/api/health

   # Check error rate dropped
   # Prometheus query: rate(http_requests_total{status="500"}[5m])

   # Verify escrow operations
   ./scripts/test-escrow-flow.sh
   ```

---

**Timeline: 60+ minutes (Resolution → Recovery)**

8. **Resolution**
   - Update status page: "Monitoring - Service restored"
   - Keep incident channel open for 1 hour
   - Monitor error rates, latency, resource usage

9. **Post-Incident** (within 24 hours)
   - Schedule post-mortem meeting
   - Update status page: "Resolved"
   - Close incident channel (export logs first)

---

### P2 - High Severity Response

**Timeline: 0-60 minutes**

1. **Alert Fires** (Slack #alerts)
2. **Triage** (0-15 min)
   - On-call reviews alert
   - Assess customer impact
   - Decide: Can this wait till business hours?

3. **Investigation** (15-45 min)
   - Same as P1 investigation steps
   - No war room required (use #alerts channel)

4. **Resolution** (45-60 min)
   - Apply fix or workaround
   - Document in incident ticket
   - Schedule permanent fix in next sprint

---

## 👥 Incident Roles

### Incident Commander (IC)

**Responsibilities:**
- Overall incident coordination
- Decision authority
- Stakeholder communication
- Escalation to executives if needed

**Qualifications:**
- Senior engineer or EM
- Familiar with all systems
- Clear communication skills

**Actions:**
```markdown
- [ ] Declare incident severity
- [ ] Assign roles (Ops Lead, Comms Lead)
- [ ] Set status update cadence (every 15min for P1)
- [ ] Make go/no-go decisions (rollback, restore, etc.)
- [ ] Declare incident resolved
```

---

### Operations Lead

**Responsibilities:**
- Hands-on technical troubleshooting
- Execute mitigation steps
- Coordinate with other engineers
- Provide technical updates to IC

**Actions:**
```bash
# Investigation checklist
- [ ] Gather logs from all affected services
- [ ] Check recent deployments/changes
- [ ] Review Prometheus metrics
- [ ] Identify root cause or failure mode
- [ ] Propose mitigation options to IC
- [ ] Execute approved mitigation
- [ ] Verify resolution
```

---

### Communications Lead

**Responsibilities:**
- Status page updates
- Customer communication
- Internal stakeholder updates
- Post-mortem scheduling

**Template - Status Page Update:**
```
**Incident:** Marketplace API Unavailable
**Status:** Investigating
**Impact:** Users unable to create orders
**Next Update:** 2025-10-22 14:30 UTC (15 minutes)

We are investigating reports of API errors.
Engineering team is actively working on resolution.
```

---

## 📢 Communication Protocols

### Internal Communication

**Slack Channels:**
- `#incidents` - Active incident coordination
- `#alerts` - Automated alert notifications
- `#ops` - General operations discussion
- `#post-mortems` - Post-incident reviews

**Status Updates (P1):**
- Every 15 minutes minimum
- Format: "Status | Current Action | ETA"
- Example: "Investigating | Analyzing database logs | 10min"

---

### External Communication

**Status Page Updates:**
- **Investigating:** Within 5 minutes of incident
- **Identified:** When root cause known
- **Monitoring:** When fix deployed
- **Resolved:** When verified stable

**Customer Communication:**
- **P1:** Email blast if >30min outage
- **P2:** In-app notification
- **P3/P4:** Release notes only

---

## 🔧 Technical Playbooks

### Playbook 1: Complete Service Outage

**Symptoms:**
- All API endpoints return 503
- Docker containers exited
- `docker compose ps` shows unhealthy services

**Response:**
```bash
# 1. Check Docker daemon
sudo systemctl status docker
sudo systemctl restart docker  # If needed

# 2. Check disk space (common cause)
df -h
# If >90% full, clear logs:
sudo docker system prune -af

# 3. Restart stack
cd 4.5/docker
sudo docker compose down
sudo docker compose up -d

# 4. Monitor startup
sudo docker compose logs -f server

# 5. Test health
curl -I http://localhost:8080/api/health
```

**Escalation:** If still failing → Database corruption (Playbook 5)

---

### Playbook 2: Database Connection Failures

**Symptoms:**
- `DatabaseUnavailable` alert firing
- Logs show "Failed to get DB connection"
- Error rate >5%

**Response:**
```bash
# 1. Check database container
sudo docker compose ps database
sudo docker compose logs database | tail -100

# 2. Check connection pool
# Prometheus query: diesel_connection_pool_size

# 3. Check file locks
ls -la /data/marketplace.db*
# Look for .db-shm or .db-wal files

# 4. Restart database (CAUTION: will cause brief outage)
sudo docker compose restart database

# 5. If corruption suspected, restore from backup
./scripts/restore-database.sh /backups/database/latest.sql.gz.gpg
```

---

### Playbook 3: Monero RPC Unreachable

**Symptoms:**
- `WalletRPCUnreachable` alert firing
- Escrow operations failing
- RPC call timeouts in logs

**Response:**
```bash
# 1. Check RPC container health
sudo docker compose ps monero-wallet-rpc-buyer
sudo docker compose ps monero-wallet-rpc-vendor
sudo docker compose ps monero-wallet-rpc-arbiter

# 2. Test RPC connectivity
curl -X POST http://127.0.0.1:18082/json_rpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"0","method":"get_balance"}'

# 3. Check Monero daemon sync status
# (wallet needs synced daemon to work)
sudo docker compose logs monero-wallet-rpc-buyer | grep -i "sync"

# 4. Restart wallet RPC
sudo docker compose restart monero-wallet-rpc-buyer

# 5. If wallet corrupted, restore from backup
./scripts/restore-wallet.sh /backups/wallets/latest.tar.gz.gpg buyer
```

---

### Playbook 4: High Error Rate (>5%)

**Symptoms:**
- `HighErrorRate` alert firing
- Prometheus shows spike in 5xx errors
- Users reporting errors

**Response:**
```bash
# 1. Identify failing endpoint
# Prometheus query:
topk(5, rate(http_requests_total{status=~"5.."}[5m]))

# 2. Check application logs
sudo docker compose logs server | grep -i "error" | tail -100

# 3. Check resource usage
sudo docker stats server

# 4. If resource exhaustion, increase limits
# Edit docker-compose.yml:
#   deploy:
#     resources:
#       limits:
#         memory: 4G  # Increase from 2G

# 5. Restart with new limits
sudo docker compose up -d server

# 6. If error persists, rollback deployment
git checkout <last_stable_commit>
sudo docker compose up -d --build server
```

---

### Playbook 5: Data Corruption / Loss

**Symptoms:**
- Database queries return corrupted data
- SQLite integrity check fails
- Escrow balances incorrect

**⚠️ CRITICAL - STOP ALL OPERATIONS IMMEDIATELY**

**Response:**
```bash
# 1. STOP application to prevent further corruption
sudo docker compose stop server

# 2. Create emergency backup of current state
cp /data/marketplace.db /data/marketplace.db.CORRUPTED-$(date +%s)

# 3. Run SQLite integrity check
sqlite3 /data/marketplace.db "PRAGMA integrity_check;"

# 4. If corruption confirmed, restore from latest backup
./scripts/restore-database.sh /backups/database/latest.sql.gz.gpg

# 5. Verify restore
./scripts/test-database-integrity.sh

# 6. Calculate data loss (RPO)
# Compare backup timestamp to current time
# RPO Target: 1 hour

# 7. Notify stakeholders of data loss window
# 8. Restart application
sudo docker compose up -d server

# 9. MANDATORY: Full post-mortem with RCA
```

**Escalation:** Immediately page Database Team + CTO

---

### Playbook 6: Security Breach

**Symptoms:**
- Unauthorized access detected
- Suspicious transactions
- Secrets exposed in logs

**⚠️ SECURITY INCIDENT - FOLLOW SECURITY PROTOCOL**

**Immediate Actions:**
```bash
# 1. ISOLATE compromised service
sudo docker compose stop server  # Or specific service

# 2. PRESERVE evidence
sudo docker compose logs server > /tmp/incident-logs-$(date +%s).log
sudo docker inspect server > /tmp/incident-container-$(date +%s).json

# 3. ROTATE all secrets
cd 4.5/security
# Generate new Age key
./scripts/setup-sops.sh --rotate

# 4. REVIEW access logs
sudo docker compose logs nginx | grep -E "POST|PUT|DELETE" > /tmp/access-review.log

# 5. NOTIFY Security Team immediately
# 6. PRESERVE compromised container (do NOT delete)
# 7. Page CISO / Security Lead
```

**DO NOT:**
- ❌ Restart services (destroys evidence)
- ❌ Modify logs
- ❌ Communicate publicly before Security approval

---

## 📝 Post-Incident Process

### Post-Mortem Meeting (within 48 hours)

**Attendees:**
- Incident Commander
- Operations Lead
- Engineering Manager
- Product Owner (if customer impact)

**Agenda:**
1. Timeline review (what happened when)
2. Root cause analysis (5 Whys)
3. What went well
4. What went wrong
5. Action items (with owners + due dates)

---

### Post-Mortem Document Template

```markdown
# Post-Mortem: [Incident Title]

**Date:** 2025-10-22
**Duration:** 14:30 - 16:45 UTC (2h 15min)
**Severity:** P1
**Affected Services:** Marketplace API, Database
**Customer Impact:** 100% of users unable to create orders

## Summary
[1-2 sentence description]

## Timeline
- 14:30 UTC: Alert fires - HighErrorRate
- 14:32 UTC: On-call acknowledges, begins investigation
- 14:40 UTC: Incident Commander paged
- 14:45 UTC: Root cause identified - database connection pool exhausted
- 15:00 UTC: Fix deployed - increased pool size 10 → 50
- 15:15 UTC: Service restored, monitoring
- 16:45 UTC: Incident resolved

## Root Cause
Database connection pool size (10) insufficient for traffic spike.
Load test did not simulate realistic concurrent user behavior.

## Impact
- Users affected: ~500 active users
- Revenue impact: $0 (testnet only)
- Data loss: None
- SLA breach: Yes (99.9% uptime target, 2h outage = 99.72% achieved)

## What Went Well
✅ Alert fired within 2 minutes of issue
✅ On-call responded promptly (2min acknowledge)
✅ Clear communication in #incidents channel
✅ No data loss

## What Went Wrong
❌ Load testing incomplete (didn't catch this scenario)
❌ No connection pool monitoring (missed early warning)
❌ Deployment had no rollback plan
❌ Status page update delayed (15min instead of 5min)

## Action Items
| # | Action | Owner | Due Date | Status |
|---|--------|-------|----------|--------|
| 1 | Add connection pool metrics to Grafana | @ops | 2025-10-25 | Open |
| 2 | Increase connection pool to 50 | @ops | 2025-10-22 | Done |
| 3 | Add pool saturation alert (>80%) | @ops | 2025-10-24 | Open |
| 4 | Improve load test scenarios | @qa | 2025-10-29 | Open |
| 5 | Update runbook with rollback steps | @ops | 2025-10-23 | Open |

## Lessons Learned
1. Load testing must simulate realistic concurrency
2. Monitor ALL resource pools (CPU, memory, connections)
3. Always have a rollback plan before deployment

## References
- Incident Slack channel: #incident-2025-10-22-1430
- Prometheus snapshot: /backups/prometheus-2025-10-22-1430.tar.gz
- Related alerts: HighErrorRate, DatabaseConnectionPoolSaturation
```

---

## 📊 Incident Metrics

**Track Monthly:**
- Total incidents by severity (P1/P2/P3/P4)
- Mean Time To Detect (MTTD)
- Mean Time To Resolve (MTTR)
- SLA breaches
- Post-mortem completion rate

**Quarterly Review:**
- Top 5 root causes
- Repeat incidents (>2 times)
- Action item completion rate
- Process improvements

---

## 🔗 Related Documents

- [SLA-RTO-RPO.md](SLA-RTO-RPO.md) - Service Level Agreement
- [DISASTER-RECOVERY.md](DISASTER-RECOVERY.md) - DR procedures
- [OPERATIONS-RUNBOOK.md](OPERATIONS-RUNBOOK.md) - Day-to-day operations
- [SECURITY-AUDIT.md](SECURITY-AUDIT.md) - Security checklist

---

## 📞 Emergency Contacts

**PagerDuty:** `marketplace-oncall` schedule

**Escalation Chain:**
1. On-Call Engineer (15min)
2. Incident Commander (30min)
3. Engineering Manager (1 hour)
4. CTO (4 hours)

**External Vendors:**
- Docker Support: support@docker.com
- Cloudflare (DDoS): dash.cloudflare.com
- PagerDuty Support: support@pagerduty.com

---

**Document Version:** 1.0
**Last Updated:** 2025-10-22
**Next Review:** 2025-11-22
**Owner:** Operations Team
