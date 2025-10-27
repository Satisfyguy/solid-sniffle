# Commandes Ubuntu pour NEXUS Marketplace

## 🚀 Démarrage Rapide (Recommandé)

```bash
# Tout en une seule commande !
./UBUNTU_QUICK_START.sh
```

Ce script fait tout automatiquement :
- ✅ Vérifie et installe Rust si nécessaire
- ✅ Installe diesel_cli et SQLite
- ✅ Crée le fichier .env
- ✅ Lance les migrations de base de données
- ✅ Compile le serveur
- ✅ Démarre le serveur sur http://127.0.0.1:8080

## 📋 Démarrage Manuel (Étape par Étape)

### 1. Installer Rust (si pas déjà fait)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Installer les dépendances Ubuntu

```bash
sudo apt update
sudo apt install -y build-essential sqlite3 libsqlite3-dev pkg-config libssl-dev
```

### 3. Installer diesel_cli

```bash
cargo install diesel_cli --no-default-features --features sqlite
```

### 4. Créer le fichier .env

```bash
cat > .env << 'EOF'
DATABASE_URL=marketplace.db
DB_ENCRYPTION_KEY=dev_key_32_bytes_minimum_length_required_here_1234567890
SESSION_SECRET_KEY=development_key_do_not_use_in_production_minimum_64_bytes_required
RUST_LOG=info,actix_web=info,server=debug
MONERO_RPC_URL=http://127.0.0.1:18082/json_rpc
MONERO_RPC_PORT=18082
EOF
```

### 5. Initialiser la base de données

```bash
diesel migration run
```

### 6. Compiler et lancer le serveur

```bash
# Compilation
cargo build --package server

# Lancement
cargo run --package server
```

## 🌐 Ouvrir le Site

Une fois le serveur démarré, ouvre ton navigateur :

```
http://127.0.0.1:8080
```

## 🎨 Ce Que Tu Vas Voir

### Header NEXUS (en haut de page) :

```
┌───────────────────────────────────────────────────────────────┐
│ NEXUS │ Browse │ Categories │ Vendors │ 🔓 LOGIN │ ➕ SIGN UP │
└───────────────────────────────────────────────────────────────┘
```

- **Bouton LOGIN** : Style transparent (ghost) avec icône de porte
- **Bouton SIGN UP** : Style rose/rouge avec animation brillante ✨

### Test du Flux d'Inscription :

1. **Clique sur "SIGN UP"**
   - Tu verras un formulaire avec fond glassmorphisme
   - Orbes animés en arrière-plan

2. **Remplis le formulaire** :
   - Username : `test_buyer`
   - Password : `testpassword123`
   - Role : Buyer

3. **Clique "REGISTER"**
   - Toast notification apparaît : "🎉 Registration Successful"
   - Tu es automatiquement connecté
   - Le header change : ton nom d'utilisateur apparaît

4. **Clique sur ton nom** → Menu dropdown avec :
   - Settings
   - Logout

## 🛠️ Commandes Utiles

### Arrêter le serveur

```bash
# Ctrl+C dans le terminal

# OU tuer le processus
killall -9 server

# OU tuer par port
kill -9 $(lsof -ti:8080)
```

### Rebuild propre

```bash
cargo clean
cargo build --package server
```

### Voir les logs en détail

```bash
RUST_LOG=debug cargo run --package server
```

### Réinitialiser la base de données

```bash
rm marketplace.db
diesel migration run
cargo run --package server
```

## 🐛 Problèmes Courants

### Erreur : "Port 8080 already in use"

```bash
# Tuer le processus existant
kill -9 $(lsof -ti:8080)
```

### Erreur : "diesel: command not found"

```bash
# Installer diesel_cli
cargo install diesel_cli --no-default-features --features sqlite

# Ajouter au PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Erreur : "failed to run custom build command for `openssl-sys`"

```bash
# Installer les dépendances OpenSSL
sudo apt install -y pkg-config libssl-dev
```

### Le serveur compile mais ne démarre pas

```bash
# Vérifier les logs
RUST_LOG=debug cargo run --package server 2>&1 | tee server.log

# Vérifier que le port 8080 est libre
lsof -i:8080
```

## 📊 Vérifier Que Tout Fonctionne

### Test 1 : Les boutons sont visibles

```bash
# Ouvre http://127.0.0.1:8080
# Tu dois voir LOGIN et SIGN UP dans le header
```

### Test 2 : L'inscription fonctionne

```bash
# 1. Clique SIGN UP
# 2. Remplis le formulaire
# 3. Clique REGISTER
# 4. Toast de succès apparaît
# 5. Header change (ton nom apparaît)
```

### Test 3 : La connexion fonctionne

```bash
# 1. Logout
# 2. Clique LOGIN
# 3. Entre tes identifiants
# 4. Toast de succès
# 5. Redirection vers homepage
```

## 🔍 Inspecter le Code

### Voir les boutons dans le code

```bash
cat templates/partials/nexus/organisms/nav.html | grep -A 20 "Guest User"
```

### Voir l'animation CSS

```bash
tail -20 static/css/nexus.css
```

### Voir les derniers commits

```bash
git log --oneline -5
```

## 📦 Structure du Projet

```
solid-sniffle/
├── server/          # Backend Rust (Actix-web)
├── templates/       # Templates Tera (HTML)
│   ├── auth/        # Login, Register
│   └── partials/nexus/organisms/nav.html  ← Boutons ici !
├── static/          # CSS, JS, images
│   └── css/nexus.css  ← Animation shine ici !
├── marketplace.db   # Base de données SQLite
└── .env             # Configuration
```

## 🎉 C'est Tout !

Si tu suis ces étapes, tu devrais voir les nouveaux boutons LOGIN et SIGN UP dans le header NEXUS.

Bonne découverte ! 🚀
