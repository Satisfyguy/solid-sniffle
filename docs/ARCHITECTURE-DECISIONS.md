# Architecture Decisions Record (ADR)
## Monero Marketplace - Hidden Service

**Date de Création:** 2025-10-16
**Version:** 1.0
**Statut:** Draft

---

## Table des Matières
1. [Vue d'Ensemble](#vue-densemble)
2. [Décisions Structurelles](#décisions-structurelles)
3. [Décisions Techniques](#décisions-techniques)
4. [Décisions Sécurité](#décisions-sécurité)
5. [Trade-offs & Justifications](#trade-offs--justifications)

---

## Vue d'Ensemble

Ce document capture les décisions architecturales majeures pour le Monero Marketplace, un hidden service Tor avec escrow Monero 2-of-3 multisig.

**Principes Directeurs:**
1. **Security First** - Sécurité prioritaire sur features
2. **Privacy by Default** - Pas de tracking, pas de logs sensibles
3. **KISS** - Keep It Simple, Stupid (éviter over-engineering)
4. **Fail Secure** - En cas d'erreur, échouer de manière sécurisée
5. **Zero Trust** - Ne jamais faire confiance aux inputs

---

## Décisions Structurelles

### ADR-001: Workspace Monorepo Rust

**Statut:** ✅ Accepté
**Date:** 2025-10-14

**Contexte:**
Besoin de séparer le code en modules logiques (common, wallet, cli, server) tout en maintenant cohérence et facilité de développement.

**Décision:**
Utiliser un Cargo workspace avec 4 crates:
```
monero-marketplace/
├── common/      # Types partagés, erreurs, utils
├── wallet/      # Logique Monero (RPC, multisig)
├── cli/         # Interface ligne de commande
└── server/      # Backend web API (à créer)
```

**Alternatives Considérées:**
1. **Mono-crate** - Tout dans un seul crate
   - ❌ Mauvaise séparation des responsabilités
   - ❌ Tests plus difficiles
   - ❌ Compilation plus lente

2. **Repos Séparés** - Un repo par composant
   - ❌ Gestion des versions complexe
   - ❌ Changements cross-crate difficiles
   - ❌ CI/CD plus complexe

**Conséquences:**
- ✅ Séparation claire des responsabilités
- ✅ Réutilisation du code facilitée
- ✅ Tests isolés par crate
- ❌ Légère complexité dans Cargo.toml

---

### ADR-002: Backend Framework - Actix-web vs Rocket

**Statut:** ⏳ Proposé (à décider)
**Date:** 2025-10-16

**Contexte:**
Besoin d'un framework web Rust performant, sécurisé et mature pour le backend API.

**Options:**

#### Option A: Actix-web 4.x (RECOMMANDÉ)
**Pros:**
- ✅ Très performant (acteur model)
- ✅ Mature et stable
- ✅ Excellent écosystème middleware
- ✅ Async/await natif
- ✅ WebSocket support intégré
- ✅ Grande communauté

**Cons:**
- ❌ API plus verbeux
- ❌ Courbe d'apprentissage moyenne

**Exemple:**
```rust
use actix_web::{web, App, HttpServer, HttpResponse};

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

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}
```

#### Option B: Rocket 0.5
**Pros:**
- ✅ API très ergonomique
- ✅ Type-safe routing
- ✅ Validation automatique
- ✅ Excellente documentation

**Cons:**
- ❌ Moins performant qu'Actix
- ❌ Communauté plus petite
- ❌ Moins de middleware disponibles

**Exemple:**
```rust
#[macro_use] extern crate rocket;

#[get("/api/health")]
fn health_check() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![health_check])
}
```

#### Option C: Axum (Alternative moderne)
**Pros:**
- ✅ Très moderne (from Tokio team)
- ✅ Excellente intégration Tokio
- ✅ Type-safe extractors
- ✅ Composition élégante

**Cons:**
- ❌ Moins mature (sorti en 2021)
- ❌ Moins d'exemples/docs

**Décision Recommandée:** **Actix-web 4.x**

**Justification:**
- Performance critique pour hidden service (latence Tor déjà élevée)
- Maturité importante pour production
- WebSocket nécessaire pour notifications temps réel
- Large communauté = plus de ressources

**Conséquences:**
- ✅ Performance optimale
- ✅ Scaling facile
- ❌ Code légèrement plus verbose

---

### ADR-003: Database - PostgreSQL vs SQLite

**Statut:** ⏳ Proposé
**Date:** 2025-10-16

**Contexte:**
Besoin d'un stockage persistant pour users, listings, orders, escrows.

**Options:**

#### Option A: PostgreSQL (RECOMMANDÉ pour Production)
**Pros:**
- ✅ ACID complet
- ✅ Concurrent writes
- ✅ Rich types (JSON, arrays)
- ✅ Full-text search
- ✅ Replication native
- ✅ Excellent pour production

**Cons:**
- ❌ Setup plus complexe
- ❌ Overhead pour petit volume
- ❌ Nécessite serveur dédié

**Use Case:** Production avec >100 users, >1000 orders

#### Option B: SQLite + sqlcipher (RECOMMANDÉ pour MVP)
**Pros:**
- ✅ Zero config (fichier unique)
- ✅ Chiffrement at-rest natif
- ✅ Très rapide pour reads
- ✅ Backup = copier fichier
- ✅ Idéal pour développement

**Cons:**
- ❌ Concurrent writes limité
- ❌ Pas de replication
- ❌ Scaling limité

**Use Case:** MVP, testnet, small scale (<100 concurrent users)

**Décision Recommandée:**
- **Phase 1-6 (Testnet):** SQLite + sqlcipher
- **Phase 7 (Mainnet):** Migration vers PostgreSQL

**Justification:**
- Simplicité pour MVP
- Chiffrement at-rest crucial
- Migration PostgreSQL facile plus tard (via diesel migrations)

**Conséquences:**
- ✅ Setup rapide
- ✅ Chiffrement gratuit
- ⚠️ Nécessitera migration pour scale

---

### ADR-004: ORM - Diesel vs SQLx

**Statut:** ⏳ Proposé
**Date:** 2025-10-16

**Options:**

#### Option A: Diesel (RECOMMANDÉ)
**Pros:**
- ✅ Compile-time query checking
- ✅ Type-safe queries
- ✅ Excellent migrations system
- ✅ Support PostgreSQL + SQLite
- ✅ Très mature

**Cons:**
- ❌ Async support récent (diesel-async)
- ❌ Courbe d'apprentissage

**Exemple:**
```rust
use diesel::prelude::*;

#[derive(Queryable, Insertable)]
#[diesel(table_name = users)]
struct User {
    id: Uuid,
    username: String,
    password_hash: String,
}

// Type-safe query
let users = users::table
    .filter(users::username.eq("alice"))
    .load::<User>(&mut conn)?;
```

#### Option B: SQLx
**Pros:**
- ✅ Async-first
- ✅ Compile-time checking (avec macros)
- ✅ Raw SQL (plus flexible)

**Cons:**
- ❌ Moins type-safe que Diesel
- ❌ Migrations moins robustes

**Décision Recommandée:** **Diesel 2.x + diesel-async**

**Justification:**
- Type safety critique pour éviter SQL injection
- Migrations robustes nécessaires
- Support SQLite + PostgreSQL pour migration

---

## Décisions Techniques

### ADR-005: Authentication - Sessions vs JWT

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Décision:** **Server-side Sessions** (PAS JWT)

**Justification:**
1. **JWT Trackable** - JWT payload décodable = fingerprinting possible
2. **Sessions Opaque** - Cookie contient juste session_id (UUID random)
3. **Révocation** - Sessions révocables immédiatement (logout)
4. **Tor-Friendly** - Pas de payload exposé = moins de metadata

**Implémentation:**
```rust
// Session storage in-memory (Redis alternative: actix-session)
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};

// Configure session
SessionMiddleware::builder(
    CookieSessionStore::default(),
    secret_key.clone()
)
.cookie_name("SESSIONID")
.cookie_http_only(true)
.cookie_same_site(SameSite::Strict)
.cookie_secure(false) // Tor hidden service n'utilise pas HTTPS
.build()
```

**Structure Session:**
```rust
struct SessionData {
    user_id: Uuid,
    role: UserRole, // buyer, vendor, arbiter
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}
```

**Conséquences:**
- ✅ Privacy préservée
- ✅ Révocation immédiate
- ❌ State server-side (scaling challenge)

---

### ADR-006: Password Hashing - Argon2id

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Décision:** Argon2id via `argon2` crate

**Alternatives:**
- bcrypt - ❌ Moins résistant aux GPU
- scrypt - ❌ Moins moderne
- PBKDF2 - ❌ Vulnérable aux ASICs

**Implémentation:**
```rust
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
```

**Paramètres:**
- Memory cost: 19456 KiB (19 MiB)
- Time cost: 2 iterations
- Parallelism: 1 thread
- Hash length: 32 bytes

---

### ADR-007: WebSocket pour Notifications Temps Réel

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Contexte:**
Besoin de notifier users en temps réel:
- État escrow (syncing → ready)
- Messages vendor ↔ buyer
- Transaction confirmations

**Décision:** WebSocket avec actix-web-actors

**Alternatives:**
- Polling - ❌ Inefficace, latence élevée
- Server-Sent Events - ❌ Unidirectionnel seulement

**Implémentation:**
```rust
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

struct WsSession {
    user_id: Uuid,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Handle incoming message
                ctx.text(text);
            }
            _ => (),
        }
    }
}
```

**Events à Diffuser:**
```rust
enum WsEvent {
    EscrowStateChanged { escrow_id: Uuid, new_state: String },
    NewMessage { from: Uuid, content: String },
    TransactionConfirmed { tx_hash: String, confirmations: u32 },
    OrderStatusChanged { order_id: Uuid, new_status: String },
}
```

---

### ADR-008: Rate Limiting Strategy

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Contexte:**
Hidden services vulnérables aux attaques DDoS.

**Décision:** Multi-layer rate limiting

**Layers:**

#### Layer 1: Global Rate Limit
```rust
use actix_governor::{Governor, GovernorConfigBuilder};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)  // 10 requests/sec global
    .burst_size(20)  // burst max 20
    .finish()
    .unwrap();

App::new().wrap(Governor::new(&governor_conf))
```

#### Layer 2: Per-IP Rate Limit (Tor Exit Node)
```rust
// Note: Tor = même IP pour tous
// Alternative: Per-session rate limit
use actix_web::middleware::RateLimiter;

// Max 5 req/sec par session
let limiter = RateLimiter::new(5, Duration::from_secs(1));
```

#### Layer 3: Per-Endpoint Rate Limit
```rust
// Endpoints sensibles
POST /api/auth/login       -> 3 req/min
POST /api/orders           -> 10 req/hour
POST /api/escrow/init      -> 5 req/hour
POST /api/listings         -> 20 req/day (vendors)

// Endpoints lecture
GET /api/listings          -> 60 req/min
GET /api/orders/:id        -> 30 req/min
```

**Conséquences:**
- ✅ Protection DDoS
- ✅ Prévention brute-force
- ⚠️ UX impacté si trop strict (à tester)

---

## Décisions Sécurité

### ADR-009: Tor Isolation Strict

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Règles:**

#### 1. Monero RPC TOUJOURS localhost
```rust
// Validation stricte dans MoneroRpcClient::new()
fn validate_rpc_url(url: &str) -> Result<()> {
    let parsed = Url::parse(url)?;
    let host = parsed.host_str()
        .ok_or_else(|| Error::InvalidInput("Missing host".to_string()))?;

    // ONLY allow 127.0.0.1 or localhost
    if host != "127.0.0.1" && host != "localhost" {
        return Err(Error::Security(format!(
            "RPC URL must be localhost only, got: {}", host
        )));
    }

    Ok(())
}
```

#### 2. Tous les appels externes via Tor
```rust
use reqwest::Proxy;

let proxy = Proxy::all("socks5h://127.0.0.1:9050")?;
let client = reqwest::Client::builder()
    .proxy(proxy)
    .timeout(Duration::from_secs(30))
    .build()?;
```

#### 3. Pas de logs de données sensibles
```rust
// ❌ INTERDIT
tracing::info!("User {} logged in from {}", username, ip_address);
tracing::debug!("Multisig address: {}", address);

// ✅ OK
tracing::info!("User login successful");
tracing::debug!("Multisig wallet created");
```

---

### ADR-010: Encryption At-Rest

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Décision:** Multiple layers

#### Layer 1: Disk Encryption (LUKS)
```bash
# Server setup
cryptsetup luksFormat /dev/sda1
cryptsetup open /dev/sda1 encrypted_disk
mkfs.ext4 /dev/mapper/encrypted_disk
```

#### Layer 2: Database Encryption (sqlcipher)
```rust
// SQLite connection avec chiffrement
let conn = SqliteConnection::establish(&format!(
    "file:{}?cipher=aes-256-gcm&key={}",
    db_path,
    hex::encode(&key)
))?;
```

#### Layer 3: Field-Level Encryption (Sensitive Data)
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};

// Chiffrer multisig_info avant stockage
fn encrypt_multisig_info(info: &str, key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce"); // TODO: random nonce
    let ciphertext = cipher.encrypt(nonce, info.as_bytes())?;
    Ok(ciphertext)
}
```

**Champs Chiffrés:**
- `users.email` (si présent)
- `escrows.buyer_wallet_info`
- `escrows.vendor_wallet_info`
- `escrows.arbiter_wallet_info`
- `messages.content` (communications vendor-buyer)

---

### ADR-011: Input Validation Stricte

**Statut:** ✅ Accepté
**Date:** 2025-10-16

**Stratégie:** Validate Everything

**Implémentation:**
```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
struct CreateListingRequest {
    #[validate(length(min = 10, max = 200))]
    title: String,

    #[validate(length(min = 50, max = 5000))]
    description: String,

    #[validate(range(min = 1_000_000, max = 1_000_000_000_000))] // 0.001 XMR to 1000 XMR
    price_xmr_atomic: u64,

    #[validate(range(min = 0, max = 10000))]
    stock: u32,
}

// Dans handler
async fn create_listing(
    req: web::Json<CreateListingRequest>
) -> Result<HttpResponse> {
    req.validate()?; // Retourne 400 si invalide
    // ...
}
```

**Validation par Type:**

| Type | Validation |
|------|------------|
| Username | 3-50 chars, alphanumeric + underscore |
| Password | 8-128 chars, au moins 1 upper, 1 lower, 1 digit |
| XMR Amount | 0.001 XMR (min) to 10000 XMR (max) |
| Monero Address | Regex: `^[48][0-9AB][1-9A-HJ-NP-Za-km-z]{93}$` |
| Multisig Info | Length 100-5000, starts with "MultisigV1" |

---

## Trade-offs & Justifications

### Trade-off 1: Performance vs Security

**Décision:** Security > Performance

**Exemples:**
- Argon2id (slow) vs bcrypt (faster) → Choisi Argon2id
- Rate limiting strict → Peut ralentir UX
- Input validation extensive → Overhead requests

**Justification:**
Marketplace finance = sécurité prioritaire. Performance optimisée dans limites sécurité.

---

### Trade-off 2: Complexity vs Maintainability

**Décision:** KISS (Keep It Simple)

**Exemples Évités:**
- ❌ Microservices (over-engineering pour MVP)
- ❌ Event sourcing (complexité excessive)
- ❌ GraphQL (REST suffit)

**Architecture Choisie:**
- ✅ Monorepo monolithique
- ✅ REST API simple
- ✅ PostgreSQL/SQLite (pas NoSQL)

---

### Trade-off 3: Features vs Time-to-Market

**Décision:** MVP First, Features Later

**Phase 1 MVP Features:**
- ✅ Escrow 2-of-3 multisig
- ✅ Listings basiques
- ✅ Orders flow
- ✅ Dispute resolution

**Post-MVP Features (Backlog):**
- ⏳ Reputation system
- ⏳ Multi-currency (BTC support)
- ⏳ Vendor analytics
- ⏳ Advanced search (full-text)

---

## Revisions & Changelog

| Version | Date | Changements | Auteur |
|---------|------|-------------|--------|
| 1.0 | 2025-10-16 | Initial ADR | Claude |
| | | | |

---

## Prochaines Décisions à Prendre

### À Décider Semaine 3-4:
- [ ] **ADR-012:** Frontend Framework (Vanilla JS vs Svelte vs Alpine.js)
- [ ] **ADR-013:** File Upload Strategy (pour images listings + disputes)
- [ ] **ADR-014:** Backup Strategy (frequency, retention, encryption)
- [ ] **ADR-015:** Monitoring Stack (Prometheus vs alternatives)

### À Décider Phase 2:
- [ ] **ADR-016:** Deployment Strategy (Docker vs systemd)
- [ ] **ADR-017:** Log Aggregation (Loki vs alternatives)
- [ ] **ADR-018:** Alerting System (PagerDuty vs self-hosted)

---

**Status:** 📋 Living Document (sera mis à jour régulièrement)
**Review Cycle:** Après chaque phase milestone
**Approval Process:** À définir (team consensus)
