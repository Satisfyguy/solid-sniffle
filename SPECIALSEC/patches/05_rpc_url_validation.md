# PATCH 5 : RPC URL Validation

**Fichier cible :** `server/src/handlers/escrow.rs`
**Temps estimé :** 30 minutes
**Risque :** Moyen
**Impact :** Bloque URL injection attacks

---

## Description

**PROBLÈME ACTUEL :**
La validation RPC URL vérifie uniquement le FORMAT (`#[validate(url)]`), mais pas que l'URL pointe vers localhost ou .onion.

**Scénario d'attaque :**
1. Attaquant s'inscrit comme buyer/vendor
2. Appelle `/api/escrow/register-wallet-rpc` avec `rpc_url: "http://attacker.com:18082/json_rpc"`
3. Système valide le format ✅ (c'est une URL valide)
4. Système stocke cette URL ❌
5. Lors d'opérations multisig, le code fait des requêtes vers **attacker.com**
6. Attaquant peut logger toutes les requêtes RPC (leak de view keys, addresses, etc.)

**Ce patch ajoute :**
Validation custom qui autorise UNIQUEMENT :
- localhost / 127.x.x.x / ::1 (IPv6 localhost)
- *.onion (Tor hidden services)

---

## Patch 5.1 : Ajouter fonction validate_rpc_url

**Localisation :** Après la fonction `validate_client_role` (ligne ~50)

### Code à ajouter :
```rust
/// Validate RPC URL: only allow localhost or .onion (no public URLs)
fn validate_rpc_url(url: &str) -> Result<(), validator::ValidationError> {
    let parsed = url::Url::parse(url)
        .map_err(|_| validator::ValidationError::new("invalid_url"))?;

    let host = parsed.host_str()
        .ok_or_else(|| validator::ValidationError::new("no_host"))?;

    // Only allow localhost, 127.x.x.x, or .onion addresses
    let is_localhost = host.starts_with("127.")
        || host.eq("localhost")
        || host.starts_with("::1");
    let is_onion = host.ends_with(".onion");

    if !is_localhost && !is_onion {
        return Err(validator::ValidationError::new(
            "rpc_url_must_be_local_or_onion"
        ));
    }

    Ok(())
}
```

### Position exacte (APRÈS) :
```rust
/// Validate that role is buyer or vendor (not arbiter)
fn validate_client_role(role: &str) -> Result<(), validator::ValidationError> {
    match role.to_lowercase().as_str() {
        "buyer" | "vendor" => Ok(()),
        "arbiter" => Err(validator::ValidationError::new(
            "role_not_allowed",
        )),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

/// Validate RPC URL: only allow localhost or .onion (no public URLs)
fn validate_rpc_url(url: &str) -> Result<(), validator::ValidationError> {
    // ... nouveau code ici
}

/// Response for successful wallet registration
```

---

## Patch 5.2 : Appliquer validation custom au champ rpc_url

**Localisation :** Struct `RegisterWalletRpcRequest`, ligne ~15

### Code actuel (MAUVAIS) :
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(url(message = "Invalid RPC URL format"))]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,
```

### Code corrigé (BON) :
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(custom = "validate_rpc_url")]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,
```

**IMPORTANT :** Supprimer `#[validate(url)]` et le remplacer par `#[validate(custom = "validate_rpc_url")]`.

---

## Validation post-patch

### 1. Compilation
```bash
cargo check
# Doit compiler sans erreur
```

### 2. Test URLs valides (doivent passer) ✅

```bash
# Test 1: localhost standard
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -H "Content-Type: application/json" \
  -H "Cookie: session=valid_session" \
  -d '{
    "rpc_url": "http://127.0.0.1:18082/json_rpc",
    "role": "buyer"
  }'
# Expected: 200 OK

# Test 2: 127.x.x.x range
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://127.0.0.2:18082/json_rpc",
    "role": "vendor"
  }'
# Expected: 200 OK

# Test 3: localhost hostname
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://localhost:18082/json_rpc",
    "role": "buyer"
  }'
# Expected: 200 OK

# Test 4: .onion address (Tor)
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://abc123xyz456def.onion:18082/json_rpc",
    "role": "vendor"
  }'
# Expected: 200 OK

# Test 5: IPv6 localhost
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://[::1]:18082/json_rpc",
    "role": "buyer"
  }'
# Expected: 200 OK
```

### 3. Test URLs INVALIDES (doivent être rejetées) ❌

```bash
# Test 1: Public IPv4 (attacker.com)
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://attacker.com:18082/json_rpc",
    "role": "buyer"
  }'
# Expected: 400 Bad Request
# Body: {"error":"Validation error: rpc_url_must_be_local_or_onion"}

# Test 2: Public IPv4 direct
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://1.2.3.4:18082/json_rpc",
    "role": "vendor"
  }'
# Expected: 400 Bad Request

# Test 3: Private network (10.x.x.x)
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://10.0.0.5:18082/json_rpc",
    "role": "buyer"
  }'
# Expected: 400 Bad Request (sauf si on veut autoriser private networks)

# Test 4: LAN address (192.168.x.x)
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://192.168.1.100:18082/json_rpc",
    "role": "vendor"
  }'
# Expected: 400 Bad Request

# Test 5: DNS rebinding attack vector
curl -X POST http://127.0.0.1:8080/api/escrow/register-wallet-rpc \
  -d '{
    "rpc_url": "http://localhost.attacker.com:18082/json_rpc",
    "role": "buyer"
  }'
# Expected: 400 Bad Request (ne finit pas par .onion)
```

---

## Scénarios de test complets

### Scénario 1 : User légitime avec wallet local ✅
- **Setup :** User a Monero wallet RPC sur 127.0.0.1:18082
- **Action :** Register RPC avec `rpc_url: "http://127.0.0.1:18082/json_rpc"`
- **Résultat attendu :** 200 OK, RPC enregistré

### Scénario 2 : User avec wallet sur Tor hidden service ✅
- **Setup :** User a Monero wallet derrière Tor (.onion)
- **Action :** Register RPC avec `rpc_url: "http://abc123...xyz.onion:18082/json_rpc"`
- **Résultat attendu :** 200 OK, RPC enregistré

### Scénario 3 : Attaquant tente URL injection (ATTAQUE) ❌
- **Setup :** Attaquant contrôle `attacker.com`
- **Action :** Register RPC avec `rpc_url: "http://attacker.com:18082/json_rpc"`
- **Résultat attendu :** 400 Bad Request, validation error

### Scénario 4 : Attaquant tente SSRF via DNS rebinding ❌
- **Setup :** Domain `evil.com` pointe vers 127.0.0.1 puis vers attacker IP
- **Action :** Register RPC avec `rpc_url: "http://evil.com:18082/json_rpc"`
- **Résultat attendu :** 400 Bad Request (pas .onion)

---

## Cas limites et edge cases

### Cas 1 : Port non-standard
- **URL :** `http://127.0.0.1:38082/json_rpc` (port 38082 au lieu de 18082)
- **Résultat :** ✅ Devrait passer (le port n'est pas validé strictement)

### Cas 2 : Path custom
- **URL :** `http://127.0.0.1:18082/custom/rpc/path`
- **Résultat :** ✅ Devrait passer (le path n'est pas validé)

### Cas 3 : Username/password dans URL
- **URL :** `http://user:pass@127.0.0.1:18082/json_rpc`
- **Résultat :** ✅ Devrait passer (auth dans URL OK)

### Cas 4 : IPv6 public address
- **URL :** `http://[2001:db8::1]:18082/json_rpc`
- **Résultat :** ❌ Devrait être rejeté (seulement ::1 autorisé)
- **FIX possible :** Ajouter check pour IPv6 loopback complet :
```rust
let is_localhost = host.starts_with("127.")
    || host.eq("localhost")
    || host.eq("::1")
    || host.eq("[::1]");
```

### Cas 5 : .onion v3 (56 chars)
- **URL :** `http://thisisaverylongonionaddressversion3examplehereabcd.onion:18082/json_rpc`
- **Résultat :** ✅ Devrait passer (.onion suffix OK)

---

## Commandes d'application

### Avec Edit tool (recommandé) :
```
# Patch 5.1 : Ajouter fonction validate_rpc_url
Edit {
  file_path: "server/src/handlers/escrow.rs"
  old_str: "/// Validate that role is buyer or vendor (not arbiter)\nfn validate_client_role(role: &str) -> Result<(), validator::ValidationError> {\n    match role.to_lowercase().as_str() {\n        \"buyer\" | \"vendor\" => Ok(()),\n        \"arbiter\" => Err(validator::ValidationError::new(\n            \"role_not_allowed\",\n        )),\n        _ => Err(validator::ValidationError::new(\"invalid_role\")),\n    }\n}\n\n/// Response for successful wallet registration"
  new_str: "/// Validate that role is buyer or vendor (not arbiter)\nfn validate_client_role(role: &str) -> Result<(), validator::ValidationError> {\n    match role.to_lowercase().as_str() {\n        \"buyer\" | \"vendor\" => Ok(()),\n        \"arbiter\" => Err(validator::ValidationError::new(\n            \"role_not_allowed\",\n        )),\n        _ => Err(validator::ValidationError::new(\"invalid_role\")),\n    }\n}\n\n/// Validate RPC URL: only allow localhost or .onion (no public URLs)\nfn validate_rpc_url(url: &str) -> Result<(), validator::ValidationError> {\n    let parsed = url::Url::parse(url)\n        .map_err(|_| validator::ValidationError::new(\"invalid_url\"))?;\n\n    let host = parsed.host_str()\n        .ok_or_else(|| validator::ValidationError::new(\"no_host\"))?;\n\n    // Only allow localhost, 127.x.x.x, or .onion addresses\n    let is_localhost = host.starts_with(\"127.\")\n        || host.eq(\"localhost\")\n        || host.starts_with(\"::1\");\n    let is_onion = host.ends_with(\".onion\");\n\n    if !is_localhost && !is_onion {\n        return Err(validator::ValidationError::new(\n            \"rpc_url_must_be_local_or_onion\"\n        ));\n    }\n\n    Ok(())\n}\n\n/// Response for successful wallet registration"
}

# Patch 5.2 : Changer validateur du champ rpc_url
Edit {
  file_path: "server/src/handlers/escrow.rs"
  old_str: "    #[validate(url(message = \"Invalid RPC URL format\"))]\n    #[validate(length(min = 10, max = 500, message = \"RPC URL must be 10-500 characters\"))]\n    pub rpc_url: String,"
  new_str: "    #[validate(custom = \"validate_rpc_url\")]\n    #[validate(length(min = 10, max = 500, message = \"RPC URL must be 10-500 characters\"))]\n    pub rpc_url: String,"
}
```

---

## Troubleshooting

### Problème : Import `url::Url` not found
**Cause :** Crate `url` non importé
**Solution :**
```rust
// Top du fichier escrow.rs
use url::Url;
```

### Problème : Users légitimes avec RPC sur LAN bloqués
**Cause :** Validation trop stricte (seulement localhost/.onion)
**Solution (si nécessaire) :** Autoriser private networks :
```rust
let is_private = host.starts_with("10.")
    || host.starts_with("192.168.")
    || host.starts_with("172.16.") // à 172.31.x.x
    ...;

if !is_localhost && !is_onion && !is_private { ... }
```

**MAIS ATTENTION :** Cela ouvre à des SSRF attacks sur le réseau local !

---

## Statut

- [ ] Fonction validate_rpc_url créée
- [ ] Validation custom appliquée à rpc_url
- [ ] Compilation OK (`cargo check`)
- [ ] Tests URLs valides passés (localhost, .onion)
- [ ] Tests URLs invalides passés (rejetées)
- [ ] Edge cases testés (IPv6, ports custom, etc.)

---

**Créé le :** 2025-11-03
**Difficulté :** Facile-Moyenne (⭐⭐☆☆☆)
**Priorité :** HAUTE 🔴
