# Phase 4.5 - Production Infrastructure

**Status:** STAGING-READY ✅
**Score:** 87/100 (Post Beta Terminal Audit)
**Last Update:** 2025-10-22

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Directory Structure](#directory-structure)
5. [Services](#services)
6. [Monitoring](#monitoring)
7. [Security](#security)
8. [Backup & Disaster Recovery](#backup--disaster-recovery)
9. [Operations](#operations)
10. [Documentation](#documentation)

---

## 🎯 Overview

Phase 4.5 provides production-grade infrastructure for the Monero Marketplace, including:

- **Docker Compose orchestration** (11 services)
- **Complete monitoring stack** (Prometheus, Grafana, Loki, Alertmanager)
- **Monero RPC exporters** (custom Python exporter for wallet metrics)
- **Encrypted secrets management** (SOPS + AGE encryption)
- **Automated backup/restore** (GPG encrypted backups with 90-day retention)
- **Blue-green deployment** support
- **TLS 1.3 + HSTS** via Nginx reverse proxy

---

## 🏗️ Architecture

### Services (11 total)

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| **server** | Custom (Rust) | 127.0.0.1:8080 | Marketplace application |
| **monero-wallet-rpc-buyer** | xmr.to/monero | 127.0.0.1:18082 | Buyer wallet RPC |
| **monero-wallet-rpc-vendor** | xmr.to/monero | 127.0.0.1:18083 | Vendor wallet RPC |
| **monero-wallet-rpc-arbiter** | xmr.to/monero | 127.0.0.1:18084 | Arbiter wallet RPC |
| **prometheus** | prom/prometheus:v2.48.0 | 127.0.0.1:9090 | Metrics collection |
| **grafana** | grafana/grafana:10.2.2 | 127.0.0.1:3000 | Metrics visualization |
| **loki** | grafana/loki:2.9.3 | 127.0.0.1:3100 | Log aggregation |
| **promtail** | grafana/promtail:2.9.3 | - | Log shipping |
| **node_exporter** | prom/node-exporter:v1.7.0 | 127.0.0.1:9100 | System metrics |
| **monero-exporter** | Custom (Python) | 127.0.0.1:9101 | Monero wallet metrics |
| **alertmanager** | prom/alertmanager:v0.26.0 | 127.0.0.1:9093 | Alert routing |

**Network Isolation:** All RPC services bind to `127.0.0.1` (localhost only)
**Healthchecks:** 11/11 services (100% coverage)

---

## 🚀 Quick Start

### Prerequisites

- Docker 24.0+
- Docker Compose v2.0+
- SOPS 3.7+ (for secrets)
- Age 1.0+ (for encryption)
- GPG 2.2+ (for backups)

### 1. Setup Secrets

```bash
# Generate Age encryption key
./scripts/setup-sops.sh

# Create .env file from template
cp docker/.env.example docker/.env

# Edit .env with secure passwords
nano docker/.env
```

### 2. Deploy Stack

```bash
# Navigate to docker directory
cd 4.5/docker

# Start all services
sudo docker compose up -d

# Check service health
sudo docker compose ps
```

### 3. Access Services

- **Grafana:** http://localhost:3000 (admin / \<GRAFANA_ADMIN_PASSWORD>)
- **Prometheus:** http://localhost:9090
- **Alertmanager:** http://localhost:9093
- **Application:** http://localhost:8080 (via Nginx reverse proxy)

---

## 📁 Directory Structure

```
4.5/
├── docker/
│   ├── docker-compose.yml          # Main orchestration (11 services)
│   ├── docker-compose.blue-green.yml  # Blue-green deployment
│   ├── Dockerfile                  # Application container
│   └── .env.example               # Environment variables template
│
├── monitoring/
│   ├── prometheus.yml             # Prometheus configuration
│   ├── alertmanager.yml           # Alert routing rules
│   ├── loki-config.yaml           # Loki log aggregation
│   ├── promtail-config.yaml       # Promtail log shipping
│   ├── grafana/
│   │   └── dashboards/           # 6 Grafana dashboards (JSON)
│   ├── monero-exporter/          # Custom Monero metrics exporter
│   │   ├── exporter.py           # Python exporter (142 lines)
│   │   ├── Dockerfile
│   │   └── README.md
│   └── alerts/
│       └── marketplace.yml        # Prometheus alerting rules
│
├── nginx/
│   └── nginx.conf                 # Reverse proxy config (TLS 1.3)
│
├── scripts/
│   ├── deploy.sh                  # Standard deployment
│   ├── deploy-blue-green.sh       # Blue-green deployment
│   ├── backup-database.sh         # Automated DB backup (GPG)
│   ├── backup-wallets.sh          # Automated wallet backup (GPG)
│   ├── restore-database.sh        # Database restore
│   ├── restore-wallet.sh          # Wallet restore
│   ├── setup-sops.sh              # SOPS + Age setup
│   ├── setup-firewall.sh          # UFW firewall rules
│   ├── validate-infrastructure.sh # Infrastructure validation
│   └── (12 more scripts...)
│
├── security/
│   ├── secrets.enc.yaml           # Encrypted secrets (SOPS+AGE)
│   ├── age.key                    # Age private key (GITIGNORED)
│   └── backup-gpg-key.asc         # GPG public key for backups
│
├── docs/
│   ├── DISASTER-RECOVERY.md       # DR procedures
│   ├── DOCKER-DEPLOYMENT.md       # Deployment guide
│   ├── OPERATIONS-RUNBOOK.md      # Operations procedures
│   ├── SECURITY-AUDIT.md          # Security checklist
│   ├── TROUBLESHOOTING.md         # Troubleshooting guide
│   └── DATABASE-OPTIMIZATIONS.md  # DB tuning guide
│
├── load-tests/
│   ├── scenarios/
│   │   ├── http-endpoints.js      # k6 HTTP load tests
│   │   └── escrow-flow.js         # k6 escrow flow tests
│   └── README.md
│
└── PHASE-4.5-COMPLETE.md          # Phase completion report
```

---

## 📊 Monitoring

### Prometheus Metrics

**System Metrics (node_exporter):**
- CPU usage, memory, disk, network I/O

**Application Metrics:**
- HTTP request rate, latency, error rate
- Active connections, escrow counts

**Monero Metrics (custom exporter):**
- `monero_wallet_balance_piconero{wallet_name="buyer|vendor|arbiter"}`
- `monero_wallet_unlocked_balance_piconero{...}`
- `monero_wallet_height{...}`
- `monero_wallet_num_unspent_outputs{...}`
- `monero_wallet_rpc_calls_total{method, status}`
- `monero_wallet_rpc_errors_total{method}`

### Grafana Dashboards

1. **System Overview** - CPU, RAM, disk, network
2. **HTTP Overview** - Request rate, latency, status codes
3. **Escrow Overview** - Active escrows, disputes, amounts locked
4. *(+3 additional dashboards)*

Access: http://localhost:3000

### Alerting Rules

- **ServiceDown** - Service unavailable
- **HighErrorRate** - >5% error rate for 5 minutes
- **WalletRPCUnreachable** - Monero RPC unreachable
- **EscrowStuckInPending** - Escrow stuck >24h

Configured in: `monitoring/alerts/marketplace.yml`

---

## 🔒 Security

### Secrets Management

**SOPS + AGE Encryption:**
```bash
# View encrypted secrets
sops 4.5/security/secrets.enc.yaml

# Edit secrets
sops 4.5/security/secrets.enc.yaml

# Encrypt new file
sops -e -i newfile.yaml
```

**Environment Variables (.env):**
- `GRAFANA_ADMIN_PASSWORD` - Grafana admin password
- `DATABASE_PASSWORD` - SQLCipher database password
- `BACKUP_GPG_PASSPHRASE` - GPG key passphrase

**NEVER commit `.env` or `age.key` to Git** (already in .gitignore)

### Network Isolation

All RPC services bind to `127.0.0.1`:
- Monero wallets: 18082-18084
- Prometheus: 9090
- Grafana: 3000
- Metrics exporters: 9100-9101
- Alertmanager: 9093

**Public access via Nginx reverse proxy only** (TLS 1.3 + HSTS)

### Security Hardening Checklist

See [docs/SECURITY-AUDIT.md](docs/SECURITY-AUDIT.md) for complete checklist:
- [x] No hardcoded credentials
- [x] Secrets encrypted at rest (SOPS+AGE)
- [x] TLS 1.3 enforced
- [x] HSTS enabled
- [x] Rate limiting active
- [x] Firewall restricts RPC
- [x] Database encrypted (SQLCipher)
- [x] Backups encrypted (GPG)
- [x] Non-root Docker users
- [x] Comprehensive logging (Loki)

---

## 💾 Backup & Disaster Recovery

### Automated Backups

**Database:**
```bash
# Manual backup
./scripts/backup-database.sh

# Automated: Run daily via cron
0 2 * * * /path/to/4.5/scripts/backup-database.sh
```

**Wallets:**
```bash
# Manual backup
./scripts/backup-wallets.sh

# Automated: Run daily via cron
0 3 * * * /path/to/4.5/scripts/backup-wallets.sh
```

**Retention:** 90 days (configurable in scripts)
**Encryption:** GPG with RSA 4096-bit key

### Restore Procedures

**Database Restore:**
```bash
./scripts/restore-database.sh /backups/database/db-TIMESTAMP.sql.gz.gpg
```

**Wallet Restore:**
```bash
./scripts/restore-wallet.sh /backups/wallets/wallet-UUID-TIMESTAMP.tar.gz.gpg buyer
```

**Complete DR Guide:** [docs/DISASTER-RECOVERY.md](docs/DISASTER-RECOVERY.md)

---

## 🛠️ Operations

### Deployment

**Standard Deployment:**
```bash
./scripts/deploy.sh <environment> <version>
# Example: ./scripts/deploy.sh staging v0.2.7
```

**Blue-Green Deployment:**
```bash
./scripts/deploy-blue-green.sh <environment> <version>
# Zero-downtime deployment
```

### Health Checks

**Service Health:**
```bash
# Check all services
sudo docker compose ps

# Service-specific health
./scripts/docker-health-check.sh
```

**Infrastructure Validation:**
```bash
# Comprehensive validation
./scripts/validate-infrastructure.sh
```

### Log Management

**View Logs:**
```bash
# All services
sudo docker compose logs -f

# Specific service
sudo docker compose logs -f prometheus

# Via Loki (Grafana Explore)
# http://localhost:3000/explore
```

### Firewall Setup

```bash
# Configure UFW firewall
sudo ./scripts/setup-firewall.sh

# Blocks direct access to:
# - Port 8080 (application)
# - Ports 18082-18084 (Monero RPC)
```

---

## 📚 Documentation

### Technical Docs

| Document | Description |
|----------|-------------|
| [DISASTER-RECOVERY.md](docs/DISASTER-RECOVERY.md) | Complete DR procedures, RTO/RPO targets |
| [DOCKER-DEPLOYMENT.md](docs/DOCKER-DEPLOYMENT.md) | Docker deployment guide |
| [OPERATIONS-RUNBOOK.md](docs/OPERATIONS-RUNBOOK.md) | Day-to-day operations |
| [SECURITY-AUDIT.md](docs/SECURITY-AUDIT.md) | Security hardening checklist |
| [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues and solutions |
| [DATABASE-OPTIMIZATIONS.md](docs/DATABASE-OPTIMIZATIONS.md) | SQLCipher tuning guide |

### Component READMEs

- [monitoring/monero-exporter/README.md](monitoring/monero-exporter/README.md) - Monero exporter docs
- [load-tests/README.md](load-tests/README.md) - Load testing guide

---

## 🎯 Production Readiness

**Current Status:** STAGING-READY ✅

**Beta Terminal Score:** 87/100

**Before Production Deploy:**
- [ ] Test database restore (2h)
- [ ] Test wallet restore (2h)
- [ ] Run k6 load tests (2h)
- [ ] Define SLA/RTO/RPO (1h)
- [ ] Test blue-green deployment (1h)
- [ ] Create incident response playbook (2h)

**Estimated time to production-ready:** ~10 hours

---

## 🤝 Support

For issues or questions:
- **Documentation:** See [docs/](docs/) directory
- **Troubleshooting:** [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)
- **Operations:** [docs/OPERATIONS-RUNBOOK.md](docs/OPERATIONS-RUNBOOK.md)

---

**Last Updated:** 2025-10-22
**Maintained by:** Monero Marketplace Team
**License:** See root LICENSE file
