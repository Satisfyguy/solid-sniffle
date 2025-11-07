# Prometheus Monitoring & Alerting System

## Overview

Comprehensive production monitoring system for Monero Marketplace using Prometheus, Grafana, and Alertmanager. Tracks critical metrics, detects anomalies, and sends alerts for immediate incident response.

**Criticality:** 🟠 **HIGH** - Essential for production operations
**Status:** ✅ **IMPLEMENTED** - Metrics + Alerts + Dashboards

---

## Why Monitoring Matters

### Real-World Impact

**Without proper monitoring:**
- 🔴 **GitLab 2017:** 300GB of production data deleted, detected 18 hours later
- 🔴 **AWS 2017:** S3 outage took 4 hours to detect root cause
- 🔴 **Knight Capital 2012:** $440M loss in 45 minutes due to undetected bug

**With Prometheus monitoring:**
- ✅ **Detect issues in <1 minute**
- ✅ **Automatic alerts to on-call team**
- ✅ **Historical trends for capacity planning**
- ✅ **SLA compliance tracking**

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Monero Marketplace Application                             │
│  ├─ /metrics endpoint (Prometheus format)                   │
│  └─ Metrics exported every 15s                              │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ HTTP scrape every 15s
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  Prometheus Server                                           │
│  ├─ Stores metrics (30 days retention)                      │
│  ├─ Evaluates alerting rules (every 15s)                    │
│  └─ Sends alerts to Alertmanager                            │
└────────────────┬────────────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
        ▼                 ▼
┌──────────────┐  ┌──────────────┐
│ Grafana      │  │ Alertmanager │
│ (Dashboard)  │  │ (Routing)    │
└──────────────┘  └──────┬───────┘
                         │
                 ┌───────┴────────┐
                 │                │
                 ▼                ▼
        ┌──────────────┐  ┌──────────────┐
        │ Slack/Email  │  │ PagerDuty    │
        └──────────────┘  └──────────────┘
```

---

## Metrics Exported

### Escrow Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `escrows_created_total` | Counter | Total escrows created |
| `escrows_funded_total` | Counter | Total escrows funded with XMR |
| `escrows_completed_total` | Counter | Total successful completions |
| `escrows_disputed_total` | Counter | Total escrows entering dispute |
| `escrows_resolved_total` | Counter | Total disputes resolved |

**Queries:**
```promql
# Escrow success rate (last hour)
(rate(escrows_completed_total[1h]) / rate(escrows_funded_total[1h])) * 100

# Dispute rate (last 24h)
(increase(escrows_disputed_total[24h]) / increase(escrows_created_total[24h])) * 100

# Active escrows (not yet completed)
escrows_funded_total - escrows_completed_total - escrows_disputed_total
```

---

### RPC Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `rpc_calls_total` | Counter | Total Monero RPC calls made |
| `rpc_calls_failed_total` | Counter | Total RPC calls that failed |

**Queries:**
```promql
# RPC success rate (last 5 minutes)
((rate(rpc_calls_total[5m]) - rate(rpc_calls_failed_total[5m])) / rate(rpc_calls_total[5m])) * 100

# RPC failure rate trend
rate(rpc_calls_failed_total[5m])
```

---

### Dispute Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `disputes_buyer_won_total` | Counter | Disputes resolved in buyer favor |
| `disputes_vendor_won_total` | Counter | Disputes resolved in vendor favor |

**Queries:**
```promql
# Buyer win percentage
(disputes_buyer_won_total / (disputes_buyer_won_total + disputes_vendor_won_total)) * 100

# Dispute resolution bias detection
abs((disputes_buyer_won_total / (disputes_buyer_won_total + disputes_vendor_won_total)) - 0.5) > 0.3
```

---

### System Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `uptime_seconds` | Counter | Server uptime in seconds |

**Queries:**
```promql
# Uptime in hours
uptime_seconds / 3600

# Detect restarts (uptime < 5 minutes)
uptime_seconds < 300
```

---

## Installation

### Step 1: Install Prometheus

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y prometheus

# Verify installation
prometheus --version
```

**Configuration:** Copy `monitoring/prometheus.yml` to `/etc/prometheus/prometheus.yml`

```bash
sudo cp monitoring/prometheus.yml /etc/prometheus/
sudo systemctl restart prometheus
sudo systemctl enable prometheus
```

**Verify:** http://localhost:9090

---

### Step 2: Install Alertmanager

```bash
# Ubuntu/Debian
sudo apt install -y prometheus-alertmanager

# Copy configuration
sudo mkdir -p /etc/alertmanager
sudo cp monitoring/alertmanager.yml /etc/alertmanager/
sudo systemctl restart prometheus-alertmanager
sudo systemctl enable prometheus-alertmanager
```

**Configure Slack webhook** (edit `/etc/alertmanager/alertmanager.yml`):

```yaml
receivers:
  - name: 'slack-critical'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#alerts-critical'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

---

### Step 3: Install Grafana

```bash
# Add Grafana APT repository
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install -y grafana

# Start Grafana
sudo systemctl start grafana-server
sudo systemctl enable grafana-server
```

**Access:** http://localhost:3000
**Default login:** admin/admin

---

### Step 4: Import Dashboard

1. **Add Prometheus data source**
   - Settings → Data Sources → Add Prometheus
   - URL: http://localhost:9090
   - Save & Test

2. **Import dashboard**
   - Dashboards → Import
   - Upload `monitoring/grafana-dashboard.json`
   - Select Prometheus data source

---

## Alert Rules

### Critical Alerts (Immediate Response < 15 min)

#### 1. High RPC Failure Rate

**Trigger:** RPC failure rate > 10% for 2 minutes

**Impact:** Escrow operations failing, users cannot fund/complete

**Actions:**
```bash
# 1. Check RPC status
systemctl status monero-wallet-rpc

# 2. Check logs
journalctl -u monero-wallet-rpc -n 100 --no-pager

# 3. Test connectivity
curl http://127.0.0.1:18082/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}'

# 4. Restart if needed
sudo systemctl restart monero-wallet-rpc
```

**Escalation:** If unresolved after 15 min, page on-call engineer

---

#### 2. Escrow Dispute Backlog

**Trigger:** More than 5 unresolved disputes for > 1 hour

**Impact:** Users stuck in dispute, funds locked

**Actions:**
```bash
# 1. Check arbiter status
curl http://localhost:8080/admin/escrow/status

# 2. Contact arbiter
# (Manual step: Call/email arbiter)

# 3. Check dispute logs
grep "dispute" /var/log/monero-marketplace/server.log | tail -20

# 4. Manual resolution if arbiter unavailable
# (Requires two admin signatures)
```

**Escalation:** Contact arbiter immediately, consider emergency arbiter rotation

---

#### 3. Server Down

**Trigger:** Server unreachable for 1 minute

**Impact:** Entire platform unavailable

**Actions:**
```bash
# 1. Check server process
systemctl status monero-marketplace

# 2. Check for crashes
journalctl -u monero-marketplace | grep -A 10 "panic"

# 3. Check resources
df -h  # Disk space
free -h  # Memory
top  # CPU

# 4. Restart if needed
sudo systemctl restart monero-marketplace

# 5. Check logs after restart
tail -f /var/log/monero-marketplace/server.log
```

**Escalation:** If restart fails, investigate root cause before retry

---

### High Priority Alerts (Response < 1 hour)

#### 4. Low Escrow Completion Rate

**Trigger:** Completion rate < 70% for 30 minutes

**Impact:** Revenue loss, user dissatisfaction

**Actions:**
1. Check for timeout issues (`grep timeout /var/log/monero-marketplace/server.log`)
2. Review escrow state distribution (Grafana dashboard)
3. Contact vendors with incomplete orders
4. Check for platform bugs

---

#### 5. High Dispute Rate

**Trigger:** Dispute rate > 5% for 1 hour

**Impact:** Platform trust issues, arbiter overload

**Actions:**
1. Identify vendors with high dispute rates
   ```sql
   SELECT vendor_id, COUNT(*) as disputes
   FROM escrows
   WHERE state = 'disputed'
   GROUP BY vendor_id
   ORDER BY disputes DESC
   LIMIT 10;
   ```
2. Review dispute reasons
3. Consider vendor suspension
4. Investigate platform bugs

---

### Warning Alerts (Response < 24 hours)

#### 6. Dispute Resolution Bias

**Trigger:** >80% or <20% buyer wins for 24 hours

**Impact:** Arbiter bias perception, unfair resolutions

**Actions:**
1. Review arbiter decisions for patterns
2. Check for fraudulent vendor/buyer behavior
3. Consider arbiter rotation
4. Document findings

---

## Grafana Dashboards

### Main Dashboard

**Panels:**
1. **Alert Status** - Active critical/high alerts
2. **Escrow Funnel** - Created → Funded → Completed
3. **Uptime** - Server uptime gauge
4. **Success Rate** - Escrow completion %
5. **RPC Health** - RPC call success rate
6. **Dispute Balance** - Buyer vs Vendor wins
7. **Creation Rate** - New escrows/hour
8. **RPC Rate** - Calls/second (total + failed)
9. **State Distribution** - Escrow states table
10. **Active Alerts** - Current firing alerts
11. **Key Metrics** - Summary statistics

**Refresh:** Every 30 seconds
**Time range:** Last 6 hours (configurable)

---

## Best Practices

### ✅ DO

1. **Set up alerting before production**
   - Configure Slack/PagerDuty webhooks
   - Test alerts with `amtool` before deployment

2. **Document runbooks**
   - Every alert should have a runbook
   - Include exact commands to run
   - Link to relevant logs/dashboards

3. **Monitor alert fatigue**
   - If alert fires >10 times/day, adjust threshold
   - Use `for: duration` to avoid flapping
   - Silence known issues during maintenance

4. **Review dashboards weekly**
   - Check for new patterns
   - Adjust thresholds based on trends
   - Add new metrics as needed

5. **Test disaster scenarios**
   - Kill server, verify alert fires
   - Simulate RPC outage
   - Test escalation chain

---

### ❌ DON'T

1. **Don't ignore warnings**
   - "Just a warning" → Critical later
   - Address warnings during business hours

2. **Don't set thresholds too tight**
   - Alerts should indicate real problems
   - Not every spike needs an alert

3. **Don't alert on everything**
   - Alert fatigue → ignored critical alerts
   - Focus on actionable metrics

4. **Don't forget to rotate secrets**
   - Webhook URLs in config files
   - Rotate every 90 days

5. **Don't skip runbook links**
   - Alerts without actions = useless
   - Always include `runbook_url` in alerts

---

## Troubleshooting

### Issue: Metrics not appearing in Prometheus

**Symptoms:** Empty graphs in Grafana, no data in Prometheus

**Diagnosis:**
```bash
# 1. Check if /metrics endpoint works
curl http://localhost:8080/metrics

# 2. Check Prometheus targets
# Visit: http://localhost:9090/targets
# Should show "monero_marketplace" as UP

# 3. Check Prometheus logs
journalctl -u prometheus -n 50
```

**Solution:**
- If endpoint returns metrics but Prometheus shows DOWN:
  - Check firewall: `sudo ufw allow 8080/tcp`
  - Check prometheus.yml configuration
  - Restart Prometheus: `sudo systemctl restart prometheus`

---

### Issue: Alerts not firing

**Symptoms:** Known issue present but no alert received

**Diagnosis:**
```bash
# 1. Check Prometheus rules
curl http://localhost:9090/api/v1/rules

# 2. Check Alertmanager status
curl http://localhost:9093/api/v1/status

# 3. Test alert manually
# Visit: http://localhost:9090/alerts
# Should show pending/firing alerts
```

**Solution:**
- If rule shows "inactive":
  - Check `prometheus-alerts.yml` syntax
  - Verify `expr` query returns data
  - Check `for: duration` not too long

- If rule "pending" but not "firing":
  - Wait for `for: duration` to elapse
  - Check Alertmanager configuration
  - Verify webhook URLs

---

### Issue: Too many false positive alerts

**Symptoms:** Alert fires frequently but issue not real

**Solution:**
1. **Increase threshold:**
   ```yaml
   # Before
   expr: rate(rpc_calls_failed_total[5m]) > 0.05

   # After
   expr: rate(rpc_calls_failed_total[5m]) > 0.10
   ```

2. **Increase duration:**
   ```yaml
   # Before
   for: 1m

   # After
   for: 5m
   ```

3. **Add time-based silencing:**
   ```bash
   # Silence during known maintenance window
   amtool silence add alertname="HighRPCFailureRate" \
     --start="2025-11-08T02:00:00Z" \
     --end="2025-11-08T04:00:00Z" \
     --comment="Scheduled maintenance"
   ```

---

## Performance Optimization

### Prometheus Storage

**Default:** 30 days retention, 10GB max
**Disk usage:** ~100MB/day for 10,000 series

**Optimization:**
```yaml
# prometheus.yml
storage:
  tsdb:
    retention.time: 15d  # Reduce retention
    retention.size: 5GB   # Reduce max size
```

**Remote write** (for long-term storage):
```yaml
remote_write:
  - url: 'https://prometheus.example.com/api/v1/write'
    write_relabel_configs:
      # Only send critical metrics
      - source_labels: [__name__]
        regex: 'escrows_.*|rpc_calls_.*'
        action: keep
```

---

### Grafana Query Optimization

**Slow dashboard?**

1. **Reduce time range:**
   - Default: 6h → 1h
   - Use template variables for dynamic range

2. **Use recording rules:**
   ```yaml
   # prometheus-alerts.yml
   groups:
     - name: recording_rules
       interval: 1m
       rules:
         - record: escrow:success_rate:1h
           expr: (rate(escrows_completed_total[1h]) / rate(escrows_funded_total[1h])) * 100
   ```

3. **Limit series:**
   ```promql
   # Instead of:
   escrows_created_total

   # Use:
   escrows_created_total{instance="main"}
   ```

---

## Security Considerations

### Metrics Endpoint Security

**Problem:** `/metrics` endpoint exposes business metrics

**Solution:**
```rust
// server/src/main.rs
.route("/metrics", web::get().to(metrics_handler).wrap(AdminAuth))
```

**Alternative:** IP whitelist in reverse proxy (Nginx):
```nginx
location /metrics {
    allow 10.0.0.0/8;    # Internal network
    deny all;
    proxy_pass http://localhost:8080;
}
```

---

### Sensitive Data in Metrics

**DO NOT expose:**
- ❌ User IDs
- ❌ Email addresses
- ❌ Wallet addresses
- ❌ Transaction amounts (exact values)

**DO expose:**
- ✅ Aggregated counts
- ✅ Success/failure rates
- ✅ Percentiles/histograms
- ✅ Error types (not error messages with PII)

---

## Monitoring Checklist

**Before Production:**
- [ ] Prometheus scraping `/metrics` successfully
- [ ] Alertmanager configured with webhook
- [ ] Grafana dashboard imported and working
- [ ] Test critical alert fires correctly
- [ ] Runbook links documented for all alerts
- [ ] On-call rotation configured
- [ ] Escalation chain tested

**Monthly:**
- [ ] Review alert thresholds
- [ ] Check for new metric opportunities
- [ ] Audit alert fatigue (false positives)
- [ ] Test disaster recovery procedures
- [ ] Rotate webhook secrets

---

## Related Documentation

- **Metrics Code:** [server/src/monitoring/metrics.rs](../../server/src/monitoring/metrics.rs)
- **Alert Rules:** [monitoring/prometheus-alerts.yml](../../monitoring/prometheus-alerts.yml)
- **Dashboard:** [monitoring/grafana-dashboard.json](../../monitoring/grafana-dashboard.json)
- **Prometheus Config:** [monitoring/prometheus.yml](../../monitoring/prometheus.yml)
- **Security Docs:** [DOX/security/](../security/)

---

**Last Updated:** 2025-11-07
**Version:** 1.0.0
**Maintainer:** DevOps Team
