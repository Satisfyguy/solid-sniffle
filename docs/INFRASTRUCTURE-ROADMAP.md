# Infrastructure & Production Readiness Roadmap

**Projet:** Monero Marketplace
**Version:** 1.0
**Date:** 2025-10-20
**Durée totale:** 4 semaines (33 jours)
**Prérequis:** Milestone 2.3 complété (Database & Encryption)

---

## Contexte

Ce document détaille la **Phase 4.5: Infrastructure & Production Readiness** - les composants manquants pour déployer le Monero Marketplace en production.

**Score Production-Ready actuel:** 65/100
- Code Quality: 90/100 ✅ Excellent
- Infrastructure: 30/100 ⚠️ Insuffisant
- Monitoring: 20/100 ⚠️ Dangereux
- Backup/DR: 10/100 ❌ Critique

**Objectif:** Atteindre 90/100 pour déploiement mainnet sécurisé.

---

## Vue d'ensemble des Milestones

| Milestone | Durée | Priorité | Risque |
|-----------|-------|----------|--------|
| 4.5.1: Containerization & Docker | 5 jours | CRITIQUE | Faible |
| 4.5.2: Monitoring & Observability | 5 jours | CRITIQUE | Moyen |
| 4.5.3: Backup & Disaster Recovery | 5 jours | CRITIQUE | Élevé |
| 4.5.4: CI/CD Pipeline | 5 jours | Haute | Faible |
| 4.5.5: Load Testing & Performance | 3 jours | Haute | Moyen |
| 4.5.6: Security Hardening | 4 jours | CRITIQUE | Élevé |
| 4.5.7: Documentation Opérationnelle | 3 jours | Moyenne | Faible |
| 4.5.8: Deployment Automation | 3 jours | Haute | Moyen |

**Total:** 33 jours (~4-5 semaines avec buffer)

---

## Success Criteria

- [ ] **Docker:** Application containerisée avec <10s cold start
- [ ] **Multi-wallet:** 3 wallets Monero RPC isolés (buyer/vendor/arbiter)
- [ ] **Prometheus:** Métriques collectées avec <5s scrape interval
- [ ] **Grafana:** 3+ dashboards opérationnels (HTTP, Escrow, System)
- [ ] **Loki:** Logs centralisés avec 30+ jours rétention
- [ ] **Alertmanager:** Alertes critiques < 2 min notification
- [ ] **Backup:** Snapshots automatiques SQLCipher toutes les 6h
- [ ] **Recovery:** RTO < 15 min, RPO < 1h
- [ ] **CI/CD:** Pipeline GitHub Actions avec tests + security scan
- [ ] **Load Test:** Support 100 req/s avec p95 < 200ms
- [ ] **TLS:** Certificats auto-renouvelés (Let's Encrypt)
- [ ] **Firewall:** UFW configuré avec whitelist stricte
- [ ] **Secrets:** Vault/SOPS pour rotation automatique
- [ ] **Tor:** Hidden service avec backup .onion address
- [ ] **Docs:** Runbook opérationnel complet (incidents, DR, scaling)

---

# Milestone 4.5.1: Containerization & Docker

**Durée:** 5 jours
**Priorité:** CRITIQUE
**Objectif:** Application complètement containerisée avec orchestration multi-wallet

## Task 4.5.1.1: Dockerfile Multi-Stage (1 jour)

**Fichier:** `Dockerfile`

```dockerfile
# ============================================================================
# Stage 1: Builder - Compilation Rust optimisée
# ============================================================================
FROM rust:1.75-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests (cache layer)
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/

# Create dummy main to cache dependencies
RUN mkdir -p server/src && \
    echo "fn main() {}" > server/src/main.rs && \
    cargo build --release --package server && \
    rm -rf server/src

# Copy actual source code
COPY server/src ./server/src
COPY server/migrations ./server/migrations

# Build application (invalidates cache only on source changes)
RUN cargo build --release --package server

# ============================================================================
# Stage 2: Runtime - Image minimale Debian
# ============================================================================
FROM debian:12-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    tor \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 marketplace && \
    mkdir -p /app/data /app/logs && \
    chown -R marketplace:marketplace /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/server /app/server

# Copy configuration
COPY server/.env.example /app/.env

# Switch to non-root user
USER marketplace

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 8080

# Run application
CMD ["/app/server"]
```

**Validation:**
```bash
# Build image
docker build -t monero-marketplace:latest .

# Check image size (target: <150MB)
docker images monero-marketplace:latest

# Run container
docker run -d \
  --name marketplace-test \
  -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  monero-marketplace:latest

# Verify health
docker exec marketplace-test curl -f http://localhost:8080/health

# Check logs
docker logs marketplace-test

# Cleanup
docker stop marketplace-test && docker rm marketplace-test
```

---

## Task 4.5.1.2: Docker Compose (2 jours)

**Fichier:** `docker-compose.yml`

```yaml
version: '3.8'

services:
  # ============================================================================
  # Main Application Server
  # ============================================================================
  server:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: marketplace-server
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:///app/data/marketplace.db?mode=rwc
      - RUST_LOG=info
      - MONERO_BUYER_RPC_URL=http://monero-wallet-rpc-buyer:18082/json_rpc
      - MONERO_VENDOR_RPC_URL=http://monero-wallet-rpc-vendor:18083/json_rpc
      - MONERO_ARBITER_RPC_URL=http://monero-wallet-rpc-arbiter:18084/json_rpc
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    depends_on:
      - monero-wallet-rpc-buyer
      - monero-wallet-rpc-vendor
      - monero-wallet-rpc-arbiter
    networks:
      - marketplace-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # ============================================================================
  # Monero Wallet RPC - Buyer
  # ============================================================================
  monero-wallet-rpc-buyer:
    image: ghcr.io/monerodocs/monero:v0.18.3.1
    container_name: monero-wallet-rpc-buyer
    restart: unless-stopped
    command: >
      monero-wallet-rpc
      --testnet
      --rpc-bind-ip 0.0.0.0
      --rpc-bind-port 18082
      --confirm-external-bind
      --wallet-dir /wallet
      --log-level 1
      --disable-rpc-login
    volumes:
      - ./wallets/buyer:/wallet
    networks:
      - marketplace-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:18082/json_rpc"]
      interval: 30s
      timeout: 10s
      retries: 3

  # ============================================================================
  # Monero Wallet RPC - Vendor
  # ============================================================================
  monero-wallet-rpc-vendor:
    image: ghcr.io/monerodocs/monero:v0.18.3.1
    container_name: monero-wallet-rpc-vendor
    restart: unless-stopped
    command: >
      monero-wallet-rpc
      --testnet
      --rpc-bind-ip 0.0.0.0
      --rpc-bind-port 18083
      --confirm-external-bind
      --wallet-dir /wallet
      --log-level 1
      --disable-rpc-login
    volumes:
      - ./wallets/vendor:/wallet
    networks:
      - marketplace-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:18083/json_rpc"]
      interval: 30s
      timeout: 10s
      retries: 3

  # ============================================================================
  # Monero Wallet RPC - Arbiter
  # ============================================================================
  monero-wallet-rpc-arbiter:
    image: ghcr.io/monerodocs/monero:v0.18.3.1
    container_name: monero-wallet-rpc-arbiter
    restart: unless-stopped
    command: >
      monero-wallet-rpc
      --testnet
      --rpc-bind-ip 0.0.0.0
      --rpc-bind-port 18084
      --confirm-external-bind
      --wallet-dir /wallet
      --log-level 1
      --disable-rpc-login
    volumes:
      - ./wallets/arbiter:/wallet
    networks:
      - marketplace-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:18084/json_rpc"]
      interval: 30s
      timeout: 10s
      retries: 3

  # ============================================================================
  # Prometheus - Metrics Collection
  # ============================================================================
  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: marketplace-prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./monitoring/alerts:/etc/prometheus/alerts
      - prometheus-data:/prometheus
    networks:
      - marketplace-network

  # ============================================================================
  # Grafana - Visualization
  # ============================================================================
  grafana:
    image: grafana/grafana:10.2.2
    container_name: marketplace-grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123_CHANGE_ME
      - GF_INSTALL_PLUGINS=grafana-piechart-panel
    volumes:
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources
      - grafana-data:/var/lib/grafana
    networks:
      - marketplace-network
    depends_on:
      - prometheus

  # ============================================================================
  # Loki - Log Aggregation
  # ============================================================================
  loki:
    image: grafana/loki:2.9.3
    container_name: marketplace-loki
    restart: unless-stopped
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml
    volumes:
      - ./monitoring/loki-config.yaml:/etc/loki/local-config.yaml
      - loki-data:/loki
    networks:
      - marketplace-network

  # ============================================================================
  # Promtail - Log Shipper
  # ============================================================================
  promtail:
    image: grafana/promtail:2.9.3
    container_name: marketplace-promtail
    restart: unless-stopped
    volumes:
      - ./logs:/var/log/marketplace
      - ./monitoring/promtail-config.yaml:/etc/promtail/config.yaml
    command: -config.file=/etc/promtail/config.yaml
    networks:
      - marketplace-network
    depends_on:
      - loki

  # ============================================================================
  # Alertmanager - Alert Routing
  # ============================================================================
  alertmanager:
    image: prom/alertmanager:v0.26.0
    container_name: marketplace-alertmanager
    restart: unless-stopped
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
    networks:
      - marketplace-network

networks:
  marketplace-network:
    driver: bridge

volumes:
  prometheus-data:
  grafana-data:
  loki-data:
```

**Validation:**
```bash
# Start all services
docker-compose up -d

# Verify all containers running
docker-compose ps

# Check logs
docker-compose logs -f server

# Test endpoints
curl http://localhost:8080/health       # Application
curl http://localhost:9090/-/healthy    # Prometheus
curl http://localhost:3000/api/health   # Grafana

# Stop all services
docker-compose down
```

---

## Task 4.5.1.3: Docker Management Scripts (1 jour)

**Fichier:** `scripts/docker-start.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "🚀 Starting Monero Marketplace Docker Stack..."

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker daemon not running"
    exit 1
fi

# Create required directories
mkdir -p data logs wallets/{buyer,vendor,arbiter} monitoring/{alerts,grafana}

# Pull latest images
echo "📦 Pulling images..."
docker-compose pull

# Start services in order
echo "🏗️  Starting infrastructure..."
docker-compose up -d monero-wallet-rpc-buyer monero-wallet-rpc-vendor monero-wallet-rpc-arbiter
sleep 10

echo "📊 Starting monitoring..."
docker-compose up -d prometheus grafana loki promtail alertmanager
sleep 5

echo "🌐 Starting application server..."
docker-compose up -d server

# Wait for health checks
echo "🏥 Waiting for health checks..."
for i in {1..30}; do
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ Application healthy!"
        break
    fi
    echo "⏳ Attempt $i/30..."
    sleep 2
done

# Display status
echo ""
echo "📋 Service Status:"
docker-compose ps

echo ""
echo "🔗 URLs:"
echo "  - Application: http://localhost:8080"
echo "  - Prometheus:  http://localhost:9090"
echo "  - Grafana:     http://localhost:3000 (admin/admin123_CHANGE_ME)"
echo "  - Alertmanager: http://localhost:9093"

echo ""
echo "📝 Logs:"
echo "  docker-compose logs -f server"
```

**Fichier:** `scripts/docker-health-check.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "🏥 Health Check - Monero Marketplace"
echo "===================================="

# Check each service
services=(
    "server:8080:/health"
    "prometheus:9090:/-/healthy"
    "grafana:3000:/api/health"
    "loki:3100:/ready"
    "alertmanager:9093:/-/healthy"
)

all_healthy=true

for service_info in "${services[@]}"; do
    IFS=':' read -r name port path <<< "$service_info"

    if curl -sf "http://localhost:${port}${path}" > /dev/null; then
        echo "✅ ${name} - HEALTHY"
    else
        echo "❌ ${name} - UNHEALTHY"
        all_healthy=false
    fi
done

# Check Monero wallets
for wallet in buyer vendor arbiter; do
    case $wallet in
        buyer)   port=18082 ;;
        vendor)  port=18083 ;;
        arbiter) port=18084 ;;
    esac

    if curl -sf "http://localhost:${port}/json_rpc" \
        -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}' \
        -H 'Content-Type: application/json' > /dev/null; then
        echo "✅ monero-wallet-rpc-${wallet} - HEALTHY"
    else
        echo "❌ monero-wallet-rpc-${wallet} - UNHEALTHY"
        all_healthy=false
    fi
done

echo ""
if $all_healthy; then
    echo "✅ All services healthy"
    exit 0
else
    echo "❌ Some services unhealthy"
    exit 1
fi
```

**Fichier:** `scripts/docker-stop.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "🛑 Stopping Monero Marketplace..."

# Graceful shutdown
docker-compose stop -t 30

# Remove containers
docker-compose down

echo "✅ Stopped"
```

**Rendre exécutables:**
```bash
chmod +x scripts/docker-*.sh
```

---

## Task 4.5.1.4: Documentation (1 jour)

**Fichier:** `docs/DOCKER-DEPLOYMENT.md`

```markdown
# Docker Deployment Guide

## Quick Start

### Development
\`\`\`bash
./scripts/docker-start.sh
\`\`\`

### Production
\`\`\`bash
# 1. Set production environment
cp .env.example .env
# Edit .env with production values

# 2. Start stack
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 3. Verify health
./scripts/docker-health-check.sh
\`\`\`

## Architecture

\`\`\`
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
\`\`\`

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
\`\`\`bash
# Check logs
docker-compose logs server

# Common issues:
# 1. Port already in use
sudo lsof -i :8080
# 2. Missing .env file
cp .env.example .env
# 3. Permission issues
sudo chown -R 1000:1000 data/ wallets/
\`\`\`

### Wallet RPC unreachable
\`\`\`bash
# Verify wallet container running
docker-compose ps monero-wallet-rpc-buyer

# Check wallet logs
docker-compose logs monero-wallet-rpc-buyer

# Restart wallet
docker-compose restart monero-wallet-rpc-buyer
\`\`\`

### Database locked
\`\`\`bash
# Stop all services
docker-compose down

# Remove lock file
rm data/marketplace.db-wal data/marketplace.db-shm

# Restart
docker-compose up -d
\`\`\`

## Performance Tuning

### Resource Limits (Production)
\`\`\`yaml
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
\`\`\`

### Wallet RPC Optimization
\`\`\`bash
# Increase RPC threads
command: >
  monero-wallet-rpc
  --rpc-threads 4
  --max-concurrency 10
\`\`\`
```

---

# Milestone 4.5.2: Monitoring & Observability

**Durée:** 5 jours
**Priorité:** CRITIQUE
**Objectif:** Monitoring complet avec métriques, logs centralisés, alertes

## Task 4.5.2.1: Prometheus Configuration (1 jour)

**Fichier:** `monitoring/prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'monero-marketplace'
    environment: 'production'

# Alertmanager configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093

# Load alert rules
rule_files:
  - '/etc/prometheus/alerts/*.yml'

# Scrape configurations
scrape_configs:
  # ============================================================================
  # Application Server
  # ============================================================================
  - job_name: 'marketplace-server'
    static_configs:
      - targets: ['server:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # ============================================================================
  # Monero Wallet RPC - Buyer
  # ============================================================================
  - job_name: 'monero-wallet-buyer'
    static_configs:
      - targets: ['monero-wallet-rpc-buyer:18082']
    metrics_path: '/metrics'
    scrape_interval: 30s

  # ============================================================================
  # Monero Wallet RPC - Vendor
  # ============================================================================
  - job_name: 'monero-wallet-vendor'
    static_configs:
      - targets: ['monero-wallet-rpc-vendor:18083']
    metrics_path: '/metrics'
    scrape_interval: 30s

  # ============================================================================
  # Monero Wallet RPC - Arbiter
  # ============================================================================
  - job_name: 'monero-wallet-arbiter'
    static_configs:
      - targets: ['monero-wallet-rpc-arbiter:18084']
    metrics_path: '/metrics'
    scrape_interval: 30s

  # ============================================================================
  # Prometheus Self-Monitoring
  # ============================================================================
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
```

**Fichier:** `monitoring/alerts/marketplace.yml`

```yaml
groups:
  - name: marketplace_alerts
    interval: 30s
    rules:
      # ========================================================================
      # High Error Rate
      # ========================================================================
      - alert: HighErrorRate
        expr: |
          rate(http_requests_total{status=~"5.."}[5m]) /
          rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "{{ $value | humanizePercentage }} of requests failing on {{ $labels.instance }}"

      # ========================================================================
      # Service Down
      # ========================================================================
      - alert: ServiceDown
        expr: up{job="marketplace-server"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Marketplace server is down"
          description: "Server {{ $labels.instance }} has been down for > 2 minutes"

      # ========================================================================
      # Slow Response Time
      # ========================================================================
      - alert: SlowResponseTime
        expr: |
          histogram_quantile(0.95,
            rate(http_request_duration_seconds_bucket[5m])
          ) > 2.0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Slow response time (p95 > 2s)"
          description: "95th percentile response time is {{ $value }}s"

      # ========================================================================
      # Escrow Stuck in Pending
      # ========================================================================
      - alert: EscrowStuckInPending
        expr: |
          escrow_total{state="pending"} > 0 AND
          time() - escrow_last_update_timestamp > 3600
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Escrow stuck in pending state"
          description: "{{ $value }} escrows pending for > 1 hour"

      # ========================================================================
      # High Database Lock Contention
      # ========================================================================
      - alert: HighDatabaseLockContention
        expr: rate(db_lock_wait_seconds_total[5m]) > 1.0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High database lock contention"
          description: "Database locks waiting {{ $value }}s/s"

      # ========================================================================
      # Wallet RPC Unreachable
      # ========================================================================
      - alert: WalletRPCUnreachable
        expr: up{job=~"monero-wallet-.*"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Monero wallet RPC unreachable"
          description: "Wallet {{ $labels.job }} down for > 5 minutes"

      # ========================================================================
      # Disk Space Low
      # ========================================================================
      - alert: DiskSpaceLow
        expr: |
          (node_filesystem_avail_bytes{mountpoint="/app/data"} /
           node_filesystem_size_bytes{mountpoint="/app/data"}) < 0.10
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Disk space critically low"
          description: "Only {{ $value | humanizePercentage }} space remaining"

      # ========================================================================
      # Memory Usage High
      # ========================================================================
      - alert: MemoryUsageHigh
        expr: |
          (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) > 0.90
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage at {{ $value | humanizePercentage }}"

      # ========================================================================
      # WebSocket Connections Spike
      # ========================================================================
      - alert: WebSocketConnectionsSpike
        expr: |
          rate(websocket_connections_total[5m]) > 100
        for: 5m
        labels:
          severity: info
        annotations:
          summary: "Unusual WebSocket activity"
          description: "{{ $value }} new connections/s (possible DoS)"

      # ========================================================================
      # SSL Certificate Expiring
      # ========================================================================
      - alert: SSLCertificateExpiring
        expr: |
          ssl_cert_expiry_seconds < (30 * 24 * 3600)
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "SSL certificate expiring soon"
          description: "Certificate expires in {{ $value | humanizeDuration }}"
```

---

## Task 4.5.2.2: Application Instrumentation (2 jours)

**Fichier:** `server/src/metrics.rs`

```rust
//! Prometheus metrics instrumentation
//!
//! This module exports metrics for:
//! - HTTP request latency/throughput
//! - Escrow state transitions
//! - Database operations
//! - WebSocket connections
//! - Monero RPC calls

use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec,
    HistogramVec, IntCounterVec, IntGaugeVec, TextEncoder, Encoder,
};
use actix_web::{HttpResponse, Result as ActixResult};

lazy_static! {
    // ========================================================================
    // HTTP Metrics
    // ========================================================================

    /// Total HTTP requests by method, path, status
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total HTTP requests received",
        &["method", "path", "status"]
    )
    .expect("Failed to register HTTP_REQUESTS_TOTAL");

    /// HTTP request duration histogram (seconds)
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latency in seconds",
        &["method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Failed to register HTTP_REQUEST_DURATION");

    // ========================================================================
    // Escrow Metrics
    // ========================================================================

    /// Current escrows by state
    pub static ref ESCROW_TOTAL: IntGaugeVec = register_int_gauge_vec!(
        "escrow_total",
        "Total escrows by state",
        &["state"]
    )
    .expect("Failed to register ESCROW_TOTAL");

    /// Escrow state transitions
    pub static ref ESCROW_STATE_TRANSITIONS: IntCounterVec = register_int_counter_vec!(
        "escrow_state_transitions_total",
        "Total escrow state transitions",
        &["from_state", "to_state"]
    )
    .expect("Failed to register ESCROW_STATE_TRANSITIONS");

    /// Last escrow update timestamp (Unix epoch)
    pub static ref ESCROW_LAST_UPDATE: IntGaugeVec = register_int_gauge_vec!(
        "escrow_last_update_timestamp",
        "Timestamp of last escrow update",
        &["escrow_id"]
    )
    .expect("Failed to register ESCROW_LAST_UPDATE");

    // ========================================================================
    // Database Metrics
    // ========================================================================

    /// Database operation duration (seconds)
    pub static ref DB_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "db_operation_duration_seconds",
        "Database operation latency",
        &["operation", "table"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    )
    .expect("Failed to register DB_OPERATION_DURATION");

    /// Database lock wait time (seconds)
    pub static ref DB_LOCK_WAIT_DURATION: HistogramVec = register_histogram_vec!(
        "db_lock_wait_seconds",
        "Time spent waiting for database locks",
        &["operation"],
        vec![0.001, 0.01, 0.1, 1.0, 10.0]
    )
    .expect("Failed to register DB_LOCK_WAIT_DURATION");

    /// Database errors by type
    pub static ref DB_ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "db_errors_total",
        "Total database errors",
        &["error_type"]
    )
    .expect("Failed to register DB_ERRORS_TOTAL");

    // ========================================================================
    // WebSocket Metrics
    // ========================================================================

    /// Active WebSocket connections
    pub static ref WEBSOCKET_CONNECTIONS: IntGaugeVec = register_int_gauge_vec!(
        "websocket_connections_active",
        "Number of active WebSocket connections",
        &["user_type"]
    )
    .expect("Failed to register WEBSOCKET_CONNECTIONS");

    /// Total WebSocket connections (counter)
    pub static ref WEBSOCKET_CONNECTIONS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "websocket_connections_total",
        "Total WebSocket connections opened",
        &["user_type"]
    )
    .expect("Failed to register WEBSOCKET_CONNECTIONS_TOTAL");

    /// WebSocket messages sent
    pub static ref WEBSOCKET_MESSAGES_SENT: IntCounterVec = register_int_counter_vec!(
        "websocket_messages_sent_total",
        "Total WebSocket messages sent",
        &["message_type"]
    )
    .expect("Failed to register WEBSOCKET_MESSAGES_SENT");

    // ========================================================================
    // Monero RPC Metrics
    // ========================================================================

    /// Monero RPC call duration (seconds)
    pub static ref MONERO_RPC_DURATION: HistogramVec = register_histogram_vec!(
        "monero_rpc_duration_seconds",
        "Monero RPC call latency",
        &["method", "wallet"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
    )
    .expect("Failed to register MONERO_RPC_DURATION");

    /// Monero RPC errors
    pub static ref MONERO_RPC_ERRORS: IntCounterVec = register_int_counter_vec!(
        "monero_rpc_errors_total",
        "Total Monero RPC errors",
        &["method", "error_type"]
    )
    .expect("Failed to register MONERO_RPC_ERRORS");

    // ========================================================================
    // System Metrics
    // ========================================================================

    /// Application uptime (seconds)
    pub static ref UPTIME_SECONDS: IntGaugeVec = register_int_gauge_vec!(
        "uptime_seconds",
        "Application uptime in seconds",
        &[]
    )
    .expect("Failed to register UPTIME_SECONDS");
}

/// Actix-Web handler to expose Prometheus metrics
pub async fn metrics_handler() -> ActixResult<HttpResponse> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Metrics encoding failed: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer))
}

/// Record HTTP request
pub fn record_http_request(method: &str, path: &str, status: u16, duration_secs: f64) {
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, path, &status.to_string()])
        .inc();

    HTTP_REQUEST_DURATION
        .with_label_values(&[method, path])
        .observe(duration_secs);
}

/// Update escrow state gauge
pub fn update_escrow_gauge(state: &str, count: i64) {
    ESCROW_TOTAL
        .with_label_values(&[state])
        .set(count);
}

/// Record escrow state transition
pub fn record_escrow_transition(from: &str, to: &str) {
    ESCROW_STATE_TRANSITIONS
        .with_label_values(&[from, to])
        .inc();
}

/// Record database operation
pub fn record_db_operation(operation: &str, table: &str, duration_secs: f64) {
    DB_OPERATION_DURATION
        .with_label_values(&[operation, table])
        .observe(duration_secs);
}

/// Increment WebSocket connection counter
pub fn increment_websocket_connections(user_type: &str) {
    WEBSOCKET_CONNECTIONS
        .with_label_values(&[user_type])
        .inc();

    WEBSOCKET_CONNECTIONS_TOTAL
        .with_label_values(&[user_type])
        .inc();
}

/// Decrement WebSocket connection counter
pub fn decrement_websocket_connections(user_type: &str) {
    WEBSOCKET_CONNECTIONS
        .with_label_values(&[user_type])
        .dec();
}

/// Record Monero RPC call
pub fn record_monero_rpc_call(method: &str, wallet: &str, duration_secs: f64) {
    MONERO_RPC_DURATION
        .with_label_values(&[method, wallet])
        .observe(duration_secs);
}

/// Record Monero RPC error
pub fn record_monero_rpc_error(method: &str, error_type: &str) {
    MONERO_RPC_ERRORS
        .with_label_values(&[method, error_type])
        .inc();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_http_request() {
        record_http_request("GET", "/api/listings", 200, 0.123);

        let metric = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/api/listings", "200"])
            .get();

        assert!(metric > 0);
    }

    #[test]
    fn test_escrow_metrics() {
        update_escrow_gauge("pending", 5);
        record_escrow_transition("pending", "funded");

        let gauge = ESCROW_TOTAL
            .with_label_values(&["pending"])
            .get();

        assert_eq!(gauge, 5);
    }
}
```

**Intégration dans `server/src/main.rs`:**

```rust
use server::metrics;

// Add metrics endpoint
.service(
    web::resource("/metrics")
        .route(web::get().to(metrics::metrics_handler))
)
```

**Middleware pour tracking automatique:**

```rust
// server/src/middleware/metrics.rs
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
use std::task::{Context, Poll};
use std::time::Instant;

pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddlewareService { service })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed().as_secs_f64();
            let status = res.status().as_u16();

            crate::metrics::record_http_request(&method, &path, status, duration);

            Ok(res)
        })
    }
}
```

---

## Task 4.5.2.3: Grafana Dashboards (1 jour)

**Fichier:** `monitoring/grafana/dashboards/http-overview.json`

```json
{
  "dashboard": {
    "title": "HTTP Overview",
    "panels": [
      {
        "title": "Request Rate (req/s)",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Error Rate (%)",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m]) / rate(http_requests_total[5m]) * 100"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Response Time (p50/p95/p99)",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Top Endpoints by Traffic",
        "targets": [
          {
            "expr": "topk(10, sum by (path) (rate(http_requests_total[5m])))"
          }
        ],
        "type": "table"
      }
    ]
  }
}
```

**Fichier:** `monitoring/grafana/dashboards/escrow-overview.json`

```json
{
  "dashboard": {
    "title": "Escrow Overview",
    "panels": [
      {
        "title": "Escrows by State",
        "targets": [
          {
            "expr": "escrow_total"
          }
        ],
        "type": "piechart"
      },
      {
        "title": "State Transitions (last 24h)",
        "targets": [
          {
            "expr": "increase(escrow_state_transitions_total[24h])"
          }
        ],
        "type": "heatmap"
      },
      {
        "title": "Stuck Escrows (pending > 1h)",
        "targets": [
          {
            "expr": "sum(escrow_total{state=\"pending\"} AND (time() - escrow_last_update_timestamp > 3600))"
          }
        ],
        "type": "singlestat",
        "thresholds": "0,1,5"
      }
    ]
  }
}
```

**Fichier:** `monitoring/grafana/dashboards/system-overview.json`

```json
{
  "dashboard": {
    "title": "System Overview",
    "panels": [
      {
        "title": "CPU Usage (%)",
        "targets": [
          {
            "expr": "100 - (avg by(instance) (irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Memory Usage (%)",
        "targets": [
          {
            "expr": "(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Disk Usage",
        "targets": [
          {
            "expr": "(node_filesystem_size_bytes - node_filesystem_avail_bytes) / node_filesystem_size_bytes * 100"
          }
        ],
        "type": "gauge"
      },
      {
        "title": "Network I/O (MB/s)",
        "targets": [
          {
            "expr": "rate(node_network_receive_bytes_total[5m]) / 1024 / 1024",
            "legendFormat": "RX"
          },
          {
            "expr": "rate(node_network_transmit_bytes_total[5m]) / 1024 / 1024",
            "legendFormat": "TX"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

---

## Task 4.5.2.4: Alertmanager Configuration (1 jour)

**Fichier:** `monitoring/alertmanager.yml`

```yaml
global:
  resolve_timeout: 5m
  smtp_from: 'alerts@monero-marketplace.local'
  smtp_smarthost: 'smtp.gmail.com:587'
  smtp_auth_username: 'your-email@gmail.com'
  smtp_auth_password: 'your-app-password'

# Routing tree
route:
  receiver: 'default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h

  routes:
    # Critical alerts → PagerDuty + Email
    - match:
        severity: critical
      receiver: 'pagerduty-critical'
      continue: true

    - match:
        severity: critical
      receiver: 'email-critical'

    # Warnings → Email only
    - match:
        severity: warning
      receiver: 'email-warning'

    # Info → Slack
    - match:
        severity: info
      receiver: 'slack-info'

# Receivers
receivers:
  - name: 'default'
    email_configs:
      - to: 'devops@monero-marketplace.local'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
        description: '{{ .GroupLabels.alertname }} - {{ .Annotations.summary }}'

  - name: 'email-critical'
    email_configs:
      - to: 'oncall@monero-marketplace.local'
        headers:
          Subject: '🚨 CRITICAL: {{ .GroupLabels.alertname }}'
        html: |
          <h2>{{ .GroupLabels.alertname }}</h2>
          <p><strong>Summary:</strong> {{ .Annotations.summary }}</p>
          <p><strong>Description:</strong> {{ .Annotations.description }}</p>
          <p><strong>Instance:</strong> {{ .Labels.instance }}</p>
          <p><strong>Fired At:</strong> {{ .StartsAt }}</p>

  - name: 'email-warning'
    email_configs:
      - to: 'devops@monero-marketplace.local'
        headers:
          Subject: '⚠️ WARNING: {{ .GroupLabels.alertname }}'

  - name: 'slack-info'
    slack_configs:
      - api_url: 'YOUR_SLACK_WEBHOOK_URL'
        channel: '#marketplace-alerts'
        title: 'ℹ️ {{ .GroupLabels.alertname }}'
        text: '{{ .Annotations.summary }}'

# Inhibition rules (suppress lower-priority alerts)
inhibit_rules:
  # If ServiceDown fires, suppress HighErrorRate
  - source_match:
      alertname: 'ServiceDown'
    target_match:
      alertname: 'HighErrorRate'
    equal: ['instance']

  # If WalletRPCUnreachable fires, suppress EscrowStuckInPending
  - source_match:
      alertname: 'WalletRPCUnreachable'
    target_match:
      alertname: 'EscrowStuckInPending'
    equal: ['instance']
```

**Validation:**
```bash
# Test alert routing
curl -XPOST http://localhost:9093/api/v1/alerts \
  -H 'Content-Type: application/json' \
  -d '[{
    "labels": {
      "alertname": "TestAlert",
      "severity": "critical",
      "instance": "server:8080"
    },
    "annotations": {
      "summary": "Test alert",
      "description": "This is a test"
    }
  }]'

# Check alert status
curl http://localhost:9093/api/v2/alerts | jq .
```

---

# Milestone 4.5.3: Backup & Disaster Recovery

**Durée:** 5 jours
**Priorité:** CRITIQUE
**Objectif:** Backups automatisés + recovery procedures testés

## Task 4.5.3.1: Automated Backup Scripts (2 jours)

**Fichier:** `scripts/backup-database.sh`

```bash
#!/bin/bash
set -euo pipefail

# Configuration
BACKUP_DIR="/var/backups/marketplace"
DATABASE_PATH="/app/data/marketplace.db"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/marketplace_${TIMESTAMP}.db"

echo "🗄️  Starting database backup..."

# Create backup directory
mkdir -p "${BACKUP_DIR}"

# SQLite backup (hot backup, safe while database is open)
sqlite3 "${DATABASE_PATH}" ".backup ${BACKUP_FILE}"

# Compress backup
gzip "${BACKUP_FILE}"

# Calculate checksum
sha256sum "${BACKUP_FILE}.gz" > "${BACKUP_FILE}.gz.sha256"

echo "✅ Backup created: ${BACKUP_FILE}.gz"

# Encrypt backup (GPG)
gpg --symmetric --cipher-algo AES256 \
    --passphrase-file /etc/marketplace/backup.key \
    --output "${BACKUP_FILE}.gz.gpg" \
    "${BACKUP_FILE}.gz"

rm "${BACKUP_FILE}.gz"  # Remove unencrypted backup

# Cleanup old backups
find "${BACKUP_DIR}" -name "marketplace_*.gz.gpg" -mtime +${RETENTION_DAYS} -delete
find "${BACKUP_DIR}" -name "marketplace_*.sha256" -mtime +${RETENTION_DAYS} -delete

# Upload to S3 (optional)
if command -v aws &> /dev/null; then
    aws s3 cp "${BACKUP_FILE}.gz.gpg" \
        s3://marketplace-backups/database/ \
        --storage-class GLACIER
    echo "✅ Uploaded to S3"
fi

# Verify backup integrity
if gpg --decrypt --passphrase-file /etc/marketplace/backup.key \
    "${BACKUP_FILE}.gz.gpg" 2>/dev/null | gunzip -t; then
    echo "✅ Backup integrity verified"
else
    echo "❌ Backup corruption detected!"
    exit 1
fi

echo "✅ Backup complete"
```

**Fichier:** `scripts/backup-wallets.sh`

```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/var/backups/marketplace/wallets"
WALLETS_DIR="/app/wallets"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "💼 Starting wallet backup..."

mkdir -p "${BACKUP_DIR}"

# Backup each wallet directory
for wallet in buyer vendor arbiter; do
    tar czf "${BACKUP_DIR}/${wallet}_${TIMESTAMP}.tar.gz" \
        -C "${WALLETS_DIR}" "${wallet}"

    # Encrypt
    gpg --symmetric --cipher-algo AES256 \
        --passphrase-file /etc/marketplace/backup.key \
        --output "${BACKUP_DIR}/${wallet}_${TIMESTAMP}.tar.gz.gpg" \
        "${BACKUP_DIR}/${wallet}_${TIMESTAMP}.tar.gz"

    rm "${BACKUP_DIR}/${wallet}_${TIMESTAMP}.tar.gz"

    echo "✅ Backed up ${wallet} wallet"
done

# Upload to S3
if command -v aws &> /dev/null; then
    aws s3 sync "${BACKUP_DIR}" \
        s3://marketplace-backups/wallets/ \
        --storage-class GLACIER \
        --exclude "*" \
        --include "*.gpg"
fi

echo "✅ Wallet backup complete"
```

**Cron job (root crontab):**
```bash
# Database backup every 6 hours
0 */6 * * * /app/scripts/backup-database.sh >> /var/log/marketplace/backup.log 2>&1

# Wallet backup daily at 2 AM
0 2 * * * /app/scripts/backup-wallets.sh >> /var/log/marketplace/backup.log 2>&1
```

---

## Task 4.5.3.2: Disaster Recovery Procedures (2 jours)

**Fichier:** `scripts/restore-database.sh`

```bash
#!/bin/bash
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <backup_file.gz.gpg>"
    exit 1
fi

BACKUP_FILE="$1"
DATABASE_PATH="/app/data/marketplace.db"

echo "⚠️  WARNING: This will REPLACE the current database!"
read -p "Are you sure? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted"
    exit 0
fi

# Stop application
echo "🛑 Stopping application..."
docker-compose stop server

# Backup current database
if [ -f "${DATABASE_PATH}" ]; then
    cp "${DATABASE_PATH}" "${DATABASE_PATH}.pre-restore.$(date +%s)"
fi

# Decrypt and restore
gpg --decrypt --passphrase-file /etc/marketplace/backup.key "${BACKUP_FILE}" \
    | gunzip > "${DATABASE_PATH}"

# Verify integrity
sqlite3 "${DATABASE_PATH}" "PRAGMA integrity_check;"

# Restart application
echo "🚀 Starting application..."
docker-compose start server

echo "✅ Database restored successfully"
```

**Fichier:** `docs/DISASTER-RECOVERY.md`

```markdown
# Disaster Recovery Runbook

## Scenarios

### Scenario 1: Database Corruption

**Detection:**
- SQLite error: "database disk image is malformed"
- Application fails to start
- Integrity check fails

**Recovery Procedure:**
\`\`\`bash
# 1. Stop application
docker-compose stop server

# 2. Verify corruption
sqlite3 /app/data/marketplace.db "PRAGMA integrity_check;"

# 3. Find latest backup
ls -lh /var/backups/marketplace/marketplace_*.gz.gpg | tail -5

# 4. Restore (replace YYYYMMDD_HHMMSS with actual timestamp)
./scripts/restore-database.sh /var/backups/marketplace/marketplace_YYYYMMDD_HHMMSS.db.gz.gpg

# 5. Verify application
docker-compose start server
curl http://localhost:8080/health
\`\`\`

**RTO:** 15 minutes
**RPO:** Up to 6 hours (depends on last backup)

---

### Scenario 2: Wallet File Loss

**Detection:**
- Monero RPC fails to open wallet
- Error: "Failed to open wallet"

**Recovery Procedure:**
\`\`\`bash
# 1. Stop wallet RPC
docker-compose stop monero-wallet-rpc-buyer

# 2. Restore from backup
LATEST_BACKUP=$(ls -t /var/backups/marketplace/wallets/buyer_*.tar.gz.gpg | head -1)
gpg --decrypt --passphrase-file /etc/marketplace/backup.key "$LATEST_BACKUP" \
    | tar xzf - -C /app/wallets

# 3. Restart wallet
docker-compose start monero-wallet-rpc-buyer

# 4. Verify wallet
curl -X POST http://localhost:18082/json_rpc \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_balance"}' \
    -H 'Content-Type: application/json'
\`\`\`

**RTO:** 10 minutes
**RPO:** Up to 24 hours

---

### Scenario 3: Complete Server Loss

**Recovery Procedure:**
\`\`\`bash
# 1. Provision new server
# 2. Install Docker + Docker Compose
# 3. Clone repository
git clone <repo-url>
cd monero-marketplace

# 4. Restore backups from S3
aws s3 sync s3://marketplace-backups/database/ /var/backups/marketplace/
aws s3 sync s3://marketplace-backups/wallets/ /var/backups/marketplace/wallets/

# 5. Restore database
./scripts/restore-database.sh /var/backups/marketplace/marketplace_LATEST.db.gz.gpg

# 6. Restore wallets
for wallet in buyer vendor arbiter; do
    LATEST=$(ls -t /var/backups/marketplace/wallets/${wallet}_*.tar.gz.gpg | head -1)
    gpg --decrypt --passphrase-file /etc/marketplace/backup.key "$LATEST" \
        | tar xzf - -C ./wallets
done

# 7. Start stack
./scripts/docker-start.sh

# 8. Verify health
./scripts/docker-health-check.sh
\`\`\`

**RTO:** 2 hours
**RPO:** Up to 24 hours
```

---

## Task 4.5.3.3: Backup Testing (1 jour)

**Fichier:** `scripts/test-backup-restore.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "🧪 Testing Backup/Restore Procedures..."

# 1. Create test database
TEST_DIR="/tmp/marketplace-backup-test"
rm -rf "${TEST_DIR}"
mkdir -p "${TEST_DIR}/data" "${TEST_DIR}/backups"

cp /app/data/marketplace.db "${TEST_DIR}/data/"

# 2. Run backup
BACKUP_DIR="${TEST_DIR}/backups" \
DATABASE_PATH="${TEST_DIR}/data/marketplace.db" \
    ./scripts/backup-database.sh

# 3. Corrupt database
dd if=/dev/urandom of="${TEST_DIR}/data/marketplace.db" bs=1024 count=10 conv=notrunc

# 4. Restore from backup
LATEST_BACKUP=$(ls -t "${TEST_DIR}/backups"/marketplace_*.gz.gpg | head -1)
gpg --decrypt --passphrase-file /etc/marketplace/backup.key "${LATEST_BACKUP}" \
    | gunzip > "${TEST_DIR}/data/marketplace.db"

# 5. Verify integrity
if sqlite3 "${TEST_DIR}/data/marketplace.db" "PRAGMA integrity_check;" | grep -q "ok"; then
    echo "✅ Backup/Restore test PASSED"
else
    echo "❌ Backup/Restore test FAILED"
    exit 1
fi

# Cleanup
rm -rf "${TEST_DIR}"
```

**Automated monthly test (cron):**
```bash
# Test backup/restore on 1st of each month at 3 AM
0 3 1 * * /app/scripts/test-backup-restore.sh >> /var/log/marketplace/backup-test.log 2>&1
```

---

# Milestone 4.5.4: CI/CD Pipeline

**Durée:** 5 jours
**Priorité:** Haute
**Objectif:** GitHub Actions pipeline avec tests + security scan + auto-deploy

## Task 4.5.4.1: GitHub Actions Workflow (2 jours)

**Fichier:** `.github/workflows/ci.yml`

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  # ==========================================================================
  # Job 1: Code Quality Checks
  # ==========================================================================
  quality:
    name: Code Quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Security Theatre Check
        run: ./scripts/check-security-theatre.sh

  # ==========================================================================
  # Job 2: Build & Test
  # ==========================================================================
  test:
    name: Build & Test
    runs-on: ubuntu-latest
    needs: quality
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libsqlite3-dev libssl-dev pkg-config

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Build workspace
        run: cargo build --workspace --verbose

      - name: Run tests
        run: cargo test --workspace --verbose

      - name: Upload test results
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: target/debug/test-*.xml

  # ==========================================================================
  # Job 3: Security Audit
  # ==========================================================================
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    needs: quality
    steps:
      - uses: actions/checkout@v4

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit

      - name: Check dependencies for vulnerabilities
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  # ==========================================================================
  # Job 4: Docker Build
  # ==========================================================================
  docker:
    name: Docker Build & Push
    runs-on: ubuntu-latest
    needs: [test, security]
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: |
            ghcr.io/${{ github.repository }}:latest
            ghcr.io/${{ github.repository }}:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  # ==========================================================================
  # Job 5: Deploy to Staging
  # ==========================================================================
  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    needs: docker
    if: github.ref == 'refs/heads/develop'
    environment:
      name: staging
      url: https://staging.marketplace.local
    steps:
      - uses: actions/checkout@v4

      - name: Deploy via SSH
        uses: appleboy/ssh-action@v1.0.0
        with:
          host: ${{ secrets.STAGING_HOST }}
          username: ${{ secrets.STAGING_USER }}
          key: ${{ secrets.STAGING_SSH_KEY }}
          script: |
            cd /opt/marketplace
            docker-compose pull
            docker-compose up -d
            docker-compose exec -T server curl -f http://localhost:8080/health

      - name: Smoke test
        run: |
          sleep 10
          curl -f https://staging.marketplace.local/health

  # ==========================================================================
  # Job 6: Deploy to Production
  # ==========================================================================
  deploy-production:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: docker
    if: github.ref == 'refs/heads/main'
    environment:
      name: production
      url: https://marketplace.onion
    steps:
      - uses: actions/checkout@v4

      - name: Deploy via SSH
        uses: appleboy/ssh-action@v1.0.0
        with:
          host: ${{ secrets.PROD_HOST }}
          username: ${{ secrets.PROD_USER }}
          key: ${{ secrets.PROD_SSH_KEY }}
          script: |
            cd /opt/marketplace
            docker-compose pull
            docker-compose up -d --no-deps server

            # Wait for health check
            for i in {1..30}; do
              if docker-compose exec -T server curl -f http://localhost:8080/health; then
                echo "Deployment successful"
                exit 0
              fi
              sleep 2
            done

            echo "Deployment failed - rolling back"
            docker-compose rollback server
            exit 1

      - name: Notify on failure
        if: failure()
        uses: 8398a7/action-slack@v3
        with:
          status: ${{ job.status }}
          text: 'Production deployment failed!'
          webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

---

## Task 4.5.4.2: Deployment Scripts (1 jour)

**Fichier:** `scripts/deploy.sh`

```bash
#!/bin/bash
set -euo pipefail

ENVIRONMENT=${1:-staging}
VERSION=${2:-latest}

echo "🚀 Deploying to ${ENVIRONMENT} (version: ${VERSION})"

case $ENVIRONMENT in
    staging)
        SSH_HOST="${STAGING_HOST}"
        SSH_USER="${STAGING_USER}"
        ;;
    production)
        SSH_HOST="${PROD_HOST}"
        SSH_USER="${PROD_USER}"
        ;;
    *)
        echo "Invalid environment: ${ENVIRONMENT}"
        exit 1
        ;;
esac

# Deploy
ssh "${SSH_USER}@${SSH_HOST}" << EOF
    set -e
    cd /opt/marketplace

    # Backup current state
    ./scripts/backup-database.sh

    # Pull new version
    docker-compose pull

    # Rolling update
    docker-compose up -d --no-deps server

    # Health check
    sleep 5
    if ! curl -f http://localhost:8080/health; then
        echo "Health check failed - rolling back"
        docker-compose rollback server
        exit 1
    fi

    echo "Deployment successful"
EOF

echo "✅ Deployed successfully"
```

---

## Task 4.5.4.3: Rollback Procedures (1 jour)

**Fichier:** `scripts/rollback.sh`

```bash
#!/bin/bash
set -euo pipefail

ENVIRONMENT=${1:-staging}

echo "⏪ Rolling back ${ENVIRONMENT}"

ssh "${DEPLOY_USER}@${DEPLOY_HOST}" << EOF
    cd /opt/marketplace

    # Get previous image
    PREVIOUS_IMAGE=\$(docker images ghcr.io/monero-marketplace --format "{{.Tag}}" | sed -n '2p')

    # Stop current
    docker-compose stop server

    # Restore from backup
    LATEST_BACKUP=\$(ls -t /var/backups/marketplace/marketplace_*.db.gz.gpg | head -1)
    ./scripts/restore-database.sh "\$LATEST_BACKUP"

    # Start with previous image
    docker-compose up -d server

    echo "Rollback complete"
EOF
```

---

## Task 4.5.4.4: Environment Configuration (1 jour)

**Fichier:** `.github/workflows/environments.yml` (GitHub Secrets required)

Required secrets:
- `STAGING_HOST`
- `STAGING_USER`
- `STAGING_SSH_KEY`
- `PROD_HOST`
- `PROD_USER`
- `PROD_SSH_KEY`
- `SLACK_WEBHOOK`

**Fichier:** `config/staging.env`

```bash
DATABASE_URL=sqlite:///app/data/marketplace-staging.db?mode=rwc
RUST_LOG=debug
MONERO_NETWORK=testnet
MONERO_BUYER_RPC_URL=http://monero-wallet-rpc-buyer:18082/json_rpc
```

**Fichier:** `config/production.env`

```bash
DATABASE_URL=sqlite:///app/data/marketplace.db?mode=rwc
RUST_LOG=info
MONERO_NETWORK=mainnet
MONERO_BUYER_RPC_URL=http://monero-wallet-rpc-buyer:18081/json_rpc
```

---

# Milestone 4.5.5: Load Testing & Performance

**Durée:** 3 jours
**Priorité:** Haute
**Objectif:** Valider capacité 100 req/s avec p95 < 200ms

## Task 4.5.5.1: Load Testing Scripts (2 jours)

**Fichier:** `load-tests/scenarios/http-endpoints.js` (k6)

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export let options = {
  stages: [
    { duration: '2m', target: 10 },   // Ramp-up to 10 users
    { duration: '5m', target: 50 },   // Ramp-up to 50 users
    { duration: '10m', target: 100 }, // Stay at 100 users
    { duration: '3m', target: 0 },    // Ramp-down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<200'], // 95% of requests < 200ms
    'errors': ['rate<0.05'],            // Error rate < 5%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // Test 1: GET /api/listings (public)
  let res = http.get(`${BASE_URL}/api/listings`);
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 200ms': (r) => r.timings.duration < 200,
  }) || errorRate.add(1);

  sleep(1);

  // Test 2: Search listings
  res = http.get(`${BASE_URL}/api/listings/search?q=test`);
  check(res, {
    'search status is 200': (r) => r.status === 200,
  }) || errorRate.add(1);

  sleep(1);

  // Test 3: Register user (write operation)
  const username = `user_${Date.now()}_${__VU}`;
  res = http.post(`${BASE_URL}/api/auth/register`, JSON.stringify({
    username: username,
    password: 'testpass123',
    role: 'buyer'
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(res, {
    'register status is 200': (r) => r.status === 200,
  }) || errorRate.add(1);

  sleep(2);
}
```

**Run load test:**
```bash
# Install k6
curl https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz -L | tar xvz
sudo mv k6-v0.47.0-linux-amd64/k6 /usr/local/bin/

# Run test
k6 run --out influxdb=http://localhost:8086/k6 load-tests/scenarios/http-endpoints.js

# Generate HTML report
k6 run --out json=results.json load-tests/scenarios/http-endpoints.js
k6-html-report results.json
```

**Fichier:** `load-tests/scenarios/escrow-flow.js`

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  scenarios: {
    escrow_creation: {
      executor: 'constant-vus',
      vus: 10,
      duration: '5m',
    },
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // 1. Register buyer
  const buyer_username = `buyer_${Date.now()}_${__VU}`;
  let res = http.post(`${BASE_URL}/api/auth/register`, JSON.stringify({
    username: buyer_username,
    password: 'password123',
    role: 'buyer'
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  const buyer_cookie = res.cookies.session_token[0].value;

  // 2. Create listing
  res = http.post(`${BASE_URL}/api/listings`, JSON.stringify({
    title: 'Load Test Product',
    description: 'Performance test',
    price_xmr: 1000000000000,
    stock: 100
  }), {
    headers: {
      'Content-Type': 'application/json',
      'Cookie': `session_token=${buyer_cookie}`
    },
  });

  check(res, {
    'listing created': (r) => r.status === 200,
  });

  const listing_id = res.json('listing_id');

  // 3. Create order
  res = http.post(`${BASE_URL}/api/orders`, JSON.stringify({
    listing_id: listing_id,
    quantity: 1
  }), {
    headers: {
      'Content-Type': 'application/json',
      'Cookie': `session_token=${buyer_cookie}`
    },
  });

  check(res, {
    'order created': (r) => r.status === 200,
  });

  sleep(3);
}
```

---

## Task 4.5.5.2: Performance Optimization (1 jour)

**Database Optimizations:**

```sql
-- Add indexes for common queries
CREATE INDEX IF NOT EXISTS idx_listings_vendor_id ON listings(vendor_id);
CREATE INDEX IF NOT EXISTS idx_listings_created_at ON listings(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_buyer_id ON orders(buyer_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_escrows_state ON escrows(state);
CREATE INDEX IF NOT EXISTS idx_transactions_order_id ON transactions(order_id);

-- Analyze query plans
EXPLAIN QUERY PLAN SELECT * FROM listings WHERE vendor_id = ? ORDER BY created_at DESC LIMIT 20;
```

**Connection Pooling (Diesel):**

```rust
// server/src/db/mod.rs
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn create_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    r2d2::Pool::builder()
        .max_size(20)              // Max connections
        .min_idle(Some(5))         // Keep 5 idle connections
        .connection_timeout(Duration::from_secs(10))
        .build(manager)
        .context("Failed to create database pool")
}
```

**Caching (Redis):**

```rust
// server/src/cache/mod.rs
use redis::Client;

pub async fn cache_listing(listing_id: i32, data: &Listing) -> Result<()> {
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_async_connection().await?;

    let json = serde_json::to_string(data)?;
    redis::cmd("SETEX")
        .arg(format!("listing:{}", listing_id))
        .arg(300) // 5 minutes TTL
        .arg(json)
        .query_async(&mut con)
        .await?;

    Ok(())
}
```

---

# Milestone 4.5.6: Security Hardening

**Durée:** 4 jours
**Priorité:** CRITIQUE
**Objectif:** Production-grade security (TLS, firewall, secrets management)

## Task 4.5.6.1: TLS/SSL Configuration (1 jour)

**Fichier:** `nginx/nginx.conf`

```nginx
# SSL/TLS Configuration
upstream marketplace_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name marketplace.onion;

    # SSL Certificates (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/marketplace.onion/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/marketplace.onion/privkey.pem;

    # TLS 1.3 only
    ssl_protocols TLSv1.3;
    ssl_prefer_server_ciphers off;
    ssl_ciphers 'TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_128_GCM_SHA256';

    # HSTS (force HTTPS)
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;

    # Security headers
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';" always;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/marketplace.onion/chain.pem;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=general:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/m;

    location / {
        limit_req zone=general burst=20 nodelay;
        proxy_pass http://marketplace_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /api/auth/ {
        limit_req zone=auth burst=5 nodelay;
        proxy_pass http://marketplace_backend;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name marketplace.onion;
    return 301 https://$host$request_uri;
}
```

**Auto-renew certificates:**
```bash
# Cron job (runs weekly)
0 0 * * 0 certbot renew --quiet && systemctl reload nginx
```

---

## Task 4.5.6.2: Firewall Configuration (1 jour)

**Fichier:** `scripts/setup-firewall.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "🔥 Configuring UFW firewall..."

# Reset to defaults
ufw --force reset

# Default policies
ufw default deny incoming
ufw default allow outgoing

# Allow SSH (change port if needed)
ufw allow 22/tcp comment 'SSH'

# Allow HTTPS
ufw allow 443/tcp comment 'HTTPS'

# Allow Prometheus (internal only)
ufw allow from 10.0.0.0/8 to any port 9090 proto tcp comment 'Prometheus'
ufw allow from 172.16.0.0/12 to any port 9090 proto tcp

# Allow Grafana (internal only)
ufw allow from 10.0.0.0/8 to any port 3000 proto tcp comment 'Grafana'

# Deny all other ports
ufw deny 8080/tcp comment 'Block direct backend access'
ufw deny 18082:18084/tcp comment 'Block Monero RPC'

# Rate limiting on SSH
ufw limit 22/tcp comment 'Rate limit SSH'

# Enable firewall
ufw --force enable

echo "✅ Firewall configured"
ufw status verbose
```

---

## Task 4.5.6.3: Secrets Management (1 jour)

**Fichier:** `scripts/setup-secrets.sh` (SOPS + Age)

```bash
#!/bin/bash
set -euo pipefail

# Install SOPS
curl -LO https://github.com/getsops/sops/releases/download/v3.8.1/sops-v3.8.1.linux.amd64
chmod +x sops-v3.8.1.linux.amd64
sudo mv sops-v3.8.1.linux.amd64 /usr/local/bin/sops

# Install Age
curl -LO https://github.com/FiloSottile/age/releases/download/v1.1.1/age-v1.1.1-linux-amd64.tar.gz
tar xzf age-v1.1.1-linux-amd64.tar.gz
sudo mv age/age /usr/local/bin/
sudo mv age/age-keygen /usr/local/bin/

# Generate encryption key
age-keygen -o /etc/marketplace/age-key.txt

# Create encrypted secrets file
cat > secrets.yaml <<EOF
database_password: CHANGEME
grafana_admin_password: CHANGEME
backup_gpg_passphrase: CHANGEME
EOF

# Encrypt with SOPS
AGE_PUBLIC_KEY=$(age-keygen -y /etc/marketplace/age-key.txt)
sops --encrypt --age "${AGE_PUBLIC_KEY}" secrets.yaml > secrets.enc.yaml

# Securely delete plaintext
shred -u secrets.yaml

echo "✅ Secrets encrypted"
echo "Public key: ${AGE_PUBLIC_KEY}"
```

**Usage:**
```bash
# Decrypt secrets (in application startup)
sops --decrypt --age $(cat /etc/marketplace/age-key.txt) secrets.enc.yaml > /tmp/secrets.yaml
source /tmp/secrets.yaml
rm /tmp/secrets.yaml
```

---

## Task 4.5.6.4: Security Audit (1 jour)

**Run automated security scanners:**

```bash
# 1. Cargo audit (dependency vulnerabilities)
cargo audit

# 2. Trivy (container vulnerabilities)
trivy image ghcr.io/monero-marketplace:latest

# 3. SQLMap (SQL injection testing)
sqlmap -u "http://localhost:8080/api/listings?page=1" --batch --random-agent

# 4. OWASP ZAP (web app scanning)
docker run -t owasp/zap2docker-stable zap-baseline.py -t http://localhost:8080

# 5. Lynis (system hardening)
sudo lynis audit system
```

**Security checklist:**
- [ ] No hardcoded credentials
- [ ] All secrets encrypted at rest
- [ ] TLS 1.3 enforced
- [ ] HSTS enabled
- [ ] CSP headers configured
- [ ] Rate limiting active
- [ ] Firewall restricts Monero RPC to localhost
- [ ] Database encrypted (SQLCipher)
- [ ] Backups encrypted (GPG)
- [ ] No .onion addresses in logs
- [ ] Tor isolation verified

---

# Milestone 4.5.7: Documentation Opérationnelle

**Durée:** 3 jours
**Priorité:** Moyenne
**Objectif:** Runbooks complets pour opérations quotidiennes

## Task 4.5.7.1: Operations Runbook (2 jours)

**Fichier:** `docs/OPERATIONS-RUNBOOK.md`

```markdown
# Operations Runbook

## Daily Operations

### Morning Checks (10 min)
\`\`\`bash
# 1. Check service health
./scripts/docker-health-check.sh

# 2. Review Grafana dashboards
# - HTTP Overview: Error rate < 1%
# - Escrow Overview: No stuck escrows
# - System Overview: CPU < 70%, Memory < 80%

# 3. Check alerts
curl http://localhost:9093/api/v2/alerts | jq '.[] | select(.status.state=="active")'

# 4. Review logs for errors
docker-compose logs --since 24h server | grep -i error
\`\`\`

### Weekly Tasks
- [ ] Review security audit logs
- [ ] Check disk space (should be > 20% free)
- [ ] Test backup restoration
- [ ] Review Prometheus alerts
- [ ] Update dependencies (cargo update)

### Monthly Tasks
- [ ] Full DR test (restore from backup)
- [ ] Security scan (cargo audit + Trivy)
- [ ] Certificate renewal verification
- [ ] Performance review (p95 response times)

---

## Incident Response

### High CPU Usage (> 90%)

**Symptoms:**
- Grafana dashboard shows CPU > 90%
- Slow response times
- Alert: "HighCPUUsage"

**Diagnosis:**
\`\`\`bash
# Check process CPU usage
docker stats

# Identify slow queries
docker-compose exec server journalctl -u marketplace | grep "slow query"
\`\`\`

**Remediation:**
\`\`\`bash
# Restart application (clears caches)
docker-compose restart server

# If persists, scale horizontally
docker-compose scale server=2
\`\`\`

---

### Database Locked Error

**Symptoms:**
- Error: "database is locked"
- Requests timing out

**Diagnosis:**
\`\`\`bash
# Check for lock file
ls -lh /app/data/marketplace.db-wal

# Check for long-running transactions
sqlite3 /app/data/marketplace.db "PRAGMA lock_status;"
\`\`\`

**Remediation:**
\`\`\`bash
# Stop application
docker-compose stop server

# Remove WAL/SHM files
rm /app/data/marketplace.db-wal /app/data/marketplace.db-shm

# Vacuum database
sqlite3 /app/data/marketplace.db "VACUUM;"

# Restart
docker-compose start server
\`\`\`

---

### Wallet RPC Unreachable

**Symptoms:**
- Alert: "WalletRPCUnreachable"
- Escrows stuck in "pending"

**Diagnosis:**
\`\`\`bash
# Check wallet container
docker-compose logs monero-wallet-rpc-buyer

# Test RPC manually
curl -X POST http://localhost:18082/json_rpc \
  -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}' \
  -H 'Content-Type: application/json'
\`\`\`

**Remediation:**
\`\`\`bash
# Restart wallet RPC
docker-compose restart monero-wallet-rpc-buyer

# If corrupted, restore from backup
./scripts/restore-wallet.sh buyer
\`\`\`
```

---

## Task 4.5.7.2: Troubleshooting Guide (1 jour)

**Fichier:** `docs/TROUBLESHOOTING.md`

```markdown
# Troubleshooting Guide

## Common Issues

### Issue: "Connection refused" to Monero RPC

**Cause:** Wallet RPC not started or wrong port

**Fix:**
\`\`\`bash
# Check if container running
docker-compose ps | grep monero-wallet

# Check correct port mapping
docker-compose port monero-wallet-rpc-buyer 18082

# Restart wallet
docker-compose restart monero-wallet-rpc-buyer
\`\`\`

---

### Issue: Slow database queries

**Cause:** Missing indexes or large dataset

**Fix:**
\`\`\`bash
# Analyze query plan
sqlite3 /app/data/marketplace.db <<EOF
EXPLAIN QUERY PLAN SELECT * FROM listings WHERE vendor_id = 123;
EOF

# Add missing indexes (see Task 4.5.5.2)

# Vacuum database
sqlite3 /app/data/marketplace.db "VACUUM;"
\`\`\`

---

### Issue: TLS certificate expired

**Cause:** Certbot auto-renewal failed

**Fix:**
\`\`\`bash
# Manual renewal
sudo certbot renew --force-renewal

# Reload nginx
sudo systemctl reload nginx

# Verify
curl -vI https://marketplace.onion 2>&1 | grep "expire date"
\`\`\`
```

---

# Milestone 4.5.8: Deployment Automation

**Durée:** 3 jours
**Priorité:** Haute
**Objectif:** Zero-downtime deployments avec blue-green strategy

## Task 4.5.8.1: Blue-Green Deployment (2 jours)

**Fichier:** `docker-compose.blue-green.yml`

```yaml
version: '3.8'

services:
  # Blue environment (current production)
  server-blue:
    build: .
    container_name: marketplace-server-blue
    environment:
      - DEPLOYMENT_COLOR=blue
    ports:
      - "8080:8080"
    networks:
      - marketplace-network

  # Green environment (new version)
  server-green:
    build: .
    container_name: marketplace-server-green
    environment:
      - DEPLOYMENT_COLOR=green
    ports:
      - "8081:8080"
    networks:
      - marketplace-network

  # Nginx load balancer
  nginx:
    image: nginx:1.25-alpine
    container_name: marketplace-nginx
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/upstream.conf:/etc/nginx/conf.d/upstream.conf
    networks:
      - marketplace-network
```

**Fichier:** `nginx/upstream.conf`

```nginx
upstream marketplace_backend {
    # Initially only blue is active
    server server-blue:8080 weight=100;
    # server server-green:8080 weight=0;
}
```

**Fichier:** `scripts/deploy-blue-green.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "🔵🟢 Starting Blue-Green Deployment..."

# 1. Deploy to green (inactive)
docker-compose -f docker-compose.blue-green.yml up -d server-green

# 2. Wait for health check
echo "⏳ Waiting for green environment to be healthy..."
for i in {1..30}; do
    if curl -f http://localhost:8081/health > /dev/null 2>&1; then
        echo "✅ Green environment healthy"
        break
    fi
    sleep 2
done

# 3. Run smoke tests on green
echo "🧪 Running smoke tests..."
./tests/smoke-test.sh http://localhost:8081

if [ $? -ne 0 ]; then
    echo "❌ Smoke tests failed - aborting deployment"
    docker-compose -f docker-compose.blue-green.yml stop server-green
    exit 1
fi

# 4. Switch traffic to green (gradual)
echo "🔀 Switching traffic to green..."
cat > nginx/upstream.conf <<EOF
upstream marketplace_backend {
    server server-blue:8080 weight=50;
    server server-green:8080 weight=50;
}
EOF
docker-compose -f docker-compose.blue-green.yml exec nginx nginx -s reload
sleep 30

# 5. Fully switch to green
cat > nginx/upstream.conf <<EOF
upstream marketplace_backend {
    server server-blue:8080 weight=0;
    server server-green:8080 weight=100;
}
EOF
docker-compose -f docker-compose.blue-green.yml exec nginx nginx -s reload

# 6. Monitor for 5 minutes
echo "📊 Monitoring green environment..."
sleep 300

# 7. Check error rate
ERROR_RATE=$(curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total{status=~"5.."}[5m])' | jq -r '.data.result[0].value[1]')

if (( $(echo "$ERROR_RATE > 0.05" | bc -l) )); then
    echo "❌ High error rate detected - rolling back"
    ./scripts/rollback-blue-green.sh
    exit 1
fi

# 8. Stop blue (old version)
echo "🛑 Stopping blue environment..."
docker-compose -f docker-compose.blue-green.yml stop server-blue

echo "✅ Deployment complete - green is now production"
```

---

## Task 4.5.8.2: Automated Rollback (1 jour)

**Fichier:** `scripts/rollback-blue-green.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "⏪ Rolling back to blue environment..."

# 1. Switch all traffic back to blue
cat > nginx/upstream.conf <<EOF
upstream marketplace_backend {
    server server-blue:8080 weight=100;
    server server-green:8080 weight=0;
}
EOF
docker-compose -f docker-compose.blue-green.yml exec nginx nginx -s reload

# 2. Stop green
docker-compose -f docker-compose.blue-green.yml stop server-green

echo "✅ Rollback complete - blue is active"
```

---

## Summary: Infrastructure Roadmap Completion

**Total Duration:** 33 days (~4-5 weeks)

**Critical Path:**
1. Milestone 4.5.1 (Containerization) → 4.5.2 (Monitoring) → 4.5.3 (Backup) → 4.5.6 (Security)

**Parallel Tracks:**
- CI/CD (4.5.4) + Load Testing (4.5.5) can run in parallel
- Documentation (4.5.7) can be written throughout

**Expected Score Improvement:**
- **Before:** 65/100 (Code: 90, Infrastructure: 30)
- **After:** 90/100 (Code: 90, Infrastructure: 90)

**Deployment Timeline:**
- **Week 1:** Docker + Monitoring
- **Week 2:** Backup + CI/CD
- **Week 3:** Load Testing + Security
- **Week 4:** Documentation + Deployment Automation

**Mainnet Ready:** 4-6 weeks after Phase 4.5 completion
