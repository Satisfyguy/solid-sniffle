# Incident Response Plan

## Overview

Step-by-step playbook for responding to security incidents in Monero Marketplace.

**Objective:** Minimize damage, restore service, prevent recurrence
**Target:** < 15 min MTTR (Mean Time To Resolve) for P0 incidents

---

## Severity Classification

| Level | Impact | Examples | Response Time | Escalation |
|-------|--------|----------|---------------|------------|
| **P0 - Critical** | Service down or data breach | RCE, DB breach, server down | < 15 min | Page on-call immediately |
| **P1 - High** | Major functionality broken | Auth bypass, payment failure | < 1 hour | Notify security team |
| **P2 - Medium** | Minor functionality broken | UI bug, slow performance | < 24 hours | Create ticket |
| **P3 - Low** | Cosmetic or minor issue | Typo, minor UX issue | < 1 week | Backlog |

---

## Incident Response Team

### Roles

| Role | Responsibilities | Contact |
|------|------------------|---------|
| **Incident Commander** | Decision making, coordination | @security-lead |
| **Technical Lead** | Investigation, mitigation | @backend-lead |
| **Communications Lead** | User communication, PR | @marketing-lead |
| **DevOps Lead** | Infrastructure, deployment | @devops-lead |

### On-Call Rotation

- **Primary:** @username (Mon-Wed)
- **Secondary:** @username (Thu-Sun)
- **Escalation:** CTO/CEO

**Contact Methods:**
- 🔴 **P0:** PagerDuty (phone call)
- 🟠 **P1:** Slack @channel
- 🟡 **P2:** Slack DM
- 🟢 **P3:** Email

---

## Response Workflow

```
┌─────────────────┐
│ 1. Detection    │  ← Prometheus alert / User report / Audit log
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 2. Triage       │  ← Assess severity (P0-P3)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 3. Containment  │  ← Stop the bleeding
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 4. Investigation│  ← Root cause analysis
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 5. Remediation  │  ← Fix + deploy
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 6. Recovery     │  ← Restore service
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 7. Post-Mortem  │  ← Document + prevent recurrence
└─────────────────┘
```

---

## Detailed Runbooks

### P0-1: Server Down

**Symptoms:**
- Prometheus alert: `ServerDown` firing
- Users report 503 errors
- Health check fails: `curl http://localhost:8080/api/health`

**Triage:**
```bash
# 1. Check process
systemctl status monero-marketplace
# → inactive (dead)

# 2. Check for crashes
journalctl -u monero-marketplace | grep -A 10 "panic"

# 3. Check resources
df -h          # Disk space
free -h        # Memory
uptime         # Load average
```

**Containment:**
```bash
# If disk full: Clean logs
sudo journalctl --vacuum-size=1G

# If memory exhausted: Kill heavy processes
ps aux --sort=-%mem | head -10
```

**Remediation:**
```bash
# Restart service
sudo systemctl restart monero-marketplace

# Verify startup
tail -f /var/log/monero-marketplace/server.log | grep "Starting"

# Wait for healthy
watch -n 1 'curl -s http://localhost:8080/api/health || echo "Not ready"'
```

**Recovery:**
- Monitor Grafana dashboard for 15 minutes
- Check escrow state consistency
- Notify users via status page

**Post-Mortem:**
- Document crash reason
- Add monitoring for root cause
- Update runbook with new findings

---

### P0-2: Database Breach

**Symptoms:**
- Unusual SQL queries in logs
- Unexpected data exports
- User reports unauthorized access

**Triage:**
```bash
# 1. Check audit logs
sqlite3 marketplace.db "SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT 100;"

# 2. Check for SQL injection attempts
grep -i "union\|select\|drop" /var/log/nginx/access.log | tail -50

# 3. Check active connections
lsof -i :8080
```

**Containment (IMMEDIATE):**
```bash
# 1. Take server offline
sudo systemctl stop monero-marketplace

# 2. Change all passwords
# (Manual step: notify all users to change passwords)

# 3. Rotate session secrets
export SESSION_SECRET_KEY="$(openssl rand -base64 48)"
# Update .env and restart

# 4. Block attacker IP (if known)
sudo ufw deny from <ATTACKER_IP>
```

**Investigation:**
```bash
# 1. Dump database for forensics
sqlite3 marketplace.db .dump > breach-dump-$(date +%Y%m%d-%H%M%S).sql

# 2. Analyze attack vector
grep -i "breach\|injection\|unauthorized" /var/log/monero-marketplace/server.log

# 3. Check for backdoors
find . -name "*.rs" -mtime -1  # Recently modified files
```

**Remediation:**
- Patch vulnerability
- Deploy fix
- Re-encrypt database with new key
- Invalidate all sessions

**Recovery:**
- Notify affected users (GDPR/CCPA compliance)
- Offer credit monitoring (if applicable)
- Publish incident report (transparency)

**Post-Mortem:**
- External security audit
- Penetration testing
- Improve input validation

---

### P0-3: Arbiter Key Compromise

**Symptoms:**
- Unexpected dispute resolutions
- Arbiter claims "I didn't sign that"
- Signature verification fails

**Triage:**
```bash
# 1. Check recent dispute resolutions
curl http://localhost:8080/admin/escrow/status | jq '.[] | select(.state == "resolved")'

# 2. Verify signatures
# (Manual: Use arbiter's public key to verify recent resolutions)

# 3. Check arbiter server logs
ssh arbiter-server 'journalctl -u arbiter-service | grep "sign"'
```

**Containment (IMMEDIATE):**
```bash
# 1. Freeze all dispute resolutions
# (Database flag: disputes_frozen = true)
sqlite3 marketplace.db "UPDATE config SET value='1' WHERE key='disputes_frozen';"

# 2. Generate new arbiter keypair
./scripts/airgap/generate-arbiter-keypair.sh

# 3. Update ARBITER_PUBKEY in .env
export ARBITER_PUBKEY="<new_public_key_hex>"

# 4. Restart server
sudo systemctl restart monero-marketplace
```

**Investigation:**
- Review how key was compromised
- Check for malware on arbiter's machine
- Audit all resolutions signed by compromised key

**Remediation:**
- Rollback fraudulent resolutions (manual, case-by-case)
- Compensate victims from platform reserves
- Rotate to new arbiter if necessary

**Recovery:**
- Unfreeze disputes with new key
- Notify community of rotation
- Document in changelog

**Post-Mortem:**
- Implement hardware security module (HSM)
- Multi-arbiter consensus (3-of-5)
- Air-gapped signing ceremony

---

### P1-1: High RPC Failure Rate

**Symptoms:**
- Prometheus alert: `HighRPCFailureRate` firing
- Users report "Cannot create escrow"
- RPC calls timing out

**Triage:**
```bash
# 1. Check RPC status
systemctl status monero-wallet-rpc

# 2. Test connectivity
curl http://127.0.0.1:18082/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}'

# 3. Check RPC logs
journalctl -u monero-wallet-rpc -n 100 | grep -i "error\|fail"
```

**Containment:**
```bash
# If RPC is stuck: Restart
sudo systemctl restart monero-wallet-rpc

# If blockchain out of sync: Wait for sync
monero-wallet-cli --wallet-file=buyer.wallet --daemon-address=127.0.0.1:28081 refresh
```

**Remediation:**
- Fix underlying issue (network, disk, sync)
- Increase RPC timeout in config
- Scale RPC instances if needed

**Recovery:**
- Monitor RPC success rate: http://localhost:3000 (Grafana)
- Re-process failed escrow creations (manual if needed)

---

### P1-2: Escrow Dispute Backlog

**Symptoms:**
- Prometheus alert: `EscrowDisputeBacklog` firing
- More than 5 disputes unresolved for > 1 hour
- Users complaining about arbiter

**Triage:**
```bash
# 1. Check arbiter availability
# (Manual: Call/message arbiter)

# 2. List pending disputes
curl http://localhost:8080/admin/escrow/status | jq '.[] | select(.state == "disputed")'

# 3. Check arbiter service
ssh arbiter-server 'systemctl status arbiter-service'
```

**Containment:**
- Contact arbiter urgently
- If arbiter unavailable: Activate backup arbiter

**Remediation:**
- Clear backlog (arbiter resolves disputes)
- If arbiter MIA: Emergency arbiter rotation
  ```bash
  # Rotate to backup arbiter
  export ARBITER_PUBKEY="<backup_arbiter_pubkey>"
  sudo systemctl restart monero-marketplace
  ```

**Recovery:**
- Monitor dispute resolution rate
- Notify users of delays

**Post-Mortem:**
- Implement multi-arbiter (3-of-5)
- Set arbiter SLA (< 24h response time)
- Automated arbiter health checks

---

## Communication Templates

### Internal (Slack)

```markdown
🚨 **P0 INCIDENT**

**Summary:** [Brief description]
**Status:** Investigating / Contained / Resolved
**Impact:** [Estimated users affected]
**ETA:** [Estimated time to resolution]
**Incident Commander:** @username

**Actions:**
- [ ] Task 1 (@owner)
- [ ] Task 2 (@owner)

**Updates:** Thread below
```

### External (Users)

```markdown
**Service Status Update**

We are currently investigating an issue affecting [feature].

**What happened:** [User-friendly explanation]
**Impact:** [Estimated % of users affected]
**Status:** [Investigating / Fixing / Monitoring]
**ETA:** [When will it be fixed]

We apologize for the inconvenience and appreciate your patience.

Follow updates: https://status.example.com
```

### Post-Mortem (Public)

```markdown
# Post-Mortem: [Incident Name]

**Date:** 2025-11-07
**Duration:** 2h 15min
**Impact:** 500 users affected

## Summary

[Brief description]

## Timeline (UTC)

- 14:00 - Incident detected via Prometheus alert
- 14:05 - Incident Commander paged
- 14:10 - Root cause identified
- 14:30 - Fix deployed
- 16:15 - Service fully restored

## Root Cause

[Technical details]

## Resolution

[What we did to fix it]

## Prevention

To prevent this from happening again:
- [ ] Add monitoring for [metric]
- [ ] Implement [safeguard]
- [ ] Update runbook with [new procedure]

## Lessons Learned

**What went well:**
- Fast detection (<1 min)
- Clear communication

**What needs improvement:**
- Faster containment (target: <5 min)
- Better alerting for [edge case]
```

---

## Tools & Resources

### Monitoring

- **Prometheus:** http://localhost:9090
- **Grafana:** http://localhost:3000
- **Alertmanager:** http://localhost:9093

### Logs

```bash
# Application logs
tail -f /var/log/monero-marketplace/server.log

# System logs
journalctl -u monero-marketplace -f

# Nginx logs (if using reverse proxy)
tail -f /var/log/nginx/error.log
```

### Forensics

```bash
# Network connections
lsof -i :8080

# Recent file modifications
find . -mtime -1 -type f

# Database dump
sqlite3 marketplace.db .dump > forensics-$(date +%Y%m%d).sql
```

---

## Training & Drills

**Frequency:** Quarterly
**Duration:** 2 hours

**Scenarios:**
1. Server down drill (P0)
2. RPC failure simulation (P1)
3. Database breach tabletop exercise (P0)
4. Arbiter rotation procedure (P1)

**Participants:**
- Security Team
- Backend Team
- DevOps Team
- On-call engineers

**Format:**
- 30 min: Review previous incidents
- 60 min: Live simulation
- 30 min: Debrief + runbook updates

---

## Related Documentation

- **Security Policy:** [SECURITY.md](../../SECURITY.md)
- **Threat Model:** [THREAT-MODEL.md](THREAT-MODEL.md)
- **Monitoring:** [../monitoring/PROMETHEUS-MONITORING.md](../monitoring/PROMETHEUS-MONITORING.md)
- **Runbook Links:** https://docs.example.com/runbooks/

---

**Version:** 1.0.0
**Last Updated:** 2025-11-07
**Next Review:** 2025-12-07
**Maintained By:** Security Team
