# INSTRUCTIONS GEMINI - Phase 4.5 Infrastructure & Production Readiness

**Projet:** Monero Marketplace
**Votre Mission:** Créer toute l'infrastructure de production dans le dossier `4.5/`
**Durée:** 33 jours (8 milestones)
**Validation:** Claude vérifiera et intégrera vos fichiers après chaque milestone

---

## 🎯 VUE D'ENSEMBLE

Vous allez créer **TOUTE l'infrastructure de production** pour le Monero Marketplace :
- Containerisation Docker (multi-stage builds)
- Monitoring complet (Prometheus + Grafana + Loki)
- Backup & Disaster Recovery automatisés
- CI/CD Pipeline (GitHub Actions)
- Load testing & Performance
- Security hardening (TLS, Firewall, Secrets)
- Documentation opérationnelle complète

**RÈGLE ABSOLUE:** Créez TOUS les fichiers dans le dossier `4.5/` à la racine du projet.

---

## 📁 STRUCTURE DU DOSSIER 4.5/

```
4.5/
├── docker/
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── docker-compose.prod.yml
│   └── .dockerignore
├── monitoring/
│   ├── prometheus.yml
│   ├── alerts/
│   │   └── marketplace.yml
│   ├── grafana/
│   │   ├── dashboards/
│   │   │   ├── http-overview.json
│   │   │   ├── escrow-overview.json
│   │   │   └── system-overview.json
│   │   └── datasources/
│   │       └── prometheus.yml
│   ├── loki-config.yaml
│   ├── promtail-config.yaml
│   └── alertmanager.yml
├── nginx/
│   ├── nginx.conf
│   └── upstream.conf
├── scripts/
│   ├── docker-start.sh
│   ├── docker-health-check.sh
│   ├── docker-stop.sh
│   ├── backup-database.sh
│   ├── backup-wallets.sh
│   ├── restore-database.sh
│   ├── restore-wallet.sh
│   ├── setup-firewall.sh
│   ├── setup-secrets.sh
│   └── test-backup-restore.sh
├── ci-cd/
│   └── github-workflows/
│       └── ci.yml
├── load-tests/
│   ├── scenarios/
│   │   ├── http-endpoints.js
│   │   └── escrow-flow.js
│   └── README.md
├── security/
│   ├── sops-config.yaml
│   └── secrets.enc.yaml
├── docs/
│   ├── DOCKER-DEPLOYMENT.md
│   ├── DISASTER-RECOVERY.md
│   ├── OPERATIONS-RUNBOOK.md
│   └── TROUBLESHOOTING.md
└── server-metrics/
    ├── metrics.rs
    └── middleware-metrics.rs
```

**Total:** ~60 fichiers à créer

---

## 📋 MILESTONE 4.5.1 : Containerization & Docker (5 jours)

### Objectif
Application Rust containerisée avec multi-stage builds + orchestration 8 services

### ✅ TÂCHES

#### Tâche 1.1 : Dockerfile Multi-Stage
**Fichier:** `4.5/docker/Dockerfile`

**Créer un Dockerfile avec :**
- Stage 1 (builder) : Rust 1.75-slim, compilation optimisée avec cache des dépendances
- Stage 2 (runtime) : Debian 12-slim, user non-root (uid 1000)
- Healthcheck : `curl -f http://localhost:8080/health`
- Taille cible : <500MB

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 72-104

#### Tâche 1.2 : Docker Compose - Stack Complet
**Fichier:** `4.5/docker/docker-compose.yml`

**Créer un docker-compose avec 8 services :**
1. `server` - Application Rust principale (port 8080)
2. `monero-wallet-rpc-buyer` - Wallet buyer (port 18082)
3. `monero-wallet-rpc-vendor` - Wallet vendor (port 18083)
4. `monero-wallet-rpc-arbiter` - Wallet arbiter (port 18084)
5. `prometheus` - Métriques (port 9090)
6. `grafana` - Visualisation (port 3000)
7. `loki` - Logs (port 3100)
8. `promtail` - Log shipper
9. `alertmanager` - Alertes (port 9093)

**Volumes nécessaires :**
- `./data:/app/data` (SQLCipher database)
- `./wallets/buyer:/wallet` (Wallet files)
- `prometheus-data`, `grafana-data`, `loki-data`

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 110-283

#### Tâche 1.3 : Scripts de Gestion Docker
**Fichiers à créer :**

1. **`4.5/scripts/docker-start.sh`**
   - Vérifier Docker daemon running
   - Créer répertoires nécessaires
   - Pull images
   - Démarrer services dans l'ordre
   - Attendre health checks
   - Afficher URLs (app, Prometheus, Grafana)

2. **`4.5/scripts/docker-health-check.sh`**
   - Vérifier santé de tous les services
   - Tester endpoints (server, prometheus, grafana, wallets RPC)
   - Exit code 0 si tout OK, 1 sinon

3. **`4.5/scripts/docker-stop.sh`**
   - Arrêt graceful (timeout 30s)
   - docker-compose down

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 289-380

#### Tâche 1.4 : Documentation Docker
**Fichier:** `4.5/docs/DOCKER-DEPLOYMENT.md`

**Sections requises :**
- Quick Start (dev + prod)
- Architecture diagram (ASCII art)
- Storage volumes (tableau avec backup requirements)
- Troubleshooting (5+ scenarios)
- Performance tuning (resource limits)

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 386-470

### ✅ VALIDATION MILESTONE 1

**Commandes de test :**
```bash
cd 4.5/

# Test build Dockerfile
docker build -f docker/Dockerfile -t monero-marketplace:test ..

# Test docker-compose
docker-compose -f docker/docker-compose.yml config

# Test scripts
bash scripts/docker-start.sh
bash scripts/docker-health-check.sh
bash scripts/docker-stop.sh
```

**Critères d'acceptance :**
- [ ] Dockerfile build sans erreur
- [ ] Image finale <500MB
- [ ] docker-compose.yml valide (8 services)
- [ ] Scripts exécutables et fonctionnels
- [ ] Documentation complète (4 sections minimum)

---

## 📋 MILESTONE 4.5.2 : Monitoring & Observability (5 jours)

### Objectif
Monitoring complet avec Prometheus, Grafana, Loki + instrumentation code

### ✅ TÂCHES

#### Tâche 2.1 : Configuration Prometheus
**Fichiers à créer :**

1. **`4.5/monitoring/prometheus.yml`**
   - Scrape configs pour : server (10s), wallets (30s), prometheus self
   - Alertmanager integration
   - Retention 30 jours

2. **`4.5/monitoring/alerts/marketplace.yml`**
   - **10 alertes minimum :**
     - HighErrorRate (>5% sur 5min)
     - ServiceDown (up==0 pendant 2min)
     - SlowResponseTime (p95 > 2s)
     - EscrowStuckInPending (>1h)
     - HighDatabaseLockContention
     - WalletRPCUnreachable
     - DiskSpaceLow (<10%)
     - MemoryUsageHigh (>90%)
     - WebSocketConnectionsSpike (>100 conn/s)
     - SSLCertificateExpiring (<30 jours)

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 478-630

#### Tâche 2.2 : Instrumentation Code
**Fichiers à créer :**

**`4.5/server-metrics/metrics.rs`** (207 lignes)

Créer les métriques suivantes avec `lazy_static!` et `prometheus` crate :

**HTTP Metrics :**
- `http_requests_total` (IntCounterVec) - labels: method, path, status
- `http_request_duration_seconds` (HistogramVec) - labels: method, path

**Escrow Metrics :**
- `escrow_total` (IntGaugeVec) - label: state
- `escrow_state_transitions_total` (IntCounterVec) - labels: from_state, to_state
- `escrow_last_update_timestamp` (IntGaugeVec) - label: escrow_id

**Database Metrics :**
- `db_operation_duration_seconds` (HistogramVec) - labels: operation, table
- `db_lock_wait_seconds` (HistogramVec) - label: operation
- `db_errors_total` (IntCounterVec) - label: error_type

**WebSocket Metrics :**
- `websocket_connections_active` (IntGaugeVec) - label: user_type
- `websocket_connections_total` (IntCounterVec) - label: user_type
- `websocket_messages_sent_total` (IntCounterVec) - label: message_type

**Monero RPC Metrics :**
- `monero_rpc_duration_seconds` (HistogramVec) - labels: method, wallet
- `monero_rpc_errors_total` (IntCounterVec) - labels: method, error_type

**Fonctions helper :**
- `metrics_handler()` - Actix-Web endpoint `/metrics`
- `record_http_request()`
- `update_escrow_gauge()`
- `record_escrow_transition()`
- `record_db_operation()`
- `increment_websocket_connections()`
- `record_monero_rpc_call()`

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 636-805

**`4.5/server-metrics/middleware-metrics.rs`** (Middleware Actix-Web)

Middleware automatique qui enregistre toutes les requêtes HTTP.

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 811-880

#### Tâche 2.3 : Dashboards Grafana
**Fichiers à créer :**

1. **`4.5/monitoring/grafana/dashboards/http-overview.json`**
   - Request Rate (req/s)
   - Error Rate (%)
   - Response Time (p50/p95/p99)
   - Top Endpoints by Traffic

2. **`4.5/monitoring/grafana/dashboards/escrow-overview.json`**
   - Escrows by State (pie chart)
   - State Transitions (heatmap)
   - Stuck Escrows (singlestat)

3. **`4.5/monitoring/grafana/dashboards/system-overview.json`**
   - CPU Usage (%)
   - Memory Usage (%)
   - Disk Usage (gauge)
   - Network I/O (MB/s)

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 886-980

#### Tâche 2.4 : Configuration Alertmanager
**Fichier:** `4.5/monitoring/alertmanager.yml`

**Routing :**
- Critical alerts → PagerDuty + Email
- Warnings → Email only
- Info → Slack

**Receivers :**
- `pagerduty-critical` (avec service_key)
- `email-critical` (avec template HTML)
- `email-warning`
- `slack-info` (webhook URL)

**Inhibition rules :** ServiceDown supprime HighErrorRate

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 986-1070

### ✅ VALIDATION MILESTONE 2

**Commandes de test :**
```bash
cd 4.5/

# Valider Prometheus config
promtool check config monitoring/prometheus.yml
promtool check rules monitoring/alerts/marketplace.yml

# Valider JSON Grafana dashboards
jq . monitoring/grafana/dashboards/*.json

# Test alert routing (via curl)
curl -XPOST http://localhost:9093/api/v1/alerts -d '[{"labels":{"alertname":"TestAlert","severity":"critical"}}]'
```

**Critères d'acceptance :**
- [ ] Prometheus config valide
- [ ] 10+ alertes définies
- [ ] metrics.rs compilable (syntaxe Rust valide)
- [ ] 3 dashboards Grafana (JSON valides)
- [ ] Alertmanager config valide

---

## 📋 MILESTONE 4.5.3 : Backup & Disaster Recovery (5 jours)

### Objectif
Backups automatisés + procédures de recovery testées (RTO < 15min, RPO < 6h)

### ✅ TÂCHES

#### Tâche 3.1 : Scripts de Backup Automatisés
**Fichiers à créer :**

1. **`4.5/scripts/backup-database.sh`**
   - Backup SQLCipher avec `sqlite3 .backup`
   - Compression gzip
   - Calcul SHA256 checksum
   - Encryption GPG (AES256)
   - Upload S3/Glacier (optionnel)
   - Cleanup vieux backups (retention 30 jours)
   - Vérification intégrité

2. **`4.5/scripts/backup-wallets.sh`**
   - Backup 3 wallets (buyer, vendor, arbiter)
   - Tar + gzip par wallet
   - Encryption GPG
   - Upload S3
   - Retention 90 jours

**Cron jobs suggérés :**
```bash
# Database backup toutes les 6h
0 */6 * * * /app/scripts/backup-database.sh >> /var/log/marketplace/backup.log 2>&1

# Wallet backup daily à 2 AM
0 2 * * * /app/scripts/backup-wallets.sh >> /var/log/marketplace/backup.log 2>&1
```

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1078-1180

#### Tâche 3.2 : Procédures de Recovery
**Fichiers à créer :**

1. **`4.5/scripts/restore-database.sh`**
   - Confirmation utilisateur (prompt "yes/no")
   - Stop application
   - Backup DB actuelle
   - Decrypt + decompress backup
   - Restore avec sqlite3
   - Vérification intégrité (`PRAGMA integrity_check`)
   - Restart application

2. **`4.5/scripts/restore-wallet.sh`**
   - Paramètre: wallet name (buyer/vendor/arbiter)
   - Stop wallet RPC
   - Restore depuis backup
   - Verify wallet files
   - Restart wallet RPC

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1186-1260

#### Tâche 3.3 : Documentation DR
**Fichier:** `4.5/docs/DISASTER-RECOVERY.md`

**3 Scenarios détaillés :**

1. **Scenario 1: Database Corruption**
   - Détection (SQLite error)
   - Recovery procedure (5 étapes)
   - RTO: 15 minutes, RPO: 6 heures

2. **Scenario 2: Wallet File Loss**
   - Détection (Monero RPC error)
   - Recovery procedure (4 étapes)
   - RTO: 10 minutes, RPO: 24 heures

3. **Scenario 3: Complete Server Loss**
   - Provisioning nouveau serveur
   - Restore backups depuis S3
   - Restore database + wallets
   - Start stack Docker
   - Verify health
   - RTO: 2 heures, RPO: 24 heures

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1266-1360

#### Tâche 3.4 : Tests de Backup/Restore
**Fichier:** `4.5/scripts/test-backup-restore.sh`

Script qui :
1. Crée test database
2. Run backup
3. Corrompt database (dd avec random data)
4. Restore depuis backup
5. Vérifie intégrité
6. Cleanup

**Cron job mensuel :** Test automatique le 1er du mois à 3 AM

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1366-1410

### ✅ VALIDATION MILESTONE 3

**Commandes de test :**
```bash
cd 4.5/

# Test scripts backup (dry-run)
bash scripts/backup-database.sh
bash scripts/backup-wallets.sh

# Vérifier scripts restore (syntaxe)
bash -n scripts/restore-database.sh
bash -n scripts/restore-wallet.sh

# Test backup/restore complet
bash scripts/test-backup-restore.sh
```

**Critères d'acceptance :**
- [ ] Scripts backup exécutables
- [ ] Encryption GPG fonctionnel
- [ ] Scripts restore avec confirmation user
- [ ] Documentation DR complète (3 scenarios)
- [ ] Test backup/restore passe

---

## 📋 MILESTONE 4.5.4 : CI/CD Pipeline (5 jours)

### Objectif
GitHub Actions pipeline complet avec tests, security scan, deploy automatique

### ✅ TÂCHES

#### Tâche 4.1 : GitHub Actions Workflow
**Fichier:** `4.5/ci-cd/github-workflows/ci.yml`

**6 Jobs parallèles/séquentiels :**

1. **Job: quality**
   - Rust fmt check
   - Clippy (strict mode)
   - Security theatre check (`./scripts/check-security-theatre.sh`)
   - Cache cargo registry/index/build

2. **Job: test** (depends on quality)
   - Build workspace
   - Run all tests
   - Upload test results si failure

3. **Job: security** (depends on quality)
   - cargo-audit
   - actions-rs/audit-check (vulnerabilities)

4. **Job: docker** (depends on test + security)
   - Build & push Docker image
   - Tags: latest + SHA commit
   - Registry: ghcr.io
   - Cache buildx

5. **Job: deploy-staging** (depends on docker, branch develop)
   - Deploy via SSH
   - Pull images
   - docker-compose up
   - Smoke test

6. **Job: deploy-production** (depends on docker, branch main)
   - Deploy via SSH avec rollback automatique si échec
   - Health check
   - Notify on failure (Slack)

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1418-1600

#### Tâche 4.2 : Scripts de Déploiement
**Fichiers à créer :**

1. **`4.5/scripts/deploy.sh`**
   - Paramètres: environment (staging/production), version
   - SSH vers serveur cible
   - Backup database avant deploy
   - Pull nouvelle image Docker
   - Rolling update
   - Health check
   - Rollback automatique si échec

2. **`4.5/scripts/rollback.sh`**
   - Paramètre: environment
   - SSH vers serveur
   - Get previous image tag
   - Stop current
   - Restore DB depuis backup
   - Start avec previous image

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1606-1680

#### Tâche 4.3 : Configuration Environnements
**Fichiers à créer :**

1. **`4.5/config/staging.env`**
```bash
DATABASE_URL=sqlite:///app/data/marketplace-staging.db?mode=rwc
RUST_LOG=debug
MONERO_NETWORK=testnet
```

2. **`4.5/config/production.env`**
```bash
DATABASE_URL=sqlite:///app/data/marketplace.db?mode=rwc
RUST_LOG=info
MONERO_NETWORK=mainnet
```

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1686-1710

### ✅ VALIDATION MILESTONE 4

**Commandes de test :**
```bash
cd 4.5/

# Valider GitHub Actions workflow
actionlint ci-cd/github-workflows/ci.yml

# Test scripts deploy (syntaxe)
bash -n scripts/deploy.sh
bash -n scripts/rollback.sh

# Vérifier env files
cat config/staging.env
cat config/production.env
```

**Critères d'acceptance :**
- [ ] Workflow YAML valide (6 jobs)
- [ ] Scripts deploy/rollback exécutables
- [ ] Env files créés (staging + prod)
- [ ] Tous secrets documentés (README)

---

## 📋 MILESTONE 4.5.5 : Load Testing & Performance (3 jours)

### Objectif
Tests de charge avec k6 + optimisations database/cache (100 req/s, p95 < 200ms)

### ✅ TÂCHES

#### Tâche 5.1 : Load Testing Scripts (k6)
**Fichiers à créer :**

1. **`4.5/load-tests/scenarios/http-endpoints.js`**
   - Stages: Ramp-up 10→50→100 users sur 20 minutes
   - Threshold: p95 < 200ms, error rate < 5%
   - Tests: GET /api/listings, Search, Register user
   - Output: InfluxDB + JSON report

2. **`4.5/load-tests/scenarios/escrow-flow.js`**
   - Scenario: 10 VUs constant pendant 5 minutes
   - Flow: Register buyer → Create listing → Create order
   - Checks: status 200, response structure

3. **`4.5/load-tests/README.md`**
   - Installation k6
   - Commandes pour run tests
   - Génération HTML report

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1718-1830

#### Tâche 5.2 : Optimizations Database
**Fichier:** `4.5/docs/DATABASE-OPTIMIZATIONS.md`

**Indexes à créer :**
```sql
CREATE INDEX IF NOT EXISTS idx_listings_vendor_id ON listings(vendor_id);
CREATE INDEX IF NOT EXISTS idx_listings_created_at ON listings(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_buyer_id ON orders(buyer_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_escrows_state ON escrows(state);
CREATE INDEX IF NOT EXISTS idx_transactions_order_id ON transactions(order_id);
```

**Connection pooling Diesel :**
```rust
r2d2::Pool::builder()
    .max_size(20)
    .min_idle(Some(5))
    .connection_timeout(Duration::from_secs(10))
```

**Redis caching (optionnel) :**
```rust
// Cache listings pour 5 minutes
redis::cmd("SETEX")
    .arg(format!("listing:{}", listing_id))
    .arg(300)
    .arg(json)
```

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1836-1890

### ✅ VALIDATION MILESTONE 5

**Commandes de test :**
```bash
cd 4.5/

# Install k6
curl -L https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz | tar xvz

# Run load tests
k6 run load-tests/scenarios/http-endpoints.js
k6 run load-tests/scenarios/escrow-flow.js
```

**Critères d'acceptance :**
- [ ] 2 scripts k6 créés
- [ ] Load tests s'exécutent sans erreur
- [ ] Documentation optimizations complète
- [ ] README load-tests avec instructions

---

## 📋 MILESTONE 4.5.6 : Security Hardening (4 jours)

### Objectif
Production-grade security (TLS 1.3, Firewall UFW, Secrets SOPS)

### ✅ TÂCHES

#### Tâche 6.1 : Configuration TLS/SSL
**Fichier:** `4.5/nginx/nginx.conf`

**Nginx reverse proxy avec :**
- TLS 1.3 uniquement
- Ciphers: AES256-GCM, CHACHA20-POLY1305, AES128-GCM
- HSTS header (63072000 max-age)
- Security headers (X-Frame-Options DENY, CSP, etc.)
- OCSP Stapling
- Rate limiting (10 req/s general, 5 req/m auth)
- Upstream keepalive

**Cron auto-renew :**
```bash
# Weekly certificate renewal
0 0 * * 0 certbot renew --quiet && systemctl reload nginx
```

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1898-1990

#### Tâche 6.2 : Firewall Configuration
**Fichier:** `4.5/scripts/setup-firewall.sh`

**UFW rules :**
- Default: deny incoming, allow outgoing
- Allow SSH (port 22, rate limited)
- Allow HTTPS (port 443)
- Allow Prometheus (9090) FROM internal IPs only
- Allow Grafana (3000) FROM internal IPs only
- DENY direct backend (8080)
- DENY Monero RPC (18082-18084)

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 1996-2040

#### Tâche 6.3 : Secrets Management (SOPS + Age)
**Fichiers à créer :**

1. **`4.5/scripts/setup-secrets.sh`**
   - Install SOPS + Age
   - Generate Age key (`age-keygen`)
   - Create secrets.yaml template
   - Encrypt with SOPS
   - Shred plaintext

2. **`4.5/security/secrets.enc.yaml`**
   - Template encrypted (placeholder values)
   - Fields: database_password, grafana_admin_password, backup_gpg_passphrase

**Usage :**
```bash
# Decrypt at runtime
sops --decrypt secrets.enc.yaml > /tmp/secrets.yaml
source /tmp/secrets.yaml
rm /tmp/secrets.yaml
```

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 2046-2100

#### Tâche 6.4 : Security Audit
**Fichier:** `4.5/docs/SECURITY-AUDIT.md`

**Documentation des scans :**
- `cargo audit` (dependency vulnerabilities)
- `trivy` (container scanning)
- `sqlmap` (SQL injection)
- `OWASP ZAP` (web app scanning)
- `lynis` (system hardening)

**Checklist :**
- [ ] No hardcoded credentials
- [ ] Secrets encrypted at rest
- [ ] TLS 1.3 enforced
- [ ] HSTS enabled
- [ ] Rate limiting active
- [ ] Firewall restricts RPC
- [ ] Database encrypted (SQLCipher)
- [ ] Backups encrypted (GPG)

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 2106-2150

### ✅ VALIDATION MILESTONE 6

**Commandes de test :**
```bash
cd 4.5/

# Valider nginx config
nginx -t -c nginx/nginx.conf

# Test firewall script
bash -n scripts/setup-firewall.sh

# Test secrets setup
bash scripts/setup-secrets.sh

# Vérifier SOPS encryption
sops --decrypt security/secrets.enc.yaml
```

**Critères d'acceptance :**
- [ ] Nginx config valide (TLS 1.3)
- [ ] UFW script fonctionnel
- [ ] SOPS + Age setup complet
- [ ] Security audit doc (5+ scans)

---

## 📋 MILESTONE 4.5.7 : Documentation Opérationnelle (3 jours)

### Objectif
Runbooks complets pour opérations quotidiennes + incident response

### ✅ TÂCHES

#### Tâche 7.1 : Operations Runbook
**Fichier:** `4.5/docs/OPERATIONS-RUNBOOK.md`

**Sections requises :**

1. **Daily Operations (10 min)**
   - Morning checks (health, dashboards, alerts, logs)

2. **Weekly Tasks**
   - Security audit logs review
   - Disk space check
   - Test backup restoration
   - Prometheus alerts review
   - Update dependencies

3. **Monthly Tasks**
   - Full DR test
   - Security scan (cargo audit + Trivy)
   - Certificate renewal verification
   - Performance review (p95 response times)

4. **Incident Response (10+ scenarios)**
   - High CPU Usage (>90%)
   - Database Locked Error
   - Wallet RPC Unreachable
   - Disk Space Low
   - Memory Leak
   - SSL Certificate Expired
   - DDoS Attack
   - Data Breach
   - Service Outage
   - Backup Corruption

Chaque scenario avec :
- Symptoms
- Diagnosis commands
- Remediation steps

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 2158-2300

#### Tâche 7.2 : Troubleshooting Guide
**Fichier:** `4.5/docs/TROUBLESHOOTING.md`

**Common Issues (15+) :**
- "Connection refused" to Monero RPC
- Slow database queries
- TLS certificate expired
- Docker container won't start
- Out of memory (OOM) killed
- High latency (>1s responses)
- WebSocket connections dropping
- Prometheus metrics missing
- Grafana dashboard empty
- Backup failed
- Restore failed
- Port already in use
- Permission denied errors
- Database migration failed
- Wallet sync stuck

Chaque issue avec :
- Cause
- Fix (commandes exactes)
- Prevention

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 2306-2380

### ✅ VALIDATION MILESTONE 7

**Critères d'acceptance :**
- [ ] Operations runbook complet (daily/weekly/monthly + 10 incidents)
- [ ] Troubleshooting guide (15+ issues)
- [ ] Toutes commandes testables
- [ ] Format markdown propre

---

## 📋 MILESTONE 4.5.8 : Deployment Automation (3 jours)

### Objectif
Zero-downtime deployments avec blue-green strategy

### ✅ TÂCHES

#### Tâche 8.1 : Blue-Green Deployment
**Fichiers à créer :**

1. **`4.5/docker/docker-compose.blue-green.yml`**
   - 2 environnements: server-blue + server-green
   - Nginx load balancer
   - Ports: blue=8080, green=8081
   - Shared volumes (database, wallets)

2. **`4.5/nginx/upstream.conf`**
   - Config upstream avec weights
   - Blue: weight=100 (active)
   - Green: weight=0 (standby)

3. **`4.5/scripts/deploy-blue-green.sh`**
   - Deploy to green (inactive)
   - Wait health check
   - Run smoke tests
   - Switch traffic gradually (50/50 puis 0/100)
   - Monitor 5 minutes
   - Check error rate
   - Stop blue si success, rollback si échec

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 2388-2520

#### Tâche 8.2 : Automated Rollback
**Fichier:** `4.5/scripts/rollback-blue-green.sh`

- Switch all traffic back to blue (100%)
- Stop green environment
- Logs rollback reason

**Référence complète :** Voir `docs/INFRASTRUCTURE-ROADMAP.md` lignes 2526-2550

### ✅ VALIDATION MILESTONE 8

**Commandes de test :**
```bash
cd 4.5/

# Valider blue-green compose
docker-compose -f docker/docker-compose.blue-green.yml config

# Test deploy script (dry-run)
bash -n scripts/deploy-blue-green.sh
bash -n scripts/rollback-blue-green.sh
```

**Critères d'acceptance :**
- [ ] docker-compose blue-green valide
- [ ] Nginx upstream config créé
- [ ] Deploy script avec health checks
- [ ] Rollback script fonctionnel

---

## ✅ VALIDATION GLOBALE PHASE 4.5

Après avoir complété les 8 milestones, **Claude vérifiera TOUT** avec :

### Checklist Finale

**Structure :**
- [ ] Dossier `4.5/` à la racine du projet
- [ ] ~60 fichiers créés (docker, monitoring, scripts, docs, ci-cd)
- [ ] Aucun fichier en dehors de `4.5/`

**Docker :**
- [ ] Dockerfile multi-stage <500MB
- [ ] docker-compose.yml (8+ services)
- [ ] Scripts docker-start/stop/health-check exécutables

**Monitoring :**
- [ ] Prometheus config + 10 alertes
- [ ] 3 dashboards Grafana (JSON valides)
- [ ] metrics.rs compilable (syntaxe Rust)
- [ ] Alertmanager config

**Backup/DR :**
- [ ] Scripts backup database + wallets
- [ ] Scripts restore avec confirmation
- [ ] Documentation DR (3 scenarios)
- [ ] Test backup/restore

**CI/CD :**
- [ ] GitHub Actions workflow (6 jobs)
- [ ] Scripts deploy/rollback
- [ ] Config staging + production

**Load Testing :**
- [ ] 2 scenarios k6 (http-endpoints + escrow-flow)
- [ ] README avec instructions
- [ ] Documentation optimizations

**Security :**
- [ ] Nginx TLS 1.3 config
- [ ] UFW firewall script
- [ ] SOPS secrets setup
- [ ] Security audit doc

**Documentation :**
- [ ] Operations runbook (daily/weekly/monthly + incidents)
- [ ] Troubleshooting guide (15+ issues)
- [ ] DOCKER-DEPLOYMENT.md
- [ ] DISASTER-RECOVERY.md

**Deployment :**
- [ ] Blue-green docker-compose
- [ ] Deploy script avec smoke tests
- [ ] Rollback automatique

---

## 🎯 WORKFLOW DE VALIDATION

### Après chaque Milestone

**Vous faites :**
```
1. Créer tous les fichiers du milestone dans 4.5/
2. M'envoyer un message : "Milestone 4.5.X terminé"
```

**Je (Claude) ferai :**
```
1. Vérifier structure fichiers (ls -R 4.5/)
2. Valider syntaxe (bash -n scripts/*.sh, docker-compose config, etc.)
3. Vérifier contenu vs spécifications
4. Vous donner feedback : ✅ OK ou 🔴 Corrections nécessaires
5. Si OK → Continuer milestone suivant
```

### Après Milestone 8 (Fin Phase 4.5)

**Je (Claude) ferai :**
```
1. Review complète des 60+ fichiers
2. Vérifier cohérence inter-fichiers
3. Tester intégration (docker build, docker-compose up, etc.)
4. Déplacer fichiers de 4.5/ vers racine projet
   - 4.5/docker/Dockerfile → ./Dockerfile
   - 4.5/docker/docker-compose.yml → ./docker-compose.yml
   - 4.5/monitoring/ → ./monitoring/
   - 4.5/scripts/ → ./scripts/ (merge avec existants)
   - etc.
5. Commit final : "feat: Phase 4.5 Infrastructure Complete"
```

---

## 📚 RÉFÉRENCES COMPLÈTES

Tous les détails (code complet, configurations exactes) sont dans :
**`docs/INFRASTRUCTURE-ROADMAP.md`**

Pour chaque milestone, référez-vous aux sections indiquées :
- Milestone 4.5.1 → lignes 62-470
- Milestone 4.5.2 → lignes 476-1070
- Milestone 4.5.3 → lignes 1072-1410
- Milestone 4.5.4 → lignes 1412-1710
- Milestone 4.5.5 → lignes 1712-1890
- Milestone 4.5.6 → lignes 1892-2150
- Milestone 4.5.7 → lignes 2152-2380
- Milestone 4.5.8 → lignes 2382-2550

---

## 🚀 COMMENCEZ MAINTENANT

**Première action :**
```
Créer le dossier 4.5/ et commencer Milestone 4.5.1 (Containerization & Docker)
```

**Bonne chance ! Je (Claude) suis prêt à valider votre travail après chaque milestone.**
