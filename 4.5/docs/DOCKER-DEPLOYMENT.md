# Docker Deployment Guide

## Quick Start

### Development
```bash
./scripts/docker-start.sh
```

### Production
```bash
# 1. Set production environment
cp .env.example .env
# Edit .env with production values

# 2. Start stack
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 3. Verify health
./scripts/docker-health-check.sh
```

## Architecture

```
┌─────────────────────────────────────────────┐
│              Load Balancer / Nginx          │
│            (SSL/TLS Termination)            │
└────────────────┬────────────────────────────┘
                 │
        ┌────────▼────────┐
        │  marketplace-   │
        │     server      │◄───────┐
        └────┬────────┬───┘        │
             │        │             │
    ┌────────▼───┐  ┌▼─────────┐   │
    │ Prometheus │  │  Grafana │   │
    └────────────┘  └──────────┘   │
             │                      │
    ┌────────▼────────────────┐    │
    │  Monero Wallet RPC      │    │
    │  - buyer:18082          │◄───┤
    │  - vendor:18083         │    │
    │  - arbiter:18084        │    │
    └─────────────────────────┘    │
             │                      │
    ┌────────▼────────┐  ┌─────────▼──┐
    │ Loki + Promtail │  │ Alertmanager│
    └─────────────────┘  └────────────┘
```

## Storage Volumes

| Volume | Purpose | Backup Required |
|--------|---------|-----------------|
| `./data` | SQLCipher database | ✅ CRITICAL |
| `./wallets` | Monero wallet files | ✅ CRITICAL |
| `prometheus-data` | Metrics (30 days) | ⚠️ Optional |
| `grafana-data` | Dashboards config | ⚠️ Optional |
| `loki-data` | Logs (30 days) | ⚠️ Optional |

## Troubleshooting

### Container won't start
```bash
# Check logs
docker-compose logs server

# Common issues:
# 1. Port already in use
sudo lsof -i :8080
# 2. Missing .env file
cp .env.example .env
# 3. Permission issues
sudo chown -R 1000:1000 data/ wallets/
```

### Wallet RPC unreachable
```bash
# Verify wallet container running
docker-compose ps monero-wallet-rpc-buyer

# Check wallet logs
docker-compose logs monero-wallet-rpc-buyer

# Restart wallet
docker-compose restart monero-wallet-rpc-buyer
```

### Database locked
```bash
# Stop all services
docker-compose down

# Remove lock file
rm data/marketplace.db-wal data/marketplace.db-shm

# Restart
docker-compose up -d
```

## Performance Tuning

### Resource Limits (Production)
```yaml
services:
  server:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          cpus: '1.0'
          memory: 1G
```

### Wallet RPC Optimization
```bash
# Increase RPC threads
command: >
  monero-wallet-rpc
  --rpc-threads 4
  --max-concurrency 10
```
