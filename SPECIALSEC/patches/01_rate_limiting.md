# PATCH 1 : Rate Limiting

**Fichier cible :** `server/src/main.rs`
**Temps estimé :** 5 minutes
**Risque :** Très bas
**Impact :** Protection contre DoS et brute-force

---

## Description

Actuellement, le rate limiting est implémenté mais **désactivé** (commenté). Ce patch le réactive pour protéger contre:
- Attaques par force brute sur login
- DoS (Denial of Service)
- Épuisement des ressources
- Scraping massif de l'API

---

## Patch 1.1 : Décommenter global_rate_limiter

**Localisation :** Ligne ~258

### Code actuel (MAUVAIS) :
```rust
// Global rate limiter (100 req/min per IP)
// .wrap(global_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ
```

### Code corrigé (BON) :
```rust
// Global rate limiter (100 req/min per IP)
.wrap(global_rate_limiter())
```

---

## Patch 1.2 : Décommenter protected_rate_limiter

**Localisation :** Ligne ~343

### Code actuel (MAUVAIS) :
```rust
web::scope("/api")
    // .wrap(protected_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ
```

### Code corrigé (BON) :
```rust
web::scope("/api")
    .wrap(protected_rate_limiter())
```

---

## Configuration des limites

Les limites actuelles (définies dans `server/src/middleware/rate_limit.rs`) :

| Limiter | Limite | Fenêtre | Endpoints |
|---------|--------|---------|-----------|
| **Global** | 100 req | 1 minute | Tous |
| **Auth** | 5 req | 15 minutes | /auth/* |
| **Protected** | 60 req | 1 minute | /api/* (sauf auth) |

---

## Validation post-patch

### 1. Compilation
```bash
cargo check
# Doit compiler sans erreur
```

### 2. Test fonctionnel (rate limiting actif)
```bash
# Terminal 1: Démarrer le serveur
cargo run --release

# Terminal 2: Envoyer 150 requêtes rapidement
for i in {1..150}; do
  echo "Request $i:"
  curl -s -w "HTTP %{http_code}\n" http://127.0.0.1:8080/api/health
done
```

**Résultat attendu :**
- Requêtes 1-100 : `HTTP 200 OK`
- Requêtes 101+ : `HTTP 429 Too Many Requests`

### 3. Test header de rate limiting
```bash
curl -i http://127.0.0.1:8080/api/health
```

**Headers attendus :**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1699999999
```

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
Edit {
  file_path: "server/src/main.rs"
  old_str: "            // Global rate limiter (100 req/min per IP)\n            // .wrap(global_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ"
  new_str: "            // Global rate limiter (100 req/min per IP)\n            .wrap(global_rate_limiter())"
}

Edit {
  file_path: "server/src/main.rs"
  old_str: "                web::scope(\"/api\")\n                    // .wrap(protected_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ"
  new_str: "                web::scope(\"/api\")\n                    .wrap(protected_rate_limiter())"
}
```

### Avec sed (alternative) :
```bash
cd server/src
sed -i 's|// \.wrap(global_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ|.wrap(global_rate_limiter())|' main.rs
sed -i 's|// \.wrap(protected_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ|.wrap(protected_rate_limiter())|' main.rs
```

---

## Troubleshooting

### Problème : Compilation échoue avec "cannot find function `global_rate_limiter`"
**Cause :** Import manquant
**Solution :**
```rust
use crate::middleware::rate_limit::{global_rate_limiter, protected_rate_limiter, auth_rate_limiter};
```

### Problème : Rate limiting trop agressif (bloque users légitimes)
**Cause :** Limites trop basses
**Solution :** Ajuster dans `middleware/rate_limit.rs` :
```rust
// Augmenter de 100 à 200 req/min
governor::RateLimiter::keyed(
    governor::Quota::per_minute(NonZeroU32::new(200).unwrap())
)
```

### Problème : Rate limiting ne fonctionne pas en prod (load balancer)
**Cause :** IP client masquée par proxy
**Solution :** Utiliser X-Forwarded-For header
```rust
let client_ip = req
    .connection_info()
    .realip_remote_addr()
    .unwrap_or("0.0.0.0");
```

---

## Statut

- [ ] Patch 1.1 appliqué
- [ ] Patch 1.2 appliqué
- [ ] Compilation OK (`cargo check`)
- [ ] Test fonctionnel passé (429 après 100 req)
- [ ] Headers rate limit vérifiés

---

**Créé le :** 2025-11-03
**Difficulté :** Facile (⭐☆☆☆☆)
**Priorité :** CRITIQUE 🔴
