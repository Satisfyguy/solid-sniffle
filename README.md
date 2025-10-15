# Monero Marketplace - Tor Hidden Service

Marketplace décentralisé avec escrow Monero 2-of-3 multisig sur Tor.

## 🎯 Statut Projet

**Version:** 0.1.0-alpha  
**Status:** 🟡 En développement  
**Security Score:** 80/100

### ✅ Fonctionnalités Implémentées
- [x] Vérification connexion Tor
- [x] Client RPC Monero (localhost isolation)
- [x] `prepare_multisig` (étape 1/6)
- [x] Système Reality Check Tor automatique
- [x] Dashboard métriques projet

### 🚧 En Cours
- [ ] `make_multisig` (étape 2/6)
- [ ] `export_multisig_info` (étape 3/6)
- [ ] `import_multisig_info` (étape 4/6)
- [ ] Setup multisig complet
- [ ] Hidden service .onion

---

## 🚀 Quick Start

### Prérequis
- Windows 10/11
- Rust 1.75+
- PowerShell 5.1+
- Tor (daemon ou browser)
- Monero CLI (testnet)

### Installation

```powershell
# 1. Cloner repo
git clone <repo-url>
cd monero-marketplace

# 2. Setup Monero testnet
.\scripts\setup-monero-testnet.ps1

# 3. Lancer Tor
tor  # OU lancer Tor Browser

# 4. Compiler
cargo build

# 5. Tester
cargo test --workspace
```

---

## 🧅 Architecture Tor

```
┌─────────────┐
│   Client    │
│  (Browser)  │
└──────┬──────┘
       │ HTTPS
       │
┌──────▼──────────────────────┐
│   Tor Hidden Service        │
│   (your-service.onion)      │
│                             │
│  ┌───────────────────────┐  │
│  │  Marketplace Server   │  │
│  │  (Rust + Actix-web)  │  │
│  └──────────┬────────────┘  │
│             │                │
│  ┌──────────▼────────────┐  │
│  │  Monero Wallet RPC    │  │
│  │  (127.0.0.1:18082)   │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
             │
             │ Tor
             │
      ┌──────▼──────┐
      │   Monero    │
      │   Daemon    │
      │  (testnet)  │
      └─────────────┘
```

**OPSEC Critical:**
- ✅ Wallet RPC bind `127.0.0.1` UNIQUEMENT
- ✅ Daemon connections via Tor
- ✅ Pas de logs contenant .onion/keys
- ✅ Tous les appels externes via SOCKS5

---

## 📊 Métriques Projet

Lancer dashboard:
```powershell
.\scripts\metrics-dashboard.ps1
```

**Dernières métriques:**
- LOC: 1034
- Functions: 23 (14 sans spec)
- Tests: 4/4 ✅
- Unwraps: 0 ✅
- Security Score: 80/100

---

## 🧪 Tests

### Tests Unitaires
```powershell
cargo test --workspace
```

### Tests Tor
```powershell
# 1. Lancer Tor
tor

# 2. Tester connexion Tor
cargo test --package wallet test_check_tor_connection
```

### Tests Monero RPC
```powershell
# 1. Setup testnet
.\scripts\setup-monero-testnet.ps1

# 2. Tester RPC
cargo test --package wallet test_prepare_multisig
```

---

## 📋 Reality Checks

Chaque fonction réseau a un **Reality Check Tor** obligatoire.

### Créer Reality Check
```powershell
.\scripts\auto-reality-check-tor.ps1 <function_name>
```

### Valider Reality Check
```powershell
.\scripts\validate-reality-check-tor.ps1 <function_name>
```

**Checks automatiques:**
- ✅ Tor daemon running
- ✅ Pas de fuites IP
- ✅ RPC isolation (localhost)
- ✅ Pas de données sensibles dans logs

---

## 🔐 OPSEC Guidelines

### Règles Absolues

1. **JAMAIS exposer RPC publiquement**
   ```bash
   # ✅ BON
   --rpc-bind-ip 127.0.0.1
   
   # ❌ MAUVAIS
   --rpc-bind-ip 0.0.0.0
   ```

2. **JAMAIS logger de données sensibles**
   - ❌ Adresses .onion
   - ❌ View/Spend keys
   - ❌ Passwords
   - ❌ IPs réelles

3. **TOUJOURS router via Tor**
   ```rust
   // ✅ BON
   let proxy = Proxy::all("socks5h://127.0.0.1:9050")?;
   
   // ❌ MAUVAIS - connexion directe
   reqwest::get("http://example.com")
   ```

4. **TOUJOURS valider inputs**
   - Pas de `.unwrap()` sans contexte
   - Retourner `Result<T, E>`
   - Valider formats (ex: MultisigV1...)

### Threat Model

**Adversaires considérés:**
- ISP / Surveillance réseau
- Exit nodes malveillants
- Blockchain analysis
- Timing correlation attacks
- Global passive adversary

**Mitigations:**
- Tout le trafic via Tor
- Monero pour paiements (privacy by default)
- Multisig 2-of-3 (arbitre neutre)
- Pas de metadata dans transactions
- Random delays pour timing

---

## 📁 Structure Projet

```
monero-marketplace/
├── .cursorrules              # Rules Cursor (Tor-aware)
├── Cargo.toml                # Workspace
├── README.md
│
├── docs/
│   ├── specs/                # Spec par fonction
│   │   ├── check_tor_connection.md
│   │   └── prepare_multisig.md
│   ├── reality-checks/       # Reality checks Tor
│   │   ├── tor-check_tor_connection-2024-12-08.md
│   │   └── tor-prepare_multisig-2024-12-08.md
│   └── metrics/              # Métriques projet
│
├── scripts/                  # Scripts PowerShell
│   ├── new-spec.ps1
│   ├── auto-reality-check-tor.ps1
│   ├── validate-reality-check-tor.ps1
│   ├── setup-monero-testnet.ps1
│   └── metrics-dashboard.ps1
│
├── common/                   # Types partagés
│   └── src/
│       ├── error.rs          # TorError, MoneroError
│       ├── types.rs          # TorStatus, MultisigInfo
│       └── lib.rs
│
├── wallet/                   # Logique Monero
│   └── src/
│       ├── tor.rs            # check_tor_connection
│       ├── rpc.rs            # MoneroRpcClient
│       └── lib.rs
│
└── cli/                      # Interface CLI (TODO)
    └── src/
        └── main.rs
```

---

## 🛠️ Scripts Disponibles

| Script | Commande | Description |
|--------|----------|-------------|
| **New Spec** | `.\scripts\new-spec.ps1 <name>` | Créer spec depuis template |
| **Reality Check Tor** | `.\scripts\auto-reality-check-tor.ps1 <name>` | Générer RC avec tests auto |
| **Validate RC** | `.\scripts\validate-reality-check-tor.ps1 <name>` | Valider RC avant merge |
| **Setup Monero** | `.\scripts\setup-monero-testnet.ps1` | Setup testnet automatique |
| **Metrics** | `.\scripts\metrics-dashboard.ps1` | Dashboard métriques |

---

## 🎓 Développement

### Workflow Standard

```powershell
# 1. Créer spec
.\scripts\new-spec.ps1 my_function

# 2. Éditer spec
code docs/specs/my_function.md

# 3. Coder (Cursor détecte mode Tor si applicable)

# 4. Reality Check
.\scripts\auto-reality-check-tor.ps1 my_function

# 5. Compléter tests manuels

# 6. Valider
.\scripts\validate-reality-check-tor.ps1 my_function

# 7. Commit
git add .
git commit -m "[CODE] Implement my_function"
```

### Cursor Rules

Le projet utilise `.cursorrules` v2.1 avec:
- ✅ Détection automatique code Tor
- ✅ Blocage si spec manquante
- ✅ Reality Check obligatoire
- ✅ Interdiction `.unwrap()`
- ✅ Validation OPSEC

---

## 🚨 Dépannage

### Tor ne se connecte pas
```powershell
# Vérifier process
Get-Process tor

# Tester manuellement
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip

# Relancer
tor
```

### Monero RPC injoignable
```powershell
# Vérifier process
Get-Process monero-wallet-rpc

# Tester
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}'

# Relancer
.\scripts\setup-monero-testnet.ps1
```

### Tests échouent
```powershell
# Vérifier que Tor + Monero tournent
.\scripts\metrics-dashboard.ps1

# Relancer setup complet
.\scripts\setup-monero-testnet.ps1

# Nettoyer et rebuild
cargo clean
cargo build
cargo test
```

---

## 📖 Ressources

- [Tor Project](https://www.torproject.org/)
- [Monero Documentation](https://www.getmonero.org/resources/developer-guides/)
- [Monero RPC Calls](https://www.getmonero.org/resources/developer-guides/wallet-rpc.html)
- [Multisig Guide](https://monerodocs.org/multisignature/)

---

## 📄 License

MIT (à définir selon besoins)

---

## 🤝 Contribution

1. Fork le repo
2. Créer branch feature (`git checkout -b feature/my-function`)
3. **TOUJOURS créer spec avant code**
4. **TOUJOURS faire Reality Check Tor**
5. Commit avec format standard
6. Push et créer PR

**Note:** PRs sans Reality Check validé seront rejetées.

---

## ⚠️ Disclaimer

**Projet éducatif en développement.**

- ❌ Ne PAS utiliser en production
- ❌ Ne PAS utiliser avec vrais fonds
- ✅ Testnet UNIQUEMENT pour l'instant

**OPSEC:** Même en testnet, suivre bonnes pratiques pour habituation.