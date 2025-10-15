# OPSEC Guidelines - Monero Marketplace

## 🎯 Objectif

Ce document définit les **pratiques de sécurité opérationnelle** (OPSEC) obligatoires pour le développement et déploiement du marketplace Monero sur Tor.

---

## 🔴 Règles Critiques (JAMAIS enfreindre)

### 1. Isolation Réseau

#### ✅ À FAIRE
- **Monero RPC** bind `127.0.0.1` UNIQUEMENT
  ```bash
  --rpc-bind-ip 127.0.0.1
  ```
- Toutes connexions externes via **Tor SOCKS5**
  ```rust
  Proxy::all("socks5h://127.0.0.1:9050")
  ```
- Daemon Monero distant via **Tor**

#### ❌ INTERDIT
- Bind RPC sur `0.0.0.0` ou IP publique
- Connexions HTTP directes (bypass Tor)
- DNS clearnet (toujours `socks5h://` pas `socks5://`)

---

### 2. Pas de Logs Sensibles

#### ❌ JAMAIS Logger
- Adresses `.onion`
- View keys / Spend keys
- Passwords / Seeds
- IPs réelles
- Multisig info complète

#### ✅ Logger (acceptable)
- Metadata (timestamp, event type)
- Error types (sans détails sensibles)
- Performance metrics

**Exemple:**
```rust
// ❌ MAUVAIS
log::info!("Connected to {}.onion", onion_address);

// ✅ BON
log::info!("Connected to hidden service");
```

---

### 3. Validation Inputs

#### Toujours Valider
- Formats (ex: `MultisigV1...`)
- Longueurs (pas > expected)
- Caractères (alphanumeric seulement si applicable)

#### Jamais `.unwrap()` Sans Contexte
```rust
// ❌ MAUVAIS
let info = rpc_call().await.unwrap();

// ✅ BON
let info = rpc_call().await
    .context("Failed to get multisig info")?;
```

---

## 🟡 Bonnes Pratiques

### User-Agent Générique
```rust
.user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
```
Utiliser **Tor Browser UA** standard.

### Timeouts Généreux
Tor = lent. Utiliser **≥30s**:
```rust
.timeout(Duration::from_secs(30))
```

### Timestamps Arrondis
Pas de `2024-12-08T16:45:23.387642Z`.  
Utiliser: `2024-12-08T16:45:00Z` (arrondi minute).

### Random Delays
Ajouter délais aléatoires (anti-timing attacks):
```rust
let delay = rand::thread_rng().gen_range(100..500);
tokio::time::sleep(Duration::from_millis(delay)).await;
```

---

## 🧅 Spécifique Tor

### Circuits Isolés
Pour identités multiples:
```rust
// Circuit 1 (buyer)
Proxy::all("socks5h://127.0.0.1:9050")?

// Circuit 2 (seller) - utiliser port différent ou stream isolation
Proxy::all("socks5h://127.0.0.1:9050")?
    .header("X-Tor-Stream-Isolation", "seller")
```

### Bridges (si Tor bloqué)
```toml
UseBridges 1
Bridge obfs4 <bridge-address>
```

### Guard Nodes
Pinning recommandé (production):
```toml
EntryNodes {fingerprint1},{fingerprint2},{fingerprint3}
```

---

## 💰 Spécifique Monero

### Daemon via Tor
Si daemon distant:
```bash
--proxy 127.0.0.1:9050
--tx-proxy tor,127.0.0.1:9050
```

### Pas de Réutilisation Adresses
1 adresse = 1 transaction.

### Churn Outputs
Envoyer à soi-même pour briser links:
```bash
monero-wallet-cli churn
```

### Multisig Info Exchange
**TOUJOURS** échanger via canaux chiffrés:
- ✅ .onion hidden service
- ✅ PGP encrypted
- ❌ Email cleartext
- ❌ Discord/Telegram

---

## 🚨 Incident Response

### Si Fuite Suspectée

1. **STOP** immédiatement toutes opérations
2. **Déconnecter** du réseau
3. **Audit** logs pour confirmer fuite
4. **Documenter** incident
5. **Rotate** toutes identités/clés
6. **Post-mortem** + améliorer mitigations

### Contacts d'Urgence
- Security lead: [email]
- PGP Key: [fingerprint]

---

## 📚 Ressources

- [Tor Best Practices](https://2019.www.torproject.org/docs/documentation.html.en)
- [Whonix Documentation](https://www.whonix.org/)
- [Monero OPSEC](https://www.getmonero.org/resources/user-guides/)
- [Breaking Monero](https://www.youtube.com/playlist?list=PLsSYUeVwrHBnAUre2G_LYDsdo-tD0ov-y) (educational)

---

**Last Updated:** 2024-12-08  
**Version:** 1.0