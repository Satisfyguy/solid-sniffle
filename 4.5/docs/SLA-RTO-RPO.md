# Service Level Agreement (SLA) - Monero Marketplace

**Version:** 1.0
**Effective Date:** 2025-10-22
**Review Cycle:** Quarterly
**Owner:** Operations Team

---

## 📊 Service Level Objectives (SLO)

### Uptime & Availability

| Service Component | Target Uptime | Measurement Window | Downtime Budget |
|-------------------|---------------|--------------------|-----------------|
| **Marketplace Application** | 99.9% | Monthly | 43.2 minutes/month |
| **Monero RPC Services** | 99.5% | Monthly | 3.6 hours/month |
| **Monitoring Stack** | 99.0% | Monthly | 7.2 hours/month |
| **Database (SQLCipher)** | 99.95% | Monthly | 21.6 minutes/month |

**Uptime Calculation:**
- Excludes planned maintenance windows (announced 48h in advance)
- Measured via Prometheus `up` metric
- Alert threshold: <99.5% triggers Page DutyOncall

---

### Performance Targets

| Metric | Target (p95) | Target (p99) | Measurement |
|--------|--------------|--------------|-------------|
| **HTTP Response Time** | <200ms | <500ms | Prometheus histogram |
| **API Endpoint Latency** | <300ms | <800ms | Per-endpoint tracking |
| **Database Query Time** | <50ms | <150ms | SQLite query logs |
| **Escrow Creation Time** | <2s | <5s | End-to-end timing |
| **Monero RPC Response** | <1s | <3s | RPC call duration |

**Latency Budgets:**
- 95% of requests must meet p95 target
- 99% of requests must meet p99 target
- Breach triggers performance investigation

---

### Error Rate Targets

| Component | Target Error Rate | Alert Threshold | Measurement |
|-----------|-------------------|-----------------|-------------|
| **HTTP 5xx Errors** | <0.1% | >1% for 5min | Prometheus counter |
| **HTTP 4xx Errors** | <5% | >10% for 10min | Client error rate |
| **Database Errors** | <0.01% | >0.1% for 5min | Connection failures |
| **RPC Call Failures** | <1% | >5% for 5min | Monero RPC errors |
| **WebSocket Disconnects** | <2% | >10% for 5min | Connection drops |

---

## 🕒 Recovery Time Objective (RTO)

**RTO:** Maximum acceptable time to restore service after an outage.

### Component RTOs

| Component | RTO Target | Recovery Procedure |
|-----------|------------|-------------------|
| **Application Server** | 15 minutes | Blue-green rollback or container restart |
| **Database** | 30 minutes | Restore from latest automated backup |
| **Monero Wallets** | 1 hour | Restore encrypted wallet files + resync |
| **Monitoring Stack** | 2 hours | Rebuild from docker-compose + dashboards |
| **Complete System** | 4 hours | Full infrastructure rebuild from backups |

**RTO Assumptions:**
- Backups are accessible and valid
- Recovery team is available (on-call)
- Infrastructure platform (Docker host) is operational

---

## 💾 Recovery Point Objective (RPO)

**RPO:** Maximum acceptable data loss in case of failure.

### Data RPOs

| Data Type | RPO Target | Backup Frequency | Retention |
|-----------|------------|------------------|-----------|
| **Database (transactions)** | 1 hour | Hourly automated | 30 days |
| **Monero Wallets** | 24 hours | Daily automated | 90 days |
| **Configuration Files** | 24 hours | Daily (Git commits) | Infinite (VCS) |
| **Prometheus Metrics** | 15 days | Continuous TSDB | 30 days |
| **Application Logs** | 7 days | Continuous (Loki) | 30 days |

**Backup Validation:**
- Weekly automated restore tests (staging environment)
- Monthly manual DR drill
- Quarterly full disaster recovery simulation

---

## 📞 Escalation & Support

### Support Tiers

| Tier | Response Time | Coverage | Contact |
|------|---------------|----------|---------|
| **L1 - On-Call** | 15 minutes | 24/7 | PagerDuty |
| **L2 - Engineering** | 1 hour | Business hours | Slack #incidents |
| **L3 - Architecture** | 4 hours | On-demand | Email escalation |

### Severity Levels

#### P1 - Critical (RTO: 15min)
**Definition:** Complete service outage, data loss, security breach
**Examples:**
- All API endpoints returning 503
- Database corruption or unavailable
- Escrow funds inaccessible

**Response:**
- Page on-call engineer immediately
- Incident commander assigned within 15 minutes
- Status page updated every 15 minutes
- Post-mortem required within 48 hours

---

#### P2 - High (RTO: 1 hour)
**Definition:** Degraded service, partial outage, performance issues
**Examples:**
- Single RPC wallet unreachable
- >5% error rate on API endpoints
- Monitoring stack unavailable

**Response:**
- Slack alert to #incidents channel
- Assigned engineer within 1 hour
- Status page updated if customer-facing
- Post-incident review within 1 week

---

#### P3 - Medium (RTO: 4 hours)
**Definition:** Non-critical issues, minor bugs, planned maintenance
**Examples:**
- Dashboard rendering issues
- Alert notification delays
- Log volume spikes

**Response:**
- Create JIRA ticket
- Assigned within 4 hours (business hours)
- Fix in next sprint
- No post-mortem required

---

#### P4 - Low (RTO: 1 week)
**Definition:** Feature requests, optimizations, cosmetic issues
**Examples:**
- Dashboard layout improvements
- Metric label changes
- Documentation updates

**Response:**
- Backlog grooming
- Prioritized in quarterly planning
- No SLA commitment

---

## 🔍 Monitoring & Alerting

### Critical Alerts (PagerDuty)

| Alert Name | Condition | Severity | Owner |
|------------|-----------|----------|-------|
| **ServiceDown** | Service unreachable for 2 minutes | P1 | On-call |
| **DatabaseUnavailable** | DB connection failures >0.1% | P1 | On-call |
| **HighErrorRate** | 5xx errors >1% for 5 minutes | P1 | On-call |
| **EscrowStuckCritical** | Escrow pending >48 hours | P1 | On-call |
| **DiskFull** | Disk usage >90% | P2 | On-call |

### Warning Alerts (Slack)

| Alert Name | Condition | Severity | Channel |
|------------|-----------|----------|---------|
| **HighLatency** | p95 latency >500ms for 10min | P2 | #alerts |
| **WalletRPCUnreachable** | RPC unreachable >5 minutes | P2 | #alerts |
| **BackupFailure** | Backup job failed | P2 | #ops |
| **CertificateExpiry** | TLS cert expires <30 days | P3 | #ops |

---

## 📋 Maintenance Windows

### Planned Maintenance

**Schedule:**
- **Weekly Patching:** Sundays 02:00-04:00 UTC
- **Monthly Updates:** First Saturday 01:00-05:00 UTC
- **Quarterly Upgrades:** Last Saturday 00:00-08:00 UTC

**Notification:**
- 7 days advance notice (quarterly)
- 48 hours advance notice (monthly)
- 24 hours advance notice (weekly)

**Approval Process:**
1. Change request submitted (JIRA)
2. Risk assessment completed
3. Rollback plan documented
4. Stakeholder approval obtained
5. Maintenance window scheduled

---

## 📊 Capacity Planning

### Resource Thresholds

| Resource | Warning (80%) | Critical (90%) | Action |
|----------|---------------|----------------|--------|
| **CPU** | Alert L2 | Alert L1 | Scale horizontally |
| **Memory** | Alert L2 | Alert L1 | Increase limits |
| **Disk** | Alert L2 | Alert L1 | Expand volume |
| **Network** | Alert L2 | Alert L1 | Rate limit review |

### Growth Projections

| Metric | Current | 6 Months | 12 Months | Capacity Plan |
|--------|---------|----------|-----------|---------------|
| **Active Users** | 100 | 500 | 2,000 | Horizontal scaling |
| **Daily Transactions** | 50 | 250 | 1,000 | Database sharding |
| **Storage (DB)** | 1 GB | 10 GB | 50 GB | Volume expansion |
| **Metrics Retention** | 30 days | 90 days | 180 days | Archive to S3 |

---

## 🧪 Testing & Validation

### SLA Validation Tests

**Monthly:**
- [ ] RTO drill: Restore database from backup (<30min target)
- [ ] RTO drill: Rollback application deployment (<15min target)
- [ ] RPO validation: Verify backup integrity (100% success rate)
- [ ] Load test: Simulate 2x normal traffic (p95 <200ms maintained)

**Quarterly:**
- [ ] Full DR scenario: Complete infrastructure rebuild (<4h target)
- [ ] Chaos engineering: Random service failures (graceful degradation)
- [ ] Security audit: Penetration testing + vulnerability scan
- [ ] Capacity test: 10x traffic spike (identify breaking points)

---

## 📝 Reporting & Review

### Monthly SLA Report

**Contents:**
1. Uptime Achievement (vs 99.9% target)
2. Performance Metrics (p95/p99 latency)
3. Error Rate Summary (by component)
4. Incident Summary (P1/P2 count, MTTR)
5. Backup Success Rate
6. Capacity Utilization Trends

**Distribution:**
- Stakeholders: First Monday of month
- Format: Executive summary + detailed appendix
- Storage: Confluence wiki + Slack #sla-reports

---

### Quarterly SLA Review

**Agenda:**
1. SLA target review (still achievable?)
2. Incident post-mortem trends
3. Capacity planning updates
4. RTO/RPO target adjustments
5. Tooling improvements
6. Budget allocation

**Attendees:**
- Product Owner
- Engineering Lead
- Operations Team
- Security Team

---

## 🔗 Related Documents

- [DISASTER-RECOVERY.md](DISASTER-RECOVERY.md) - Complete DR procedures
- [OPERATIONS-RUNBOOK.md](OPERATIONS-RUNBOOK.md) - Day-to-day operations
- [INCIDENT-RESPONSE.md](INCIDENT-RESPONSE.md) - Incident management playbook
- [SECURITY-AUDIT.md](SECURITY-AUDIT.md) - Security hardening checklist

---

## 📞 Emergency Contacts

**On-Call Rotation:** PagerDuty schedule `marketplace-oncall`

**Escalation Chain:**
1. On-Call Engineer (15min response)
2. Engineering Lead (1 hour response)
3. CTO (4 hour response)

**External Vendors:**
- **Docker Support:** support@docker.com (Enterprise SLA)
- **Monero RPC:** community@getmonero.org (community support)
- **Age Encryption:** age-users@googlegroups.com (community)

---

**Last Reviewed:** 2025-10-22
**Next Review:** 2026-01-22
**Document Owner:** Operations Team
**Approver:** CTO
