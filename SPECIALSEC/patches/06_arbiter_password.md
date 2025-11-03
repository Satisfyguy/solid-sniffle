# PATCH 6 : Arbiter Password - Random Generation

**Fichier cible :** `server/src/main.rs`
**Temps estimé :** 45 minutes
**Risque :** Très bas
**Impact :** Operational security

---

## Description

**PROBLÈME ACTUEL :**
Le système arbiter est créé avec un mot de passe **hardcodé** : `arbiter_system_2024`.

**Risques :**
1. Password connu de tout développeur qui lit le code
2. Password identique sur toutes les instances du marketplace
3. Si le code est public (GitHub), password est public
4. Brute-force trivial si le password est connu

**Ce patch ajoute :**
- Génération aléatoire d'un password de 16 caractères (alphanumeric)
- Logging du password AU DÉMARRAGE (car pas d'autre moyen de le récupérer)
- Warning explicite que le password doit être changé immédiatement

---

## Patch 6.1 : Générer password aléatoire pour arbiter

**Localisation :** Fonction de création arbiter system, ligne ~150

### Code actuel (MAUVAIS - hardcodé) :
```rust
if arbiter_exists.is_none() {
    info!("No arbiter found, creating system arbiter...");
    let password = "arbiter_system_2024";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .context("Failed to hash password")?
        .to_string();
```

### Code corrigé (BON - aléatoire) :
```rust
if arbiter_exists.is_none() {
    info!("No arbiter found, creating system arbiter...");

    // Generate random 16-character password
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let password: String = (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                .chars()
                .nth(idx)
                .unwrap()
        })
        .collect();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .context("Failed to hash password")?
        .to_string();
```

**Pourquoi 16 caractères alphanumériques ?**
- Espace de recherche : 62^16 ≈ 4.77 × 10^28 combinaisons
- Résistance brute-force : Infaisable même avec GPUs
- Mémorisable pour copy-paste (ni trop court ni trop long)

---

## Patch 6.2 : Logger le password généré

**Localisation :** Après insertion du system arbiter, ligne ~175

### Code actuel (MAUVAIS - affiche le vieux password) :
```rust
info!("✅ System arbiter created successfully (username: arbiter_system, password: arbiter_system_2024)");
```

### Code corrigé (BON - affiche le nouveau password avec warnings) :
```rust
info!("⚠️  ✅ System arbiter created successfully");
info!("📋 SAVE THIS IMMEDIATELY - Arbiter credentials:");
info!("   Username: arbiter_system");
info!("   Password: {}", password);
info!("⚠️  This password will NOT be shown again. Change it immediately after first login.");
```

**Pourquoi logger le password ?**
- Pas de système de récupération de password implémenté
- Pas d'email pour envoyer le password
- Pas d'interface admin pour reset
- C'est le SEUL moment où le password est accessible
- L'opérateur DOIT le sauvegarder immédiatement

---

## Validation post-patch

### 1. Compilation
```bash
cargo check
cargo build --release
# Doit compiler sans erreur
```

### 2. Test génération password (runtime)
```bash
# Supprimer DB pour forcer création arbiter
rm marketplace.db

# Démarrer serveur
cargo run --release 2>&1 | tee server_startup.log

# Vérifier logs
grep "SAVE THIS IMMEDIATELY" server_startup.log
# Doit afficher:
# ⚠️  ✅ System arbiter created successfully
# 📋 SAVE THIS IMMEDIATELY - Arbiter credentials:
#    Username: arbiter_system
#    Password: aB3xK9pQw2mN7vL5
# ⚠️  This password will NOT be shown again. Change it immediately after first login.
```

### 3. Test login avec password généré
```bash
# Extraire le password du log
PASSWORD=$(grep "Password:" server_startup.log | awk '{print $3}')

# Tester login
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"arbiter_system\",
    \"password\": \"$PASSWORD\"
  }"

# Expected: 200 OK avec session cookie
```

### 4. Test unicité des passwords (plusieurs runs)
```bash
# Run 1
rm marketplace.db
cargo run --release 2>&1 | grep "Password:" > pass1.txt

# Run 2
rm marketplace.db
cargo run --release 2>&1 | grep "Password:" > pass2.txt

# Compare
diff pass1.txt pass2.txt
# Les passwords doivent être DIFFÉRENTS
```

---

## Scénarios de test complets

### Scénario 1 : Premier démarrage (DB vide) ✅
- **Setup :** marketplace.db n'existe pas
- **Action :** `cargo run --release`
- **Résultat attendu :**
  - DB créée
  - System arbiter créé avec password aléatoire
  - Password loggé au démarrage
  - Login avec ce password fonctionne

### Scénario 2 : Démarrages subséquents ✅
- **Setup :** marketplace.db existe, arbiter déjà créé
- **Action :** `cargo run --release`
- **Résultat attendu :**
  - Pas de message de création arbiter
  - Pas de nouveau password généré
  - Ancien password toujours valide

### Scénario 3 : Recréation après suppression DB ✅
- **Setup :** Supprimer marketplace.db après premier run
- **Action :** `cargo run --release`
- **Résultat attendu :**
  - Nouveau arbiter créé
  - NOUVEAU password généré (différent du premier)
  - Nouveau password loggé

---

## Améliorations futures recommandées

### 1. Forcer changement password au premier login
```rust
// Dans la table users, ajouter colonne
must_change_password BOOLEAN DEFAULT FALSE

// Lors de création system arbiter
must_change_password = TRUE

// Middleware qui check avant chaque request
if user.must_change_password {
    return HttpResponse::Forbidden().json({
        "error": "Password change required",
        "redirect": "/change-password"
    });
}
```

### 2. Stocker password hash dans fichier séparé (au lieu de logs)
```rust
// Écrire dans arbiter_initial_password.txt
std::fs::write("arbiter_initial_password.txt", &password)?;
info!("⚠️  Initial arbiter password saved to: arbiter_initial_password.txt");
info!("⚠️  DELETE THIS FILE after setting a new password!");
```

### 3. Envoyer password par email (si configured)
```rust
if let Ok(admin_email) = env::var("ADMIN_EMAIL") {
    send_email(
        &admin_email,
        "System Arbiter Password",
        &format!("Username: arbiter_system\nPassword: {}", password)
    ).await?;
    info!("✅ Arbiter credentials sent to {}", admin_email);
}
```

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
# Patch 6.1 : Générer password aléatoire
Edit {
  file_path: "server/src/main.rs"
  old_str: "        if arbiter_exists.is_none() {\n            info!(\"No arbiter found, creating system arbiter...\");\n            let password = \"arbiter_system_2024\";\n            let salt = SaltString::generate(&mut OsRng);\n            let argon2 = Argon2::default();\n            let password_hash = argon2\n                .hash_password(password.as_bytes(), &salt)\n                .context(\"Failed to hash password\")?\n                .to_string();"
  new_str: "        if arbiter_exists.is_none() {\n            info!(\"No arbiter found, creating system arbiter...\");\n\n            // Generate random 16-character password\n            use rand::Rng;\n            let mut rng = rand::thread_rng();\n            let password: String = (0..16)\n                .map(|_| {\n                    let idx = rng.gen_range(0..62);\n                    \"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\"\n                        .chars()\n                        .nth(idx)\n                        .unwrap()\n                })\n                .collect();\n\n            let salt = SaltString::generate(&mut OsRng);\n            let argon2 = Argon2::default();\n            let password_hash = argon2\n                .hash_password(password.as_bytes(), &salt)\n                .context(\"Failed to hash password\")?\n                .to_string();"
}

# Patch 6.2 : Logger le password
Edit {
  file_path: "server/src/main.rs"
  old_str: "            info!(\"✅ System arbiter created successfully (username: arbiter_system, password: arbiter_system_2024)\");"
  new_str: "            info!(\"⚠️  ✅ System arbiter created successfully\");\n            info!(\"📋 SAVE THIS IMMEDIATELY - Arbiter credentials:\");\n            info!(\"   Username: arbiter_system\");\n            info!(\"   Password: {}\", password);\n            info!(\"⚠️  This password will NOT be shown again. Change it immediately after first login.\");"
}
```

---

## Troubleshooting

### Problème : rand crate not found
**Cause :** Dépendance `rand` manquante dans Cargo.toml
**Solution :**
```toml
[dependencies]
rand = "0.8"
```

### Problème : Password non loggé (pas de output)
**Cause :** Niveau de log trop bas (error only)
**Solution :** Vérifier RUST_LOG :
```bash
export RUST_LOG=info
cargo run --release
```

### Problème : Password contient caractères spéciaux cassant shell
**Cause :** Charset inclut `&`, `$`, etc.
**Solution :** Notre charset est alphanumeric UNIQUEMENT (a-zA-Z0-9), safe pour shell

---

## Sécurité du logging du password

**Q: Est-ce sécurisé de logger un password ?**

**R:** Dans CE cas précis, OUI, car :
1. C'est le password INITIAL qui DOIT être changé
2. Pas d'autre moyen de communiquer le password (no email, no UI)
3. Logs sont en production sur serveur (pas exposés publiquement)
4. Alternative serait d'écrire dans un fichier `.txt` (même risque)

**MAIS :**
- ⚠️ Ne JAMAIS logger les passwords d'utilisateurs normaux
- ⚠️ Logs ne doivent PAS être exportés vers SIEM/centralized logging
- ⚠️ Arbiter DOIT changer le password immédiatement

---

## Statut

- [ ] Password aléatoire généré (16 chars)
- [ ] Password loggé au démarrage
- [ ] Warning "change password" affiché
- [ ] Compilation OK (`cargo check`)
- [ ] Test runtime passé (password dans logs)
- [ ] Test login avec nouveau password réussi
- [ ] Test unicité passwords (plusieurs runs)

---

**Créé le :** 2025-11-03
**Difficulté :** Facile (⭐☆☆☆☆)
**Priorité :** MOYENNE ⚠️
