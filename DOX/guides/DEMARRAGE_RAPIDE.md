# 🚀 Démarrage Rapide - Tu as Déjà Tout Installé

## Si tu lances normalement avec `./target/release/server`

### ✅ Étapes Ultra-Rapides

```bash
# 1. Pull les dernières modifications
git pull origin claude/analyze-nexus-file-011CUWkcfFgRT7bCTg96dBJi

# 2. Recompiler (IMPORTANT pour charger les nouveaux templates !)
cargo build --release --package server

# 3. Tuer l'ancien serveur s'il tourne
killall -9 server 2>/dev/null || true

# 4. Lancer le nouveau serveur
./target/release/server
```

### 🎯 Pourquoi Recompiler ?

Même si les templates sont chargés au runtime, il faut recompiler pour :
- ✅ Mettre à jour les routes (si modifiées)
- ✅ Mettre à jour les handlers Rust
- ✅ S'assurer que tout est synchronisé

**Bon à savoir** : Tera (le moteur de templates) charge les fichiers `.html` directement depuis `templates/`, donc même sans recompiler, les changements de templates sont visibles après un refresh. MAIS les changements dans le code Rust nécessitent une recompilation.

## 🌐 Ouvrir le Site

```
http://127.0.0.1:8080
```

## 🎨 Ce Que Tu Dois Voir

### Header NEXUS (en haut) :

```
NEXUS | Browse | Categories | Vendors | [🔓 LOGIN] [➕ SIGN UP]
```

**Si tu ne vois PAS les boutons LOGIN et SIGN UP :**

### Diagnostic Rapide

```bash
# 1. Vérifier que les templates ont les modifications
grep "LOGIN" templates/partials/nexus/organisms/nav.html

# Résultat attendu : plusieurs lignes avec "LOGIN"

# 2. Vérifier que tu utilises le bon template de base
grep "base-nexus.html" templates/listings/index.html

# Résultat attendu : {% extends "base-nexus.html" %}

# 3. Vérifier les logs du serveur
# Le serveur doit afficher : "Tera template engine initialized"
```

### Si les boutons ne sont toujours pas visibles

**Option 1 : Force refresh du navigateur**
```
Ctrl + Shift + R (Linux/Windows)
Cmd + Shift + R (Mac)
```

**Option 2 : Vider le cache**
```
F12 → Network → Disable cache
```

**Option 3 : Recompiler en mode debug (plus de logs)**
```bash
cargo build --package server
RUST_LOG=debug ./target/debug/server
```

**Option 4 : Vérifier que logged_in est false**

Le serveur doit passer `logged_in: false` au template. Vérifie dans les logs :
```
Rendered homepage
```

## 🧪 Test Complet

### 1. Page d'accueil (non connecté)

```bash
# Ouvre http://127.0.0.1:8080
# Dans le header, tu DOIS voir :
# - Logo NEXUS (gauche)
# - Browse, Categories, Vendors (centre)
# - LOGIN et SIGN UP (droite)
```

### 2. Clique sur SIGN UP

```bash
# Tu dois être redirigé vers http://127.0.0.1:8080/register
# Tu dois voir :
# - Fond dark avec orbes animés
# - Formulaire glassmorphisme
# - Titre "CREATE ACCOUNT"
```

### 3. Remplis le formulaire

```
Username : test_user_001
Password : testpassword123
Role     : Buyer
```

### 4. Clique REGISTER

```bash
# Tu dois voir :
# - Toast notification "🎉 Registration Successful"
# - Redirection vers homepage
# - Header change : ton nom apparaît au lieu de LOGIN/SIGN UP
```

### 5. Clique sur ton nom

```bash
# Menu dropdown apparaît avec :
# - Settings
# - Logout (en rouge)
```

### 6. Clique Logout

```bash
# Redirection vers /login
# Header redevient : LOGIN et SIGN UP
```

## 📊 État Actuel des Modifications

### Fichiers Modifiés (déjà pushés) :

✅ `templates/partials/nexus/organisms/nav.html` - Boutons LOGIN/SIGN UP
✅ `static/css/nexus.css` - Animation shine
✅ `templates/auth/login.html` - Toast notifications
✅ `templates/auth/register.html` - Toast notifications
✅ `server/src/handlers/frontend.rs` - CSRF tokens et user_name
✅ 6 templates migrés vers base-nexus.html

### Commits :

```bash
git log --oneline -5

# Tu devrais voir :
# 4872f93 docs: Add Ubuntu-specific startup scripts
# 763c679 docs: Add server startup guide
# 2bb911e feat: Enhanced NEXUS navigation with premium auth buttons ← Les boutons !
# e47b50e docs: Add comprehensive NEXUS authentication
# 4d042a5 feat: Complete NEXUS authentication system
```

## 🔍 Vérification Manuelle

Si tu veux voir le code exact des boutons :

```bash
# Voir les boutons dans le template
cat templates/partials/nexus/organisms/nav.html | grep -A 15 "Guest User"

# Résultat attendu : HTML avec deux <a> tags
# - Un pour LOGIN (nexus-btn-ghost)
# - Un pour SIGN UP (nexus-btn-primary)
```

## ⚡ Commandes Utiles

```bash
# Arrêter le serveur
killall -9 server

# Rebuild rapide
cargo build --release --package server

# Rebuild avec logs détaillés
RUST_LOG=debug cargo build --package server

# Nettoyer et rebuild complet
cargo clean
cargo build --release --package server

# Vérifier le port
lsof -i:8080
```

## 🎯 Résumé en 3 Commandes

```bash
git pull origin claude/analyze-nexus-file-011CUWkcfFgRT7bCTg96dBJi
cargo build --release --package server
./target/release/server
```

Puis ouvre **http://127.0.0.1:8080** et regarde le header ! 🚀
