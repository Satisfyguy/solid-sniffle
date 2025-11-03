# Plan de Sécurisation Backend - Patches Exactes

**Durée totale : 6-7h**
**Résultat : Production-ready, zéro théâtre**

---

## PHASE 1 : FIXES CODE (3-4h)

### FIX 1 : Rate Limiting (5 min)

**Fichier : `server/src/main.rs`**

#### Patch 1.1 : Décommenter global_rate_limiter
```
FIND (ligne ~258):
            // Global rate limiter (100 req/min per IP)
            // .wrap(global_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ

REPLACE WITH:
            // Global rate limiter (100 req/min per IP)
            .wrap(global_rate_limiter())
```

#### Patch 1.2 : Décommenter protected_rate_limiter
```
FIND (ligne ~343):
                web::scope("/api")
                    // .wrap(protected_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ

REPLACE WITH:
                web::scope("/api")
                    .wrap(protected_rate_limiter())
```

**Vérification :** Aucune ligne ne devrait avoir `// TEMPORAIREMENT DÉSACTIVÉ` après cette étape.

---

### FIX 2 : Escrow Authorization - refund_funds (45 min)

**Fichier : `server/src/handlers/escrow.rs`**

#### Patch 2.1 : Ajouter imports
```
FIND (ligne ~1):
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};

REPLACE WITH:
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use crate::db::db_load_escrow;
```

#### Patch 2.2 : Ajouter vérification vendor/arbiter dans refund_funds
```
FIND (dans refund_funds, ligne ~280):
    // Refund funds via orchestrator
    match escrow_orchestrator
        .refund_funds(escrow_id, user_id, payload.buyer_address.clone())
        .await

REPLACE WITH:
    // Load escrow to verify requester is vendor or arbiter
    let escrow = match db_load_escrow(&pool, escrow_id).await {
        Ok(e) => e,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Escrow not found"
            }))
        }
    };

    // Verify user is vendor or arbiter
    if user_id.to_string() != escrow.vendor_id && user_id.to_string() != escrow.arbiter_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only vendor or arbiter can refund"
        }));
    }

    // Refund funds via orchestrator
    match escrow_orchestrator
        .refund_funds(escrow_id, user_id, payload.buyer_address.clone())
        .await
```

**Validation :** refund_funds vérifie maintenant que seul le vendor/arbiter de l'escrow peut refund.

---

### FIX 3 : Escrow Authorization - resolve_dispute (45 min)

**Fichier : `server/src/handlers/escrow.rs`**

#### Patch 3.1 : Ajouter vérification arbiter-only dans resolve_dispute
```
FIND (dans resolve_dispute, ligne ~360):
    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Resolve dispute via orchestrator
    match escrow_orchestrator

REPLACE WITH:
    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Load escrow to verify requester is the assigned arbiter
    let escrow = match db_load_escrow(&pool, escrow_id).await {
        Ok(e) => e,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Escrow not found"
            }))
        }
    };

    // Verify user is the assigned arbiter
    if user_id.to_string() != escrow.arbiter_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only the assigned arbiter can resolve disputes"
        }));
    }

    // Resolve dispute via orchestrator
    match escrow_orchestrator
```

**Validation :** resolve_dispute vérifie maintenant que seul l'arbiter assigné peut résoudre.

---

### FIX 4 : Orders Authorization - cancel_order (30 min)

**Fichier : `server/src/handlers/orders.rs`**

#### Patch 4.1 : Ajouter imports pour db_load_escrow
```
FIND (ligne ~1):
use crate::db::DbPool;

REPLACE WITH:
use crate::db::{DbPool, db_load_escrow};
```

#### Patch 4.2 : Ajouter vérification buyer dans cancel_order
```
FIND (dans cancel_order, ligne ~620):
    // Check if order is funded (needs refund)
    let needs_refund = order.status == "funded";

    if needs_refund {
        // Validate escrow exists for this order
        let escrow_id_str = match &order.escrow_id {
            Some(id) => id,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Order is funded but has no associated escrow"
                }))
            }
        };

        let escrow_uuid = match Uuid::parse_str(escrow_id_str) {
            Ok(uuid) => uuid,
            Err(_) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Invalid escrow ID format"
                }))
            }
        };

REPLACE WITH:
    // Check if order is funded (needs refund)
    let needs_refund = order.status == "funded";

    if needs_refund {
        // Validate escrow exists for this order
        let escrow_id_str = match &order.escrow_id {
            Some(id) => id,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Order is funded but has no associated escrow"
                }))
            }
        };

        let escrow_uuid = match Uuid::parse_str(escrow_id_str) {
            Ok(uuid) => uuid,
            Err(_) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Invalid escrow ID format"
                }))
            }
        };

        // Load escrow and verify buyer is the one cancelling
        let escrow = match db_load_escrow(&pool, escrow_uuid).await {
            Ok(e) => e,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Escrow not found"
                }))
            }
        };

        if escrow.buyer_id != user_id.to_string() {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only the buyer can cancel this order"
            }));
        }
```

**Validation :** cancel_order vérifie maintenant que le buyer de l'escrow cancelle bien.

---

### FIX 5 : RPC URL Validation (30 min)

**Fichier : `server/src/handlers/escrow.rs`**

#### Patch 5.1 : Remplacer validation RPC URL
```
FIND (ligne ~35-50):
/// Validate that role is buyer or vendor (not arbiter)
fn validate_client_role(role: &str) -> Result<(), validator::ValidationError> {
    match role.to_lowercase().as_str() {
        "buyer" | "vendor" => Ok(()),
        "arbiter" => Err(validator::ValidationError::new(
            "role_not_allowed",
        )),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

REPLACE WITH:
/// Validate that role is buyer or vendor (not arbiter)
fn validate_client_role(role: &str) -> Result<(), validator::ValidationError> {
    match role.to_lowercase().as_str() {
        "buyer" | "vendor" => Ok(()),
        "arbiter" => Err(validator::ValidationError::new(
            "role_not_allowed",
        )),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

/// Validate RPC URL: only allow localhost or .onion (no public URLs)
fn validate_rpc_url(url: &str) -> Result<(), validator::ValidationError> {
    let parsed = url::Url::parse(url)
        .map_err(|_| validator::ValidationError::new("invalid_url"))?;

    let host = parsed.host_str()
        .ok_or_else(|| validator::ValidationError::new("no_host"))?;

    // Only allow localhost, 127.x.x.x, or .onion addresses
    let is_localhost = host.starts_with("127.")
        || host.eq("localhost")
        || host.starts_with("::1");
    let is_onion = host.ends_with(".onion");

    if !is_localhost && !is_onion {
        return Err(validator::ValidationError::new(
            "rpc_url_must_be_local_or_onion"
        ));
    }

    Ok(())
}
```

#### Patch 5.2 : Appliquer validation au champ rpc_url
```
FIND (ligne ~15):
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(url(message = "Invalid RPC URL format"))]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,

REPLACE WITH:
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(custom = "validate_rpc_url")]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,
```

**Validation :** RPC URL validation maintenant bloque toutes URLs publiques.

---

### FIX 6 : Credentials - Arbiter Password (45 min)

**Fichier : `server/src/main.rs`**

#### Patch 6.1 : Générer password aléatoire pour arbiter
```
FIND (ligne ~150):
        if arbiter_exists.is_none() {
            info!("No arbiter found, creating system arbiter...");
            let password = "arbiter_system_2024";
            let salt = SaltString::generate(&mut OsRng);

REPLACE WITH:
        if arbiter_exists.is_none() {
            info!("No arbiter found, creating system arbiter...");

            // Generate random 16-character password
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let password: String = (0..16)
                .map(|_| {
                    let idx = rng.gen_range(0..62);
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                        .chars()
                        .nth(idx)
                        .unwrap()
                })
                .collect();

            let salt = SaltString::generate(&mut OsRng);
```

#### Patch 6.2 : Logger le password généré
```
FIND (ligne ~175):
            info!("✅ System arbiter created successfully (username: arbiter_system, password: arbiter_system_2024)");

REPLACE WITH:
            info!("⚠️  ✅ System arbiter created successfully");
            info!("📋 SAVE THIS IMMEDIATELY - Arbiter credentials:");
            info!("   Username: arbiter_system");
            info!("   Password: {}", password);
            info!("⚠️  This password will NOT be shown again. Change it immediately after first login.");
```

**Validation :** Arbiter password est maintenant aléatoire et loggé au démarrage.

---

### FIX 7 : Session Secret - Production Safety (30 min)

**Fichier : `server/src/main.rs`**

#### Patch 7.1 : Ajouter panic en production si SESSION_SECRET_KEY manquant
```
FIND (ligne ~135):
    // 4. Session secret key
    // IMPORTANT: In production, load from secure environment variable
    // This should be a 64-byte cryptographically random key
    let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| {
        tracing::warn!("SESSION_SECRET_KEY not set, using development key - NOT FOR PRODUCTION");
        "development_key_do_not_use_in_production_minimum_64_bytes_required".to_string()
    });

REPLACE WITH:
    // 4. Session secret key
    // IMPORTANT: In production, load from secure environment variable
    // This should be a 64-byte cryptographically random key
    let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            tracing::warn!("SESSION_SECRET_KEY not set, using development key (dev mode only)");
            "development_key_do_not_use_in_production_minimum_64_bytes_required".to_string()
        } else {
            panic!("❌ FATAL: SESSION_SECRET_KEY environment variable MUST be set in production!");
        }
    });
```

**Validation :** Production builds vont panic si SESSION_SECRET_KEY absent.

---

## PHASE 2 : TESTS (1-2h)

### TEST 1 : Rate Limiting

```bash
# Terminal 1: Start server
cargo run --release

# Terminal 2: Send 150 rapid requests
for i in {1..150}; do
  echo "Request $i:"
  curl -s -w "HTTP %{http_code}\n" http://127.0.0.1:8080/api/health
done

# Expected: First ~100 return 200, then 429 (Too Many Requests)
```

**Success criteria :** Après ~100 requêtes, voir `HTTP 429`.

---

### TEST 2 : Escrow Authorization - refund_funds

```bash
# Setup: Create order, fund it, get escrow_id from response
ESCROW_ID="your-escrow-id-from-setup"
BUYER_ID="buyer-uuid"
WRONG_VENDOR_ID="different-vendor-uuid"

# Test 1: Wrong vendor tries to refund → expect 403
curl -X POST http://127.0.0.1:8080/api/escrow/$ESCROW_ID/refund \
  -H "Content-Type: application/json" \
  -d '{"buyer_address":"addr123..."}' \
  -b "session=your-session-cookie"
  # Should return: {"error":"Only vendor or arbiter can refund"}
  # HTTP 403
```

**Success criteria :** Response HTTP 403 avec message "Only vendor or arbiter can refund".

---

### TEST 3 : Escrow Authorization - resolve_dispute

```bash
# Test: Non-arbiter tries to resolve → expect 403
curl -X POST http://127.0.0.1:8080/api/escrow/$ESCROW_ID/resolve \
  -H "Content-Type: application/json" \
  -d '{"resolution":"buyer","recipient_address":"addr123..."}' \
  -b "session=your-session-cookie"
  # Should return: {"error":"Only the assigned arbiter can resolve disputes"}
  # HTTP 403
```

**Success criteria :** Response HTTP 403 avec message "Only the assigned arbiter can resolve disputes".

---

### TEST 4 : RPC URL Validation

```bash
# Test 1: Public URL → expect 400
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -H "Content-Type: application/json" \
  -d '{
    "rpc_url":"http://attacker.com:18082/json_rpc",
    "role":"buyer"
  }' \
  -b "session=your-session-cookie"
  # Should return 400 with validation error

# Test 2: Localhost → expect success (or proper response)
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -H "Content-Type: application/json" \
  -d '{
    "rpc_url":"http://127.0.0.1:18082/json_rpc",
    "rpc_user":"user",
    "rpc_password":"pass",
    "role":"buyer"
  }' \
  -b "session=your-session-cookie"
  # Should NOT return validation error about URL

# Test 3: .onion → expect success
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -H "Content-Type: application/json" \
  -d '{
    "rpc_url":"http://abc123xyz.onion:18082/json_rpc",
    "role":"vendor"
  }' \
  -b "session=your-session-cookie"
  # Should NOT return validation error about URL
```

**Success criteria :**
- Public URL → 400 validation error
- localhost/127.x.x.x → Pas d'erreur URL
- .onion → Pas d'erreur URL

---

### TEST 5 : Credentials - Arbiter Password

```bash
# Test 1: Development mode - should work without SESSION_SECRET_KEY
unset SESSION_SECRET_KEY
cargo run
# Should start with warning, not panic

# Test 2: Production mode - should panic without SESSION_SECRET_KEY
unset SESSION_SECRET_KEY
cargo run --release
# Should panic with "FATAL: SESSION_SECRET_KEY environment variable MUST be set"

# Test 3: Production mode - should work with SESSION_SECRET_KEY
export SESSION_SECRET_KEY="your-64-byte-random-key-here-minimum-64-bytes-required-12345"
cargo run --release
# Should start successfully
```

**Success criteria :**
- Dev mode: warning but starts
- Release without var: panic with FATAL message
- Release with var: starts successfully

---

## PHASE 3 : VALIDATION FINALE (1h)

### Étape 1 : Tests unitaires
```bash
cargo test --workspace --lib
# Tous les tests doivent passer
```

### Étape 2 : Security audit
```bash
cargo audit
# Doit afficher: "0 vulnerabilities found"
```

### Étape 3 : Smoke test complet
```bash
# 1. Démarrer le serveur
cargo run --release

# 2. Register user
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -d '{"username":"testuser","password":"secure123"}'

# 3. Login
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -d '{"username":"testuser","password":"secure123"}'

# 4. Create listing
curl -X POST http://127.0.0.1:8080/api/listings \
  -d '{"title":"Test","description":"Long desc here","price_xmr":1000,"stock":5,"category":"test"}'

# 5. Rate limiting test
for i in {1..120}; do curl -s http://127.0.0.1:8080/api/health; done
# Doit bloquer après ~100

# 6. Verify all endpoints respond
curl http://127.0.0.1:8080/api/health  # Should be 200 or 429
```

---

## SUMMARY

| Fix | Fichier | Temps | Risque | Impact |
|-----|---------|-------|--------|--------|
| 1. Rate Limiting | main.rs | 5 min | Très bas | Protection DOS |
| 2. refund_funds auth | escrow.rs | 15 min | Haut | Empêche unauthorized refunds |
| 3. resolve_dispute auth | escrow.rs | 15 min | Haut | Empêche non-arbiter disputes |
| 4. cancel_order auth | orders.rs | 10 min | Moyen | Escrow consistency |
| 5. RPC URL validation | escrow.rs | 20 min | Moyen | Bloque URL injection |
| 6. Arbiter password | main.rs | 15 min | Très bas | Operational security |
| 7. Session secret | main.rs | 10 min | Haut | Production safety |

**Total : 6-7h incluant tests et validation**

---

## Commandes rapides pour appliquer les patches

```bash
# Après chaque patch, vérifier compilation
cargo check

# Après tous les patches
cargo build --release

# Tests
cargo test --workspace
cargo audit

# Run server
cargo run --release

# Production
export SESSION_SECRET_KEY="$(openssl rand -base64 48)"
./target/release/server
```

---

**Document créé le : 2025-11-03**
**Source : Plan détaillé fourni par l'utilisateur**
**Version : 1.0**
