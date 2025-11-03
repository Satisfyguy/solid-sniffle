# Bug Analysis: Wallet Address Not Persisting in Database

**Date:** 2025-11-03
**Severity:** HIGH
**Component:** Settings page - Wallet address update
**Status:** Under investigation

---

## 🔴 SYMPTÔME

L'utilisateur (vendor) tente de sauvegarder son adresse Monero wallet dans Settings:
1. Va sur http://127.0.0.1:8080/settings
2. Entre une adresse Monero valide (95-106 caractères)
3. Clique "SAVE WALLET ADDRESS"
4. **La page se rafraîchit mais l'adresse disparaît** - pas de persistence

**Impact:** Le vendor ne peut pas marquer les commandes comme "shipped" car le backend vérifie que `wallet_address IS NOT NULL` avant d'autoriser le shipping.

---

## 🔍 INVESTIGATION MENÉE

### 1. Vérification Réseau (Chrome DevTools)

```
Request:
POST http://127.0.0.1:8080/api/settings/update-wallet
Status: 200 OK
Response Size: 116 bytes
Content-Type: application/json
```

**Observation critique:** La réponse fait **116 bytes** et est en JSON, alors qu'elle devrait faire ~250 bytes et être en HTML.

### 2. Analyse du Code Backend

**Fichier:** `server/src/handlers/auth.rs:432-519`

```rust
#[post("/update-wallet")]
pub async fn update_wallet_address(
    pool: web::Data<DbPool>,
    req: web::Form<UpdateWalletRequest>,
    http_req: HttpRequest,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    use diesel::prelude::*;
    use crate::schema::users;

    // LIGNE 442: Détection HTMX
    let is_htmx = is_htmx_request(&http_req);

    // Authentification
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return if is_htmx {
                Ok(htmx_error_response("Not authenticated"))
            } else {
                Err(ApiError::Unauthorized("Not authenticated".to_string()))
            };
        }
    };

    // Validation CSRF token (ligne 457)
    if !validate_csrf_token(&session, &req.csrf_token) {
        return if is_htmx {
            Ok(htmx_error_response("Invalid CSRF token"))
        } else {
            Err(ApiError::Forbidden("Invalid CSRF token".to_string()))
        };
    }

    // Validation format adresse Monero (ligne 466)
    if !is_valid_monero_address(&req.wallet_address) {
        return if is_htmx {
            Ok(htmx_error_response("Invalid Monero address format..."))
        } else {
            Err(ApiError::Internal("Invalid Monero address format".to_string()))
        };
    }

    // LIGNE 480-489: DEBUG logs (ajoutés pour investigation)
    info!("DEBUG: Attempting to update wallet for user_id: {}", uid);
    info!("DEBUG: Wallet address to save: {}", wallet_addr);

    // UPDATE DATABASE
    let update_result = web::block(move || -> Result<usize, diesel::result::Error> {
        let rows_affected = diesel::update(users::table.filter(users::id.eq(&uid)))
            .set(users::wallet_address.eq(Some(&wallet_addr)))
            .execute(&mut conn)?;

        info!("DEBUG: Rows affected by UPDATE: {}", rows_affected);
        Ok(rows_affected)
    }).await;

    match update_result {
        Ok(Ok(rows_affected)) => {
            if rows_affected == 0 {
                error!("CRITICAL: UPDATE affected 0 rows! User ID not found: {}", user_id);
                return if is_htmx {
                    Ok(htmx_error_response("User not found in database"))
                } else {
                    Err(ApiError::Internal("User not found".to_string()))
                };
            }

            // LIGNE 509-518: Réponse conditionnelle
            if is_htmx {
                // ~250 bytes HTML response
                Ok(HttpResponse::Ok().content_type("text/html").body(
                    r#"<div class="alert alert-success">
                        ✅ Wallet address updated successfully!
                    </div>"#
                ))
            } else {
                // 116 bytes JSON response
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Wallet address updated successfully"
                })))
            }
        }
    }
}
```

**Problème identifié:** Le backend retourne la branche `else` (JSON 116 bytes), ce qui signifie que **`is_htmx = false`**.

### 3. Fonction de Détection HTMX

```rust
fn is_htmx_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|h| h.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}
```

**Conclusion:** Le header `HX-Request: true` n'est PAS envoyé par le navigateur.

### 4. Vérification du Template HTML

**Fichier:** `templates/settings.html:10`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Settings - NEXUS</title>
    <meta name="description" content="...">
    <link rel="icon" href="/static/favicon.ico" type="image/x-icon">
    <link rel="stylesheet" href="/static/css/main.css">
    <!-- LIGNE 10: HTMX library -->
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <style>
        /* ... */
    </style>
</head>
```

**Formulaire (lignes 198-208):**

```html
<form
    hx-post="/api/settings/update-wallet"
    hx-target="#wallet-response"
    hx-swap="innerHTML"
    hx-indicator=".htmx-indicator"
    class="settings-form"
>
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">

    <div class="form-group">
        <label for="wallet_address">Monero Wallet Address</label>
        <input
            type="text"
            id="wallet_address"
            name="wallet_address"
            placeholder="9wviCeWe2D8..."
            value="{{ wallet_address | default(value='') }}"
            required
        >
    </div>

    <button type="submit" class="btn-primary">
        SAVE WALLET ADDRESS
    </button>
</form>

<div id="wallet-response"></div>
```

**Observation:** Le template HTML source contient bien:
- ✅ La balise `<script src="https://unpkg.com/htmx.org@1.9.10"></script>`
- ✅ Les attributs HTMX sur le formulaire: `hx-post`, `hx-target`, `hx-swap`

### 5. Vérification du HTML Servi par le Serveur

```bash
$ curl -s http://127.0.0.1:8080/settings | grep -c "htmx.org"
0
```

**🚨 PROBLÈME CRITIQUE TROUVÉ:** Le serveur ne retourne PAS la balise HTMX dans le HTML!

Même après:
- Modification du fichier template
- Recompilation: `cargo build --release --bin server`
- Redémarrage du serveur
- Vérification que le binaire est frais: `stat -c "%y" target/release/server`

**Le HTML servi ne contient toujours pas HTMX.**

### 6. Logs Serveur

```
[2025-11-03T17:43:21.668609Z] INFO actix_web::middleware::logger: 127.0.0.1 "GET /settings HTTP/1.1" 302 0
[2025-11-03T17:29:03.950532Z] INFO actix_web::middleware::logger: 127.0.0.1 "POST /api/settings/update-wallet HTTP/1.1" 200 116
```

**Observations:**
- GET /settings → **302 redirect** (probablement vers login, mais curl ne suit pas les redirects)
- POST /update-wallet → **200 OK 116 bytes** (JSON, pas HTML)
- ❌ **Les logs DEBUG ne s'affichent JAMAIS** - le code ne va jamais jusqu'à la partie UPDATE database

---

## 🧩 HYPOTHÈSES

### Hypothèse #1: Redirection 302 (CONFIRMÉE ✅)
Le serveur retourne un **302 redirect** sur `/settings` quand curl est utilisé (pas de cookies de session).

**Preuve:**
```bash
$ curl -s http://127.0.0.1:8080/settings
# Empty response - redirected to /login
```

**Impact:** Impossible de vérifier le HTML servi avec curl sans suivre les redirects et avoir une session valide.

### Hypothèse #2: Tera Template Caching (PROBABLE ⚠️)
Tera charge les templates au runtime avec `Tera::new("templates/**/*.html")`, mais peut-être que:
- Le serveur est démarré AVANT la dernière compilation
- Plusieurs processus serveur tournent simultanément
- Le template est en cache quelque part

**Vérification effectuée:**
```bash
$ pkill -9 server; killall -9 server
$ ps aux | grep "[t]arget/release/server"  # Aucun résultat
$ ./target/release/server > server.log 2>&1 &
```

**Problème persiste même après kill total.**

### Hypothèse #3: Template Embedding at Compile Time (TRÈS PROBABLE 🔥)
En mode `release`, Tera peut compiler/embedder les templates dans le binaire.

**Fichier:** `server/src/main.rs:240`

```rust
let tera = Tera::new("templates/**/*.html")
    .context("Failed to initialize Tera templates")?;
```

**Problème potentiel:**
- Les templates sont peut-être lus au moment de la compilation
- Le binaire `target/release/server` contient les ANCIENS templates (sans HTMX)
- Même en modifiant `templates/settings.html`, le binaire ne voit pas les changements

**Tentatives de résolution:**
```bash
# 1. Touch pour forcer recompilation
$ touch server/src/main.rs templates/settings.html
$ cargo build --release --bin server

# 2. Clean build
$ cargo clean -p server
$ cargo build --release --bin server

# Problème persiste
```

### Hypothèse #4: Validation échoue AVANT la DB (POSSIBLE ⚠️)
Le code retourne 116 bytes JSON sans atteindre les logs DEBUG.

**Points de sortie possibles:**
- ❌ Authentification échoue (ligne 445) - **Peu probable** (user_id en session)
- ❌ CSRF token invalide (ligne 457) - **Peu probable** (token existe)
- ❌ Format adresse invalide (ligne 466) - **Peu probable** (validation regex OK)

**Mais si `is_htmx = false`, toutes ces validations retournent JSON au lieu de HTML.**

### Hypothèse #5: Multiple Template Files (ÉLIMINÉE ✅)
Peut-être que plusieurs fichiers `settings.html` existent?

```bash
$ find . -name "settings.html"
./templates/settings.html
```

**Résultat:** Un seul fichier trouvé. Hypothèse éliminée.

---

## 🔬 TESTS SUPPLÉMENTAIRES À EFFECTUER

### Test #1: Vérifier si HTMX se charge dans le navigateur

**Étapes:**
1. Ouvrir http://127.0.0.1:8080/settings dans Chrome
2. Ouvrir DevTools → Network → Reload
3. Chercher la requête: `https://unpkg.com/htmx.org@1.9.10`

**Résultat attendu:**
- ✅ Si présent: Template servi correctement, HTMX bloque ailleurs
- ❌ Si absent: Template n'est pas servi correctement

### Test #2: Inspecter le HTML source dans le navigateur

**Étapes:**
1. Ouvrir http://127.0.0.1:8080/settings
2. Clic droit → "View Page Source" (Ctrl+U)
3. Chercher "htmx" (Ctrl+F)

**Résultat attendu:**
- ✅ Si trouvé: HTMX est dans le HTML mais ne s'exécute pas
- ❌ Si absent: Le template n'est PAS celui qu'on pense

### Test #3: Vérifier le POST avec DevTools

**Étapes:**
1. Ouvrir http://127.0.0.1:8080/settings
2. DevTools → Network → XHR
3. Remplir l'adresse wallet et cliquer SAVE
4. Inspecter la requête POST

**Headers attendus si HTMX fonctionne:**
```
HX-Request: true
HX-Target: wallet-response
HX-Current-URL: http://127.0.0.1:8080/settings
Content-Type: application/x-www-form-urlencoded
```

**Headers actuels (probablement):**
```
Content-Type: application/x-www-form-urlencoded
# Pas de HX-Request!
```

### Test #4: Console JavaScript

**Étapes:**
1. Ouvrir DevTools → Console
2. Taper: `typeof htmx`

**Résultat attendu:**
- `"object"` → HTMX chargé
- `"undefined"` → HTMX PAS chargé

### Test #5: Forcer le header HTMX manuellement

**Étapes:**
```bash
$ curl -X POST http://127.0.0.1:8080/api/settings/update-wallet \
  -H "HX-Request: true" \
  -H "Cookie: session=..." \
  -d "csrf_token=xxx&wallet_address=9wviCeWe2D8..."
```

**Résultat attendu:**
- Response size: ~250 bytes (HTML)
- Content-Type: text/html

Si ça marche → Le problème est bien que HTMX n'envoie pas le header.

---

## 📊 SCHÉMA DE LA BASE DE DONNÉES

**Fichier:** `server/src/schema.rs:88-98`

```rust
diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password_hash -> Text,
        role -> Text,
        wallet_address -> Nullable<Text>,  // ← CIBLE
        wallet_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
```

**Requête UPDATE (ligne 484-486):**

```rust
diesel::update(users::table.filter(users::id.eq(&uid)))
    .set(users::wallet_address.eq(Some(&wallet_addr)))
    .execute(&mut conn)?;
```

**SQL équivalent:**
```sql
UPDATE users
SET wallet_address = '9wviCeWe2D8...'
WHERE id = 'user_uuid';
```

**Note:** L'adresse wallet est stockée en **texte clair** (pas chiffrée) car c'est une adresse publique.

---

## 🎯 DIAGNOSTIC ACTUEL

**État du bug:**

| Composant | État | Détails |
|-----------|------|---------|
| Template HTML source | ✅ OK | HTMX ligne 10, attributs hx-* présents |
| Template HTML servi | ❌ KO | HTMX absent du HTML retourné par le serveur |
| Backend logic | ✅ OK | Code Rust valide, UPDATE query correct |
| Database schema | ✅ OK | Colonne `wallet_address` existe |
| HTMX header | ❌ KO | `HX-Request: true` pas envoyé → `is_htmx = false` |
| Response | ❌ KO | 116 bytes JSON au lieu de ~250 bytes HTML |
| Database persistence | ⚠️ INCONNU | Logs DEBUG jamais atteints, impossible de vérifier |

**Conclusion:**
Le bug a **2 couches**:

1. **Couche Frontend:** HTMX n'est PAS chargé dans le HTML servi par le serveur
   - Template source: ✅ contient HTMX
   - HTML servi: ❌ ne contient PAS HTMX
   - **Root cause probable:** Template embedding/caching issue

2. **Couche Backend:** Comme HTMX ne charge pas, le header `HX-Request` n'est pas envoyé
   - Backend détecte `is_htmx = false`
   - Retourne JSON au lieu de HTML
   - **Effet secondaire:** Impossible de savoir si la DB UPDATE fonctionne

---

## 🛠️ SOLUTIONS PROPOSÉES

### Solution A: Déboguer le Template Loading

**Objectif:** Comprendre pourquoi le HTML servi n'a pas HTMX

**Étapes:**
1. Ajouter des logs dans `server/src/handlers/frontend.rs:1261`:

```rust
match tera.render("settings.html", &ctx) {
    Ok(html) => {
        // DEBUG: Log first 500 chars
        info!("Rendered HTML preview: {}", &html[..500.min(html.len())]);
        info!("HTML contains 'htmx': {}", html.contains("htmx"));

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    }
    Err(e) => { /* ... */ }
}
```

2. Rebuild + restart
3. Reload /settings
4. Vérifier les logs: Est-ce que `html.contains("htmx")` retourne `true`?

**Résultats possibles:**
- `true` → Le template Tera contient HTMX mais le client ne le reçoit pas (problème réseau/proxy?)
- `false` → Tera charge le MAUVAIS template (cache? ancien fichier?)

### Solution B: Mode Development de Tera

**Objectif:** Forcer le rechargement des templates à chaque requête

**Modification:** `server/src/main.rs:240`

```rust
// Mode DEVELOPMENT - recharge à chaque requête
let mut tera = Tera::new("templates/**/*.html")
    .context("Failed to initialize Tera templates")?;

tera.autoescape_on(vec!["html"]);
// AJOUT:
tera.full_reload()
    .context("Failed to reload templates")?;
```

**Note:** Performance hit, mais force le reload.

### Solution C: Bypass HTMX Temporairement

**Objectif:** Tester si le UPDATE database fonctionne indépendamment du problème HTMX

**Modification:** `server/src/handlers/auth.rs:442`

```rust
// TEMPORARY HACK: Force is_htmx = true
let is_htmx = true; // Override for debugging
```

**Étapes:**
1. Forcer `is_htmx = true`
2. Rebuild
3. Essayer de sauvegarder l'adresse
4. Vérifier si les logs DEBUG s'affichent
5. Vérifier si l'adresse persiste après refresh

**Résultat attendu:**
- Si l'adresse persiste → Le problème est 100% frontend (HTMX)
- Si l'adresse ne persiste pas → Il y a AUSSI un problème backend/DB

### Solution D: Utiliser un Form Standard (Sans HTMX)

**Objectif:** Contourner complètement HTMX

**Modification:** `templates/settings.html:198-208`

```html
<!-- ANCIENNE VERSION (HTMX) -->
<form
    hx-post="/api/settings/update-wallet"
    hx-target="#wallet-response"
    hx-swap="innerHTML"
>

<!-- NOUVELLE VERSION (Standard POST) -->
<form
    method="POST"
    action="/api/settings/update-wallet"
>
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">

    <div class="form-group">
        <label for="wallet_address">Monero Wallet Address</label>
        <input
            type="text"
            id="wallet_address"
            name="wallet_address"
            placeholder="9wviCeWe2D8..."
            value="{{ wallet_address | default(value='') }}"
            required
        >
    </div>

    <button type="submit" class="btn-primary">
        SAVE WALLET ADDRESS
    </button>
</form>
```

**Modification backend:** `server/src/handlers/auth.rs:509-519`

```rust
// Toujours retourner JSON, jamais HTML
Ok(HttpResponse::Ok().json(serde_json::json!({
    "success": true,
    "message": "Wallet address updated successfully"
})))
```

**Avantages:**
- ✅ Pas de dépendance HTMX
- ✅ Fonctionne avec HTTP standard
- ✅ Simpler debugging

**Inconvénients:**
- ❌ Page reload complète (pas AJAX)
- ❌ Perd l'expérience utilisateur moderne

---

## 📝 PROCHAINES ÉTAPES RECOMMANDÉES

### Priorité 1: Diagnostic Frontend

1. **Ouvrir http://127.0.0.1:8080/settings dans Chrome**
2. **View Page Source** (Ctrl+U)
3. **Chercher "htmx"**

**Si trouvé:**
- → HTMX est chargé mais ne fonctionne pas
- → Tester la console: `typeof htmx`
- → Vérifier les erreurs JavaScript

**Si absent:**
- → Le template n'est PAS celui qu'on pense
- → Implémenter Solution B (Tera full_reload)
- → Ou implémenter Solution D (Form standard)

### Priorité 2: Test Backend Isolé

1. Implémenter **Solution C** (forcer `is_htmx = true`)
2. Rebuild + restart
3. Tester la sauvegarde
4. Vérifier les logs DEBUG

**Objectif:** Confirmer que le UPDATE database fonctionne indépendamment du problème frontend.

### Priorité 3: Solution Définitive

**Si le problème est uniquement frontend:**
- Option A: Résoudre le template embedding issue
- Option B: Utiliser un form standard (plus simple, plus fiable)

**Si le problème touche aussi le backend:**
- Investiguer pourquoi `rows_affected = 0`
- Vérifier que `user_id` en session correspond à un user en DB
- Vérifier les permissions Diesel

---

## 🔗 FICHIERS IMPLIQUÉS

| Fichier | Lignes | Description |
|---------|--------|-------------|
| `server/src/handlers/auth.rs` | 432-519 | Handler POST /update-wallet |
| `server/src/handlers/frontend.rs` | 1240-1272 | Handler GET /settings (render template) |
| `templates/settings.html` | 10, 198-208 | Template HTML avec HTMX |
| `server/src/schema.rs` | 88-98 | Schéma DB table `users` |
| `server/src/main.rs` | 240 | Initialisation Tera |

---

## 💡 CONTEXTE BUSINESS

**Pourquoi c'est critique:**

Le vendor DOIT configurer son adresse Monero wallet pour pouvoir recevoir les paiements.

**Flow attendu:**
1. Buyer paie → Fonds dans escrow multisig
2. Vendor ship la commande → Clique "Mark as Shipped"
3. **Backend vérifie:** `vendor.wallet_address IS NOT NULL`
4. Si NULL → **400 Bad Request**: "Configure your wallet first"
5. Si présent → Statut = "shipped", fonds débloqués vers `vendor.wallet_address`

**Impact du bug:**
- ❌ Vendor ne peut pas configurer son wallet
- ❌ Vendor ne peut pas marquer "shipped"
- ❌ Buyers bloqués avec fonds en escrow
- ❌ Plateforme inutilisable

**Priority:** CRITIQUE - Blocker pour production

---

## 🧪 LOGS UTILES

### Logs actuels (pas de DEBUG):

```
[2025-11-03T17:43:21.570274Z] INFO server: Starting HTTP server on http://127.0.0.1:8080
[2025-11-03T17:43:21.574055Z] INFO actix_server::server: starting service
[2025-11-03T17:43:25.069529Z] INFO actix_web::middleware::logger: 127.0.0.1 "GET /settings HTTP/1.1" 302 0
[2025-11-03T17:29:03.950532Z] INFO actix_web::middleware::logger: 127.0.0.1 "POST /api/settings/update-wallet HTTP/1.1" 200 116
```

### Logs attendus avec DEBUG:

```
[INFO] DEBUG: Attempting to update wallet for user_id: abc123-def-456
[INFO] DEBUG: Wallet address to save: 9wviCeWe2D8XS82k2ovp5EUYLzBt9pYNW2LXUFsZiv8S3Mt21FZ5qQaAroko1enzw3eGr9qC7X1D7Geoo2RrAotYPwq9Gm8
[INFO] DEBUG: Rows affected by UPDATE: 1
[INFO] Wallet address updated successfully user_id=abc123-def-456 rows=1
```

**Observation:** Ces logs DEBUG ne sont JAMAIS apparus, ce qui signifie que le code retourne AVANT la partie UPDATE.

---

**FIN DU RAPPORT**

Prochaine action: Exécuter **Test #2** (View Page Source dans Chrome) pour confirmer si HTMX est présent ou non dans le HTML réellement servi au client.
