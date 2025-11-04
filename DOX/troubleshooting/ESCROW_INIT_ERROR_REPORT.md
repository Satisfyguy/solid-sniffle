# Rapport de Débogage: Erreur d'Initialisation Escrow

**Date:** 2025-11-04
**Phase:** Phase 6 - Non-Custodial Frontend (QR Code + Educational Messaging)
**Statut:** ⚠️ EN COURS - Erreur CSRF 403

---

## 📋 Contexte Initial

**Objectif:** Implémenter Phase 6 du roadmap non-custodial
- Ajouter QRCode.js pour afficher l'adresse multisig
- Ajouter une bannière éducative expliquant l'architecture non-custodiale
- Permettre au buyer de scanner le QR code pour payer depuis n'importe quel wallet Monero

**Point de départ:** Phases 1-5 complétées, base de données opérationnelle

---

## 🔴 Problème Principal

**Erreur actuelle (2025-11-04 12:55):**
```
POST http://localhost:8080/api/orders/.../init-escrow 403 (Forbidden)
Error: Invalid or missing CSRF token
```

**Fichier:** `static/js/fund-escrow.js:66`

---

## 🔍 Historique des Erreurs

### Erreur #1: Violation CSP (RÉSOLUE ✅)

**Symptôme:**
```
Loading the script 'https://cdnjs.cloudflare.com/ajax/libs/qrcodejs/1.0.0/qrcode.min.js'
violates the following Content Security Policy directive:
"script-src 'self' https://unpkg.com..."
```

**Cause:**
QRCode.js library chargée depuis CDN non autorisé dans la Content Security Policy

**Solution Appliquée:**
- Fichier: `server/src/middleware/security_headers.rs:106`
- Action: Ajout de `https://cdnjs.cloudflare.com` et `https://cdn.jsdelivr.net` à la directive `script-src`
- Compilation: Succès (5m 43s)
- Redémarrage: OK

**Résultat:** ✅ CSP violation résolue, QRCode.js se charge correctement

---

### Erreur #2: Échec Création Escrow (RÉSOLUE ✅)

**Symptôme:**
```
[ERROR] server::handlers::orders: Failed to initialize escrow:
Failed to create escrow in database
```

**Problème:** Message d'erreur trop générique, cause réelle masquée

**Solution #1 - Enhanced Error Logging:**
- Fichier: `server/src/db/mod.rs:69-90`
- Action: Ajout de logs détaillés avec `tracing::error!`
- Code ajouté:
```rust
.map_err(|e| {
    tracing::error!("Database insert error for escrow {}: {:?}", escrow_id, e);
    anyhow::anyhow!("Failed to insert escrow: {}", e)
})?;
```

**Résultat:** ✅ Logs révèlent la vraie erreur:
```
ERROR server::db: Failed to retrieve escrow a385dfb0-... after insert:
DatabaseError(Unknown, "no such column: escrows.buyer_temp_wallet_id")
```

---

### Erreur #3: Colonnes Database Manquantes (RÉSOLUE ✅)

**Symptôme:**
```
DatabaseError(Unknown, "no such column: escrows.buyer_temp_wallet_id")
```

**Cause Racine Identifiée:**
La migration Phase 1 n'a jamais été appliquée à `marketplace.db`

**Colonnes Manquantes:**
- `buyer_temp_wallet_id`
- `vendor_temp_wallet_id`
- `arbiter_temp_wallet_id`

**Vérifications Effectuées:**

1. **Schema Rust:** ✅ Colonnes présentes dans `server/src/schema.rs:25-27`
```rust
buyer_temp_wallet_id -> Nullable<Text>,
vendor_temp_wallet_id -> Nullable<Text>,
arbiter_temp_wallet_id -> Nullable<Text>,
```

2. **Fichier Migration:** ✅ Existe dans `server/migrations/2025-11-03-221723-0000_add_temp_wallet_ids_to_escrows/up.sql`
```sql
ALTER TABLE escrows ADD COLUMN buyer_temp_wallet_id TEXT DEFAULT NULL;
ALTER TABLE escrows ADD COLUMN vendor_temp_wallet_id TEXT DEFAULT NULL;
ALTER TABLE escrows ADD COLUMN arbiter_temp_wallet_id TEXT DEFAULT NULL;
```

3. **Database Réelle:** ❌ Colonnes absentes de `marketplace.db`

**Problème:** Base de données SQLCipher chiffrée - impossible d'appliquer migrations via `diesel migration run`

---

## 🛠️ Solutions Tentées

### Tentative #1: Diesel CLI Standard ❌

**Commande:**
```bash
DATABASE_URL=marketplace.db diesel migration run
```

**Résultat:** ÉCHEC
- Raison: Database chiffrée avec SQLCipher
- Diesel CLI ne peut pas se connecter sans clé de chiffrement

---

### Tentative #2: Création Database Fresh ❌

**Plan:**
1. Backup: `mv marketplace.db marketplace.db.backup`
2. Laisser le serveur créer une nouvelle database
3. Appliquer toutes les migrations

**Résultat:** ÉCHEC
- Le serveur ne crée PAS automatiquement les tables
- Erreur: `no such table: users`
- Database restaurée: `mv marketplace.db.backup marketplace.db`

---

### Tentative #3: Utilitaire Rust Personnalisé ✅

**Solution Finale Appliquée:**

1. **Création de l'utilitaire:** `server/src/bin/apply_migration.rs`
   - Se connecte à la database avec la clé de chiffrement
   - Exécute les ALTER TABLE directement via `diesel::sql_query()`
   - Crée les indexes de performance

2. **Configuration:** Ajout dans `server/Cargo.toml`
```toml
[[bin]]
name = "apply_migration"
path = "src/bin/apply_migration.rs"
```

3. **Compilation:**
```bash
cargo build --release --bin apply_migration
```
Durée: 9.12s
Warnings: 1 (dead code `DbPool` - non critique)

4. **Exécution:**
```bash
./target/release/apply_migration
```

**Résultat de l'exécution:**
```
🔧 Phase 1 Migration Utility - Adding temp wallet columns
======================================================================
📂 Database: /home/malix/Desktop/monero.marketplace/marketplace.db
🔐 Using encryption key from DB_ENCRYPTION_KEY environment variable
✅ Successfully connected to encrypted database

🔍 Checking if columns already exist...
📝 Columns do not exist - proceeding with migration...

🔨 Step 1/4: Adding buyer_temp_wallet_id column...
   ✅ buyer_temp_wallet_id added
🔨 Step 2/4: Adding vendor_temp_wallet_id column...
   ✅ vendor_temp_wallet_id added
🔨 Step 3/4: Adding arbiter_temp_wallet_id column...
   ✅ arbiter_temp_wallet_id added
🔨 Step 4/4: Creating indexes for performance...
   ✅ idx_escrows_buyer_temp_wallet created
   ✅ idx_escrows_vendor_temp_wallet created
   ✅ idx_escrows_arbiter_temp_wallet created

======================================================================
🎉 MIGRATION COMPLETED SUCCESSFULLY!

✅ All Phase 1 temp wallet columns added:
   • buyer_temp_wallet_id
   • vendor_temp_wallet_id
   • arbiter_temp_wallet_id

✅ All indexes created for performance

🚀 You can now restart the server and escrow initialization will work!
======================================================================
```

**Résultat:** ✅ Migration Phase 1 appliquée avec succès!

---

## 🔄 Actions de Correction Appliquées

### 1. Réactivation du CSRF Backend ✅

**Fichier:** `server/src/handlers/orders.rs:964-976`

**Action:** Réactivation de la validation CSRF (était temporairement désactivée pour debugging)

**Avant (commenté):**
```rust
// TEMPORARY: CSRF validation disabled for database debugging
// TODO: Re-enable after fixing database issue
// let csrf_token = http_req
//     .headers()
//     .get("X-CSRF-Token")
//     .and_then(|h| h.to_str().ok())
//     .unwrap_or("");
//
// if !validate_csrf_token(&session, csrf_token) {
//     return HttpResponse::Forbidden().json(serde_json::json!({
//         "error": "Invalid or missing CSRF token"
//     }));
// }
```

**Après (réactivé):**
```rust
let csrf_token = http_req
    .headers()
    .get("X-CSRF-Token")
    .and_then(|h| h.to_str().ok())
    .unwrap_or("");

if !validate_csrf_token(&session, csrf_token) {
    return HttpResponse::Forbidden().json(serde_json::json!({
        "error": "Invalid or missing CSRF token"
    }));
}
```

**Compilation:** Succès (5m 43s)

---

### 2. Redémarrage du Serveur ✅

**Commandes:**
```bash
pkill -9 server
killall -9 server
cargo build --release --package server
./target/release/server > server.log 2>&1 &
```

**Statut:** ✅ Serveur redémarré avec:
- Phase 1 migration appliquée
- CSP fix actif
- CSRF réactivé
- Enhanced error logging

---

## ⚠️ Problème Actuel (NON RÉSOLU)

### Erreur CSRF 403 Forbidden

**Symptôme:**
```
POST http://localhost:8080/api/orders/a6981078.../init-escrow 403 (Forbidden)
Error: Invalid or missing CSRF token
```

**Cause Probable:**
Le frontend a toujours le code CSRF **commenté** dans `static/js/fund-escrow.js`

**Code Frontend Actuel (lignes 60-70):**
```javascript
// TEMPORARY: CSRF check disabled for testing database error
// const csrfToken = getCsrfToken();
// if (!csrfToken) {
//     throw new Error('CSRF token not found. Please refresh the page.');
// }

const response = await fetch(`/api/orders/${orderId}/init-escrow`, {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
        // 'X-CSRF-Token': csrfToken
    }
});
```

**Problème:**
- Backend attend maintenant un CSRF token valide
- Frontend n'envoie PLUS le token (commenté pour debugging)
- Résultat: 403 Forbidden

---

## 🔧 Solution Nécessaire

### Action Requise: Réactiver CSRF Frontend

**Fichier:** `static/js/fund-escrow.js:60-70`

**Modifications nécessaires:**
1. Décommenter `getCsrfToken()`
2. Décommenter la vérification du token
3. Ajouter le header `X-CSRF-Token` dans la requête fetch

**Code Corrigé Attendu:**
```javascript
const csrfToken = getCsrfToken();
if (!csrfToken) {
    throw new Error('CSRF token not found. Please refresh the page.');
}

const response = await fetch(`/api/orders/${orderId}/init-escrow`, {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken
    }
});
```

---

## 📊 État des Composants

| Composant | Statut | Notes |
|-----------|--------|-------|
| CSP Headers | ✅ OK | cdnjs.cloudflare.com + cdn.jsdelivr.net autorisés |
| QRCode.js Load | ✅ OK | Se charge sans erreur CSP |
| Database Schema | ✅ OK | Colonnes temp_wallet présentes |
| Migration Phase 1 | ✅ OK | Appliquée manuellement via utility |
| CSRF Backend | ✅ OK | Validation active |
| CSRF Frontend | ❌ KO | **Code commenté - doit être réactivé** |
| Error Logging | ✅ OK | Logs détaillés actifs |
| Server Status | ✅ OK | En fonctionnement |

---

## 🎯 Prochaines Étapes

### Immédiat (Pour Résoudre 403)

1. **Réactiver CSRF Frontend:**
   - Fichier: `static/js/fund-escrow.js:60-70`
   - Action: Décommenter le code CSRF
   - Rebuild: Non nécessaire (fichier statique)
   - Test: Recharger la page + hard refresh

2. **Vérifier Génération Token:**
   - S'assurer que `getCsrfToken()` retourne un token valide
   - Vérifier que le token est présent dans le DOM/cookie

3. **Test Complet:**
   - Naviguer vers checkout
   - Initialiser escrow
   - Vérifier logs serveur

### Si Échec Persiste

1. **Debug Token Generation:**
   - Ajouter `console.log()` dans `getCsrfToken()`
   - Vérifier présence du meta tag CSRF
   - Vérifier cookie de session

2. **Vérifier Session:**
   - S'assurer que l'utilisateur est authentifié
   - Vérifier que la session n'a pas expiré

---

## 📝 Leçons Apprises

### 1. SQLCipher et Migrations
**Problème:** Diesel CLI standard ne fonctionne pas avec SQLCipher
**Solution:** Créer un utilitaire Rust personnalisé qui utilise la même connection pool que le serveur

### 2. Enhanced Logging Crucial
**Problème:** Message d'erreur générique masquait la vraie cause
**Solution:** Ajouter `tracing::error!` avec détails complets de l'erreur Diesel

### 3. Debugging Temporaire Oublié
**Problème:** Code CSRF commenté pour debugging, puis oublié
**Solution:** Toujours documenter les changements temporaires avec TODO et date

### 4. CSP Progressive
**Problème:** Ajout progressif de CDNs selon les besoins
**Solution:** Maintenir une liste centralisée des CDNs autorisés dans CLAUDE.md

---

## 🔗 Fichiers Modifiés

### Phase 6 Implementation
- `templates/checkout/index.html:265-280` - QR Code container + banner
- `templates/checkout/index.html:528` - QRCode.js script tag
- `static/js/checkout.js:410-454` - QR code generation logic

### CSP Fix
- `server/src/middleware/security_headers.rs:106` - Ajout CDNs

### Database Fix
- `server/src/bin/apply_migration.rs` - **NOUVEAU** - Utilitaire migration
- `server/Cargo.toml:66-68` - Configuration binary

### Error Logging
- `server/src/db/mod.rs:69-90` - Enhanced logging

### CSRF (Réactivé)
- `server/src/handlers/orders.rs:964-976` - Backend validation active
- `static/js/fund-escrow.js:60-70` - ⚠️ **Frontend toujours commenté**

---

## 📞 Contact & Références

- **Migration Script:** `./target/release/apply_migration`
- **Server Logs:** `tail -f server.log`
- **Database:** `marketplace.db` (SQLCipher encrypted)
- **Encryption Key:** `.env:DB_ENCRYPTION_KEY`

---

**Dernière Mise à Jour:** 2025-11-04 12:55 UTC
**Statut Global:** 🟡 Migration résolue, CSRF frontend à réactiver
**Blocage Actuel:** 403 Forbidden - CSRF token missing in frontend request
