# Monitoring & Observability

Production monitoring setup for Monero Marketplace using Prometheus and Grafana.

## Quick Start

```bash
# 1. Install Prometheus and Grafana
sudo apt update
sudo apt install -y prometheus grafana

# 2. Copy Prometheus config
sudo cp prometheus.yml /etc/prometheus/prometheus.yml

# 3. Start services
sudo systemctl start prometheus
sudo systemctl start grafana-server
sudo systemctl enable prometheus
sudo systemctl enable grafana-server

# 4. Access dashboards
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000 (admin/admin)
```

## Exposed Metrics

The `/metrics` endpoint exposes the following metrics in Prometheus format:

### Escrow Metrics

```promql
escrows_created_total       # Total escrows created
escrows_funded_total        # Total escrows funded
escrows_completed_total     # Total escrows completed successfully
escrows_disputed_total      # Total escrows that entered dispute
escrows_resolved_total      # Total disputed escrows resolved
```

### RPC Metrics

```promql
rpc_calls_total             # Total Monero RPC calls made
rpc_calls_failed_total      # Total Monero RPC calls that failed
```

### Dispute Metrics

```promql
disputes_buyer_won_total    # Total disputes resolved in buyer favor
disputes_vendor_won_total   # Total disputes resolved in vendor favor
```

### System Metrics

```promql
uptime_seconds              # Server uptime in seconds
```

## Useful Prometheus Queries

### Escrow Success Rate

```promql
rate(escrows_completed_total[5m]) / rate(escrows_created_total[5m]) * 100
```

### Dispute Rate

```promql
rate(escrows_disputed_total[5m]) / rate(escrows_created_total[5m]) * 100
```

### RPC Failure Rate

```promql
rate(rpc_calls_failed_total[5m]) / rate(rpc_calls_total[5m]) * 100
```

### Arbiter Bias Detection

```promql
# Should be close to 50/50 if arbiter is fair
disputes_buyer_won_total / (disputes_buyer_won_total + disputes_vendor_won_total) * 100
```

## Grafana Dashboard Setup

### Create New Dashboard

1. Open Grafana: http://localhost:3000
2. Login: admin/admin (change on first login)
3. Configuration → Data Sources → Add Prometheus
   - URL: http://localhost:9090
   - Save & Test
4. Create → Dashboard → Add Panel

### Example Panels

#### Panel 1: Escrow Flow

```promql
# Graph showing escrow state transitions
escrows_created_total
escrows_funded_total
escrows_completed_total
```

#### Panel 2: Dispute Resolution

```promql
# Pie chart of dispute outcomes
disputes_buyer_won_total
disputes_vendor_won_total
```

#### Panel 3: RPC Health

```promql
# Graph showing RPC success rate over time
(rpc_calls_total - rpc_calls_failed_total) / rpc_calls_total * 100
```

## Alerts (Production)

Create `/etc/prometheus/alerts.yml`:

```yaml
groups:
  - name: marketplace_alerts
    interval: 30s
    rules:
      # Alert if RPC failure rate > 10%
      - alert: HighRPCFailureRate
        expr: rate(rpc_calls_failed_total[5m]) / rate(rpc_calls_total[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High RPC failure rate detected"
          description: "RPC failure rate is {{ $value }}% (threshold: 10%)"

      # Alert if dispute rate > 20%
      - alert: HighDisputeRate
        expr: rate(escrows_disputed_total[1h]) / rate(escrows_created_total[1h]) > 0.2
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "High dispute rate detected"
          description: "Dispute rate is {{ $value }}% (threshold: 20%)"

      # Alert if server down
      - alert: ServerDown
        expr: up{job="marketplace-server"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Marketplace server is down"
          description: "Server has been down for 1 minute"
```

Enable alerts in `prometheus.yml`:

```yaml
rule_files:
  - 'alerts.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']
```

## Retention & Storage

Prometheus default retention: 15 days

Increase retention:

```bash
# Edit /etc/default/prometheus
ARGS="--storage.tsdb.retention.time=90d"

sudo systemctl restart prometheus
```

## Security

### OPSEC Considerations

- Metrics endpoint does NOT expose:
  - ✅ User IDs
  - ✅ Transaction amounts
  - ✅ Monero addresses
  - ✅ Wallet keys
  - ✅ IP addresses

- Metrics are aggregate counts only (no PII)
- Bind Prometheus to localhost only (not 0.0.0.0)
- Use SSH tunnel for remote access:

```bash
# Remote access via SSH tunnel
ssh -L 9090:localhost:9090 user@marketplace-server
# Then access: http://localhost:9090
```

## Testing

Verify metrics endpoint:

```bash
# Check metrics are exposed
curl http://localhost:8080/metrics

# Should return Prometheus format:
# # HELP escrows_created_total Total escrows created
# # TYPE escrows_created_total counter
# escrows_created_total 0
# ...
```

Test Prometheus scraping:

```bash
# Check Prometheus is scraping
curl http://localhost:9090/api/v1/targets

# Should show marketplace-server target as "up"
```

## Logs vs Metrics

**Metrics (Prometheus):**
- Counters, gauges, histograms
- Time-series data
- Fast queries, aggregation
- Retention: 90 days

**Logs (server.log):**
- Detailed events, errors
- Text search
- Debugging, forensics
- Retention: 30 days (rotated)

Use both:
- Metrics for monitoring, alerting
- Logs for root cause analysis

## Integration with Code

The metrics are tracked in handlers:

```rust
use server::monitoring::Metrics;
use actix_web::web;

async fn create_escrow(
    metrics: web::Data<Metrics>,
    // ... other params
) -> Result<HttpResponse> {
    // Create escrow...

    metrics.record_escrow_created();

    Ok(HttpResponse::Created().json(escrow))
}
```

All handlers should call appropriate metric recording methods.

## Production Deployment

For production, also consider:

1. **Node Exporter** - System metrics (CPU, RAM, disk)
2. **Alert Manager** - Alert routing, deduplication
3. **Grafana Cloud** - Managed Grafana (avoid self-hosting)
4. **Loki** - Log aggregation (complement to metrics)

Minimal production stack:

```
Marketplace Server → Prometheus → Grafana
                  ↓
              Alertmanager → Email/Slack
```

## Troubleshooting

### Metrics not updating

1. Check server is running: `ps aux | grep server`
2. Check metrics endpoint: `curl http://localhost:8080/metrics`
3. Check Prometheus targets: http://localhost:9090/targets
4. Check server logs: `tail -f server.log | grep metrics`

### Prometheus not scraping

1. Verify prometheus.yml syntax: `promtool check config /etc/prometheus/prometheus.yml`
2. Check Prometheus logs: `sudo journalctl -u prometheus -f`
3. Verify firewall: `sudo ufw status | grep 8080`

### Grafana can't reach Prometheus

1. Test connection: `curl http://localhost:9090/api/v1/status/config`
2. Check Grafana logs: `sudo journalctl -u grafana-server -f`
3. Verify data source URL in Grafana settings

## Next Steps

1. Set up basic Grafana dashboard
2. Configure alerting rules
3. Enable log rotation
4. Add node_exporter for system metrics
5. Set up off-site monitoring (UptimeRobot, etc.)
