# PATCH 2 : Escrow Authorization - refund_funds

**Fichier cible :** `server/src/handlers/escrow.rs`
**Temps estimé :** 45 minutes
**Risque :** Haut (bug de sécurité critique)
**Impact :** Empêche unauthorized refunds

---

## Description

**BUG CRITIQUE ACTUEL :**
N'importe quel vendor peut refund N'IMPORTE QUEL escrow, pas juste les siens.

**Scénario d'attaque :**
1. Alice (vendor) crée escrow #123 avec Bob (buyer)
2. Mallory (autre vendor, attaquant) appelle `/api/escrow/123/refund`
3. Système vérifie que Mallory est vendor ✅ (n'importe quel vendor)
4. Refund exécuté ❌ (mais Mallory n'est pas LE vendor de cet escrow!)

**Ce patch ajoute :**
Vérification que le user_id du requester == vendor_id OU arbiter_id **de cet escrow spécifique**.

---

## Patch 2.1 : Ajouter import db_load_escrow

**Localisation :** Top du fichier, après les autres imports (ligne ~1)

### Code à ajouter :
```rust
use crate::db::db_load_escrow;
```

### Code actuel (AVANT) :
```rust
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
```

### Code corrigé (APRÈS) :
```rust
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::db::db_load_escrow;
```

---

## Patch 2.2 : Ajouter vérification vendor/arbiter dans refund_funds

**Localisation :** Fonction `refund_funds`, ligne ~280

### Code actuel (VULNÉRABLE) :
```rust
pub async fn refund_funds(
    escrow_id: web::Path<String>,
    payload: web::Json<RefundRequest>,
    pool: web::Data<DbPool>,
    session: Session,
    escrow_orchestrator: web::Data<Arc<RwLock<EscrowOrchestrator>>>,
) -> impl Responder {
    // Get user_id from session
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // Parse escrow_id
    let escrow_id = match Uuid::parse_str(&escrow_id.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id format"
            }))
        }
    };

    // Refund funds via orchestrator
    match escrow_orchestrator
        .read()
        .await
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
}
```

### Code corrigé (SÉCURISÉ) :
```rust
pub async fn refund_funds(
    escrow_id: web::Path<String>,
    payload: web::Json<RefundRequest>,
    pool: web::Data<DbPool>,
    session: Session,
    escrow_orchestrator: web::Data<Arc<RwLock<EscrowOrchestrator>>>,
) -> impl Responder {
    // Get user_id from session
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // Parse escrow_id
    let escrow_id = match Uuid::parse_str(&escrow_id.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id format"
            }))
        }
    };

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
        .read()
        .await
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
}
```

---

## Validation post-patch

### 1. Compilation
```bash
cargo check
# Doit compiler sans erreur ni warning
```

### 2. Test unitaire (à créer)
```rust
#[tokio::test]
async fn test_refund_funds_wrong_vendor() {
    // Setup: Create escrow with vendor_a
    let escrow_id = create_test_escrow("vendor_a_uuid", "buyer_uuid", "arbiter_uuid").await;

    // Test: vendor_b tries to refund vendor_a's escrow → expect 403
    let result = refund_funds_as_user(escrow_id, "vendor_b_uuid").await;

    assert_eq!(result.status(), StatusCode::FORBIDDEN);
    assert!(result.body().contains("Only vendor or arbiter can refund"));
}
```

### 3. Test manuel (avec curl)
```bash
# Setup: Créer escrow (noter escrow_id et vendor_id)
ESCROW_ID="abc123-escrow-uuid"
CORRECT_VENDOR_SESSION="session-cookie-of-vendor-a"
WRONG_VENDOR_SESSION="session-cookie-of-vendor-b"

# Test 1: Correct vendor → expect 200 OK
curl -X POST http://127.0.0.1:8080/api/escrow/$ESCROW_ID/refund \
  -H "Content-Type: application/json" \
  -H "Cookie: monero_marketplace_session=$CORRECT_VENDOR_SESSION" \
  -d '{"buyer_address":"9w7Qr8...xyz"}'
# Expected: {"success":true,"tx_hash":"..."}

# Test 2: Wrong vendor → expect 403 Forbidden
curl -X POST http://127.0.0.1:8080/api/escrow/$ESCROW_ID/refund \
  -H "Content-Type: application/json" \
  -H "Cookie: monero_marketplace_session=$WRONG_VENDOR_SESSION" \
  -d '{"buyer_address":"9w7Qr8...xyz"}'
# Expected: {"error":"Only vendor or arbiter can refund"}
```

---

## Scénarios de test complets

### Scénario 1 : Vendor légitime refund son propre escrow ✅
- **Setup :** Escrow créé avec vendor_id = "aaa"
- **Action :** User "aaa" appelle refund
- **Résultat attendu :** 200 OK, refund exécuté

### Scénario 2 : Arbiter refund un escrow en dispute ✅
- **Setup :** Escrow créé avec arbiter_id = "bbb"
- **Action :** User "bbb" appelle refund
- **Résultat attendu :** 200 OK, refund exécuté

### Scénario 3 : Autre vendor tente refund (ATTAQUE) ❌
- **Setup :** Escrow créé avec vendor_id = "aaa"
- **Action :** User "zzz" (autre vendor) appelle refund
- **Résultat attendu :** 403 Forbidden, "Only vendor or arbiter can refund"

### Scénario 4 : Buyer tente refund (cas limite) ❌
- **Setup :** Escrow créé avec buyer_id = "ccc"
- **Action :** User "ccc" appelle refund
- **Résultat attendu :** 403 Forbidden (buyer ne peut pas refund directement)

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
Edit {
  file_path: "server/src/handlers/escrow.rs"
  old_str: "use actix_session::Session;\nuse actix_web::{web, HttpResponse, Responder};\nuse serde::{Deserialize, Serialize};"
  new_str: "use actix_session::Session;\nuse actix_web::{web, HttpResponse, Responder};\nuse serde::{Deserialize, Serialize};\nuse crate::db::db_load_escrow;"
}

Edit {
  file_path: "server/src/handlers/escrow.rs"
  old_str: "    // Refund funds via orchestrator\n    match escrow_orchestrator\n        .read()\n        .await\n        .refund_funds(escrow_id, user_id, payload.buyer_address.clone())\n        .await"
  new_str: "    // Load escrow to verify requester is vendor or arbiter\n    let escrow = match db_load_escrow(&pool, escrow_id).await {\n        Ok(e) => e,\n        Err(_) => {\n            return HttpResponse::BadRequest().json(serde_json::json!({\n                \"error\": \"Escrow not found\"\n            }))\n        }\n    };\n\n    // Verify user is vendor or arbiter\n    if user_id.to_string() != escrow.vendor_id && user_id.to_string() != escrow.arbiter_id {\n        return HttpResponse::Forbidden().json(serde_json::json!({\n            \"error\": \"Only vendor or arbiter can refund\"\n        }));\n    }\n\n    // Refund funds via orchestrator\n    match escrow_orchestrator\n        .read()\n        .await\n        .refund_funds(escrow_id, user_id, payload.buyer_address.clone())\n        .await"
}
```

---

## Troubleshooting

### Problème : `db_load_escrow` not found
**Cause :** Fonction non exportée depuis `db/mod.rs`
**Solution :**
```rust
// Dans server/src/db/mod.rs
pub async fn db_load_escrow(pool: &DbPool, escrow_id: Uuid) -> Result<Escrow, diesel::result::Error> {
    use crate::schema::escrows::dsl::*;
    let mut conn = pool.get().expect("Failed to get DB connection");

    escrows
        .filter(id.eq(escrow_id.to_string()))
        .first::<Escrow>(&mut conn)
}
```

### Problème : Arbiter ne peut plus refund
**Cause :** Code oublie de vérifier arbiter_id
**Solution :** Vérifier que le patch inclut bien `|| user_id.to_string() != escrow.arbiter_id`

---

## Statut

- [ ] Import db_load_escrow ajouté
- [ ] Vérification vendor/arbiter ajoutée
- [ ] Compilation OK (`cargo check`)
- [ ] Test unitaire créé et passé
- [ ] Test manuel avec curl passé
- [ ] Scénario attaque testé (403 retourné)

---

**Créé le :** 2025-11-03
**Difficulté :** Moyenne (⭐⭐⭐☆☆)
**Priorité :** CRITIQUE 🔴
