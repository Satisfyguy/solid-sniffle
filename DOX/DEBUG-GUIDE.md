# Guide de Debug - Monero Marketplace

Guide complet pour diagnostiquer et résoudre les problèmes courants dans le projet Monero Marketplace.

## Table des matières

1. [Erreurs de Template](#erreurs-de-template)
2. [Erreurs CSS / MIME Type](#erreurs-css--mime-type)
3. [Erreurs de Base de Données](#erreurs-de-base-de-données)
4. [Erreurs de Compilation Rust](#erreurs-de-compilation-rust)
5. [Problèmes de Serveur](#problèmes-de-serveur)
6. [Problèmes IPFS](#problèmes-ipfs)
7. [Problèmes Monero RPC](#problèmes-monero-rpc)
8. [Content Security Policy (CSP)](#content-security-policy-csp)
9. [Outils de Debug](#outils-de-debug)

---

## Erreurs de Template

### ❌ Erreur: "Failed to render 'X.html' (error happened in a parent template)"

**Symptôme:**
```
Template error rendering X: Failed to render 'X.html' (error happened in a parent template)
```

**Cause:**
Le template parent (`base-marketplace.html` ou `base-nexus.html`) utilise des variables qui ne sont pas fournies par le handler.

**Variables requises par `base-marketplace.html`:**
- `username` - Nom d'utilisateur connecté
- `user_name` - Alias pour le nom d'utilisateur (utilisé dans la nav)
- `logged_in` - Boolean (true/false)
- `role` - Rôle de l'utilisateur ("vendor" ou "buyer")
- `user_role` - Alias pour le rôle (utilisé dans le menu)
- `is_vendor` - Boolean (true si vendor, false sinon)
- `csrf_token` - Token CSRF pour les formulaires

**Solution:**

Dans votre handler (`server/src/handlers/frontend.rs`), assurez-vous d'insérer toutes les variables:

```rust
pub async fn your_handler(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Render template
    match tera.render("your_template.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
```

**Vérification rapide:**
```bash
# Chercher tous les handlers qui manquent ces variables
grep -n "pub async fn show_" server/src/handlers/frontend.rs | while read line; do
    func_line=$(echo $line | cut -d: -f1)
    func_name=$(echo $line | cut -d' ' -f4)
    if ! sed -n "${func_line},$((func_line+50))p" server/src/handlers/frontend.rs | grep -q "user_role"; then
        echo "⚠️  $func_name manque 'user_role'"
    fi
done
```

---

## Erreurs CSS / MIME Type

### ❌ Erreur: "Refused to apply style... MIME type ('') is not supported"

**Symptôme:**
```
Refused to apply style from '<URL>' because its MIME type ('')
is not a supported stylesheet MIME type, and strict MIME checking is enabled.
```

**Cause:**
Le template étend `base-nexus.html` qui référence des fichiers CSS inexistants (Nexus/Brutalist design system qui a été remplacé).

**Solution:**

1. **Changer le template parent:**
   ```html
   <!-- ❌ Ancien -->
   {% extends "base-nexus.html" %}
   {% import "partials/nexus-macros.html" as nx %}

   <!-- ✅ Nouveau -->
   {% extends "base-marketplace.html" %}
   ```

2. **Remplacer les classes CSS:**
   ```bash
   # Script de remplacement rapide
   sed -i 's/class="nexus-card"/class="card"/g' templates/your_template.html
   sed -i 's/class="nexus-btn nexus-btn-primary"/class="btn btn-primary"/g' templates/your_template.html
   sed -i 's/class="nexus-input"/class="input"/g' templates/your_template.html
   sed -i 's/class="nexus-label"/class="label"/g' templates/your_template.html
   ```

3. **Remplacer les variables CSS:**
   ```bash
   sed -i 's/var(--nexus-fg, #f9fafb)/hsl(var(--foreground))/g' templates/your_template.html
   sed -i 's/var(--nexus-muted-fg, #9ca3af)/hsl(var(--muted-foreground))/g' templates/your_template.html
   sed -i 's/var(--nexus-glass-border, rgba(255, 255, 255, 0.1))/hsl(var(--border))/g' templates/your_template.html
   ```

4. **Remplacer les macros Nexus par du HTML standard:**
   ```html
   <!-- ❌ Ancien -->
   {{ nx::button(text="Submit", variant="primary") }}

   <!-- ✅ Nouveau -->
   <button class="btn btn-primary">Submit</button>
   ```

**Classes disponibles dans le design system marketplace:**

| Composant | Classes |
|-----------|---------|
| Boutons | `btn`, `btn-primary`, `btn-secondary`, `btn-ghost`, `btn-destructive`, `btn-lg`, `btn-sm` |
| Cartes | `card`, `card-header`, `card-content`, `card-footer` |
| Inputs | `input`, `textarea`, `select` |
| Badges | `badge`, `badge-success`, `badge-warning`, `badge-destructive` |
| Layout | `container`, `section`, `section-header`, `section-title`, `section-subtitle` |

**Trouver tous les templates affectés:**
```bash
grep -l "base-nexus" templates/**/*.html
```

---

## Erreurs de Base de Données

### ❌ Erreur: "Column count mismatch"

**Symptôme:**
```
Failed to retrieve created listing
Thread 'tokio-runtime-worker' panicked at 'Column count mismatch'
```

**Cause:**
La structure Rust ne correspond pas au schéma de la base de données après une migration.

**Solution:**

1. **Vérifier les migrations appliquées:**
   ```bash
   DATABASE_URL=marketplace.db diesel migration list
   ```
   Toutes les migrations doivent afficher `[X]` (appliquées), pas `[ ]` (en attente).

2. **Appliquer les migrations manquantes:**
   ```bash
   DATABASE_URL=marketplace.db diesel migration run
   ```

3. **Régénérer le schéma Rust:**
   ```bash
   diesel print-schema > server/src/schema.rs
   ```

4. **Vérifier la correspondance:**
   ```bash
   # Comparer le nombre de colonnes dans la DB
   sqlite3 marketplace.db "PRAGMA table_info(listings);" | wc -l

   # Comparer avec le nombre de champs dans la struct Rust
   grep -A 20 "pub struct Listing {" server/src/models/listing.rs | grep "pub " | wc -l
   ```

5. **Recompiler:**
   ```bash
   cargo build --release --package server
   ```

### ❌ Erreur: "Failed to retrieve created listing" (persistent)

**Cause:**
Le serveur utilise un ancien binaire compilé avant les migrations.

**Solution:**
```bash
# 1. Tuer TOUS les processus serveur
killall -9 server
pkill -9 -f "target/release/server"

# 2. Vérifier qu'aucun serveur ne tourne
ps aux | grep "[t]arget/release/server"

# 3. Recompiler
cargo build --release --package server

# 4. Vérifier la date du binaire
stat -c "%y" target/release/server

# 5. Redémarrer
./target/release/server > server.log 2>&1 &
```

---

## Erreurs de Compilation Rust

### ❌ Erreur: "unused variable" / "unused import"

**Symptôme:**
```
warning: unused import: `error`
  --> server/src/main.rs:11:19
   |
11 | use tracing::{debug, error, info, warn};
   |                      ^^^^^
```

**Solution:**
Ces warnings n'empêchent pas la compilation. Pour les corriger:

```rust
// Supprimer les imports non utilisés
use tracing::{debug, info, warn}; // Retiré 'error'

// Ou préfixer la variable avec _
let _unused_var = value;
```

### ❌ Erreur: "cannot find type `X` in this scope"

**Cause:**
Import manquant ou mauvais chemin de module.

**Solution:**
```rust
// Vérifier les imports nécessaires
use crate::models::listing::Listing;
use crate::models::user::User;
use crate::error::Error;
```

---

## Problèmes de Serveur

### ❌ Erreur: "Address already in use (os error 98)"

**Symptôme:**
```
Error: Failed to bind to 127.0.0.1:8080
Caused by: Address already in use (os error 98)
```

**Cause:**
Un autre processus serveur est déjà en cours d'exécution sur le port 8080.

**Solution:**
```bash
# Option 1: Tuer tous les serveurs
killall -9 server

# Option 2: Trouver et tuer le processus spécifique
sudo lsof -i :8080
# Notez le PID, puis:
kill -9 <PID>

# Option 3: Tuer par pattern
pkill -9 -f "target/release/server"

# Vérifier qu'aucun serveur ne tourne
ps aux | grep "[s]erver"

# Redémarrer
./target/release/server > server.log 2>&1 &
```

### ❌ Le serveur démarre mais retourne 500 sur toutes les pages

**Diagnostic:**
```bash
# Vérifier les logs du serveur
tail -50 server.log | grep -i "error\|panic\|failed"

# Vérifier les erreurs de template
tail -100 server.log | grep "Template error"
```

**Solutions courantes:**
1. Migrations non appliquées → Voir [Erreurs de Base de Données](#erreurs-de-base-de-données)
2. Variables de template manquantes → Voir [Erreurs de Template](#erreurs-de-template)
3. Fichiers CSS manquants → Voir [Erreurs CSS](#erreurs-css--mime-type)

---

## Problèmes IPFS

### ❌ Erreur: "GET http://127.0.0.1:8081/ipfs/... ERR_CONNECTION_REFUSED"

**Symptôme:**
Les images des listings ne chargent pas.

**Cause:**
Le daemon IPFS n'est pas démarré.

**Solution:**
```bash
# Démarrer IPFS
ipfs daemon > /tmp/ipfs.log 2>&1 &

# Vérifier qu'il tourne
curl http://127.0.0.1:5001/api/v0/version

# Vérifier les logs
tail -20 /tmp/ipfs.log
```

### Configuration IPFS pour le projet

```bash
# Port API: 5001 (utilisé par le serveur Rust)
# Port Gateway: 8081 (utilisé par le navigateur)

# Vérifier la configuration
ipfs config Addresses.API
ipfs config Addresses.Gateway
```

---

## Problèmes Monero RPC

### ❌ Erreur: "Monero RPC unreachable"

**Cause:**
Le démon Monero wallet-rpc n'est pas démarré ou n'écoute pas sur le bon port.

**Solution:**
```bash
# Vérifier si le wallet-rpc tourne
ps aux | grep "[m]onero-wallet-rpc"

# Démarrer wallet-rpc (testnet)
monero-wallet-rpc \
    --testnet \
    --daemon-address testnet.xmr.ditatompel.com:443 \
    --rpc-bind-port 18082 \
    --rpc-bind-ip 127.0.0.1 \
    --wallet-dir ~/testnet-wallets \
    --disable-rpc-login \
    --log-level 2 &

# Vérifier la connectivité
curl -X POST http://127.0.0.1:18082/json_rpc \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}' \
    -H 'Content-Type: application/json'
```

### ⚠️ Sécurité CRITIQUE

**JAMAIS exposer le RPC Monero publiquement:**
```bash
# ✅ CORRECT - localhost uniquement
--rpc-bind-ip 127.0.0.1

# ❌ DANGEREUX - accessible depuis l'extérieur
--rpc-bind-ip 0.0.0.0
```

---

## Content Security Policy (CSP)

### ❌ Erreur: "Refused to execute inline script"

**Symptôme:**
```
Refused to execute inline script because it violates the following
Content Security Policy directive: "script-src 'self'"
```

**Cause:**
Le code JavaScript est inline dans le HTML au lieu d'être dans un fichier externe.

**Solution:**

1. **Extraire le JavaScript dans un fichier externe:**
   ```html
   <!-- ❌ Ancien (inline) -->
   <script>
       document.addEventListener('DOMContentLoaded', function() {
           // Code here
       });
   </script>

   <!-- ✅ Nouveau (externe) -->
   <script src="/static/js/your-script.js"></script>
   ```

2. **Créer le fichier JS externe:**
   ```bash
   cat > static/js/your-script.js << 'EOF'
   document.addEventListener('DOMContentLoaded', function() {
       // Code here
   });
   EOF
   ```

3. **Éviter les event handlers inline:**
   ```html
   <!-- ❌ Mauvais -->
   <button onclick="handleClick()">Click</button>

   <!-- ✅ Bon -->
   <button id="myButton">Click</button>
   <script src="/static/js/button-handler.js"></script>
   ```

### ❌ Erreur: "Refused to execute inline event handler"

**Solution:**
Utiliser `addEventListener` au lieu d'attributs inline:

```javascript
// static/js/your-handler.js
document.addEventListener('DOMContentLoaded', function() {
    const images = document.querySelectorAll('.product-image');

    images.forEach(function(img) {
        img.addEventListener('error', function() {
            this.style.display = 'none';
            const placeholder = this.nextElementSibling;
            if (placeholder) {
                placeholder.style.display = 'flex';
            }
        });
    });
});
```

---

## Outils de Debug

### Vérification rapide de l'état du système

```bash
#!/bin/bash
# scripts/quick-status.sh

echo "=== Monero Marketplace Status ==="

# Serveur
if ps aux | grep -q "[t]arget/release/server"; then
    echo "✅ Server: Running"
else
    echo "❌ Server: Not running"
fi

# IPFS
if curl -s http://127.0.0.1:5001/api/v0/version > /dev/null 2>&1; then
    echo "✅ IPFS: Running"
else
    echo "❌ IPFS: Not running"
fi

# Monero RPC
if curl -s -X POST http://127.0.0.1:18082/json_rpc \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}' \
    -H 'Content-Type: application/json' | grep -q "result"; then
    echo "✅ Monero RPC: Running"
else
    echo "❌ Monero RPC: Not running"
fi

# Base de données
if [ -f "marketplace.db" ]; then
    echo "✅ Database: Found"

    # Vérifier les migrations
    pending=$(DATABASE_URL=marketplace.db diesel migration list 2>/dev/null | grep "\[ \]" | wc -l)
    if [ "$pending" -eq 0 ]; then
        echo "✅ Migrations: All applied"
    else
        echo "⚠️  Migrations: $pending pending"
    fi
else
    echo "❌ Database: Not found"
fi

# Logs récents
echo ""
echo "=== Recent Errors (last 10) ==="
tail -100 server.log 2>/dev/null | grep -i "error\|panic" | tail -10
```

### Commandes de debug utiles

```bash
# Voir les requêtes HTTP en temps réel
tail -f server.log | grep "HTTP"

# Voir uniquement les erreurs
tail -f server.log | grep -i "error\|panic\|failed"

# Tester un endpoint spécifique
curl -v http://127.0.0.1:8080/listings

# Vérifier le schéma de la base de données
sqlite3 marketplace.db ".schema listings"

# Compter les enregistrements
sqlite3 marketplace.db "SELECT COUNT(*) FROM listings;"

# Voir les dernières erreurs de template
grep "Template error" server.log | tail -20
```

### Checklist avant de déployer

```bash
# 1. Vérifier les migrations
DATABASE_URL=marketplace.db diesel migration list | grep "\[ \]" && echo "⚠️  Migrations en attente"

# 2. Compiler sans warnings critiques
cargo clippy --workspace -- -D warnings

# 3. Tester les routes principales
curl -s http://127.0.0.1:8080/ > /dev/null && echo "✅ Home"
curl -s http://127.0.0.1:8080/listings > /dev/null && echo "✅ Listings"

# 4. Vérifier les fichiers statiques
ls static/css/marketplace-*.css > /dev/null 2>&1 && echo "✅ CSS files"
ls static/js/htmx.min.js > /dev/null 2>&1 && echo "✅ HTMX"

# 5. Audit de sécurité rapide
./scripts/audit-pragmatic.sh
```

---

## Patterns de Debug Récurrents

### Pattern 1: Erreur 500 sur une nouvelle page

**Checklist:**
1. ✅ Le template étend-il `base-marketplace.html`?
2. ✅ Le handler insère-t-il `user_role` et `is_vendor`?
3. ✅ Le handler insère-t-il le `csrf_token`?
4. ✅ Les variables utilisées dans le template sont-elles toutes passées?

### Pattern 2: CSS ne charge pas

**Checklist:**
1. ✅ Le template utilise-t-il `base-marketplace.html` (pas `base-nexus.html`)?
2. ✅ Les classes utilisées existent-elles dans `marketplace-components.css`?
3. ✅ Pas de référence à des fichiers CSS inexistants?

### Pattern 3: JavaScript ne s'exécute pas

**Checklist:**
1. ✅ Le JavaScript est-il dans un fichier externe (pas inline)?
2. ✅ Le fichier JS est-il chargé avec `<script src="...">`?
3. ✅ Le code attend-il `DOMContentLoaded` avant de s'exécuter?
4. ✅ Pas d'attributs `onclick`, `onerror`, etc. inline?

---

## Contacts et Ressources

- **Documentation complète:** `DOX/`
- **Guide développeur:** `docs/DEVELOPER-GUIDE.md`
- **Sécurité:** `docs/SECURITY-THEATRE-PREVENTION.md`
- **Tests:** `docs/TESTING.md`

### Scripts utiles

```bash
# Audit de sécurité rapide
./scripts/audit-pragmatic.sh

# Vérification de sécurité theatre
./scripts/check-security-theatre.sh

# Dashboard de sécurité
./scripts/security-dashboard.sh

# Pre-commit checks
./scripts/pre-commit.sh
```

---

**Dernière mise à jour:** 2025-10-31
**Version du guide:** 1.0.0
