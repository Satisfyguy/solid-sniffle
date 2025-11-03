# PATCH 3 : Escrow Authorization - resolve_dispute

**Fichier cible :** `server/src/handlers/escrow.rs`
**Temps estimé :** 45 minutes
**Risque :** Haut (bug de sécurité critique)
**Impact :** Empêche non-arbiter dispute resolution

---

## Description

**BUG CRITIQUE ACTUEL :**
N'importe quel user peut résoudre N'IMPORTE QUEL dispute, même s'il n'est pas l'arbiter assigné.

**Scénario d'attaque :**
1. Alice (buyer) et Bob (vendor) ont un dispute sur escrow #456
2. Arbiter assigné = Charlie
3. Mallory (attaquant) appelle `/api/escrow/456/resolve` avec `resolution: "buyer"`
4. Système ne vérifie PAS si Mallory == Charlie
5. Dispute résolu ❌ et funds transférés à Alice (alors que Mallory décide!)

**Ce patch ajoute :**
Vérification stricte que user_id == arbiter_id **de cet escrow spécifique**.

---

## Patch 3.1 : Ajouter vérification arbiter-only dans resolve_dispute

**Localisation :** Fonction `resolve_dispute`, ligne ~360

**Note :** L'import `db_load_escrow` a déjà été ajouté dans PATCH 2.

### Code actuel (VULNÉRABLE) :
```rust
pub async fn resolve_dispute(
    path: web::Path<String>,
    payload: web::Json<ResolveDisputeRequest>,
    pool: web::Data<DbPool>,
    session: Session,
    escrow_orchestrator: web::Data<Arc<RwLock<EscrowOrchestrator>>>,
) -> impl Responder {
    // Get user_id from session
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(e) => return e,
    };

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
        .read()
        .await
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
}
```

### Code corrigé (SÉCURISÉ) :
```rust
pub async fn resolve_dispute(
    path: web::Path<String>,
    payload: web::Json<ResolveDisputeRequest>,
    pool: web::Data<DbPool>,
    session: Session,
    escrow_orchestrator: web::Data<Arc<RwLock<EscrowOrchestrator>>>,
) -> impl Responder {
    // Get user_id from session
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(e) => return e,
    };

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
        .read()
        .await
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
}
```

---

## Validation post-patch

### 1. Compilation
```bash
cargo check
# Doit compiler sans erreur
```

### 2. Test unitaire (à créer)
```rust
#[tokio::test]
async fn test_resolve_dispute_wrong_arbiter() {
    // Setup: Create escrow with arbiter_a
    let escrow_id = create_test_escrow_in_dispute("vendor_uuid", "buyer_uuid", "arbiter_a_uuid").await;

    // Test: arbiter_b tries to resolve arbiter_a's dispute → expect 403
    let result = resolve_dispute_as_user(escrow_id, "arbiter_b_uuid", "buyer").await;

    assert_eq!(result.status(), StatusCode::FORBIDDEN);
    assert!(result.body().contains("Only the assigned arbiter can resolve disputes"));
}
```

### 3. Test manuel (avec curl)
```bash
# Setup: Créer escrow en dispute (noter escrow_id et arbiter_id)
ESCROW_ID="xyz789-escrow-in-dispute"
CORRECT_ARBITER_SESSION="session-cookie-of-arbiter-a"
WRONG_USER_SESSION="session-cookie-of-random-user"

# Test 1: Correct arbiter → expect 200 OK
curl -X POST http://127.0.0.1:8080/api/escrow/$ESCROW_ID/resolve \
  -H "Content-Type: application/json" \
  -H "Cookie: monero_marketplace_session=$CORRECT_ARBITER_SESSION" \
  -d '{
    "resolution": "buyer",
    "recipient_address": "9w7Qr8...buyer_address"
  }'
# Expected: {"success":true,"resolution":"buyer","tx_hash":"..."}

# Test 2: Wrong user (not arbiter) → expect 403 Forbidden
curl -X POST http://127.0.0.1:8080/api/escrow/$ESCROW_ID/resolve \
  -H "Content-Type: application/json" \
  -H "Cookie: monero_marketplace_session=$WRONG_USER_SESSION" \
  -d '{
    "resolution": "vendor",
    "recipient_address": "9w7Qr8...vendor_address"
  }'
# Expected: {"error":"Only the assigned arbiter can resolve disputes"}
```

---

## Scénarios de test complets

### Scénario 1 : Arbiter légitime résout dispute ✅
- **Setup :** Escrow en dispute avec arbiter_id = "arbiter_charlie"
- **Action :** User "arbiter_charlie" appelle resolve avec resolution="buyer"
- **Résultat attendu :** 200 OK, funds transférés au buyer

### Scénario 2 : Arbiter résout en faveur du vendor ✅
- **Setup :** Escrow en dispute avec arbiter_id = "arbiter_charlie"
- **Action :** User "arbiter_charlie" appelle resolve avec resolution="vendor"
- **Résultat attendu :** 200 OK, funds transférés au vendor

### Scénario 3 : Autre arbiter tente résolution (ATTAQUE) ❌
- **Setup :** Escrow en dispute avec arbiter_id = "arbiter_charlie"
- **Action :** User "arbiter_mallory" (autre arbiter) appelle resolve
- **Résultat attendu :** 403 Forbidden, "Only the assigned arbiter can resolve disputes"

### Scénario 4 : Buyer tente self-resolution (ATTAQUE) ❌
- **Setup :** Escrow en dispute avec buyer_id = "alice"
- **Action :** User "alice" appelle resolve avec resolution="buyer" (auto-attribution)
- **Résultat attendu :** 403 Forbidden

### Scénario 5 : Vendor tente self-resolution (ATTAQUE) ❌
- **Setup :** Escrow en dispute avec vendor_id = "bob"
- **Action :** User "bob" appelle resolve avec resolution="vendor"
- **Résultat attendu :** 403 Forbidden

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
Edit {
  file_path: "server/src/handlers/escrow.rs"
  old_str: "    // Parse escrow_id from path\n    let escrow_id_str = path.into_inner();\n    let escrow_id = match escrow_id_str.parse::<Uuid>() {\n        Ok(id) => id,\n        Err(_) => {\n            return HttpResponse::BadRequest().json(serde_json::json!({\n                \"error\": \"Invalid escrow_id\"\n            }));\n        }\n    };\n\n    // Resolve dispute via orchestrator\n    match escrow_orchestrator"
  new_str: "    // Parse escrow_id from path\n    let escrow_id_str = path.into_inner();\n    let escrow_id = match escrow_id_str.parse::<Uuid>() {\n        Ok(id) => id,\n        Err(_) => {\n            return HttpResponse::BadRequest().json(serde_json::json!({\n                \"error\": \"Invalid escrow_id\"\n            }));\n        }\n    };\n\n    // Load escrow to verify requester is the assigned arbiter\n    let escrow = match db_load_escrow(&pool, escrow_id).await {\n        Ok(e) => e,\n        Err(_) => {\n            return HttpResponse::BadRequest().json(serde_json::json!({\n                \"error\": \"Escrow not found\"\n            }))\n        }\n    };\n\n    // Verify user is the assigned arbiter\n    if user_id.to_string() != escrow.arbiter_id {\n        return HttpResponse::Forbidden().json(serde_json::json!({\n            \"error\": \"Only the assigned arbiter can resolve disputes\"\n        }));\n    }\n\n    // Resolve dispute via orchestrator\n    match escrow_orchestrator"
}
```

---

## Troubleshooting

### Problème : Arbiter system ne peut pas résoudre disputes
**Cause :** Arbiter system UUID ne match pas escrow.arbiter_id
**Solution :** Vérifier que l'escrow a bien été créé avec le bon arbiter_id :
```rust
// Lors de la création d'escrow
let arbiter_id = get_system_arbiter_id(pool).await?;
```

### Problème : Error "Escrow not found" même si escrow existe
**Cause :** db_load_escrow retourne erreur si escrow pas en status "dispute"
**Solution :** Ne PAS filtrer par status dans db_load_escrow, filtrer après :
```rust
let escrow = db_load_escrow(&pool, escrow_id).await?;

if escrow.status != "dispute" {
    return HttpResponse::BadRequest().json(serde_json::json!({
        "error": "Escrow is not in dispute status"
    }));
}
```

---

## Statut

- [ ] Vérification arbiter ajoutée
- [ ] Compilation OK (`cargo check`)
- [ ] Test unitaire créé et passé
- [ ] Test manuel avec curl passé (arbiter légitime)
- [ ] Test attaque passé (403 pour non-arbiter)
- [ ] Test état escrow vérifié (doit être "dispute")

---

**Créé le :** 2025-11-03
**Difficulté :** Moyenne (⭐⭐⭐☆☆)
**Priorité :** CRITIQUE 🔴
