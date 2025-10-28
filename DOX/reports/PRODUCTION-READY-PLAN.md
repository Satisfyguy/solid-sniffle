# Plan de Transformation Production-Ready - Monero Marketplace

## 📊 État Actuel Analysé

### Composants Prêts pour Production ✅
- **Wallet Crate** (100% production-ready)
  - 0 TODOs/placeholders
  - Toutes les fonctions multisig implémentées avec gestion d'erreurs réelle
  - Tests E2E complets
  - Zero security theatre violations

- **CLI Crate** (100% production-ready)
  - 0 TODOs/placeholders
  - Interface fonctionnelle avec commandes complètes

- **Common Crate** (100% production-ready)
  - Types, erreurs, constantes définis

- **Base de données Server** (95% production-ready)
  - ✅ Migrations SQL complètes avec tous les champs
  - ✅ Models Diesel avec CRUD réel (User, Escrow)
  - ✅ Pool de connexions R2D2 fonctionnel
  - ✅ Encryption AES-256-GCM implémentée correctement
  - ✅ Opérations async avec tokio::spawn_blocking

- **Services Escrow** (90% production-ready)
  - ✅ Logique d'orchestration réelle (pas de placeholders hardcodés)
  - ✅ Arbiter assignment via requête DB
  - ✅ Encryption/decryption fonctionnelle
  - ✅ Validation des inputs
  - ✅ Méthodes: init, prepare, make_multisig, release, dispute

### Composants Nécessitant Transformation 🔧

**1. WalletManager** ([server/src/wallet_manager.rs](server/src/wallet_manager.rs))
- Statut: STUBS VALIDÉS avec documentation claire
- Fonctions stubbed: `make_multisig()`, `prepare_multisig()`, `export_multisig_info()`, `import_multisig_info()`
- **Raison**: Nécessite intégration RPC Monero réelle (actuellement retourne placeholders déterministes)

**2. WebSocketServer** ([server/src/websocket.rs](server/src/websocket.rs))
- Statut: MODE LOGGING (pas de WebSocket réel)
- **Raison**: Nécessite implémentation actix-web-actors

**3. Encryption Key Management** ([server/.env.example](server/.env.example))
- Statut: Clé éphémère générée au démarrage
- **Raison**: Nécessite gestion persistante pour production

---

## 🎯 Stratégie "Production-First" (Nouveau Paradigme)

### Principe Fondamental
**STOP au développement "test puis production". À partir de maintenant:**
- ✅ Coder DIRECTEMENT pour la production
- ✅ Utiliser des feature flags pour basculer entre testnet/mainnet
- ✅ Configuration via variables d'environnement (dev/staging/prod)
- ✅ Tests d'intégration contre services RÉELS (Monero testnet, pas de mocks)
- ✅ Code = Production code dès le premier commit

### Configuration Multi-Environnement
```rust
// Nouveau fichier: server/src/config.rs
pub enum Environment {
    Development,  // Testnet, logs verbeux, limits relâchés
    Staging,      // Testnet, production-like config
    Production,   // Mainnet, strict limits, monitoring
}
```

Fini les branches "dev" vs "prod". Un seul code, configuration différente.

---

## 🚀 Plan d'Action Concret (Priorités)

### PHASE 1: Transformation Immédiate (Semaine 1-2) - CRITIQUE ⚡

#### Action 1.1: Implémenter WalletManager Production (3 jours)
**Fichiers à modifier:**
- [server/src/wallet_manager.rs](server/src/wallet_manager.rs)

**Tâches:**
1. Créer `WalletInstance` struct pour gérer chaque wallet (buyer/vendor/arbiter) séparément
2. Implémenter `make_multisig()` en appelant `monero_client.make_multisig()` (déjà dans wallet crate)
3. Implémenter `prepare_multisig()` via RPC réel
4. Implémenter `export_multisig_info()` et `import_multisig_info()`
5. Ajouter gestion d'état des wallets (track quel wallet est à quelle étape)
6. Tests d'intégration contre Monero testnet

**Critères de succès:**
- ✅ Création réelle d'un multisig 2-of-3 via le WalletManager
- ✅ Aucun placeholder/stub restant
- ✅ Test E2E passant avec 3 wallets testnet

#### Action 1.2: Implémenter WebSocket Réel (2 jours)
**Fichiers à modifier:**
- [server/src/websocket.rs](server/src/websocket.rs)
- [server/src/main.rs](server/src/main.rs) (ajout route WebSocket)

**Tâches:**
1. Implémenter `WebSocketSession` actor avec actix-web-actors
2. Maintenir `HashMap<Uuid, Vec<Addr<WebSocketSession>>>` pour tracking connections
3. Implémenter heartbeat/ping-pong (30s timeout)
4. Gérer connect/disconnect dans lifecycle Actor
5. Serializer WsEvent en JSON pour envoi

**Critères de succès:**
- ✅ Client peut se connecter à `/ws`
- ✅ Notifications envoyées en temps réel lors d'événements escrow
- ✅ Gestion automatique des déconnexions

#### Action 1.3: Encryption Key Management Production (1 jour)
**Fichiers à créer/modifier:**
- [server/src/crypto/key_manager.rs](server/src/crypto/key_manager.rs) (nouveau)
- [server/src/main.rs](server/src/main.rs)
- [server/.env.example](server/.env.example)

**Tâches:**
1. Créer fonction `load_or_generate_key(path: &Path)` qui:
   - Vérifie si fichier clé existe
   - Si oui, charge et valide (32 bytes)
   - Si non, génère nouvelle clé et l'écrit (permissions 600)
2. Ajouter variable d'env `ENCRYPTION_KEY_FILE=/path/to/secure/keyfile`
3. Charger la clé au démarrage du serveur
4. ⚠️ AVERTISSEMENT: En production, utiliser un KMS (AWS KMS, HashiCorp Vault)

**Critères de succès:**
- ✅ Clé persiste entre redémarrages
- ✅ Données chiffrées avec une clé restent déchiffrables après restart
- ✅ Permissions fichier clé = 600 (owner read/write only)

---

### PHASE 2: Configuration Production-First (Semaine 2) - HAUTE 🔥

#### Action 2.1: Système de Configuration Multi-Environnement (2 jours)
**Fichiers à créer:**
- [server/src/config.rs](server/src/config.rs) (nouveau)
- [server/.env.development](server/.env.development)
- [server/.env.staging](server/.env.staging)
- [server/.env.production](server/.env.production)

**Structure config.rs:**
```rust
pub struct AppConfig {
    pub environment: Environment,
    pub database_url: String,
    pub monero_network: MoneroNetwork, // Testnet/Mainnet
    pub max_escrow_amount_xmr: u64,
    pub rate_limit_requests_per_minute: u32,
    pub session_timeout_hours: u64,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        // Load from .env.{ENV}
    }
}
```

**Différences par environnement:**
| Paramètre | Development | Staging | Production |
|-----------|-------------|---------|------------|
| Monero Network | Testnet | Testnet | Mainnet |
| Max Escrow | 10 XMR | 1 XMR | 5 XMR (initialement) |
| Rate Limit | 1000/min | 100/min | 60/min |
| Log Level | debug | info | warn |
| Session Timeout | 24h | 8h | 2h |

**Critères de succès:**
- ✅ Un seul binaire fonctionne dans tous les environnements
- ✅ `ENV=production cargo run` utilise config production
- ✅ Impossible de lancer en production sans toutes les configs requises

#### Action 2.2: Feature Flags (1 jour)
**Fichiers à modifier:**
- [server/Cargo.toml](server/Cargo.toml)
- Code concerné (conditional compilation)

**Features à définir:**
```toml
[features]
default = ["testnet"]
testnet = []
mainnet = []
monitoring = ["prometheus", "opentelemetry"]
```

**Usage:**
```rust
#[cfg(feature = "mainnet")]
const DEFAULT_MAX_ESCROW: u64 = 5_000_000_000_000; // 5 XMR

#[cfg(feature = "testnet")]
const DEFAULT_MAX_ESCROW: u64 = 10_000_000_000_000; // 10 XMR
```

**Critères de succès:**
- ✅ Build testnet vs mainnet via flags
- ✅ Protections supplémentaires en production (e.g., double-check confirmations)

---

### PHASE 3: Qualité & Sécurité Production (Semaine 3) - CRITIQUE ⚡

#### Action 3.1: Renforcer Pre-commit Hooks (1 jour)
**Fichiers à modifier:**
- [scripts/pre-commit.sh](scripts/pre-commit.sh)
- [scripts/check-security-theatre.sh](scripts/check-security-theatre.sh)

**Nouveaux checks à ajouter:**
1. **Vérifier aucun TODO/FIXME dans code Rust (sauf docs/tests)**
   ```bash
   # Rejeter si TODO trouvé dans server/src/*.rs (hors commentaires doc)
   ```
2. **Vérifier configuration environment correcte**
   ```bash
   # Si ENV=production, bloquer commit si secrets en clair dans .env
   ```
3. **Vérifier dépendances à jour (cargo audit)**
   ```bash
   cargo audit
   ```
4. **Vérifier aucun secret hardcodé (gitleaks)**
   ```bash
   gitleaks detect --no-git
   ```

**Critères de succès:**
- ✅ Impossible de commit du code avec placeholders
- ✅ Impossible de commit avec dépendances vulnérables
- ✅ Impossible de commit avec secrets hardcodés

#### Action 3.2: Tests d'Intégration Production-Ready (2 jours)
**Fichiers à créer:**
- [server/tests/production_integration_tests.rs](server/tests/production_integration_tests.rs)

**Tests à implémenter:**
1. **Test Full Escrow Flow (bout en bout)**
   - Créer 3 users (buyer, vendor, arbiter)
   - Créer listing
   - Buyer crée order → escrow init
   - Les 3 parties préparent multisig (RPC réel)
   - Vérifier multisig address généré
   - Simuler funding (testnet)
   - Release funds (2-of-3 signatures)
   - Vérifier transaction broadcasted

2. **Test Dispute Resolution**
   - Setup escrow
   - Buyer ouvre dispute
   - Arbiter résout (refund buyer)
   - Vérifier funds retournés

3. **Test WebSocket Notifications**
   - Client se connecte via WebSocket
   - Trigger événement escrow
   - Vérifier notification reçue en temps réel

**Critères de succès:**
- ✅ Tests passent contre Monero testnet RÉEL (pas de mocks)
- ✅ Tests peuvent tourner en CI/CD
- ✅ Tests couvrent happy path + error cases

#### Action 3.3: Monitoring & Observability (2 jours)
**Fichiers à créer:**
- [server/src/telemetry.rs](server/src/telemetry.rs)
- [server/prometheus_metrics.rs](server/prometheus_metrics.rs)

**Métriques à tracker:**
- Nombre d'escrows actifs par statut
- Latence RPC Monero (p50, p95, p99)
- Taux d'erreur par endpoint API
- Nombre de connexions WebSocket actives
- Volume XMR en escrow

**Logs structurés:**
```rust
tracing::info!(
    escrow_id = %escrow.id,
    status = %escrow.status,
    "Escrow status changed"
);
```

**Alertes à configurer:**
- ⚠️ Monero RPC down > 1 minute
- ⚠️ Transaction stuck (0 confirmations après 30 min)
- ⚠️ Erreur rate > 5%
- 🚨 Fonds perdus détectés (balance mismatch)

**Critères de succès:**
- ✅ Dashboard Grafana montrant toutes les métriques
- ✅ Alertes fonctionnelles (test avec conditions artificielles)
- ✅ Logs JSON parsables par outils d'analyse

---

### PHASE 4: Deployment Production (Semaine 4) - HAUTE 🔥

#### Action 4.1: Infrastructure as Code (2 jours)
**Fichiers à créer:**
- [deployment/docker-compose.production.yml](deployment/docker-compose.production.yml)
- [deployment/Dockerfile.server](deployment/Dockerfile.server)
- [deployment/nginx.conf](deployment/nginx.conf)
- [deployment/tor/torrc](deployment/tor/torrc)

**Services à containerizer:**
1. **Server Actix-web** (Rust binary)
2. **Monero Wallet RPC** (3 instances: buyer, vendor, arbiter)
3. **Monero Daemon** (monerod)
4. **Tor Hidden Service**
5. **Nginx** (reverse proxy + rate limiting)
6. **Prometheus + Grafana**

**Critères de succès:**
- ✅ `docker-compose up` lance tout l'environnement production-like
- ✅ Tor hidden service accessible
- ✅ Toutes les connexions inter-services fonctionnelles

#### Action 4.2: Scripts de Déploiement (1 jour)
**Fichiers à créer:**
- [scripts/deploy-production.sh](scripts/deploy-production.sh)
- [scripts/backup-database.sh](scripts/backup-database.sh)
- [scripts/restore-database.sh](scripts/restore-database.sh)
- [scripts/health-check-production.sh](scripts/health-check-production.sh)

**Checklist déploiement:**
```bash
#!/bin/bash
# deploy-production.sh

# 1. Vérifier environnement
check_environment_production || exit 1

# 2. Backup database
backup_database || exit 1

# 3. Pull latest code
git pull origin main || exit 1

# 4. Build release
cargo build --release --features mainnet || exit 1

# 5. Run migrations
diesel migration run || exit 1

# 6. Restart services (zero-downtime)
systemctl restart monero-marketplace || exit 1

# 7. Health check
wait_for_healthy || rollback

# 8. Cleanup
cleanup_old_versions
```

**Critères de succès:**
- ✅ Déploiement scriptable en une commande
- ✅ Rollback automatique si health check échoue
- ✅ Backups automatiques avant chaque déploiement

#### Action 4.3: Documentation Production (1 jour)
**Fichiers à créer:**
- [docs/PRODUCTION-DEPLOYMENT-GUIDE.md](docs/PRODUCTION-DEPLOYMENT-GUIDE.md)
- [docs/INCIDENT-RESPONSE-PLAYBOOK.md](docs/INCIDENT-RESPONSE-PLAYBOOK.md)
- [docs/RUNBOOK.md](docs/RUNBOOK.md)

**Contenu RUNBOOK:**
- Comment vérifier health du système
- Comment investiguer un escrow bloqué
- Comment redémarrer Monero RPC sans perdre de données
- Comment gérer une panne Tor
- Procédure d'urgence si fonds bloqués

**Critères de succès:**
- ✅ N'importe quel développeur peut déployer en suivant le guide
- ✅ Playbook incident response couvre les scénarios critiques
- ✅ Documentation à jour avec le code

---

## 🛡️ Quality Gates (Automatisation)

### Gate 1: Pre-commit (Local)
```bash
# Exécuté automatiquement avant chaque commit
1. cargo fmt --check           # Code formaté
2. cargo clippy -- -D warnings # Pas de warnings
3. cargo test --workspace      # Tous les tests passent
4. ./scripts/check-security-theatre.sh  # 0 violations
5. gitleaks detect             # Pas de secrets
6. cargo audit                 # Pas de vulns connues
```
**SI ÉCHEC → Commit bloqué**

### Gate 2: CI/CD (GitHub Actions / GitLab CI)
```yaml
# .github/workflows/ci.yml
stages:
  - lint
  - test
  - integration_test
  - security_scan

lint:
  - cargo fmt --check
  - cargo clippy --all-features -- -D warnings

test:
  - cargo test --workspace --all-features

integration_test:
  - docker-compose -f docker-compose.test.yml up -d
  - cargo test --test production_integration_tests

security_scan:
  - cargo audit
  - cargo deny check
  - semgrep --config=auto
```
**SI ÉCHEC → Merge bloqué**

### Gate 3: Pre-deployment (Production)
```bash
# Exécuté avant déploiement production
1. Vérifier tous les tests CI/CD sont ✅
2. Vérifier aucun TODO dans server/src/**/*.rs
3. Vérifier cargo audit clean
4. Vérifier backup database récent (<24h)
5. Vérifier monitoring opérationnel
6. Vérifier encryption key accessible
```
**SI ÉCHEC → Déploiement bloqué**

---

## 📋 Checklist Production-Ready (À valider avant mainnet)

### Code Quality ✅
- [ ] 0 TODO/FIXME dans code production (server/src/)
- [ ] 0 `.unwrap()` ou `.expect()` sans justification
- [ ] Code coverage >85%
- [ ] Tous les tests passent (unit + integration + E2E)
- [ ] 0 warnings clippy
- [ ] 0 security theatre violations

### Sécurité ✅
- [ ] Audit externe complété (tous criticals fixés)
- [ ] Penetration testing passé
- [ ] Bug bounty actif 4+ semaines
- [ ] Encryption key gérée de manière sécurisée (KMS ou fichier 600)
- [ ] Rate limiting sur tous les endpoints
- [ ] CSRF protection activée
- [ ] Session management sécurisé

### Infrastructure ✅
- [ ] Hidden service .onion opérationnel
- [ ] Monero daemon syncé (mainnet)
- [ ] 3 wallets RPC fonctionnels (buyer, vendor, arbiter)
- [ ] Database backups automatiques (quotidiens)
- [ ] Monitoring opérationnel (Prometheus + Grafana)
- [ ] Alertes configurées et testées
- [ ] Logs centralisés et sécurisés (pas de données sensibles)

### Opérations ✅
- [ ] Runbook documenté et testé
- [ ] Incident response playbook prêt
- [ ] Équipe on-call définie (24/7 pendant 2 semaines post-launch)
- [ ] Procédure rollback testée
- [ ] Procédure backup/restore testée

### Légal & Compliance ✅
- [ ] Terms of Service rédigés
- [ ] Privacy Policy conforme
- [ ] Content moderation policy définie
- [ ] DMCA procedure (si applicable)
- [ ] Warrant canary mis en place

---

## 🎯 Timeline Réaliste

| Phase | Durée | Description | Blockers |
|-------|-------|-------------|----------|
| **Phase 1** | 1-2 semaines | WalletManager + WebSocket + Key mgmt | Accès Monero testnet |
| **Phase 2** | 1 semaine | Config multi-env + feature flags | Aucun |
| **Phase 3** | 1 semaine | Tests integration + monitoring | Aucun |
| **Phase 4** | 1 semaine | Deployment + docs | VPS/infra |
| **Audit** | 4-8 semaines | Audit externe professionnel | Budget $50k-$150k |
| **Beta** | 4-6 semaines | Beta testnet avec users réels | 50+ beta testers |

**Total avant production mainnet: 12-18 semaines (~3-4 mois)**

---

## 💡 Principes à Respecter Dès Maintenant

1. **Code = Production Code**
   - Pas de branches "dev" vs "prod"
   - Configuration via env vars, pas via code séparé

2. **Feature Flags > Branches**
   - Disable features incomplètes via flags
   - Ne jamais merger du code non-production dans main

3. **Tests Réels > Mocks**
   - Tests d'intégration contre Monero testnet
   - Tests WebSocket contre vrai serveur
   - Tests database contre vraie DB

4. **Fail Fast**
   - Validation stricte au démarrage (config, keys, DB)
   - Crash si environnement invalide
   - Pas de mode "dégradé" silencieux

5. **Observable dès Jour 1**
   - Logs structurés partout
   - Métriques sur toutes les opérations critiques
   - Alertes configurées même en dev

---

## 🚦 Feux Verts pour Lancer en Production

### ✅ GO si:
- Tous les checks de la checklist validés
- Audit externe: 0 criticals, <5 highs
- Beta testnet: 50+ users, 100+ escrows, 0 incidents fonds perdus
- Équipe disponible 24/7 (2 semaines)
- Backups testés et fonctionnels
- Monitoring et alertes opérationnels

### 🛑 NO-GO si:
- Un seul critical non-résolu
- Code contient TODOs dans server/src/
- Tests d'intégration échouent
- Monero RPC instable
- Pas d'équipe on-call disponible
- Documentation incomplète

---

**Ce plan transforme IMMÉDIATEMENT le développement en mode production-first. Fini les allers-retours test/production. Un seul code, production-ready dès le premier jour.**
