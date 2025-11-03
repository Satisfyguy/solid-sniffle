# PATCH 7 : Session Secret - Production Safety

**Fichier cible :** `server/src/main.rs`
**Temps estimé :** 30 minutes
**Risque :** Haut (production security)
**Impact :** Prevents session hijacking in production

---

## Description

**PROBLÈME ACTUEL :**
Si `SESSION_SECRET_KEY` n'est pas défini, le serveur utilise un fallback **hardcodé** : `development_key_do_not_use_in_production_minimum_64_bytes_required`.

**Risques en production :**
1. **Session prediction** : Clé connue → attaquant peut forger sessions valides
2. **Session hijacking** : Attaquant peut voler sessions d'autres users
3. **Privilege escalation** : Forger session admin si le secret est connu
4. **Reproductibilité** : Toutes les instances utilisent la même clé → attack scale

**Pourquoi c'est CRITIQUE :**
Si le code est public (GitHub), le fallback secret est public → TOUTES les instances non-configurées sont vulnérables.

**Ce patch ajoute :**
- **Dev mode** (debug build) : Warning + fallback (inchangé, OK pour dev local)
- **Production mode** (release build) : **PANIC** si SESSION_SECRET_KEY non défini

---

## Patch 7.1 : Ajouter panic en production si SESSION_SECRET_KEY manquant

**Localisation :** Configuration session secret, ligne ~135

### Code actuel (DANGEREUX en prod) :
```rust
// 4. Session secret key
// IMPORTANT: In production, load from secure environment variable
// This should be a 64-byte cryptographically random key
let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| {
    tracing::warn!("SESSION_SECRET_KEY not set, using development key - NOT FOR PRODUCTION");
    "development_key_do_not_use_in_production_minimum_64_bytes_required".to_string()
});
```

### Code corrigé (SAFE en prod) :
```rust
// 4. Session secret key
// IMPORTANT: In production, load from secure environment variable
// This should be a 64-byte cryptographically random key
let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| {
    if cfg!(debug_assertions) {
        tracing::warn!("SESSION_SECRET_KEY not set, using development key (dev mode only)");
        "development_key_do_not_use_in_production_minimum_64_bytes_required".to_string()
    } else {
        panic!("❌ FATAL: SESSION_SECRET_KEY environment variable MUST be set in production!");
    }
});
```

**Explication du changement :**
- `cfg!(debug_assertions)` = true → **Debug build** (cargo run, cargo build)
- `cfg!(debug_assertions)` = false → **Release build** (cargo build --release)
- En release, si SESSION_SECRET_KEY absent → **panic!** (serveur ne démarre pas)

---

## Validation post-patch

### 1. Compilation
```bash
cargo check
cargo build          # Debug build (doit compiler)
cargo build --release  # Release build (doit compiler)
```

### 2. Test DEBUG mode (avec fallback) ✅
```bash
# Désactiver SESSION_SECRET_KEY
unset SESSION_SECRET_KEY

# Build en debug
cargo build

# Run (doit démarrer avec warning)
cargo run 2>&1 | grep "SESSION_SECRET_KEY"

# Expected output:
# WARN SESSION_SECRET_KEY not set, using development key (dev mode only)
```

### 3. Test RELEASE mode SANS SESSION_SECRET_KEY (doit panic) ❌
```bash
# Désactiver SESSION_SECRET_KEY
unset SESSION_SECRET_KEY

# Build en release
cargo build --release

# Run (doit panic immédiatement)
./target/release/server 2>&1

# Expected output:
# thread 'main' panicked at 'FATAL: SESSION_SECRET_KEY environment variable MUST be set in production!'
# note: run with `RUST_BACKTRACE=1` for a backtrace
```

### 4. Test RELEASE mode AVEC SESSION_SECRET_KEY (doit démarrer) ✅
```bash
# Générer secret key aléatoire
export SESSION_SECRET_KEY="$(openssl rand -base64 48)"

# Vérifier que la variable est définie
echo $SESSION_SECRET_KEY

# Build en release
cargo build --release

# Run (doit démarrer normalement)
./target/release/server

# Expected: Serveur démarre, écoute sur 127.0.0.1:8080
# No panic, no warning about session key
```

---

## Scénarios de test complets

### Scénario 1 : Dev local (debug, no env var) ✅
- **Build :** `cargo run` (debug mode)
- **Env var :** SESSION_SECRET_KEY non défini
- **Résultat attendu :** Warning loggé, serveur démarre avec fallback key

### Scénario 2 : Dev local (debug, avec env var) ✅
- **Build :** `cargo run`
- **Env var :** SESSION_SECRET_KEY="custom_dev_key_12345..."
- **Résultat attendu :** Aucun warning, utilise custom key

### Scénario 3 : Production (release, no env var) ❌ PANIC
- **Build :** `cargo build --release`
- **Env var :** SESSION_SECRET_KEY non défini
- **Résultat attendu :** **PANIC** au démarrage, message FATAL

### Scénario 4 : Production (release, avec env var) ✅
- **Build :** `cargo build --release`
- **Env var :** SESSION_SECRET_KEY défini
- **Résultat attendu :** Serveur démarre normalement

### Scénario 5 : Production avec key trop courte ⚠️
- **Build :** `cargo build --release`
- **Env var :** SESSION_SECRET_KEY="short"
- **Résultat attendu :** Serveur démarre, **MAIS** sessions faibles
- **FIX future :** Valider longueur minimale (64 bytes)

---

## Génération de SESSION_SECRET_KEY sécurisée

### Méthode 1 : OpenSSL (recommandé)
```bash
# Génère 48 bytes random → base64 (64 chars)
openssl rand -base64 48

# Example output:
# dK3mN8pQvL2xYwZ6tB9jR4sA7fH1gE5nC0uI2oM3kP8vT6qX9rW1lJ4hD7yS0bF=
```

### Méthode 2 : /dev/urandom (Linux/Mac)
```bash
# Génère 64 bytes random → hex (128 chars)
head -c 64 /dev/urandom | xxd -p -c 64
```

### Méthode 3 : Python
```bash
python3 -c "import secrets; print(secrets.token_urlsafe(48))"
```

### Méthode 4 : Rust (générer programmatically)
```rust
use rand::RngCore;

let mut key = [0u8; 64];
rand::thread_rng().fill_bytes(&mut key);
let encoded = base64::encode(&key);
println!("SESSION_SECRET_KEY={}", encoded);
```

---

## Configuration en production

### Dans un fichier .env (recommandé pour dev/staging)
```bash
# .env
SESSION_SECRET_KEY=dK3mN8pQvL2xYwZ6tB9jR4sA7fH1gE5nC0uI2oM3kP8vT6qX9rW1lJ4hD7yS0bF=
```

```bash
# Charger .env et run
source .env
./target/release/server
```

### Dans systemd service (recommandé pour prod)
```ini
# /etc/systemd/system/monero-marketplace.service
[Unit]
Description=Monero Marketplace Server
After=network.target

[Service]
Type=simple
User=marketplace
WorkingDirectory=/opt/monero-marketplace
Environment="SESSION_SECRET_KEY=dK3mN8pQvL2xYwZ6tB9jR4sA7fH1gE5nC0uI2oM3kP8vT6qX9rW1lJ4hD7yS0bF="
ExecStart=/opt/monero-marketplace/target/release/server
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### Dans Docker (secret management)
```dockerfile
# Dockerfile
ENV SESSION_SECRET_KEY=${SESSION_SECRET_KEY}

# docker-compose.yml
services:
  marketplace:
    image: monero-marketplace:latest
    environment:
      - SESSION_SECRET_KEY=${SESSION_SECRET_KEY}
    # Ou avec secrets:
    secrets:
      - session_secret_key

secrets:
  session_secret_key:
    external: true
```

### Dans Kubernetes (Secret object)
```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: marketplace-secrets
type: Opaque
data:
  session-secret-key: ZEs...base64...

# deployment.yaml
env:
  - name: SESSION_SECRET_KEY
    valueFrom:
      secretKeyRef:
        name: marketplace-secrets
        key: session-secret-key
```

---

## Rotation de SESSION_SECRET_KEY

**Q: Faut-il changer le SESSION_SECRET_KEY périodiquement ?**

**R:** OUI, recommandations :
- **Changement régulier :** Tous les 90 jours (quarterly)
- **Après incident :** Immédiatement si suspicion de leak
- **Après départ employé :** Si admin avec accès au secret quitte

**Impact du changement :**
- ⚠️ **TOUTES les sessions actives sont invalidées**
- Users doivent se reconnecter
- Sessions cookies deviennent invalides

**Procédure de rotation :**
```bash
# 1. Générer nouveau secret
NEW_SECRET=$(openssl rand -base64 48)

# 2. Update env var (méthode dépend de déploiement)
export SESSION_SECRET_KEY="$NEW_SECRET"

# 3. Restart serveur
systemctl restart monero-marketplace

# 4. Notifier users qu'ils doivent se reconnecter
```

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
Edit {
  file_path: "server/src/main.rs"
  old_str: "    // 4. Session secret key\n    // IMPORTANT: In production, load from secure environment variable\n    // This should be a 64-byte cryptographically random key\n    let session_secret = env::var(\"SESSION_SECRET_KEY\").unwrap_or_else(|_| {\n        tracing::warn!(\"SESSION_SECRET_KEY not set, using development key - NOT FOR PRODUCTION\");\n        \"development_key_do_not_use_in_production_minimum_64_bytes_required\".to_string()\n    });"
  new_str: "    // 4. Session secret key\n    // IMPORTANT: In production, load from secure environment variable\n    // This should be a 64-byte cryptographically random key\n    let session_secret = env::var(\"SESSION_SECRET_KEY\").unwrap_or_else(|_| {\n        if cfg!(debug_assertions) {\n            tracing::warn!(\"SESSION_SECRET_KEY not set, using development key (dev mode only)\");\n            \"development_key_do_not_use_in_production_minimum_64_bytes_required\".to_string()\n        } else {\n            panic!(\"❌ FATAL: SESSION_SECRET_KEY environment variable MUST be set in production!\");\n        }\n    });"
}
```

---

## Troubleshooting

### Problème : Serveur panic en dev (alors que debug build)
**Cause :** Build optimisé avec `--release` flag par erreur
**Solution :**
```bash
# Vérifier le build profile
cargo run  # Debug (dev mode)
cargo run --release  # Release (prod mode)
```

### Problème : Warning "SESSION_SECRET_KEY not set" en production
**Cause :** Build en debug mode au lieu de release
**Solution :** Build avec `--release` flag

### Problème : Sessions invalides après restart
**Cause :** SESSION_SECRET_KEY a changé entre redémarrages
**Solution :** Persister le secret dans env var/config file

---

## Améliorations futures recommandées

### 1. Valider longueur minimale du secret
```rust
let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(...);

if session_secret.len() < 64 {
    panic!("SESSION_SECRET_KEY must be at least 64 characters (got {})", session_secret.len());
}
```

### 2. Valider entropie du secret
```rust
use sha2::{Sha256, Digest};

fn has_sufficient_entropy(secret: &str) -> bool {
    let hash = Sha256::digest(secret.as_bytes());
    // Check for patterns, repeated chars, etc.
    ...
}
```

### 3. Automatic rotation avec grace period
```rust
// Supporter 2 secrets simultanément (old + new)
let current_secret = env::var("SESSION_SECRET_KEY")?;
let previous_secret = env::var("SESSION_SECRET_KEY_OLD").ok();

// Valider session avec current, fallback sur previous
```

---

## Statut

- [ ] Panic ajouté pour release build sans env var
- [ ] Warning conservé pour debug build
- [ ] Compilation OK (debug et release)
- [ ] Test debug mode passé (warning affiché)
- [ ] Test release sans env var passé (panic)
- [ ] Test release avec env var passé (démarrage OK)
- [ ] Documentation configuration prod créée

---

**Créé le :** 2025-11-03
**Difficulté :** Facile (⭐☆☆☆☆)
**Priorité :** CRITIQUE 🔴
