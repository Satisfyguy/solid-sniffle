# PATCHES EXACTS - Format str_replace (copy-paste)

**Version : 1.0**
**Date : 2025-11-03**
**Durée estimée : 3-4h pour application**

---

## FIX 1.1 : Décommenter global_rate_limiter - main.rs

**Fichier :** `server/src/main.rs` (ligne ~258)

```
old_str:
            // Global rate limiter (100 req/min per IP)
            // .wrap(global_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ

new_str:
            // Global rate limiter (100 req/min per IP)
            .wrap(global_rate_limiter())
```

**Validation :**
```bash
cargo check
# Doit compiler sans erreur
```

---

## FIX 1.2 : Décommenter protected_rate_limiter - main.rs

**Fichier :** `server/src/main.rs` (ligne ~343)

```
old_str:
                web::scope("/api")
                    // .wrap(protected_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ

new_str:
                web::scope("/api")
                    .wrap(protected_rate_limiter())
```

**Validation :**
```bash
cargo check
# Doit compiler sans erreur
```

---

## FIX 2.1 : Ajouter import db_load_escrow - escrow.rs (TOP)

**Fichier :** `server/src/handlers/escrow.rs` (ligne ~1)

```
old_str:
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

new_str:
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::db::db_load_escrow;
```

**Validation :**
```bash
cargo check
```

---

## FIX 2.2 : Ajouter validation vendor/arbiter dans refund_funds - escrow.rs

**Fichier :** `server/src/handlers/escrow.rs` (fonction refund_funds, ligne ~280)

```
old_str:
    // Refund funds via orchestrator
    match escrow_orchestrator
        .refund_funds(escrow_id, user_id, payload.buyer_address.clone())
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tx_hash": tx_hash,
            "message": "Funds refunded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to refund funds: {}", e)
        })),
    }

new_str:
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
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tx_hash": tx_hash,
            "message": "Funds refunded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to refund funds: {}", e)
        })),
    }
```

**Validation :**
```bash
cargo check
```

---

## FIX 3.1 : Ajouter validation arbiter-only dans resolve_dispute - escrow.rs

**Fichier :** `server/src/handlers/escrow.rs` (fonction resolve_dispute, ligne ~360)

```
old_str:
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
        .resolve_dispute(
            escrow_id,
            user_id,
            &payload.resolution,
            payload.recipient_address.clone(),
        )
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "resolution": &payload.resolution,
            "tx_hash": tx_hash,
            "message": format!("Dispute resolved in favor of {}, funds transferred", &payload.resolution)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to resolve dispute: {}", e)
        })),
    }

new_str:
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
        .resolve_dispute(
            escrow_id,
            user_id,
            &payload.resolution,
            payload.recipient_address.clone(),
        )
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "resolution": &payload.resolution,
            "tx_hash": tx_hash,
            "message": format!("Dispute resolved in favor of {}, funds transferred", &payload.resolution)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to resolve dispute: {}", e)
        })),
    }
```

**Validation :**
```bash
cargo check
```

---

## FIX 4.1 : Ajouter import db_load_escrow - orders.rs (TOP)

**Fichier :** `server/src/handlers/orders.rs` (ligne ~1)

```
old_str:
use crate::db::DbPool;
use crate::middleware::csrf::validate_csrf_token;

new_str:
use crate::db::{DbPool, db_load_escrow};
use crate::middleware::csrf::validate_csrf_token;
```

**Validation :**
```bash
cargo check
```

---

## FIX 4.2 : Ajouter validation buyer dans cancel_order - orders.rs

**Fichier :** `server/src/handlers/orders.rs` (fonction cancel_order, ligne ~620)

```
old_str:
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

        // Get buyer's wallet address for refund

new_str:
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

        // Get buyer's wallet address for refund
```

**Validation :**
```bash
cargo check
```

---

## FIX 5.1 : Ajouter fonction validate_rpc_url - escrow.rs (APRÈS validate_client_role)

**Fichier :** `server/src/handlers/escrow.rs` (ligne ~50)

```
old_str:
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

/// Response for successful wallet registration

new_str:
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

/// Response for successful wallet registration
```

**Validation :**
```bash
cargo check
```

---

## FIX 5.2 : Appliquer validation custom au champ rpc_url - escrow.rs

**Fichier :** `server/src/handlers/escrow.rs` (struct RegisterWalletRpcRequest, ligne ~15)

```
old_str:
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(url(message = "Invalid RPC URL format"))]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,

new_str:
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(custom = "validate_rpc_url")]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,
```

**Validation :**
```bash
cargo check
```

---

## FIX 6.1 : Générer password aléatoire pour arbiter - main.rs

**Fichier :** `server/src/main.rs` (ligne ~150)

```
old_str:
        if arbiter_exists.is_none() {
            info!("No arbiter found, creating system arbiter...");
            let password = "arbiter_system_2024";
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .context("Failed to hash password")?
                .to_string();

new_str:
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
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .context("Failed to hash password")?
                .to_string();
```

**Validation :**
```bash
cargo check
```

---

## FIX 6.2 : Logger le password généré - main.rs

**Fichier :** `server/src/main.rs` (ligne ~175)

```
old_str:
            info!("✅ System arbiter created successfully (username: arbiter_system, password: arbiter_system_2024)");

new_str:
            info!("⚠️  ✅ System arbiter created successfully");
            info!("📋 SAVE THIS IMMEDIATELY - Arbiter credentials:");
            info!("   Username: arbiter_system");
            info!("   Password: {}", password);
            info!("⚠️  This password will NOT be shown again. Change it immediately after first login.");
```

**Validation :**
```bash
cargo check
```

---

## FIX 7.1 : Ajouter panic en production si SESSION_SECRET_KEY manquant - main.rs

**Fichier :** `server/src/main.rs` (ligne ~135)

```
old_str:
    // 4. Session secret key
    // IMPORTANT: In production, load from secure environment variable
    // This should be a 64-byte cryptographically random key
    let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| {
        tracing::warn!("SESSION_SECRET_KEY not set, using development key - NOT FOR PRODUCTION");
        "development_key_do_not_use_in_production_minimum_64_bytes_required".to_string()
    });

new_str:
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

**Validation :**
```bash
cargo check
cargo build --release
# Doit compiler sans erreur
```

---

## Application des Patches - Ordre Recommandé

1. **FIX 1.1 + 1.2** : Rate limiting (rapide, faible risque)
2. **FIX 7.1** : Session secret (rapide, faible risque)
3. **FIX 6.1 + 6.2** : Arbiter password (rapide, faible risque)
4. **FIX 2.1 + 2.2** : Escrow refund auth (critique)
5. **FIX 3.1** : Escrow resolve auth (critique)
6. **FIX 4.1 + 4.2** : Orders cancel auth (critique)
7. **FIX 5.1 + 5.2** : RPC URL validation (critique)

## Validation Finale

Après chaque patch :
```bash
cargo check
```

Après tous les patches :
```bash
cargo build --release
cargo test --workspace
cargo audit
```

---

## Notes d'Application

- **Backup recommandé** avant d'appliquer les patches
- Utiliser `Edit` tool avec les strings exactes (attention aux espaces)
- Vérifier numéros de ligne (peuvent varier légèrement)
- Si old_str introuvable, chercher manuellement et ajuster
- Commit après chaque patch validé (granularité)

---

## Génération Session Secret Key

```bash
# Linux/Mac
export SESSION_SECRET_KEY="$(openssl rand -base64 48)"

# Windows PowerShell
$env:SESSION_SECRET_KEY = [Convert]::ToBase64String((1..48 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
```

---

**Document créé le : 2025-11-03**
**Source : Plan de Sécurisation Backend détaillé**
**Version : 1.0**
