## Spec: create_order_from_cart

### Objectif
Créer une commande (Order) à partir du panier du buyer avec chiffrement AES-256-GCM de l'adresse de livraison

### Préconditions
- [ ] User authentifié avec session valide
- [ ] User possède le rôle "buyer"
- [ ] Panier (Cart) non vide dans la session
- [ ] Tous les items du panier proviennent du même vendeur (single-vendor cart)
- [ ] Clé de chiffrement AES-256-GCM disponible dans app_data
- [ ] Base de données accessible (DbPool)

### Input
```rust
pool: web::Data<DbPool>,                        // Connection pool database
session: Session,                                // Session actix pour auth + cart
http_req: HttpRequest,                           // Pour extraire CSRF token
req: web::Json<CreateOrderFromCartRequest>,      // Body JSON avec shipping address
websocket: web::Data<Addr<WebSocketServer>>,    // Pour notif vendor
encryption_key: web::Data<Vec<u8>>,              // Clé AES-256 (32 bytes)

// Structure CreateOrderFromCartRequest:
pub struct CreateOrderFromCartRequest {
    pub checkout_mode: String,                   // "cart" | "listing"

    #[validate(length(min = 10, max = 500))]
    pub shipping_address: String,                // Adresse postale (JSON)

    #[validate(length(max = 200))]
    pub shipping_notes: Option<String>,          // Instructions livraison
}
```

### Output
```rust
HttpResponse // JSON response
// Success (201 Created):
{
    "success": true,
    "order_id": "uuid-string",
    "total_xmr": 1000000000000,  // piconeros
    "message": "Order created successfully"
}

// Error (400/403/500):
{
    "error": "Message d'erreur explicite"
}
```

### Erreurs Possibles
- **400 BadRequest** "Validation error: ..." - Champs invalides (trop courts/longs)
- **400 BadRequest** "Cart is empty" - Panier vide dans session
- **400 BadRequest** "Multi-vendor carts not yet supported" - Items de vendors différents
- **400 BadRequest** "Cannot purchase your own listings" - Self-purchase attempt
- **400 BadRequest** "Invalid order total" - Total <= 0
- **400 BadRequest** "Order total exceeds maximum allowed value" - Total > 10,000 XMR
- **403 Forbidden** "Invalid or missing CSRF token" - Token CSRF invalide
- **403 Forbidden** "Buyer role required to create orders" - Pas de rôle buyer
- **500 InternalServerError** "Failed to encrypt shipping address" - Chiffrement échoué
- **500 InternalServerError** "Database connection failed" - Pool DB indisponible
- **500 InternalServerError** "Failed to create order" - INSERT order échoué

### Dépendances
```toml
[dependencies]
actix-web = "4"
actix-session = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
validator = { version = "0.18", features = ["derive"] }
diesel = { version = "2", features = ["sqlite", "r2d2"] }
uuid = { version = "1", features = ["v4"] }
base64 = "0.22"                  # Pour encoder bytes chiffrés
tracing = "0.1"

# Internal dependencies
crate::crypto::encryption         # encrypt_field()
crate::db::DbPool
crate::middleware::csrf          # validate_csrf_token()
crate::models::cart::Cart
crate::models::order::{NewOrder, Order, OrderStatus}
crate::websocket::{NotifyUser, WebSocketServer, WsEvent}
```

### Flux de Traitement

**Étape 1: Validation CSRF + Authentification**
- Extraire CSRF token du header X-CSRF-Token
- Valider token avec session
- Extraire buyer_id depuis session
- Vérifier rôle "buyer"

**Étape 2: Validation Input**
- Valider CreateOrderFromCartRequest avec validator crate
- Contraintes: shipping_address 10-500 chars, shipping_notes max 200 chars

**Étape 3: Récupération & Validation Cart**
- Lire Cart depuis session
- Vérifier non-vide
- Valider single-vendor (tous items même vendor_id)
- Vérifier pas de self-purchase (vendor_id != buyer_id)
- Calculer total_xmr = sum(item.unit_price_xmr * item.quantity)
- Valider total > 0 et < 10,000 XMR (10^16 piconeros)

**Étape 4: Chiffrement Adresse** (CRITICAL SECURITY)
- Appeler encrypt_field(&req.shipping_address, &encryption_key)
- AES-256-GCM avec nonce aléatoire (12 bytes)
- Résultat: [nonce][ciphertext][auth_tag] (format brut)
- Encoder en base64 pour stockage DB
- En cas d'erreur de chiffrement → HTTP 500

**Étape 5: Création Order en DB**
- Obtenir connection DB depuis pool
- Créer NewOrder avec:
  - id: UUID v4
  - buyer_id, vendor_id (du cart)
  - listing_id: premier item du cart (pour compatibilité)
  - escrow_id: None (set lors de init_escrow)
  - status: "pending"
  - total_xmr: calculé
  - shipping_address: Some(encrypted_base64)
  - shipping_notes: Option<String>
- INSERT via Order::create()

**Étape 6: Post-Processing**
- Vider le cart: cart.clear() + session.insert("cart", &cart)
- Logger: order_id, buyer_id, vendor_id, total (NO shipping address in logs)
- Envoyer WebSocket notification au vendor (OrderStatusChanged event)

**Étape 7: Response**
- HTTP 201 Created
- JSON avec order_id, total_xmr, success message

### Test de Validation (Shell)

```bash
# Prerequisites
# 1. Server running on localhost:8080
# 2. User logged in as buyer with valid session cookie
# 3. Cart contains at least 1 item

# Test 1: Succès nominal avec adresse valide
curl -X POST http://127.0.0.1:8080/api/orders/create \
  -H "Content-Type: application/json" \
  -H "Cookie: session_cookie_here" \
  -H "X-CSRF-Token: csrf_token_here" \
  -d '{
    "checkout_mode": "cart",
    "shipping_address": "{\"street\":\"123 Rue Test\",\"city\":\"Paris\",\"postal_code\":\"75001\",\"country\":\"France\"}",
    "shipping_notes": "Sonnez 3 fois"
  }'

# Expected output:
# {"success":true,"order_id":"uuid-v4","total_xmr":1000000000000,"message":"Order created successfully"}
# Status code: 201 Created

# Test 2: Échec - Adresse trop courte (< 10 chars)
curl -X POST http://127.0.0.1:8080/api/orders/create \
  -H "Content-Type: application/json" \
  -H "Cookie: session_cookie_here" \
  -H "X-CSRF-Token: csrf_token_here" \
  -d '{
    "checkout_mode": "cart",
    "shipping_address": "short",
    "shipping_notes": null
  }'

# Expected output:
# {"error":"Validation error: shipping_address: Shipping address must be between 10 and 500 characters"}
# Status code: 400 Bad Request

# Test 3: Échec - Cart vide
curl -X POST http://127.0.0.1:8080/api/orders/create \
  -H "Content-Type: application/json" \
  -H "Cookie: session_cookie_here_with_empty_cart" \
  -H "X-CSRF-Token: csrf_token_here" \
  -d '{
    "checkout_mode": "cart",
    "shipping_address": "123 Valid Street Address Here",
    "shipping_notes": null
  }'

# Expected output:
# {"error":"Cart is empty"}
# Status code: 400 Bad Request

# Test 4: Échec - CSRF token invalide
curl -X POST http://127.0.0.1:8080/api/orders/create \
  -H "Content-Type: application/json" \
  -H "Cookie: session_cookie_here" \
  -H "X-CSRF-Token: invalid_token" \
  -d '{
    "checkout_mode": "cart",
    "shipping_address": "123 Valid Street",
    "shipping_notes": null
  }'

# Expected output:
# {"error":"Invalid or missing CSRF token"}
# Status code: 403 Forbidden

# Verification: L'adresse est chiffrée en DB
sqlite3 marketplace.db "SELECT id, shipping_address FROM orders LIMIT 1;"
# Output doit montrer une chaîne base64, PAS du plaintext
# Exemple: uuid|dGVzdCBlbmNyeXB0ZWQgZGF0YQ==...
```

### Security Checklist
- [x] CSRF token validation
- [x] Buyer role authorization
- [x] Input validation (length constraints)
- [x] AES-256-GCM field-level encryption pour shipping_address
- [x] No .unwrap() or .expect() (proper Result<> error handling)
- [x] No sensitive data in logs (shipping address never logged plaintext)
- [x] Integer overflow protection (checked_mul for totals)
- [x] Maximum order value limit (10,000 XMR)
- [x] Self-purchase prevention
- [x] Session-based cart (no unauthorized access)

### Performance Considerations
- Cart read from session (fast, in-memory)
- Single DB INSERT (Order::create)
- AES-256-GCM encryption: ~5-10µs (negligible)
- WebSocket notification: async, non-blocking
- **Estimated response time**: < 50ms

### Estimation
- Code: 60 min (already implemented)
- Test: 30 min
- Security review: 20 min
- Documentation: 30 min (this spec)
- **Total**: 140 min

### Status
- [x] Spec validée
- [x] Code écrit (server/src/handlers/orders.rs lines 111-311)
- [x] Chiffrement AES-256-GCM implémenté
- [x] CSRF protection active
- [x] Client-side logs sanitized
- [ ] Reality check Tor (N/A - pas d'appels réseau externes dans cette fonction)
- [ ] Tests unitaires ajoutés
- [ ] Tests E2E avec vraie session
- [ ] Production-ready validation > 80/100

### Related Documentation
- [CLAUDE.md](../../CLAUDE.md) - Security patterns & encryption requirements
- [server/src/crypto/encryption.rs](../../server/src/crypto/encryption.rs) - AES-256-GCM implementation
- [server/src/models/order.rs](../../server/src/models/order.rs) - Order & NewOrder structs
- [server/src/models/cart.rs](../../server/src/models/cart.rs) - Cart struct

### Future Enhancements
- [ ] Support multi-vendor carts (separate orders per vendor) - See line 177 TODO
- [ ] Vendor-specific encryption keys (currently uses global encryption_key)
- [ ] Stock reservation during order creation (currently only in create_order)
- [ ] Automatic cart expiration after X hours
