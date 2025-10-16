# Plan Complet - Monero Marketplace Tor v2.0
## De l'Alpha à la Production Mainnet

**Version:** 2.0
**Date de Création:** 2025-10-16
**Dernière Mise à Jour:** 2025-10-16
**Statut:** 📋 Planification Approuvée

---

## 📑 Table des Matières

0. [Setup Ubuntu/WSL](#setup-ubuntuwsl) ⚡ **NOUVEAU**
1. [État Actuel du Projet](#état-actuel-du-projet)
2. [Vision & Objectifs](#vision--objectifs)
3. [Roadmap Complète (7 Phases)](#roadmap-complète-7-phases)
4. [Phase 1: Multisig Core (Semaines 1-6)](#phase-1-multisig-core)
5. [Phase 2: Backend Web Service (Semaines 7-14)](#phase-2-backend-web-service)
6. [Phase 3: Escrow & Transactions (Semaines 15-20)](#phase-3-escrow--transactions)
7. [Phase 4: Frontend & UX (Semaines 21-28)](#phase-4-frontend--ux)
8. [Phase 5: Sécurité & Audit (Semaines 29-40)](#phase-5-sécurité--audit)
9. [Phase 6: Production Testnet (Semaines 41-46)](#phase-6-production-testnet)
10. [Phase 7: Mainnet Launch (Semaine 47+)](#phase-7-mainnet-launch)
11. [Architecture Technique](#architecture-technique)
12. [Stack Technologique](#stack-technologique)
13. [Sécurité & OPSEC](#sécurité--opsec)
14. [Budget & Ressources](#budget--ressources)
15. [Risques & Mitigations](#risques--mitigations)
16. [Métriques de Succès](#métriques-de-succès)
17. [Actions Immédiates](#actions-immédiates)

---

## Setup Ubuntu/WSL

### 🎯 Environnement Actuel: WSL Ubuntu

Vous êtes déjà sur **WSL (Windows Subsystem for Linux)** à `/mnt/c/Users/Lenovo/monero-marketplace$`

### ⚡ Quick Start (5 minutes)

```bash
# 1. Vérifier l'environnement
./scripts/check-environment.sh

# 2. Setup automatique complet (si besoin)
chmod +x scripts/*.sh
./scripts/ubuntu-setup.sh

# 3. Vérifier Tor
sudo service tor status
# ou (selon WSL version)
systemctl status tor

# 4. Setup Monero testnet
./scripts/setup-monero-testnet.sh

# 5. Build du projet
cargo build --workspace

# 6. Tests
cargo test --workspace
```

### 📚 Documentation Complète

- **[UBUNTU-SETUP.md](UBUNTU-SETUP.md)** - Guide installation détaillé
- **[MIGRATION-UBUNTU.md](MIGRATION-UBUNTU.md)** - Migration Windows → Ubuntu
- **[CLAUDE.md](CLAUDE.md)** - Instructions développement (màj Ubuntu)
- **[scripts/README.md](scripts/README.md)** - Documentation scripts Bash

### 🔧 Commandes Essentielles

```bash
# Build & Test
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --workspace

# Pre-commit
./scripts/pre-commit.sh

# Security checks
./scripts/check-security-theatre.sh
./scripts/check-environment.sh

# Monero RPC
./scripts/test-rpc.sh
```

### ⚠️ Notes WSL

**Tor:** Sur WSL1, utiliser `service` au lieu de `systemctl`:
```bash
# Démarrer Tor
sudo service tor start

# Status
sudo service tor status
```

**Permissions:** Rendre les scripts exécutables:
```bash
chmod +x scripts/*.sh
chmod +x .git/hooks/pre-commit
```

---

## État Actuel du Projet

### 📊 Snapshot (2025-10-16)

| Métrique | Valeur |
|----------|--------|
| **Version** | 0.1.0-alpha |
| **Score Sécurité** | 80/100 |
| **Statut** | 🟡 Développement actif - Testnet uniquement |
| **Lines of Code** | ~1034 |
| **Tests** | 4/4 passing ✅ |
| **Code Coverage** | ~60% |
| **Security Theatre Violations** | 0 ✅ |
| **Reality Checks Validés** | 6 |

### ✅ Composants Complétés

**Architecture de Base:**
- [x] Workspace Cargo avec 3 crates (common, wallet, cli)
- [x] Structure de projet documentée
- [x] CI/CD pipeline basique (pre-commit hooks)
- [x] Security theatre detection automatique

**Monero Integration:**
- [x] Client RPC Monero (`MoneroRpcClient`)
- [x] Isolation localhost stricte (127.0.0.1 only)
- [x] Rate limiting (max 5 concurrent RPC calls)
- [x] Retry logic avec exponential backoff
- [x] Type-safe RPC calls

**Multisig Workflow:**
- [x] `prepare_multisig()` - Étape 1/6
- [x] `make_multisig()` - Étape 2/6
- [x] `export_multisig_info()` - Étape 3/6
- [x] `import_multisig_info()` - Étape 4/6
- [x] `is_multisig()` - Vérification état
- [x] `sync_multisig_round()` - Helper pour sync

**CLI Interface:**
- [x] `monero-marketplace` CLI avec clap
- [x] Commandes: status, info, multisig (prepare, make, export, import, check)
- [x] Intégration complète avec wallet crate

**Documentation:**
- [x] 34+ fichiers de documentation
- [x] 12 specs techniques
- [x] 6 Reality Checks Tor validés
- [x] CLAUDE.md (instructions pour développement)
- [x] DEVELOPER-GUIDE.md
- [x] SECURITY-THEATRE-PREVENTION.md

**Testing:**
- [x] Tests unitaires (wallet, common)
- [x] Tests d'intégration (wallet/tests/integration.rs)
- [x] Reality Checks automatiques

### 🚧 Lacunes Critiques

**Multisig (À Compléter):**
- [ ] Tests end-to-end avec 3 wallets simultanés
- [ ] Création de transactions multisig
- [ ] Signature de transactions (collect 2-of-3 signatures)
- [ ] Finalization & broadcast de transactions
- [ ] Monitoring des confirmations blockchain
- [ ] Edge cases handling (timeouts, errors, retries)

**Infrastructure (Manquante):**
- [ ] Backend web service (API REST)
- [ ] Hidden service .onion (Tor v3)
- [ ] Database (PostgreSQL ou SQLite chiffré)
- [ ] WebSocket pour notifications temps réel
- [ ] Frontend web interface
- [ ] Authentication & session management

**Sécurité (Non Auditée):**
- [ ] Audit de sécurité externe
- [ ] Penetration testing
- [ ] Bug bounty programme
- [ ] Incident response plan
- [ ] Production monitoring & alerting

---

## Vision & Objectifs

### 🎯 Vision du Produit

**Monero Marketplace** est un marketplace décentralisé et anonyme permettant des transactions sécurisées entre acheteurs et vendeurs via un système d'escrow 2-of-3 multisig basé sur Monero, accessible uniquement via Tor.

**Principes Fondamentaux:**
1. **Privacy by Default** - Aucun tracking, aucun KYC
2. **Security First** - Sécurité prioritaire sur features
3. **Trustless Escrow** - Multisig 2-of-3 avec arbitre neutre
4. **Open Source** - Code auditable publiquement
5. **Tor Only** - Pas d'accès clearnet

### 🎪 Cas d'Usage Cible

**Marketplace Légal pour:**
- Produits digitaux (ebooks, software, art)
- Services freelance (développement, design, consulting)
- Biens physiques (art, artisanat, collectibles)

**Protection pour:**
- Acheteurs contre scams vendors
- Vendors contre chargebacks frauduleux
- Les deux via arbitre neutre

### 🚫 Hors Scope (Interdits)

- Drogues illégales
- Armes
- Données volées
- Contenu illégal
- Services illicites

**Note:** Terms of Service strictes avec modération réactive.

---

## Roadmap Complète (7 Phases)

### Timeline Visuel

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     MONERO MARKETPLACE ROADMAP                          │
│                         8-11 Mois (32-46 Semaines)                      │
└─────────────────────────────────────────────────────────────────────────┘

Mois 1-2 │ ████████████ │ Phase 1: Multisig Core (6 sem)
         │              │ ├─ Tests e2e 3 wallets
         │              │ ├─ Transactions multisig
         │              │ └─ Edge cases handling
         │
Mois 3-4 │ ████████████████████ │ Phase 2: Backend API (8 sem)
         │                      │ ├─ Hidden service .onion
         │                      │ ├─ API REST (listings, orders, escrow)
         │                      │ ├─ Database (PostgreSQL/SQLite)
         │                      │ └─ WebSocket notifications
         │
Mois 5-6 │ ████████████ │ Phase 3: Escrow Flow (6 sem)
         │              │ ├─ Escrow initialization
         │              │ ├─ Release & dispute resolution
         │              │ └─ Blockchain monitoring
         │
Mois 6-8 │ ████████████████████ │ Phase 4: Frontend (8 sem)
         │                      │ ├─ UI/UX design
         │                      │ ├─ Pages (marketplace, checkout, orders)
         │                      │ └─ OPSEC hardening
         │
Mois 8-11│ ████████████████████████████ │ Phase 5: Security (12 sem)
         │                              │ ├─ Internal security review
         │                              │ ├─ External audit ($50k-$150k)
         │                              │ └─ Bug bounty programme
         │
Mois 11  │ ██████ │ Phase 6: Testnet Beta (6 sem)
         │        │ ├─ Infrastructure setup
         │        │ ├─ Beta testing (50+ users)
         │        │ └─ Bug fixes & iteration
         │
Mois 12+ │ ██ │ Phase 7: Mainnet Launch
         │    │ └─ Production deployment (si audit OK)

```

### 📅 Calendrier Détaillé

| Phase | Durée | Dates Estimées | Statut |
|-------|-------|----------------|--------|
| **Phase 1** | 6 semaines | 2025-10-16 → 2025-11-27 | 🚀 Next |
| **Phase 2** | 8 semaines | 2025-11-28 → 2026-01-22 | ⏳ Planifié |
| **Phase 3** | 6 semaines | 2026-01-23 → 2026-03-05 | ⏳ Planifié |
| **Phase 4** | 8 semaines | 2026-03-06 → 2026-04-30 | ⏳ Planifié |
| **Phase 5** | 12 semaines | 2026-05-01 → 2026-07-23 | ⏳ Planifié |
| **Phase 6** | 6 semaines | 2026-07-24 → 2026-09-03 | ⏳ Planifié |
| **Phase 7** | Variable | 2026-09-04 → TBD | ⏳ Conditionnel |

**Total:** 46 semaines (~11 mois)

---

## Phase 1: Multisig Core

**Durée:** 6 semaines (Semaines 1-6)
**Priorité:** 🔴 CRITIQUE
**Objectif:** Workflow multisig 2-of-3 entièrement fonctionnel et testé

### 🎯 Success Criteria

- ✅ 3 wallets testnet créent multisig 2-of-3 sans erreur
- ✅ Transactions créées, signées (2-of-3), finalisées et diffusées
- ✅ Code coverage >80% pour `wallet/`
- ✅ Zero `.unwrap()` ou `panic!` possibles
- ✅ Tous les Reality Checks Tor validés
- ✅ Tests automatisés passent end-to-end

### 📋 Milestones

#### Milestone 1.1: Tests End-to-End (Semaine 1-2)

**Délivrables:**

**Task 1.1.1: Setup 3 Wallets Testnet (2 jours)**
```bash
# Créer script automatique
scripts/setup-3-wallets-testnet.sh

# Fonctionnalités:
- Vérifier Monero daemon testnet running
- Créer 3 wallets (buyer, vendor, arbiter)
- Bind sur ports différents (18082, 18083, 18084)
- Démarrer les 3 RPC simultanément
- Health checks automatiques
```

**Task 1.1.2: Test E2E Multisig Setup (3 jours)**
```rust
// wallet/tests/multisig_e2e.rs
#[tokio::test]
async fn test_full_multisig_2of3_setup() -> Result<()> {
    // 1. Setup 3 clients
    // 2. Prepare multisig (parallel)
    // 3. Make multisig (collect infos)
    // 4. Sync round 1 (export → import)
    // 5. Sync round 2 (export → import)
    // 6. Verify all is_multisig() == true
    // 7. Assert same multisig_address
}
```

**Task 1.1.3: Diagramme de Séquence (1 jour)**
- Créer `docs/MULTISIG-SEQUENCE-DIAGRAM.md`
- Mermaid diagram du flow complet
- Documentation de chaque étape

**Temps Total:** 6 jours (Semaine 1-2)

---

#### Milestone 1.2: Transactions Multisig (Semaine 3-4)

**Délivrables:**

**Task 1.2.1: `create_transaction()` (2 jours)**
```rust
/// Create an unsigned multisig transaction
pub async fn create_transaction(
    &self,
    destinations: Vec<(String, u64)>, // (address, amount_atomic)
    priority: u32,
) -> Result<UnsignedTransaction>
```

**Spec:** `docs/specs/create_transaction.md`
**Tests:** Créer TX avec montant valide, invalide, insufficient funds
**Reality Check:** `docs/reality-checks/tor-create_transaction-2025-10-XX.md`

---

**Task 1.2.2: `sign_multisig_transaction()` (2 jours)**
```rust
/// Sign a multisig transaction with this wallet's key
pub async fn sign_multisig_transaction(
    &self,
    tx_metadata: String,
) -> Result<SignedTransaction>
```

**Spec:** `docs/specs/sign_multisig_transaction.md`
**Tests:** Sign valid TX, invalid TX, already signed
**Reality Check:** Validé

---

**Task 1.2.3: `finalize_multisig_transaction()` (2 jours)**
```rust
/// Finalize with collected signatures (2-of-3)
pub async fn finalize_multisig_transaction(
    &self,
    signatures: Vec<String>,
) -> Result<FinalizedTransaction>
```

**Spec:** `docs/specs/finalize_multisig_transaction.md`
**Tests:** Finalize with 2 sigs, 1 sig (error), 3 sigs (ok)

---

**Task 1.2.4: `broadcast_transaction()` (1 jour)**
```rust
/// Broadcast to Monero network
pub async fn broadcast_transaction(
    &self,
    tx_hex: String,
) -> Result<String> // tx_hash
```

**Spec:** `docs/specs/broadcast_transaction.md`
**Tests:** Broadcast valid TX, invalid TX

---

**Task 1.2.5: Test E2E Transaction (3 jours)**
```rust
#[tokio::test]
async fn test_full_multisig_transaction_2of3() -> Result<()> {
    // 1. Setup 3 multisig wallets (from 1.1.2)
    // 2. Fund multisig address (testnet faucet)
    // 3. Wait for confirmations (10 blocks)
    // 4. Buyer creates transaction (release to vendor)
    // 5. Buyer signs (signer 1/2)
    // 6. Vendor signs (signer 2/2)
    // 7. Finalize with 2 signatures
    // 8. Broadcast
    // 9. Wait for confirmations
    // 10. Verify vendor received funds
}
```

**Temps Total:** 10 jours (Semaine 3-4)

---

#### Milestone 1.3: Edge Cases & Polish (Semaine 5-6)

**Task 1.3.1: Tests Edge Cases (3 jours)**
```rust
// wallet/tests/multisig_edge_cases.rs
- test_already_multisig_error()
- test_invalid_multisig_info()
- test_insufficient_funds()
- test_timeout_handling()
- test_double_spend_prevention()
- test_invalid_signatures()
- test_wrong_threshold()
```

**Task 1.3.2: Documentation (2 jours)**
- `docs/ERROR-CODES.md` - Tous les codes d'erreur
- Update `README.md` avec instructions Phase 1
- Update `CHANGELOG.md` avec v0.2.0

**Task 1.3.3: Code Review & Refactoring (2 jours)**
- Code review complet
- Refactoring si nécessaire
- Performance optimization
- Memory leak check

**Task 1.3.4: Final Validation (3 jours)**
- Run full test suite (cargo test --workspace)
- Code coverage report (>80%)
- Clippy strict mode (cargo clippy -- -D warnings)
- Security theatre scan (all pass)
- All Reality Checks validated

**Temps Total:** 10 jours (Semaine 5-6)

---

### 📦 Délivrables Phase 1

- [x] 18+ tests automatisés passing
- [x] Code coverage >80%
- [x] 6+ nouvelles specs techniques
- [x] 4+ Reality Checks Tor validés
- [x] Documentation ERROR-CODES.md
- [x] Updated README.md & CHANGELOG.md
- [x] Demo video (optional)

---

## Phase 2: Backend Web Service

**Durée:** 8 semaines (Semaines 7-14)
**Priorité:** 🟠 HAUTE
**Objectif:** Hidden service .onion avec API REST fonctionnelle

### 🎯 Success Criteria

- ✅ Hidden service .onion accessible via Tor
- ✅ API REST complète (listings, orders, escrow)
- ✅ Database avec encryption at-rest
- ✅ Authentication & authorization fonctionnels
- ✅ WebSocket notifications temps réel
- ✅ Rate limiting & DDoS protection

### 📋 Milestones

#### Milestone 2.1: Tor Hidden Service (Semaine 7-8)

**Délivrables:**

**Task 2.1.1: Nouveau Crate `server/` (1 jour)**
```toml
# Cargo.toml
[workspace]
members = ["common", "wallet", "cli", "server"]

# server/Cargo.toml
[dependencies]
actix-web = "4.4"
actix-session = "0.9"
actix-web-actors = "4.3"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
# ...
```

**Task 2.1.2: Configuration Tor (1 jour)**
```bash
# /etc/tor/torrc
HiddenServiceDir /var/lib/tor/marketplace/
HiddenServicePort 80 127.0.0.1:8080
HiddenServiceVersion 3
```

**Task 2.1.3: Basic Actix-web Server (2 jours)**
```rust
// server/src/main.rs
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/health", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

**Task 2.1.4: Health Check & Testing (1 jour)**
```bash
# Test 1: Direct access
curl http://127.0.0.1:8080/api/health
# Expected: {"status": "ok"}

# Test 2: Via Tor
curl --socks5-hostname 127.0.0.1:9050 http://your-onion-address.onion/api/health
# Or: torify curl http://your-onion-address.onion/api/health
```

**Task 2.1.5: Reality Check Tor (2 jours)**
- Vérifier isolation réseau
- Pas de fuites IP
- Hidden service accessible
- RPC localhost only

**Temps Total:** 7 jours (Semaine 7)

---

#### Milestone 2.2: API REST Core (Semaine 9-11)

**Architecture API:**
```
/api/v1/
├── /auth
│   ├── POST /register
│   ├── POST /login
│   ├── GET /whoami
│   └── POST /logout
├── /listings
│   ├── GET /listings (public)
│   ├── GET /listings/:id
│   ├── POST /listings (vendor only)
│   ├── PUT /listings/:id (vendor only)
│   └── DELETE /listings/:id (vendor only)
├── /orders
│   ├── POST /orders (buyer)
│   ├── GET /orders/:id
│   ├── GET /orders/user/:user_id
│   └── PUT /orders/:id/status
├── /escrow
│   ├── POST /escrow/init
│   ├── POST /escrow/:id/prepare
│   ├── POST /escrow/:id/make
│   ├── POST /escrow/:id/sync
│   ├── GET /escrow/:id/status
│   ├── POST /escrow/:id/release
│   └── POST /escrow/:id/dispute
└── /users
    ├── GET /users/me
    └── PUT /users/me
```

**Task 2.2.1: Authentication Endpoints (3 jours)**
```rust
// server/src/handlers/auth.rs

#[derive(Deserialize, Validate)]
struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    username: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
    #[validate(custom = "validate_role")]
    role: UserRole, // buyer, vendor, arbiter
}

async fn register(
    req: web::Json<RegisterRequest>,
    db: web::Data<DbPool>,
) -> Result<HttpResponse> {
    req.validate()?;

    // Hash password (Argon2id)
    let password_hash = hash_password(&req.password)?;

    // Create user in DB
    let user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        password_hash,
        role: req.role,
        created_at: Utc::now(),
    };

    db_insert_user(&db, &user).await?;

    Ok(HttpResponse::Created().json(UserResponse::from(user)))
}
```

**Task 2.2.2: Listings Endpoints (4 jours)**
- GET /listings - Pagination, filters, search
- POST /listings - Vendor creates listing
- PUT /listings/:id - Vendor updates listing
- DELETE /listings/:id - Soft delete

**Task 2.2.3: Orders Endpoints (3 jours)**
- POST /orders - Buyer creates order
- GET /orders/:id - View order details
- Authorization checks (owner only)

**Task 2.2.4: Middleware (2 jours)**
```rust
// Rate limiting
use actix_governor::{Governor, GovernorConfigBuilder};

// CSRF protection
use actix_web_csrf::{CsrfGuard, CsrfMiddleware};

// Session management
use actix_session::{SessionMiddleware, storage::CookieSessionStore};

// Logging (sans données sensibles)
use tracing_actix_web::TracingLogger;
```

**Temps Total:** 12 jours (Semaine 9-11)

---

#### Milestone 2.3: Database (Semaine 12-14)

**Task 2.3.1: Schema Design (2 jours)**
```sql
-- database/schema.sql

CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('buyer', 'vendor', 'arbiter', 'admin')),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE listings (
    id UUID PRIMARY KEY,
    vendor_id UUID REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    price_xmr BIGINT NOT NULL CHECK (price_xmr > 0),
    stock INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE orders (
    id UUID PRIMARY KEY,
    buyer_id UUID REFERENCES users(id) ON DELETE SET NULL,
    vendor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    listing_id UUID REFERENCES listings(id) ON DELETE SET NULL,
    escrow_id UUID UNIQUE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    total_xmr BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE escrows (
    id UUID PRIMARY KEY,
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    buyer_wallet_info TEXT, -- ENCRYPTED
    vendor_wallet_info TEXT, -- ENCRYPTED
    arbiter_wallet_info TEXT, -- ENCRYPTED
    multisig_address VARCHAR(95),
    status VARCHAR(50) NOT NULL DEFAULT 'init',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    escrow_id UUID REFERENCES escrows(id) ON DELETE CASCADE,
    tx_hash VARCHAR(64) UNIQUE,
    amount_xmr BIGINT NOT NULL,
    confirmations INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_listings_vendor ON listings(vendor_id);
CREATE INDEX idx_orders_buyer ON orders(buyer_id);
CREATE INDEX idx_orders_vendor ON orders(vendor_id);
CREATE INDEX idx_escrows_order ON escrows(order_id);
CREATE INDEX idx_transactions_escrow ON transactions(escrow_id);
```

**Task 2.3.2: Diesel Setup & Migrations (2 jours)**
```rust
// server/src/db/mod.rs
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn create_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(10)
        .build(manager)?;
    Ok(pool)
}
```

**Task 2.3.3: Models & Queries (3 jours)**
```rust
// server/src/models/user.rs
#[derive(Queryable, Insertable, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// CRUD operations
impl User {
    pub async fn create(db: &DbPool, user: NewUser) -> Result<User>;
    pub async fn find_by_id(db: &DbPool, id: Uuid) -> Result<User>;
    pub async fn find_by_username(db: &DbPool, username: &str) -> Result<User>;
    pub async fn update(db: &DbPool, id: Uuid, update: UpdateUser) -> Result<User>;
    pub async fn delete(db: &DbPool, id: Uuid) -> Result<()>;
}
```

**Task 2.3.4: Encryption (2 jours)**
```rust
// server/src/crypto/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};

/// Encrypt sensitive field before storing in DB
pub fn encrypt_field(plaintext: &str, key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&generate_nonce());
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
    Ok(ciphertext)
}

/// Decrypt when reading from DB
pub fn decrypt_field(ciphertext: &[u8], key: &[u8]) -> Result<String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&extract_nonce(ciphertext));
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(String::from_utf8(plaintext)?)
}
```

**Task 2.3.5: Integration Tests (2 jours)**
- Test CRUD operations
- Test encryption/decryption
- Test foreign key constraints
- Test indexes performance

**Temps Total:** 11 jours (Semaine 12-14)

---

### 📦 Délivrables Phase 2

- [x] Hidden service .onion fonctionnel
- [x] API REST complète (20+ endpoints)
- [x] Database avec schema complet
- [x] Encryption at-rest pour données sensibles
- [x] Authentication & sessions
- [x] Rate limiting middleware
- [x] 30+ tests API
- [x] OpenAPI documentation (swagger)

---

## Phase 3: Escrow & Transactions

**Durée:** 6 semaines (Semaines 15-20)
**Priorité:** 🟠 HAUTE
**Objectif:** Flow escrow complet de A à Z

### 🎯 Success Criteria

- ✅ Buyer peut créer order → escrow initialisé
- ✅ 3 parties (buyer, vendor, arbiter) setup multisig automatiquement
- ✅ Buyer dépose funds → multisig address
- ✅ Release normal (buyer + vendor signs)
- ✅ Dispute resolution (arbiter décide)
- ✅ Monitoring blockchain en temps réel
- ✅ Notifications WebSocket pour tous les événements

### 📋 Milestones

#### Milestone 3.1: Escrow Initialization (Semaine 15-16)

**Flow Orchestration:**
```
1. Buyer crée order → POST /api/orders
   └─> Backend crée escrow (status: init)
   └─> Backend assigne arbiter (round-robin)

2. Backend notifie via WebSocket:
   - Buyer: "Prepare votre wallet pour multisig"
   - Vendor: "Nouvelle order reçue, préparez multisig"
   - Arbiter: "Vous êtes assigné à escrow {id}"

3. Chaque partie appelle:
   POST /api/escrow/:id/prepare
   └─> Backend orchestre prepare_multisig()
   └─> Backend stocke multisig_info (encrypted)

4. Quand 3 infos reçues:
   POST /api/escrow/:id/make (automatique)
   └─> Backend appelle make_multisig() pour chaque partie
   └─> Backend stocke multisig_address

5. Sync rounds (automatique):
   POST /api/escrow/:id/sync (round 1)
   POST /api/escrow/:id/sync (round 2)
   └─> Backend orchestre export/import pour les 3

6. Escrow status → "ready"
   └─> WebSocket notification: "Déposez funds à {address}"
```

**Task 3.1.1: Escrow Orchestration Service (4 jours)**
```rust
// server/src/services/escrow.rs

pub struct EscrowOrchestrator {
    wallet_manager: Arc<WalletManager>,
    db: DbPool,
    websocket: Arc<WebSocketServer>,
}

impl EscrowOrchestrator {
    /// Initialize new escrow (step 1)
    pub async fn init_escrow(
        &self,
        order_id: Uuid,
        buyer_id: Uuid,
        vendor_id: Uuid,
    ) -> Result<Escrow> {
        // 1. Assign arbiter (round-robin from available arbiters)
        let arbiter_id = self.assign_arbiter().await?;

        // 2. Create escrow in DB
        let escrow = Escrow {
            id: Uuid::new_v4(),
            order_id,
            status: EscrowStatus::Init,
            created_at: Utc::now(),
        };

        db_insert_escrow(&self.db, &escrow).await?;

        // 3. Notify parties via WebSocket
        self.websocket.notify(buyer_id, WsEvent::EscrowInit { escrow_id: escrow.id }).await?;
        self.websocket.notify(vendor_id, WsEvent::EscrowInit { escrow_id: escrow.id }).await?;
        self.websocket.notify(arbiter_id, WsEvent::EscrowAssigned { escrow_id: escrow.id }).await?;

        Ok(escrow)
    }

    /// Collect prepare_multisig from party (step 2)
    pub async fn collect_prepare_info(
        &self,
        escrow_id: Uuid,
        user_id: Uuid,
        multisig_info: String,
    ) -> Result<()> {
        // Validate & encrypt
        validate_multisig_info(&multisig_info)?;
        let encrypted = encrypt_field(&multisig_info, &self.encryption_key)?;

        // Store in DB
        db_store_multisig_info(&self.db, escrow_id, user_id, encrypted).await?;

        // Check if all 3 received
        let count = db_count_multisig_infos(&self.db, escrow_id).await?;
        if count == 3 {
            // Trigger make_multisig automatically
            self.make_multisig(escrow_id).await?;
        }

        Ok(())
    }

    /// Make multisig for all 3 parties (step 3)
    async fn make_multisig(&self, escrow_id: Uuid) -> Result<()> {
        // Load 3 multisig_infos
        let infos = db_load_multisig_infos(&self.db, escrow_id).await?;

        // Call make_multisig for each party (parallel)
        let (buyer_result, vendor_result, arbiter_result) = tokio::try_join!(
            self.wallet_manager.make_multisig(buyer_wallet, 2, vec![vendor_info, arbiter_info]),
            self.wallet_manager.make_multisig(vendor_wallet, 2, vec![buyer_info, arbiter_info]),
            self.wallet_manager.make_multisig(arbiter_wallet, 2, vec![buyer_info, vendor_info]),
        )?;

        // Verify same address
        assert_eq!(buyer_result.address, vendor_result.address);
        assert_eq!(buyer_result.address, arbiter_result.address);

        // Store multisig address
        db_update_escrow_address(&self.db, escrow_id, &buyer_result.address).await?;

        // Update status
        db_update_escrow_status(&self.db, escrow_id, EscrowStatus::Syncing).await?;

        // Trigger sync rounds
        self.sync_round_1(escrow_id).await?;

        Ok(())
    }

    /// Sync round 1 (step 4a)
    async fn sync_round_1(&self, escrow_id: Uuid) -> Result<()> {
        // Export from all 3
        let exports = self.export_from_all_parties(escrow_id).await?;

        // Import to all 3 (each imports 2 others)
        self.import_to_all_parties(escrow_id, exports).await?;

        // Trigger round 2
        self.sync_round_2(escrow_id).await?;

        Ok(())
    }

    /// Sync round 2 (step 4b)
    async fn sync_round_2(&self, escrow_id: Uuid) -> Result<()> {
        // Export from all 3 again
        let exports = self.export_from_all_parties(escrow_id).await?;

        // Import to all 3
        self.import_to_all_parties(escrow_id, exports).await?;

        // Verify all is_multisig()
        let all_ready = self.verify_all_multisig(escrow_id).await?;
        if !all_ready {
            return Err(Error::Escrow("Sync failed".to_string()));
        }

        // Update status
        db_update_escrow_status(&self.db, escrow_id, EscrowStatus::Ready).await?;

        // Notify parties
        let escrow = db_load_escrow(&self.db, escrow_id).await?;
        self.websocket.notify_escrow_ready(escrow).await?;

        Ok(())
    }
}
```

**Task 3.1.2: WebSocket Server (3 jours)**
```rust
// server/src/websocket/mod.rs

use actix::{Actor, StreamHandler, Addr};
use actix_web_actors::ws;
use std::collections::HashMap;

pub struct WebSocketServer {
    sessions: Arc<Mutex<HashMap<Uuid, Addr<WsSession>>>>,
}

impl WebSocketServer {
    /// Send event to specific user
    pub async fn notify(&self, user_id: Uuid, event: WsEvent) -> Result<()> {
        let sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(&user_id) {
            session.do_send(SendMessage(serde_json::to_string(&event)?));
        }
        Ok(())
    }

    /// Broadcast to multiple users
    pub async fn broadcast(&self, user_ids: Vec<Uuid>, event: WsEvent) -> Result<()> {
        for user_id in user_ids {
            self.notify(user_id, event.clone()).await?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize)]
pub enum WsEvent {
    EscrowInit { escrow_id: Uuid },
    EscrowAssigned { escrow_id: Uuid },
    EscrowStatusChanged { escrow_id: Uuid, new_status: String },
    TransactionConfirmed { tx_hash: String, confirmations: u32 },
    NewMessage { from: Uuid, content: String },
    OrderStatusChanged { order_id: Uuid, new_status: String },
}
```

**Task 3.1.3: Tests Integration (3 jours)**
```rust
#[tokio::test]
async fn test_full_escrow_initialization() -> Result<()> {
    // Setup: Create buyer, vendor, arbiter accounts
    let (buyer, vendor, arbiter) = create_test_users().await?;

    // 1. Buyer creates order
    let order = create_order(buyer.id, listing.id).await?;

    // 2. Escrow initialized automatically
    let escrow = get_escrow_by_order(order.id).await?;
    assert_eq!(escrow.status, "init");

    // 3. Each party prepares multisig
    prepare_multisig_for_party(escrow.id, buyer.id).await?;
    prepare_multisig_for_party(escrow.id, vendor.id).await?;
    prepare_multisig_for_party(escrow.id, arbiter.id).await?;

    // 4. Wait for auto-orchestration
    wait_for_escrow_status(escrow.id, "ready", Duration::from_secs(60)).await?;

    // 5. Verify multisig address generated
    let escrow = get_escrow(escrow.id).await?;
    assert!(escrow.multisig_address.is_some());

    Ok(())
}
```

**Temps Total:** 10 jours (Semaine 15-16)

---

#### Milestone 3.2: Release & Dispute (Semaine 17-18)

**Task 3.2.1: Normal Release Flow (3 jours)**
```rust
// server/src/services/escrow.rs

impl EscrowOrchestrator {
    /// Release funds (buyer + vendor agree)
    pub async fn release_funds(
        &self,
        escrow_id: Uuid,
        requester_id: Uuid, // buyer or vendor
    ) -> Result<String> {
        let escrow = db_load_escrow(&self.db, escrow_id).await?;

        // 1. Verify requester is buyer or vendor
        if requester_id != escrow.buyer_id && requester_id != escrow.vendor_id {
            return Err(Error::Unauthorized);
        }

        // 2. Create transaction (release to vendor)
        let unsigned_tx = self.wallet_manager.create_transaction(
            vec![(escrow.vendor_address.clone(), escrow.amount_xmr)],
            0 // default priority
        ).await?;

        // 3. Collect signatures from buyer + vendor
        let buyer_sig = self.wallet_manager.sign_transaction(buyer_wallet, &unsigned_tx.tx_metadata).await?;
        let vendor_sig = self.wallet_manager.sign_transaction(vendor_wallet, &unsigned_tx.tx_metadata).await?;

        // 4. Finalize with 2-of-3 signatures
        let finalized_tx = self.wallet_manager.finalize_transaction(
            vec![buyer_sig.signature, vendor_sig.signature]
        ).await?;

        // 5. Broadcast
        let tx_hash = self.wallet_manager.broadcast_transaction(finalized_tx.tx_hex).await?;

        // 6. Store transaction in DB
        db_insert_transaction(&self.db, Transaction {
            id: Uuid::new_v4(),
            escrow_id,
            tx_hash: tx_hash.clone(),
            amount_xmr: escrow.amount_xmr,
            confirmations: 0,
            created_at: Utc::now(),
        }).await?;

        // 7. Update escrow status
        db_update_escrow_status(&self.db, escrow_id, EscrowStatus::Released).await?;

        // 8. Notify parties
        self.websocket.notify_release_completed(escrow).await?;

        Ok(tx_hash)
    }
}
```

**Task 3.2.2: Dispute Resolution Flow (4 jours)**
```rust
impl EscrowOrchestrator {
    /// Open dispute
    pub async fn open_dispute(
        &self,
        escrow_id: Uuid,
        requester_id: Uuid,
        reason: String,
    ) -> Result<Dispute> {
        // 1. Create dispute in DB
        let dispute = Dispute {
            id: Uuid::new_v4(),
            escrow_id,
            opened_by: requester_id,
            reason,
            status: DisputeStatus::Open,
            created_at: Utc::now(),
        };

        db_insert_dispute(&self.db, &dispute).await?;

        // 2. Update escrow status
        db_update_escrow_status(&self.db, escrow_id, EscrowStatus::Disputed).await?;

        // 3. Notify arbiter
        let escrow = db_load_escrow(&self.db, escrow_id).await?;
        self.websocket.notify(escrow.arbiter_id, WsEvent::DisputeOpened { dispute_id: dispute.id }).await?;

        Ok(dispute)
    }

    /// Arbiter resolves dispute
    pub async fn resolve_dispute(
        &self,
        dispute_id: Uuid,
        arbiter_id: Uuid,
        decision: DisputeDecision, // RefundBuyer or ReleaseTo Vendor
    ) -> Result<String> {
        let dispute = db_load_dispute(&self.db, dispute_id).await?;
        let escrow = db_load_escrow(&self.db, dispute.escrow_id).await?;

        // 1. Verify arbiter
        if arbiter_id != escrow.arbiter_id {
            return Err(Error::Unauthorized);
        }

        // 2. Create transaction according to decision
        let recipient_address = match decision {
            DisputeDecision::RefundBuyer => escrow.buyer_address.clone(),
            DisputeDecision::ReleaseVendor => escrow.vendor_address.clone(),
        };

        let unsigned_tx = self.wallet_manager.create_transaction(
            vec![(recipient_address, escrow.amount_xmr)],
            0
        ).await?;

        // 3. Collect signatures: arbiter + winner
        let arbiter_sig = self.wallet_manager.sign_transaction(arbiter_wallet, &unsigned_tx.tx_metadata).await?;

        let winner_sig = match decision {
            DisputeDecision::RefundBuyer => {
                self.wallet_manager.sign_transaction(buyer_wallet, &unsigned_tx.tx_metadata).await?
            },
            DisputeDecision::ReleaseVendor => {
                self.wallet_manager.sign_transaction(vendor_wallet, &unsigned_tx.tx_metadata).await?
            },
        };

        // 4. Finalize & broadcast
        let finalized_tx = self.wallet_manager.finalize_transaction(
            vec![arbiter_sig.signature, winner_sig.signature]
        ).await?;

        let tx_hash = self.wallet_manager.broadcast_transaction(finalized_tx.tx_hex).await?;

        // 5. Update dispute & escrow
        db_update_dispute_status(&self.db, dispute_id, DisputeStatus::Resolved).await?;
        db_update_escrow_status(&self.db, escrow.id, EscrowStatus::Resolved).await?;

        // 6. Notify all parties
        self.websocket.notify_dispute_resolved(escrow, decision).await?;

        Ok(tx_hash)
    }
}
```

**Task 3.2.3: Evidence Upload System (2 jours)**
```rust
// server/src/handlers/disputes.rs

/// Upload evidence for dispute
async fn upload_evidence(
    dispute_id: web::Path<Uuid>,
    user_id: Session, // from auth middleware
    files: Multipart,
) -> Result<HttpResponse> {
    // 1. Validate user is party to dispute
    let dispute = db_load_dispute(&db, *dispute_id).await?;
    let escrow = db_load_escrow(&db, dispute.escrow_id).await?;

    if user_id != escrow.buyer_id
        && user_id != escrow.vendor_id
        && user_id != escrow.arbiter_id {
        return Err(Error::Unauthorized);
    }

    // 2. Process files
    let mut evidence_files = Vec::new();

    while let Ok(Some(field)) = files.try_next().await {
        // Validate file type (images, PDFs only)
        let content_type = field.content_type();
        if !is_allowed_type(content_type) {
            return Err(Error::InvalidFileType);
        }

        // Validate file size (max 5MB)
        let file_data = read_field(field).await?;
        if file_data.len() > 5_000_000 {
            return Err(Error::FileTooLarge);
        }

        // Scan for malware (ClamAV)
        scan_file(&file_data).await?;

        // Encrypt & store
        let file_id = Uuid::new_v4();
        let encrypted_data = encrypt_file(&file_data)?;
        store_file(file_id, encrypted_data).await?;

        evidence_files.push(file_id);
    }

    // 3. Store evidence metadata
    db_insert_evidence(&db, Evidence {
        id: Uuid::new_v4(),
        dispute_id: *dispute_id,
        uploaded_by: user_id,
        file_ids: evidence_files,
        created_at: Utc::now(),
    }).await?;

    Ok(HttpResponse::Ok().json(json!({"status": "uploaded"})))
}
```

**Temps Total:** 9 jours (Semaine 17-18)

---

#### Milestone 3.3: Blockchain Monitoring (Semaine 19-20)

**Task 3.3.1: Background Worker (3 jours)**
```rust
// server/src/workers/blockchain_monitor.rs

pub struct BlockchainMonitor {
    wallet_manager: Arc<WalletManager>,
    db: DbPool,
    websocket: Arc<WebSocketServer>,
}

impl BlockchainMonitor {
    /// Start monitoring loop
    pub async fn start(&self) {
        loop {
            // Scan all pending transactions
            let pending_txs = db_load_pending_transactions(&self.db).await
                .unwrap_or_default();

            for tx in pending_txs {
                match self.check_transaction_status(&tx).await {
                    Ok(confirmations) => {
                        // Update DB
                        db_update_transaction_confirmations(&self.db, tx.id, confirmations).await.ok();

                        // Notify if milestone reached
                        if confirmations == 1 {
                            self.websocket.notify_tx_confirmed(tx.clone(), 1).await.ok();
                        } else if confirmations >= 10 {
                            self.websocket.notify_tx_finalized(tx.clone()).await.ok();

                            // Mark escrow as completed
                            db_update_escrow_status(&self.db, tx.escrow_id, EscrowStatus::Completed).await.ok();
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to check TX {}: {}", tx.tx_hash, e);
                    }
                }
            }

            // Sleep 30 seconds
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    async fn check_transaction_status(&self, tx: &Transaction) -> Result<u32> {
        // Query Monero daemon for TX status
        let tx_info = self.wallet_manager.get_transaction_info(&tx.tx_hash).await?;
        Ok(tx_info.confirmations)
    }
}
```

**Task 3.3.2: Alerts & Notifications (2 jours)**
```rust
impl BlockchainMonitor {
    /// Alert if transaction stuck
    async fn check_stuck_transactions(&self) {
        let stuck_txs = db_load_stuck_transactions(&self.db).await
            .unwrap_or_default();

        for tx in stuck_txs {
            // TX older than 1 hour with 0 confirmations
            if tx.created_at < Utc::now() - Duration::hours(1) && tx.confirmations == 0 {
                tracing::error!("STUCK TX: {} for escrow {}", tx.tx_hash, tx.escrow_id);

                // Alert admin
                self.send_admin_alert(format!("TX {} stuck", tx.tx_hash)).await.ok();

                // Notify parties
                let escrow = db_load_escrow(&self.db, tx.escrow_id).await.ok();
                if let Some(escrow) = escrow {
                    self.websocket.notify_tx_stuck(escrow, tx.tx_hash.clone()).await.ok();
                }
            }
        }
    }
}
```

**Task 3.3.3: Dashboard Admin (2 jours)**
```rust
// server/src/handlers/admin.rs

/// GET /api/admin/escrows (admin only)
async fn list_all_escrows(
    session: Session,
    query: web::Query<EscrowFilters>,
) -> Result<HttpResponse> {
    // Verify admin role
    let user = session.get_user()?;
    if user.role != UserRole::Admin {
        return Err(Error::Unauthorized);
    }

    // Load escrows with filters
    let escrows = db_load_escrows(&db, &query).await?;

    // Include statistics
    let stats = EscrowStats {
        total: escrows.len(),
        by_status: count_by_status(&escrows),
        total_volume_xmr: sum_volume(&escrows),
    };

    Ok(HttpResponse::Ok().json(json!({
        "escrows": escrows,
        "stats": stats
    })))
}
```

**Temps Total:** 7 jours (Semaine 19-20)

---

### 📦 Délivrables Phase 3

- [x] Escrow orchestration service complet
- [x] WebSocket notifications temps réel
- [x] Release flow (2-of-3 signatures)
- [x] Dispute resolution workflow
- [x] Evidence upload system
- [x] Blockchain monitoring background worker
- [x] Admin dashboard
- [x] 25+ tests end-to-end

---

## Phase 4: Frontend & UX

**Durée:** 8 semaines (Semaines 21-28)
**Priorité:** 🟡 MOYENNE
**Objectif:** Interface web complète et OPSEC-hardened

### 🎯 Success Criteria

- ✅ Interface responsive et intuitive
- ✅ 9 pages principales fonctionnelles
- ✅ Real-time updates via WebSocket
- ✅ OPSEC: pas de fingerprinting, pas de CDN
- ✅ Accessibility (WCAG 2.1 Level AA)
- ✅ Performance: <2s load time (via Tor)

### 📋 Stack Technique

**Décision:** **HTML/CSS/Vanilla JS** (pas de framework)

**Justification:**
- ✅ Pas de fingerprinting framework
- ✅ Contrôle total sur le code
- ✅ Taille minimale (important pour Tor)
- ✅ OPSEC-friendly

**Alternative:** Svelte/Alpine.js (si besoin réactivité)

### 📋 Pages à Créer

1. **Homepage** (`/`)
2. **Listings** (`/listings`)
3. **Product Detail** (`/listings/:id`)
4. **Checkout** (`/checkout/:listing_id`)
5. **My Orders** (`/orders`)
6. **Vendor Dashboard** (`/vendor/dashboard`)
7. **Escrow Tracker** (`/escrow/:id`)
8. **Admin Panel** (`/admin`)
9. **Settings** (`/settings`)

### Milestones détaillés disponibles dans [PRODUCTION-ROADMAP.md](PRODUCTION-ROADMAP.md)

---

## Phase 5: Sécurité & Audit

**Durée:** 12 semaines (Semaines 29-40)
**Priorité:** 🔴 CRITIQUE
**Objectif:** Production-ready security posture

### 🎯 Success Criteria

- ✅ Audit externe complété (tous les criticals fixés)
- ✅ Bug bounty actif (4+ semaines)
- ✅ Penetration testing passed
- ✅ Code coverage >90%
- ✅ Zero critical vulnerabilities
- ✅ [SECURITY-CHECKLIST-PRODUCTION.md](SECURITY-CHECKLIST-PRODUCTION.md) 100% complétée

### 📋 Milestones

#### Milestone 5.1: Internal Security Review (Semaines 29-32)

**Activités:**
- Code review ligne par ligne
- Threat modeling (STRIDE framework)
- Static analysis (cargo-audit, semgrep, clippy pedantic)
- Dependency audit (cargo-deny)
- Secrets scanning (gitleaks)
- Fuzzing (cargo-fuzz)

#### Milestone 5.2: External Audit (Semaines 33-40)

**Scope:**
1. Cryptographic review (2 semaines)
2. Network security (Tor isolation) (2 semaines)
3. Application security (3 semaines)
4. Infrastructure security (1 semaine)

**Budget:** $50k-$150k

**Auditeurs Recommandés:**
- Trail of Bits
- Kudelski Security
- NCC Group
- Cure53

#### Milestone 5.3: Bug Bounty (Semaines 41-42)

**Platform:** HackerOne ou Bugcrowd
**Rewards:** $100-$10k selon severity

---

## Phase 6: Production Testnet

**Durée:** 6 semaines (Semaines 41-46)
**Priorité:** 🟠 HAUTE
**Objectif:** Beta testing avec utilisateurs réels

### 📋 Milestones

#### Milestone 6.1: Infrastructure (Semaines 43-44)
- VPS anonyme (Njalla, 1984 Hosting)
- Server hardening (Debian 12 minimal)
- Monitoring (Prometheus + Grafana)

#### Milestone 6.2: Beta Testing (Semaines 45-46)
- 50+ beta testers
- Testnet XMR faucet
- Feedback collection
- Bug fixing

---

## Phase 7: Mainnet Launch

**Durée:** Variable (Semaine 47+)
**Priorité:** 🔴 CRITIQUE
**Objectif:** Production mainnet avec monitoring 24/7

### ⚠️ Pre-Launch Checklist (MANDATORY)

**Launch UNIQUEMENT si:**
- [ ] Audit externe complété (tous criticals fixés)
- [ ] Bug bounty actif 4+ semaines sans critical
- [ ] Testnet beta réussi (50+ users, 100+ escrows)
- [ ] [SECURITY-CHECKLIST-PRODUCTION.md](SECURITY-CHECKLIST-PRODUCTION.md) 100%
- [ ] Team disponible 24/7 (2 semaines)
- [ ] Monitoring & alerting opérationnels
- [ ] Incident response plan documenté
- [ ] Legal compliance vérifiée

### 🚀 Launch Strategy

**Week 1-2: Soft Launch**
- Invite-only (10-20 users)
- Limits: Max 0.1 XMR par escrow

**Week 3-4: Limited Public**
- Open registration
- Limits: Max 0.5 XMR par escrow

**Week 5+: Full Launch**
- Remove invite requirement
- Increase limits: Max 5 XMR par escrow

---

## Architecture Technique

### 🏗️ Vue d'Ensemble

```
┌─────────────────────────────────────────────────────────────┐
│                         CLIENT                              │
│                    (Tor Browser)                            │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTPS (via Tor)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   TOR NETWORK                               │
│                  (Hidden Service v3)                        │
│            your-marketplace.onion                           │
└──────────────────────┬──────────────────────────────────────┘
                       │ localhost:8080
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              BACKEND SERVER (Rust)                          │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Actix-web (API REST)                      │   │
│  │  ┌──────────┐  ┌───────────┐  ┌──────────────┐    │   │
│  │  │  Auth    │  │ Listings  │  │  Escrow      │    │   │
│  │  │ Handlers │  │ Handlers  │  │ Orchestrator │    │   │
│  │  └──────────┘  └───────────┘  └──────────────┘    │   │
│  │                                                     │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │        WebSocket Server                      │  │   │
│  │  │     (Real-time notifications)                │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
│                       │                                     │
│                       ▼                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Database (PostgreSQL/SQLite)                │   │
│  │        (Encryption at-rest: sqlcipher)              │   │
│  └─────────────────────────────────────────────────────┘   │
│                       │                                     │
│                       ▼                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Wallet Manager                            │   │
│  │                                                     │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │   │
│  │  │  Buyer   │  │  Vendor  │  │ Arbiter  │         │   │
│  │  │  Wallet  │  │  Wallet  │  │  Wallet  │         │   │
│  │  └──────────┘  └──────────┘  └──────────┘         │   │
│  └─────────────────────────────────────────────────────┘   │
│                       │                                     │
│                       │ localhost:18082-18084              │
│                       ▼                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │      Monero Wallet RPC (3 instances)                │   │
│  │         STRICT LOCALHOST BIND                       │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────┬──────────────────────────────────────┘
                       │ localhost:18081 (or via Tor)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│             Monero Daemon (monerod)                         │
│                   (Testnet/Mainnet)                         │
└─────────────────────────────────────────────────────────────┘
                       │
                       │ P2P via Tor
                       ▼
                 Monero Network
```

---

## Stack Technologique

### Backend

| Composant | Technologie | Version | Justification |
|-----------|-------------|---------|---------------|
| **Language** | Rust | 1.75+ | Performance, safety, async |
| **Web Framework** | Actix-web | 4.4+ | Performance, mature |
| **Database** | SQLite + sqlcipher | 3.42+ | Encryption at-rest |
| **ORM** | Diesel | 2.1+ | Type-safe, migrations |
| **Authentication** | argon2 | 0.5+ | Password hashing |
| **Session** | actix-session | 0.9+ | Server-side sessions |
| **WebSocket** | actix-web-actors | 4.3+ | Real-time notifications |
| **Serialization** | serde | 1.0+ | JSON handling |
| **Async Runtime** | tokio | 1.35+ | Async I/O |

### Frontend

| Composant | Technologie | Justification |
|-----------|-------------|---------------|
| **HTML/CSS/JS** | Vanilla | No fingerprinting |
| **Build Tool** | esbuild | Fast, minimal |
| **Icons** | SVG inline | No external fonts |
| **Styles** | CSS custom | No frameworks |

### Infrastructure

| Composant | Technologie | Justification |
|-----------|-------------|---------------|
| **OS** | Debian 12 | Stable, secure |
| **Tor** | Tor 0.4.7+ | Hidden service v3 |
| **Monero** | Monero 0.18.3+ | Multisig support |
| **Reverse Proxy** | Nginx | Performance |
| **Monitoring** | Prometheus + Grafana | Open-source |
| **Logs** | tracing | Rust-native |

---

## Sécurité & OPSEC

### 🔐 Principes de Sécurité

1. **Defense in Depth** - Multiples couches de sécurité
2. **Least Privilege** - Accès minimum requis
3. **Fail Secure** - Échouer de manière sécurisée
4. **Zero Trust** - Ne jamais faire confiance aux inputs
5. **Privacy by Default** - Pas de tracking, pas de logs sensibles

### 🛡️ Mesures de Sécurité

**Cryptographie:**
- Argon2id pour passwords
- AES-256-GCM pour encryption
- OsRng pour random generation

**Network:**
- Tor isolation stricte
- Monero RPC localhost only
- Rate limiting multi-layer
- DDoS protection

**Application:**
- Input validation stricte
- SQL prepared statements only
- XSS prevention (escaping)
- CSRF tokens
- Session management secure

**Infrastructure:**
- Disk encryption (LUKS)
- Database encryption (sqlcipher)
- Firewall (ufw)
- Automatic security updates

### 🚨 Threat Model

**Adversaires Considérés:**
1. **ISP/Network Surveillance** → Mitigé par Tor
2. **Exit Node Operators** → Mitigé par .onion (no exit)
3. **Blockchain Analysis** → Mitigé par Monero
4. **Global Passive Adversary** → Partiellement mitigé

**Hors Scope:**
- Attaques physiques sur serveur
- Compromission complète de Tor network
- Backdoor dans Monero protocol

---

## Budget & Ressources

### 💰 Estimation Budgétaire

| Phase | Durée | Coût Dev (Freelance) | Coût Infra | Total |
|-------|-------|----------------------|------------|-------|
| **Phase 1** | 6 sem | $15k-$25k | $0 | $15k-$25k |
| **Phase 2** | 8 sem | $25k-$40k | $0 | $25k-$40k |
| **Phase 3** | 6 sem | $20k-$30k | $0 | $20k-$30k |
| **Phase 4** | 8 sem | $20k-$35k | $0 | $20k-$35k |
| **Phase 5** | 12 sem | $10k-$20k | $50k-$150k (audit) | $60k-$170k |
| **Phase 6** | 6 sem | $10k-$20k | $500/mois × 2 | $11k-$21k |
| **Phase 7** | Ongoing | - | $500/mois | $500/mois |

**Total An 1:** $151k-$321k + $6k/an infra
**Total Développement:** $100k-$170k
**Total Audit:** $50k-$150k

**Note:** Open-source bénévole = gratuit mais plus lent (×2-3)

### 👥 Équipe Idéale

| Rôle | Temps | Compétences |
|------|-------|-------------|
| **Rust Developer** | Full-time | Rust, async, Monero, Tor |
| **Security Engineer** | Part-time | OPSEC, cryptography, audit |
| **Frontend Developer** | Half-time | HTML/CSS/JS, OPSEC |
| **DevOps** | Part-time | Linux, Tor, Monero, monitoring |

**Minimum:** 1 développeur Rust full-stack + audit externe

---

## Risques & Mitigations

### 🚨 Risques Techniques

| Risque | Prob. | Impact | Mitigation |
|--------|-------|--------|------------|
| Vulnérabilité critique post-launch | Moyenne | Très Haut | Bug bounty, audits réguliers |
| Monero RPC instable | Moyenne | Haut | Health checks, retry logic, failover |
| Tor network down/censored | Faible | Haut | Bridges, backup .onion |
| Database corruption | Faible | Très Haut | Backups quotidiens, réplication |
| DDoS sur hidden service | Moyenne | Moyen | Rate limiting, Tor PoW |

### ⚖️ Risques Légaux

| Risque | Prob. | Impact | Mitigation |
|--------|-------|--------|------------|
| Saisie serveurs | Faible-Moyenne | Très Haut | Encryption at-rest, pas de KYC |
| Responsabilité contenus illégaux | Moyenne | Haut | Terms of Service, modération |
| Contrainte juridique (backdoor) | Faible | Très Haut | Canary, open-source |

### 💼 Risques Business

| Risque | Prob. | Impact | Mitigation |
|--------|-------|--------|------------|
| Pas assez d'utilisateurs | Moyenne | Haut | Marketing Tor/Monero communities |
| Vendor scams | Moyenne | Moyen | Reputation system, arbiters |
| Competitors | Élevée | Moyen | Meilleure OPSEC, meilleur UX |

---

## Métriques de Succès

### 📊 KPIs Techniques

**Phase 1-3 (Testnet):**
- [ ] 100% tests passing
- [ ] Code coverage >80%
- [ ] Zero security theatre violations
- [ ] <5% error rate transactions

**Phase 4-6 (Beta):**
- [ ] 50+ beta testers
- [ ] 100+ completed escrows on testnet
- [ ] User satisfaction >4/5
- [ ] Zero security incidents

**Phase 7 (Mainnet):**
- [ ] 500+ registered users (mois 1)
- [ ] 100+ completed escrows (mois 1)
- [ ] Uptime >99.5%
- [ ] Response time <500ms
- [ ] Zero fund loss incidents

### 💹 KPIs Business

**Mois 1:**
- 500+ users
- 100+ escrows
- $10k+ volume (XMR equivalent)

**Mois 3:**
- 2000+ users
- 500+ escrows
- $50k+ volume

**Mois 6:**
- 5000+ users
- 2000+ escrows
- $200k+ volume

---

## Actions Immédiates

### 🚀 Cette Semaine (Semaine 1)

**Jour 1-2: Setup Environnement**
1. [ ] Vous êtes déjà sur WSL Ubuntu ✅
2. [ ] Vérifier compilation: `cargo build --workspace`
3. [ ] Setup Monero testnet: `./scripts/setup-monero-testnet.sh`

**Jour 3-4: Task 1.1.1**
4. [ ] Créer `scripts/setup-3-wallets-testnet.sh`
5. [ ] Tester script: 3 wallets créés et running

**Jour 5: Task 1.1.2 (Début)**
6. [ ] Créer fichier `wallet/tests/multisig_e2e.rs`
7. [ ] Structure de base du test

### 📅 Semaine 2

**Lundi-Mercredi:**
- [ ] Compléter test e2e multisig setup
- [ ] Test DOIT passer end-to-end

**Jeudi:**
- [ ] Créer diagramme de séquence

**Vendredi:**
- [ ] Code review & refactoring

### 📅 Semaine 3-4

- [ ] Implémenter fonctions transactions (create, sign, finalize, broadcast)
- [ ] Test e2e transaction complet

### 📅 Semaine 5-6

- [ ] Tests edge cases
- [ ] Documentation ERROR-CODES.md
- [ ] Final validation Phase 1

---

## 📚 Ressources & Documentation

### Documents Projet

1. [PRODUCTION-ROADMAP.md](PRODUCTION-ROADMAP.md) - Roadmap détaillée
2. [PHASE-1-IMPLEMENTATION.md](PHASE-1-IMPLEMENTATION.md) - Plan Phase 1
3. [ARCHITECTURE-DECISIONS.md](ARCHITECTURE-DECISIONS.md) - ADRs
4. [SECURITY-CHECKLIST-PRODUCTION.md](SECURITY-CHECKLIST-PRODUCTION.md) - Checklist
5. [COMPILATION-WINDOWS.md](COMPILATION-WINDOWS.md) - Fix Windows
6. [NEXT-STEPS.md](NEXT-STEPS.md) - Actions immédiates
7. [CLAUDE.md](CLAUDE.md) - Instructions Claude Code

### Documentation Externe

**Monero:**
- [Wallet RPC Documentation](https://www.getmonero.org/resources/developer-guides/wallet-rpc.html)
- [Multisig Guide](https://monerodocs.org/multisignature/)
- [Testnet Explorer](https://testnet.xmrchain.net/)

**Tor:**
- [Hidden Service Guide](https://community.torproject.org/onion-services/)
- [Tor Project](https://www.torproject.org/)

**Rust:**
- [Rust Security Guide](https://anssi-fr.github.io/rust-guide/)
- [Actix-web Documentation](https://actix.rs/)

---

## 📝 Changelog

| Version | Date | Changements | Auteur |
|---------|------|-------------|--------|
| 1.0 | 2025-10-14 | Plan initial | Claude |
| 2.0 | 2025-10-16 | Plan complet détaillé | Claude |

---

## ✅ Next Review

**Date:** Fin de Semaine 2 (2025-10-30)
**Agenda:**
- Review progrès Phase 1 Milestone 1.1
- Ajuster timeline si nécessaire
- Identifier blockers

---

**🎯 Let's Build Something Great! 🚀**

**Statut:** 📋 Approuvé et Prêt à Exécuter
**Contact:** (À définir)
