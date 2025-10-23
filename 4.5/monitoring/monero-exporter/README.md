# Monero Prometheus Exporter

Custom Prometheus exporter for Monero wallet RPC metrics.

## Features

- Exports wallet balance (locked + unlocked)
- Exports sync height
- Exports number of unspent outputs
- Tracks RPC call success/failure rates
- Supports multiple wallets (buyer, vendor, arbiter)

## Metrics Exported

| Metric | Type | Description |
|--------|------|-------------|
| `monero_wallet_balance_piconero` | Gauge | Total wallet balance in piconero (1 XMR = 1e12 piconero) |
| `monero_wallet_unlocked_balance_piconero` | Gauge | Unlocked (spendable) balance in piconero |
| `monero_wallet_height` | Gauge | Current wallet sync height |
| `monero_wallet_num_unspent_outputs` | Gauge | Number of unspent transaction outputs |
| `monero_wallet_rpc_calls_total` | Counter | Total RPC calls made (by method and status) |
| `monero_wallet_rpc_errors_total` | Counter | Total RPC errors (by method) |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EXPORTER_PORT` | 9101 | Port to expose Prometheus metrics |
| `POLL_INTERVAL` | 30 | Seconds between wallet polls |
| `BUYER_RPC_HOST` | monero-wallet-rpc-buyer | Buyer wallet RPC host |
| `BUYER_RPC_PORT` | 18082 | Buyer wallet RPC port |
| `VENDOR_RPC_HOST` | monero-wallet-rpc-vendor | Vendor wallet RPC host |
| `VENDOR_RPC_PORT` | 18083 | Vendor wallet RPC port |
| `ARBITER_RPC_HOST` | monero-wallet-rpc-arbiter | Arbiter wallet RPC host |
| `ARBITER_RPC_PORT` | 18084 | Arbiter wallet RPC port |

## Usage

### Docker Compose (Recommended)

Already integrated in `4.5/docker/docker-compose.yml`:

```yaml
monero-exporter:
  build: ../monitoring/monero-exporter
  container_name: marketplace-monero-exporter
  restart: unless-stopped
  ports:
    - "9101:9101"
  environment:
    - EXPORTER_PORT=9101
    - POLL_INTERVAL=30
    - BUYER_RPC_HOST=monero-wallet-rpc-buyer
    - VENDOR_RPC_HOST=monero-wallet-rpc-vendor
    - ARBITER_RPC_HOST=monero-wallet-rpc-arbiter
  networks:
    - marketplace-network
```

### Standalone

```bash
# Install dependencies
pip install -r requirements.txt

# Run exporter
python3 exporter.py
```

### Test Metrics Endpoint

```bash
curl http://localhost:9101/metrics
```

## Prometheus Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'monero-wallets'
    static_configs:
      - targets: ['monero-exporter:9101']
    scrape_interval: 30s
```

## Security Considerations

- Exporter runs as non-root user (UID 1000)
- Only connects to localhost wallet RPCs (no external exposure)
- No authentication credentials in metrics
- Metrics port (9101) should only be accessible to Prometheus

## Troubleshooting

### No metrics appearing

- Check wallet RPC is running: `curl http://monero-wallet-rpc-buyer:18082/json_rpc`
- Check exporter logs: `docker logs marketplace-monero-exporter`
- Verify network connectivity between exporter and wallet RPC

### Balance showing as 0

- Wallet may not be synced yet (check `monero_wallet_height`)
- Wallet may actually be empty (testnet wallets need faucet funds)
- RPC authentication required but not configured

## Development

```bash
# Build Docker image
docker build -t monero-exporter:latest .

# Run locally for testing
docker run -p 9101:9101 \
  -e BUYER_RPC_HOST=localhost \
  monero-exporter:latest
```

## References

- [Monero Wallet RPC Documentation](https://www.getmonero.org/resources/developer-guides/wallet-rpc.html)
- [Prometheus Python Client](https://github.com/prometheus/client_python)
