# FRONTEND-ZIGZAG.md

## Guide de Survie du Labyrinthe Frontend

**Objectif:** Éviter de se perdre dans le zigzag de templates, CSS et serveurs zombies pendant le développement frontend.

Ce document explique les pièges courants, comment les identifier, et comment ne pas perdre 2 heures à modifier le mauvais fichier.

---

## 📁 Anatomie des Templates - Éviter les Sosies

### ⚠️ PIÈGE #1: Les Templates Homonymes

**Problème:** Plusieurs fichiers semblent être la "page d'accueil" mais un seul est réellement chargé.

```
templates/
├── index.html          ← VRAI homepage (route "/")
├── v2_index.html       ← NE PAS TOUCHER (route "/new-home")
├── home2.html          ← Autre template non utilisé
└── auth/
    ├── index.html      ← Page auth (route "/auth")
    ├── login.html      ← ANCIEN, peut être supprimé
    └── register.html   ← ANCIEN, peut être supprimé
```

**Comment vérifier quel template est chargé:**

```bash
# Méthode 1: Chercher la route dans main.rs
grep -n '\.route.*"/"' server/src/main.rs

# Méthode 2: Chercher le handler dans handlers/frontend.rs
grep -n 'pub async fn index' server/src/handlers/frontend.rs
# Ligne ~65: tera.render("index.html", &ctx)
```

**Test visuel rapide:**

Ajoutez un carré rouge de test dans le header:

```html
{% include "header.html" %}
<!-- TEST: Carré rouge pour vérifier que c'est le bon template -->
<div style="position: fixed; top: 100px; left: 50%; width: 50px; height: 50px; background: red; z-index: 9999;"></div>
```

Rechargez la page. Si vous ne voyez PAS le carré, vous modifiez le mauvais fichier.

---

## 🎨 CSS - Le Double Système de Variables

### ⚠️ PIÈGE #2: Deux Systèmes de Variables CSS Incompatibles

**Le projet utilise DEUX systèmes de variables:**

#### Système 1: Variables Hexadécimales (main.css)
```css
/* static/css/main.css */
:root {
    --color-background: #1A1A1A;
    --color-foreground: #FFFFFF;
    --color-accent: #C9A445;
    --color-border: rgba(255, 255, 255, 0.1);
}

/* Utilisation */
.element {
    background-color: var(--color-background);
    color: var(--color-accent);
}
```

#### Système 2: Variables HSL (marketplace-variables.css)
```css
/* static/css/marketplace-variables.css */
:root {
    --background: 0 0% 10%;
    --foreground: 0 0% 98%;
    --accent: 45 65% 55%;
    --card: 0 0% 12%;
    --border: 0 0% 15%;
    --muted: 0 0% 15%;
    --muted-foreground: 0 0% 65%;
    --destructive: 0 84% 60%;
}

/* Utilisation avec hsl() */
.element {
    background-color: hsl(var(--background));
    color: hsl(var(--accent));
}
```

### 📋 Checklist: Quel Système Utiliser?

**Si votre template utilise:**
- `var(--color-accent)` → Chargez seulement `main.css`
- `hsl(var(--accent))` → Chargez `main.css` **ET** `marketplace-variables.css`

**Exemple de chargement correct (profile page):**

```html
<head>
    <link rel="stylesheet" href="/static/css/main.css">
    <link rel="stylesheet" href="/static/css/marketplace-variables.css">
</head>
```

**Symptômes d'un fichier CSS manquant:**
- Le mot "Profil" n'est pas jaune/doré
- Les cards sont transparentes ou blanches
- Les bordures sont invisibles
- Les tabs ne changent pas de couleur au clic

---

## 🏗️ Templates Autonomes vs Base Templates

### ⚠️ PIÈGE #3: Hériter de base-marketplace.html par Accident

**Mauvaise pratique (ancienne méthode):**

```html
{% extends "base-marketplace.html" %}

{% block content %}
    <div class="cart-page">
        <!-- Contenu de la page cart -->
    </div>
{% endblock %}
```

**Problème:** Vous héritez de TOUS les styles et scripts de la base, ce qui peut causer:
- Conflits de CSS
- Headers dupliqués
- Scripts chargés plusieurs fois
- Impossible de personnaliser le `<head>`

**Bonne pratique (templates autonomes):**

Suivre strictement le [IMPLEMENTATION-GUIDE.md](IMPLEMENTATION-GUIDE.md):

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ma Page - Nexus Marketplace</title>

    <link rel="icon" href="/static/favicon.ico" type="image/x-icon">
    <link rel="stylesheet" href="/static/css/main.css">
    <!-- Si vous utilisez hsl(var(--accent)), ajouter: -->
    <link rel="stylesheet" href="/static/css/marketplace-variables.css">

    <style>
        /* Styles spécifiques à cette page */
        .cart-page { padding-top: 6rem; }
    </style>
</head>
<body>
    {% include "header.html" %}

    <main class="cart-page">
        <!-- Votre contenu ici -->
    </main>

    <!-- Scripts obligatoires -->
    <script src="/static/js/lucide.min.js"></script>
    <script src="/static/js/base.js"></script>

    <!-- Scripts spécifiques à cette page -->
    <script src="/static/js/cart.js"></script>

    <!-- Initialisation Lucide (si icônes utilisées) -->
    <script>
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    </script>
</body>
</html>
```

### 📋 Checklist Template Autonome

- ✅ Structure HTML complète (`<!DOCTYPE html>` à `</html>`)
- ✅ Charge `/static/css/main.css`
- ✅ Inclut `{% include "header.html" %}`
- ✅ Charge `lucide.min.js` et `base.js` en fin de body
- ✅ Initialise Lucide si icônes présentes
- ✅ N'hérite PAS de `base-marketplace.html`

---

## 🧟 Zombies de Serveur - La Horde Cachée

### ⚠️ PIÈGE #4: Multiples Instances de Serveur qui Tournent

**Symptôme:** Vous modifiez du code, redémarrez le serveur, mais vos changements n'apparaissent pas.

**Cause:** Vous avez 15 instances de serveur qui tournent en arrière-plan, et vous testez avec l'ancienne.

**Diagnostic:**

```bash
# Voir toutes les instances
ps aux | grep "[t]arget.*server"

# Compter les zombies
ps aux | grep "[t]arget.*server" | wc -l
```

Si vous voyez plus de 1 ligne, vous avez des zombies.

**Solution radicale (nucléaire):**

```bash
# Méthode 1: pkill
pkill -9 server

# Méthode 2: killall
killall -9 server

# Méthode 3: Les deux pour être sûr
pkill -9 server; killall -9 server 2>/dev/null

# Vérification
ps aux | grep "[t]arget.*server"
# Devrait être vide
```

**Redémarrage propre:**

```bash
# Tuer tous les zombies
pkill -9 server; killall -9 server 2>/dev/null

# Attendre 2 secondes (important!)
sleep 2

# Redémarrer proprement
cargo run --bin server
```

**Alternative: Script de redémarrage propre**

Créez `scripts/restart-server.sh`:

```bash
#!/bin/bash
set -e

echo "🧟 Killing zombie servers..."
pkill -9 server 2>/dev/null || true
killall -9 server 2>/dev/null || true

echo "⏳ Waiting for cleanup..."
sleep 2

echo "🔍 Checking for survivors..."
SURVIVORS=$(ps aux | grep "[t]arget.*server" | wc -l)
if [ "$SURVIVORS" -gt 0 ]; then
    echo "⚠️  WARNING: $SURVIVORS zombie(s) still alive!"
    ps aux | grep "[t]arget.*server"
    exit 1
fi

echo "✅ All zombies eliminated"
echo "🚀 Starting fresh server..."
cargo run --bin server
```

```bash
chmod +x scripts/restart-server.sh
./scripts/restart-server.sh
```

---

## 🧭 Navigation entre Pages - Liens vs Routes

### ⚠️ PIÈGE #5: Lien qui Pointe Vers /new-home au Lieu de /

**Problème courant dans le code React converti:**

```html
<!-- ❌ MAUVAIS: Pointe vers v2_index.html -->
<a href="/new-home">Accueil</a>

<!-- ✅ BON: Pointe vers index.html (vraie homepage) -->
<a href="/">Accueil</a>
```

**Mapping Routes → Templates:**

| Route | Handler | Template | Description |
|-------|---------|----------|-------------|
| `/` | `frontend::index` | `index.html` | **Page d'accueil principale** |
| `/new-home` | `frontend::new_home` | `v2_index.html` | Ancienne page de test |
| `/auth` | `frontend::show_auth` | `auth/index.html` | Page d'authentification |
| `/profile` | `frontend::show_profile` | `profile/index.html` | Page profil utilisateur |
| `/cart` | `frontend::show_cart` | `cart/index.html` | Panier d'achat |
| `/listings` | `listings::index` | `listings/index.html` | Liste des annonces |
| `/listings/new` | `listings::create_form` | `listings/create.html` | Créer une annonce |

**Comment trouver la route d'une page:**

```bash
# Méthode 1: Chercher dans main.rs
grep -A 2 "show_cart" server/src/main.rs
# .route("/cart", web::get().to(frontend::show_cart))

# Méthode 2: Chercher le handler dans frontend.rs
grep -B 5 "tera.render.*cart" server/src/handlers/frontend.rs
# pub async fn show_cart(...)
```

---

## 🎯 Debugging Frontend - Techniques de Guerre

### Technique 1: Carré Rouge de Test

```html
<div style="position: fixed; top: 100px; left: 50%; width: 50px; height: 50px; background: red; z-index: 9999;"></div>
```

Placez ce div dans le template que vous PENSEZ modifier. Si vous ne le voyez pas sur la page, vous êtes dans le mauvais fichier.

### Technique 2: Console Log de Template

```html
<script>
    console.log("📄 Template chargé: cart/index.html");
    console.log("🎨 CSS variables:", {
        accent: getComputedStyle(document.documentElement).getPropertyValue('--accent'),
        colorAccent: getComputedStyle(document.documentElement).getPropertyValue('--color-accent')
    });
</script>
```

Ouvrez la console du navigateur (F12) et vérifiez quel template est chargé.

### Technique 3: Timestamp dans le Header

```html
<meta name="last-modified" content="2025-11-02 17:45:00">
```

Changez le timestamp à chaque modification. Vérifiez le source HTML (Ctrl+U) pour confirmer que la nouvelle version est chargée.

### Technique 4: Vérifier les Variables CSS Chargées

Ouvrez la console du navigateur:

```javascript
// Vérifier si marketplace-variables.css est chargé
getComputedStyle(document.documentElement).getPropertyValue('--accent')
// Retourne: "45 65% 55%" si chargé, "" si absent

// Vérifier si main.css est chargé
getComputedStyle(document.documentElement).getPropertyValue('--color-accent')
// Retourne: "#C9A445" ou "rgb(201, 164, 69)"
```

---

## 🚨 Erreurs Courantes et Solutions

### Erreur 1: "Le menu déroulant ne fonctionne pas"

**Cause:** `base.js` n'est pas chargé.

**Solution:**

```html
<script src="/static/js/base.js"></script>
```

Vérifiez dans la console:
```
Uncaught ReferenceError: userMenuBtn is not defined
```

### Erreur 2: "Les icônes Lucide n'apparaissent pas"

**Cause:** `lucide.min.js` pas chargé OU pas initialisé.

**Solution:**

```html
<script src="/static/js/lucide.min.js"></script>
<script>
    if (typeof lucide !== 'undefined') {
        lucide.createIcons();
    }
</script>
```

### Erreur 3: "Les couleurs sont moches/incorrectes"

**Diagnostic:**

```bash
# Chercher quel système de variables est utilisé
grep "hsl(var(--" templates/profile/index.html
# Si résultat: vous devez charger marketplace-variables.css

grep "var(--color-" templates/profile/index.html
# Si résultat: main.css suffit
```

**Solution:** Ajouter le CSS manquant dans `<head>`.

### Erreur 4: "Template error: template not found"

**Cause:** Faute de frappe dans le nom du template OU template dans le mauvais dossier.

**Vérification:**

```bash
# Chercher le template
find templates/ -name "cart*"

# Vérifier le handler
grep "tera.render" server/src/handlers/frontend.rs | grep cart
# tera.render("cart/index.html", &ctx)
```

**Solution:** Le chemin dans `tera.render()` doit correspondre EXACTEMENT au chemin dans `templates/`.

---

## 📊 Checklist de Création d'une Nouvelle Page

Suivez cette checklist pour créer une nouvelle page frontend sans se perdre:

### 1. Créer le Template (Autonome)

```bash
# Créer le fichier
touch templates/my-page/index.html
```

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Page - Nexus Marketplace</title>

    <link rel="icon" href="/static/favicon.ico" type="image/x-icon">
    <link rel="stylesheet" href="/static/css/main.css">
    <!-- Si utilisation de hsl(var(--accent)): -->
    <link rel="stylesheet" href="/static/css/marketplace-variables.css">

    <style>
        .my-page { padding-top: 6rem; }
    </style>
</head>
<body>
    {% include "header.html" %}

    <main class="my-page">
        <h1>My Page</h1>
    </main>

    <script src="/static/js/lucide.min.js"></script>
    <script src="/static/js/base.js"></script>
    <script>
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    </script>
</body>
</html>
```

### 2. Créer le Handler (Backend)

```rust
// server/src/handlers/frontend.rs

pub async fn show_my_page(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let logged_in = session.get::<String>("user_id").unwrap_or(None).is_some();
    let username = session.get::<String>("username").unwrap_or(None);
    let role = session.get::<String>("role").unwrap_or(None);
    let csrf_token = session.get::<String>("csrf_token").unwrap_or(None);

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in", &logged_in);
    ctx.insert("username", &username.unwrap_or_else(|| "Guest".to_string()));
    ctx.insert("role", &role.unwrap_or_else(|| "visitor".to_string()));
    ctx.insert("csrf_token", &csrf_token.unwrap_or_else(|| "".to_string()));

    match tera.render("my-page/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error rendering my-page: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
```

### 3. Ajouter la Route (main.rs)

```rust
// server/src/main.rs

use crate::handlers::frontend;

HttpServer::new(move || {
    App::new()
        .route("/my-page", web::get().to(frontend::show_my_page))
        // ... autres routes
})
```

### 4. Redémarrer le Serveur

```bash
pkill -9 server; killall -9 server 2>/dev/null
sleep 2
cargo run --bin server
```

### 5. Tester

- Visiter http://127.0.0.1:8080/my-page
- Vérifier la console (F12) pour erreurs
- Tester le header (menu déroulant, liens)
- Tester les icônes Lucide

---

## 🎓 Résumé: Les 5 Commandements du Frontend

1. **Tu vérifieras quel template est chargé** avant de modifier quoi que ce soit
2. **Tu chargeras les bons fichiers CSS** (main.css + marketplace-variables.css si HSL)
3. **Tu créeras des templates autonomes** (pas d'extends base-marketplace.html)
4. **Tu tueras tous les zombies** avant de redémarrer le serveur
5. **Tu utiliseras le carré rouge de test** pour confirmer que tu modifies le bon fichier

---

## 📚 Références

- [IMPLEMENTATION-GUIDE.md](IMPLEMENTATION-GUIDE.md) - Guide officiel pour créer des pages
- [server/src/main.rs](../../server/src/main.rs) - Mapping routes → handlers
- [server/src/handlers/frontend.rs](../../server/src/handlers/frontend.rs) - Handlers → templates
- [static/css/main.css](../../static/css/main.css) - Variables hex + styles de base
- [static/css/marketplace-variables.css](../../static/css/marketplace-variables.css) - Variables HSL
- [templates/header.html](../../templates/header.html) - Header partagé

---

**Créé le:** 2025-11-02
**Dernière mise à jour:** 2025-11-02
**Maintenu par:** L'équipe qui s'est perdue dans le zigzag 🌀
