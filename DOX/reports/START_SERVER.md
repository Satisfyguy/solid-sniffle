# Comment Démarrer le Serveur NEXUS

## Prérequis

1. **Rust installé** (1.70+)
2. **SQLite** avec SQLCipher
3. **Tor daemon** (optionnel pour le dev local)

## Étapes de Démarrage

### 1. Cloner/Pull les dernières modifications

```bash
git pull origin claude/analyze-nexus-file-011CUWkcfFgRT7bCTg96dBJi
```

### 2. Installer les dépendances Rust

```bash
cargo build --workspace
```

### 3. Configurer la base de données

```bash
# Créer le fichier .env si nécessaire
cp .env.example .env

# Appliquer les migrations
diesel migration run
```

### 4. Démarrer le serveur

```bash
# Mode développement (avec logs détaillés)
RUST_LOG=debug cargo run --package server

# OU mode release (plus rapide)
cargo run --release --package server
```

### 5. Ouvrir ton navigateur

```
http://127.0.0.1:8080
```

## 🎯 Ce que tu devrais voir

### En tant qu'invité (non connecté) :

**Dans le header NEXUS (en haut de page)** :
```
[NEXUS Logo] | Browse | Categories | Vendors | [🔓 LOGIN] [➕ SIGN UP]
```

- **Bouton LOGIN** : Style transparent (ghost) avec icône de porte
- **Bouton SIGN UP** : Style rose/rouge avec animation brillante

### Test du flux complet :

1. **Clique sur "Sign Up"** → Tu vois le formulaire d'inscription avec :
   - Fond glassmorphisme
   - Orbes animés en arrière-plan
   - Formulaire avec Username, Password, Role

2. **Remplis le formulaire** :
   - Username : `test_user_001`
   - Password : `testpassword123`
   - Role : Buyer

3. **Clique "REGISTER"** → Tu devrais voir :
   - Toast notification "🎉 Registration Successful"
   - Redirection automatique vers la page d'accueil
   - Le header change : les boutons LOGIN/SIGNUP sont remplacés par ton nom d'utilisateur avec menu dropdown

4. **Clique sur ton nom d'utilisateur** → Menu avec :
   - Settings
   - Logout

5. **Clique Logout** → Retour à la page de login

## 🐛 Troubleshooting

### Erreur "Database not found"

```bash
# Initialiser la DB
diesel migration run
```

### Erreur "Port already in use"

```bash
# Tuer le processus existant
killall -9 server
# Ou trouver et tuer le processus sur le port 8080
lsof -ti:8080 | xargs kill -9
```

### Erreur de compilation

```bash
# Nettoyer et rebuilder
cargo clean
cargo build --workspace
```

## 📁 Fichiers Modifiés (pour vérifier)

Les modifications sont dans :

1. **Navigation** : `templates/partials/nexus/organisms/nav.html`
   - Lignes 78-98 : Boutons LOGIN et SIGN UP

2. **CSS** : `static/css/nexus.css`
   - Animation `@keyframes nexus-btn-shine`

3. **Templates** : Tous migrés vers `base-nexus.html`
   - `templates/listings/show.html`
   - `templates/orders/show.html`
   - `templates/escrow/show.html`
   - etc.

## 🔍 Vérifier sans compiler

Tu peux voir les modifications dans le code :

```bash
# Voir le nouveau header avec boutons
cat templates/partials/nexus/organisms/nav.html | grep -A 20 "Guest User"

# Voir l'animation CSS
tail -20 static/css/nexus.css
```

## ✅ Derniers Commits

```bash
git log --oneline -5
```

Tu devrais voir :
- `2bb911e` - feat: Enhanced NEXUS navigation with premium auth buttons
- `e47b50e` - docs: Add comprehensive NEXUS authentication system documentation
- `4d042a5` - feat: Complete NEXUS authentication system integration
