# PATCH 4 : Orders Authorization - cancel_order

**Fichier cible :** `server/src/handlers/orders.rs`
**Temps estimé :** 30 minutes
**Risque :** Moyen
**Impact :** Assure consistency escrow-order

---

## Description

**PROBLÈME ACTUEL :**
Quand un order est cancelled avec refund, le code ne vérifie PAS que le user qui cancelle est bien le buyer de l'escrow associé.

**Scénario problématique :**
1. Alice (buyer) crée order #123, funded dans escrow #456
2. Bob (autre buyer) appelle `/api/orders/123/cancel`
3. Système vérifie que Bob est authentifié ✅
4. Système initie refund via escrow #456
5. **BUG :** Bob n'est pas le buyer de escrow #456, mais le refund est tenté quand même

**Ce patch ajoute :**
Vérification que le user qui cancelle est bien le buyer de l'escrow (si order est funded).

---

## Patch 4.1 : Ajouter import db_load_escrow

**Localisation :** Top du fichier, dans les imports (ligne ~5)

### Code actuel (AVANT) :
```rust
use crate::db::DbPool;
use crate::middleware::csrf::validate_csrf_token;
```

### Code corrigé (APRÈS) :
```rust
use crate::db::{DbPool, db_load_escrow};
use crate::middleware::csrf::validate_csrf_token;
```

---

## Patch 4.2 : Ajouter vérification buyer dans cancel_order

**Localisation :** Fonction `cancel_order`, ligne ~620 (section "Check if order is funded")

### Code actuel (INCOMPLET) :
```rust
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
```

### Code corrigé (COMPLET) :
```rust
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
async fn test_cancel_order_wrong_buyer() {
    // Setup: Create funded order with buyer_a
    let order_id = create_test_funded_order("buyer_a_uuid", "vendor_uuid").await;

    // Test: buyer_b tries to cancel buyer_a's order → expect 403
    let result = cancel_order_as_user(order_id, "buyer_b_uuid").await;

    assert_eq!(result.status(), StatusCode::FORBIDDEN);
    assert!(result.body().contains("Only the buyer can cancel this order"));
}
```

### 3. Test manuel (avec curl)
```bash
# Setup: Créer funded order (noter order_id, buyer_id, escrow_id)
ORDER_ID="order_abc123"
CORRECT_BUYER_SESSION="session-cookie-of-buyer-a"
WRONG_BUYER_SESSION="session-cookie-of-buyer-b"

# Test 1: Correct buyer → expect 200 OK (cancel + refund)
curl -X POST http://127.0.0.1:8080/api/orders/$ORDER_ID/cancel \
  -H "Content-Type: application/json" \
  -H "Cookie: monero_marketplace_session=$CORRECT_BUYER_SESSION" \
  -d '{
    "csrf_token": "your-csrf-token"
  }'
# Expected: {"success":true,"message":"Order cancelled and refund initiated"}

# Test 2: Wrong buyer → expect 403 Forbidden
curl -X POST http://127.0.0.1:8080/api/orders/$ORDER_ID/cancel \
  -H "Content-Type: application/json" \
  -H "Cookie: monero_marketplace_session=$WRONG_BUYER_SESSION" \
  -d '{
    "csrf_token": "valid-csrf-token-for-buyer-b"
  }'
# Expected: {"error":"Only the buyer can cancel this order"}
```

---

## Scénarios de test complets

### Scénario 1 : Buyer légitime cancelle son funded order ✅
- **Setup :** Order funded avec buyer_id = "alice", escrow #123
- **Action :** User "alice" appelle cancel_order
- **Résultat attendu :** 200 OK, order cancelled, refund initié

### Scénario 2 : Buyer cancelle pending order (no escrow) ✅
- **Setup :** Order pending (status = "pending"), pas d'escrow
- **Action :** User (buyer) appelle cancel_order
- **Résultat attendu :** 200 OK, order cancelled (no refund needed)

### Scénario 3 : Autre buyer tente cancel funded order (ATTAQUE) ❌
- **Setup :** Order funded avec buyer_id = "alice"
- **Action :** User "bob" (autre buyer) appelle cancel_order
- **Résultat attendu :** 403 Forbidden, "Only the buyer can cancel this order"

### Scénario 4 : Vendor tente cancel order ❌
- **Setup :** Order funded avec vendor_id = "vendor_xyz"
- **Action :** User "vendor_xyz" appelle cancel_order
- **Résultat attendu :** 403 Forbidden (vendors can't cancel, only ship/dispute)

---

## Cas limites importants

### Cas 1 : Order shipped → cancel impossible
- **Status order :** "shipped"
- **Résultat attendu :** 400 Bad Request, "Cannot cancel shipped order"
- **Check existant :** À vérifier dans le code actuel

### Cas 2 : Order completed → cancel impossible
- **Status order :** "completed"
- **Résultat attendu :** 400 Bad Request, "Cannot cancel completed order"

### Cas 3 : Escrow en dispute → cancel impossible
- **Status escrow :** "dispute"
- **Résultat attendu :** 400 Bad Request, "Cannot cancel order in dispute"

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
Edit {
  file_path: "server/src/handlers/orders.rs"
  old_str: "use crate::db::DbPool;\nuse crate::middleware::csrf::validate_csrf_token;"
  new_str: "use crate::db::{DbPool, db_load_escrow};\nuse crate::middleware::csrf::validate_csrf_token;"
}

Edit {
  file_path: "server/src/handlers/orders.rs"
  old_str: "        let escrow_uuid = match Uuid::parse_str(escrow_id_str) {\n            Ok(uuid) => uuid,\n            Err(_) => {\n                return HttpResponse::InternalServerError().json(serde_json::json!({\n                    \"error\": \"Invalid escrow ID format\"\n                }))\n            }\n        };\n\n        // Get buyer's wallet address for refund"
  new_str: "        let escrow_uuid = match Uuid::parse_str(escrow_id_str) {\n            Ok(uuid) => uuid,\n            Err(_) => {\n                return HttpResponse::InternalServerError().json(serde_json::json!({\n                    \"error\": \"Invalid escrow ID format\"\n                }))\n            }\n        };\n\n        // Load escrow and verify buyer is the one cancelling\n        let escrow = match db_load_escrow(&pool, escrow_uuid).await {\n            Ok(e) => e,\n            Err(_) => {\n                return HttpResponse::BadRequest().json(serde_json::json!({\n                    \"error\": \"Escrow not found\"\n                }))\n            }\n        };\n\n        if escrow.buyer_id != user_id.to_string() {\n            return HttpResponse::Forbidden().json(serde_json::json!({\n                \"error\": \"Only the buyer can cancel this order\"\n            }));\n        }\n\n        // Get buyer's wallet address for refund"
}
```

---

## Troubleshooting

### Problème : user_id est Uuid mais escrow.buyer_id est String
**Cause :** Mismatch de types
**Solution :** Convertir avec `.to_string()` comme dans le patch :
```rust
if escrow.buyer_id != user_id.to_string() { ... }
```

### Problème : Buyer ne peut plus cancel pending orders
**Cause :** Check appliqué même quand pas de refund
**Solution :** Vérifier que le check est bien DANS le `if needs_refund { ... }` block

---

## Statut

- [ ] Import db_load_escrow ajouté
- [ ] Vérification buyer ajoutée
- [ ] Compilation OK (`cargo check`)
- [ ] Test unitaire créé et passé
- [ ] Test manuel passé (buyer légitime)
- [ ] Test attaque passé (403 pour wrong buyer)
- [ ] Cas limites testés (shipped/completed orders)

---

**Créé le :** 2025-11-03
**Difficulté :** Facile-Moyenne (⭐⭐☆☆☆)
**Priorité :** MOYENNE ⚠️
