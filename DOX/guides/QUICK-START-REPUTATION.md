# Quick Start - Système de Réputation Intégré

**Date:** 2025-10-23
**Milestone:** REP.3 & REP.4 - Intégration complète

---

## 🚀 Installation rapide (Ubuntu/Debian)

### 1. Installer les dépendances système

```bash
# Exécuter le script d'installation
bash install-deps.sh

# OU manuellement:
sudo apt update
sudo apt install -y pkg-config libssl-dev build-essential
```

### 2. Build du module WASM

```bash
cd reputation/wasm
./build.sh

# Vérifier que les fichiers sont créés
ls -lh ../../static/wasm/
# Doit afficher:
# reputation_wasm_bg.wasm (226K)
# reputation_wasm.js (16K)
```

### 3. Tester le module reputation

```bash
cd reputation
cargo test --workspace

# Résultat attendu:
# 9 tests passés (4 common + 5 crypto)
```

### 4. Compiler le serveur

```bash
cd server
cargo build --release

# Le build peut prendre 5-10 minutes la première fois
```

### 5. Configurer l'environnement

```bash
cd server
cp .env.example .env

# Éditer .env avec vos valeurs:
nano .env
```

**Variables requises:**
```bash
DATABASE_URL=marketplace.db
DB_ENCRYPTION_KEY=your_32_byte_sqlcipher_key_here_change_me
SESSION_SECRET_KEY=your_64_byte_secret_key_here_change_me_minimum_64_bytes_required
RUST_LOG=info
```

### 6. Initialiser la base de données

```bash
# Installer diesel_cli si pas déjà fait
cargo install diesel_cli --no-default-features --features sqlite

# Lancer les migrations
diesel migration run
```

### 7. Démarrer le serveur

```bash
cargo run --release

# Le serveur démarre sur http://127.0.0.1:8080
```

---

## 🧪 Tests manuels

### Test 1: Accéder à la page d'accueil

```bash
curl http://127.0.0.1:8080/
# Doit retourner HTML
```

### Test 2: Vérifier les fichiers statiques WASM

```bash
curl -I http://127.0.0.1:8080/static/wasm/reputation_wasm_bg.wasm
# Doit retourner: 200 OK

curl -I http://127.0.0.1:8080/static/js/reputation-verify.js
# Doit retourner: 200 OK

curl -I http://127.0.0.1:8080/static/css/reputation.css
# Doit retourner: 200 OK
```

### Test 3: Page profil vendeur (nécessite un vendor_id valide)

```bash
# Exemple avec un UUID
VENDOR_ID="550e8400-e29b-41d4-a716-446655440000"
curl http://127.0.0.1:8080/vendor/$VENDOR_ID

# Doit retourner HTML avec template vendor_profile.html
```

### Test 4: API Réputation

```bash
# Obtenir la réputation d'un vendeur
curl http://127.0.0.1:8080/api/reputation/$VENDOR_ID

# Doit retourner JSON:
# {
#   "vendor_id": "...",
#   "reputation": {
#     "format_version": "1.0",
#     "vendor_pubkey": "...",
#     "reviews": [],
#     "stats": {...}
#   }
# }
```

### Test 5: Formulaire soumission avis (requiert authentification)

```bash
curl http://127.0.0.1:8080/review/submit
# Sans auth: redirect vers /login
# Avec auth: affiche formulaire
```

---

## 🌐 Tests dans le navigateur

### 1. Ouvrir dans un navigateur

```
http://127.0.0.1:8080
```

### 2. Créer un compte

1. Cliquer "Register"
2. Remplir le formulaire
3. Se connecter

### 3. Tester le profil vendeur

```
http://127.0.0.1:8080/vendor/550e8400-e29b-41d4-a716-446655440000
```

**Vérifications à faire:**
- ✅ Page charge sans erreur
- ✅ Console browser: "✅ Reputation WASM v0.1.0 loaded"
- ✅ Badge de vérification affiché
- ✅ Statistiques affichées (même si 0 avis)
- ✅ Design glassmorphism appliqué

### 4. Tester le formulaire d'avis

```
http://127.0.0.1:8080/review/submit
```

**Vérifications:**
- ✅ Redirect si non connecté
- ✅ Formulaire affiché si connecté
- ✅ Sélecteur 5 étoiles interactif
- ✅ Compteur de caractères (max 500)
- ✅ Token CSRF présent

### 5. Console navigateur - Vérification WASM

Ouvrir DevTools (F12) et aller dans Console:

```javascript
// Le WASM doit être chargé automatiquement
// Vérifier dans console:
✅ Reputation WASM v0.1.0 loaded

// Tester manuellement (si sur page vendor):
// Les fonctions doivent être disponibles:
typeof verifyReputation
// Doit retourner: "function"
```

---

## 🔍 Débogage

### Problème: WASM ne charge pas

**Symptômes:**
- Console browser: erreur "Failed to fetch WASM"
- Badge de vérification ne s'affiche pas

**Solutions:**
1. Vérifier que le fichier existe:
```bash
ls -lh static/wasm/reputation_wasm_bg.wasm
```

2. Vérifier les permissions:
```bash
chmod 644 static/wasm/*
```

3. Vérifier les logs serveur:
```bash
# Dans terminal du serveur, chercher:
GET /static/wasm/reputation_wasm_bg.wasm
# Status doit être 200
```

### Problème: Routes 404

**Symptômes:**
- `/vendor/{id}` retourne 404
- `/review/submit` retourne 404

**Solutions:**
1. Vérifier que les handlers sont compilés:
```bash
grep -n "vendor_profile" server/src/handlers/frontend.rs
# Doit afficher ligne 449
```

2. Vérifier les routes dans main.rs:
```bash
grep -n "vendor/{vendor_id}" server/src/main.rs
# Doit afficher ligne 161
```

3. Recompiler:
```bash
cd server && cargo build --release
```

### Problème: Erreur de compilation

**Symptôme:**
```
error: failed to run custom build command for `openssl-sys`
```

**Solution:**
```bash
bash install-deps.sh
# Puis recompiler
```

### Problème: Database error

**Symptômes:**
- Erreur "Database connection error"
- Erreur "Failed to get connection from pool"

**Solutions:**
1. Vérifier .env:
```bash
cat server/.env | grep DATABASE_URL
```

2. Lancer migrations:
```bash
cd server && diesel migration run
```

3. Vérifier permissions:
```bash
ls -lh marketplace.db
chmod 644 marketplace.db
```

---

## 📊 Vérification de l'intégration

### Checklist complète

- [ ] **Build WASM réussi** (226 KB)
  ```bash
  ls -lh static/wasm/reputation_wasm_bg.wasm
  ```

- [ ] **Tests passent** (9/9)
  ```bash
  cd reputation && cargo test --workspace
  ```

- [ ] **Serveur compile** sans erreurs
  ```bash
  cd server && cargo check
  ```

- [ ] **Serveur démarre** sans crash
  ```bash
  cargo run
  # Vérifier: "Actix runtime found; starting..."
  ```

- [ ] **Routes répondent**
  ```bash
  curl -I http://127.0.0.1:8080/
  curl -I http://127.0.0.1:8080/vendor/550e8400-e29b-41d4-a716-446655440000
  curl -I http://127.0.0.1:8080/static/wasm/reputation_wasm_bg.wasm
  ```

- [ ] **WASM charge en browser**
  - Ouvrir DevTools Console
  - Chercher: "Reputation WASM v0.1.0 loaded"

- [ ] **Vérification fonctionne**
  - Aller sur profil vendeur
  - Badge "✅ Verified" affiché
  - Pas d'erreurs console

---

## 🎯 Prochaines étapes

Une fois que tous les tests manuels passent:

### 1. Créer des données de test

```sql
-- Insérer un vendeur de test
INSERT INTO users (id, username, email, role)
VALUES ('550e8400-e29b-41d4-a716-446655440000', 'test_vendor', 'vendor@test.com', 'vendor');

-- Insérer un avis de test
INSERT INTO reviews (id, vendor_id, reviewer_id, txid, rating, comment, buyer_pubkey, signature, created_at)
VALUES (
  'review-uuid-here',
  '550e8400-e29b-41d4-a716-446655440000',
  'buyer-uuid-here',
  'test-tx-hash',
  5,
  'Test review!',
  'base64_pubkey',
  'base64_signature',
  datetime('now')
);
```

### 2. Tests E2E automatisés

À implémenter avec Playwright ou Cypress:
- Flow complet soumission avis
- Vérification WASM
- Export IPFS

### 3. Performance testing

```bash
# Load testing avec Apache Bench
ab -n 1000 -c 10 http://127.0.0.1:8080/vendor/550e8400-e29b-41d4-a716-446655440000
```

### 4. Security audit

- Penetration testing
- CSRF validation
- SQL injection tests
- XSS tests

---

## 📖 Documentation complète

Pour plus de détails:

- **REPUTATION-INTEGRATION.md** - Vue d'ensemble de l'intégration
- **REP-3-4-SUMMARY.md** - Résumé exécutif
- **reputation/BUILD-AND-TEST.md** - Guide build détaillé
- **reputation/REP-3-4-COMPLETE.md** - Documentation technique complète

---

## ✅ Succès!

Si tous les tests passent, vous avez maintenant:

✅ Module WASM de vérification cryptographique
✅ Frontend complet avec HTMX
✅ API REST sécurisée
✅ Intégration serveur Actix-Web
✅ Zero-trust client-side verification

**Le système de réputation est opérationnel! 🎉**

---

*Développé avec ❤️ et zero security theatre*
