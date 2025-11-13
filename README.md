# Monero Marketplace - Tor Hidden Service

**Privacy-first marketplace. Monero-only by design. No compromises.**

Decentralized marketplace with Monero 2-of-3 multisig escrow on Tor. Non-custodial architecture with cryptographic privacy guarantees at the protocol level.

## 🎯 Project Status

**Version:** 0.1.0-alpha
**Status:** 🟡 In Development
**Security Score:** 80/100

### ✅ Implemented Features
- [x] Tor connection verification
- [x] Monero RPC client (localhost isolation)
- [x] `prepare_multisig` (step 1/6)
- [x] Automatic Tor Reality Check system
- [x] Project metrics dashboard

### 🚧 In Progress
- [ ] `make_multisig` (step 2/6)
- [ ] `export_multisig_info` (step 3/6)
- [ ] `import_multisig_info` (step 4/6)
- [ ] Complete multisig setup
- [ ] .onion hidden service

---

## 💎 Why Monero-Only?

**This marketplace exclusively supports Monero (XMR). This is not a limitation—it's an architectural requirement.**

### Core Guarantees

- **🔒 Privacy at Protocol Level:** Ring signatures, stealth addresses, RingCT provide unlinkability and fungibility
- **🚫 No Transparent Chains:** Bitcoin/Ethereum expose transaction graphs—incompatible with privacy-first mandate
- **🎯 Single Attack Surface:** One RPC implementation, one multisig protocol, focused security hardening
- **✅ Technical Honesty:** "Privacy marketplace" backed by cryptography, not marketing claims

### Trade-Off Accepted

- **Market size:** Structurally limited (~0.5% crypto market cap, ~50-100K daily users)
- **Why it's worth it:** Cryptographic privacy guarantees without compromise

**Question:** "Isn't Monero-only too niche?"
**Answer:** The niche size is a direct consequence of architectural integrity. This is success, not failure.

📖 **Full rationale:** See [ADR-001: Monero-Only Architecture](DOX/architecture/ADR-001-MONERO-ONLY-RATIONALE.md)

---

## 🚀 Quick Start

### Prerequisites
- Windows 10/11
- Rust 1.75+
- PowerShell 5.1+
- Tor (daemon or browser)
- Monero CLI (testnet)

### Installation

```powershell
# 1. Clone repository
git clone <repo-url>
cd monero-marketplace

# 2. Setup Monero testnet
.\scripts\setup-monero-testnet.ps1

# 3. Start Tor
tor  # OR launch Tor Browser

# 4. Build
cargo build

# 5. Test
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
- ✅ Wallet RPC bind `127.0.0.1` ONLY
- ✅ Daemon connections via Tor
- ✅ No logs containing .onion/keys
- ✅ All external calls via SOCKS5

---

## 📊 Project Metrics

Launch dashboard:
```powershell
.\scripts\metrics-dashboard.ps1
```

**Latest metrics:**
- LOC: 1034
- Functions: 23 (14 without spec)
- Tests: 4/4 ✅
- Unwraps: 0 ✅
- Security Score: 80/100

---

## 🧪 Tests

### Unit Tests
```powershell
cargo test --workspace
```

### Tor Tests
```powershell
# 1. Start Tor
tor

# 2. Test Tor connection
cargo test --package wallet test_check_tor_connection
```

### Monero RPC Tests
```powershell
# 1. Setup testnet
.\scripts\setup-monero-testnet.ps1

# 2. Test RPC
cargo test --package wallet test_prepare_multisig
```

---

## 📋 Reality Checks

Every network function requires a **Tor Reality Check**.

### Create Reality Check
```powershell
.\scripts\auto-reality-check-tor.ps1 <function_name>
```

### Validate Reality Check
```powershell
.\scripts\validate-reality-check-tor.ps1 <function_name>
```

**Automatic checks:**
- ✅ Tor daemon running
- ✅ No IP leaks
- ✅ RPC isolation (localhost)
- ✅ No sensitive data in logs

---

## 🔐 OPSEC Guidelines

### Absolute Rules

1. **NEVER expose RPC publicly**
   ```bash
   # ✅ GOOD
   --rpc-bind-ip 127.0.0.1

   # ❌ BAD
   --rpc-bind-ip 0.0.0.0
   ```

2. **NEVER log sensitive data**
   - ❌ .onion addresses
   - ❌ View/Spend keys
   - ❌ Passwords
   - ❌ Real IP addresses

3. **ALWAYS route via Tor**
   ```rust
   // ✅ GOOD
   let proxy = Proxy::all("socks5h://127.0.0.1:9050")?;

   // ❌ BAD - direct connection
   reqwest::get("http://example.com")
   ```

4. **ALWAYS validate inputs**
   - No `.unwrap()` without context
   - Return `Result<T, E>`
   - Validate formats (e.g., MultisigV1...)

### Threat Model

**Adversaries considered:**
- ISP / Network surveillance
- Malicious exit nodes
- Blockchain analysis
- Timing correlation attacks
- Global passive adversary

**Mitigations:**
- All traffic via Tor
- Monero for payments (privacy by default)
- Multisig 2-of-3 (neutral arbiter)
- No metadata in transactions
- Random delays for timing

---

## 📁 Project Structure

```
monero-marketplace/
├── .cursorrules              # Cursor rules (Tor-aware)
├── Cargo.toml                # Workspace
├── README.md
│
├── docs/
│   ├── specs/                # Spec per function
│   │   ├── check_tor_connection.md
│   │   └── prepare_multisig.md
│   ├── reality-checks/       # Tor Reality Checks
│   │   ├── tor-check_tor_connection-2024-12-08.md
│   │   └── tor-prepare_multisig-2024-12-08.md
│   └── metrics/              # Project metrics
│
├── scripts/                  # PowerShell scripts
│   ├── new-spec.ps1
│   ├── auto-reality-check-tor.ps1
│   ├── validate-reality-check-tor.ps1
│   ├── setup-monero-testnet.ps1
│   └── metrics-dashboard.ps1
│
├── common/                   # Shared types
│   └── src/
│       ├── error.rs          # TorError, MoneroError
│       ├── types.rs          # TorStatus, MultisigInfo
│       └── lib.rs
│
├── wallet/                   # Monero logic
│   └── src/
│       ├── tor.rs            # check_tor_connection
│       ├── rpc.rs            # MoneroRpcClient
│       └── lib.rs
│
└── cli/                      # CLI interface (TODO)
    └── src/
        └── main.rs
```

---

## 🛠️ Available Scripts

| Script | Command | Description |
|--------|----------|-------------|
| **New Spec** | `.\scripts\new-spec.ps1 <name>` | Create spec from template |
| **Reality Check Tor** | `.\scripts\auto-reality-check-tor.ps1 <name>` | Generate RC with auto tests |
| **Validate RC** | `.\scripts\validate-reality-check-tor.ps1 <name>` | Validate RC before merge |
| **Setup Monero** | `.\scripts\setup-monero-testnet.ps1` | Automatic testnet setup |
| **Metrics** | `.\scripts\metrics-dashboard.ps1` | Metrics dashboard |

---

## 🎓 Development

### Standard Workflow

```powershell
# 1. Create spec
.\scripts\new-spec.ps1 my_function

# 2. Edit spec
code docs/specs/my_function.md

# 3. Code (Cursor detects Tor mode if applicable)

# 4. Reality Check
.\scripts\auto-reality-check-tor.ps1 my_function

# 5. Complete manual tests

# 6. Validate
.\scripts\validate-reality-check-tor.ps1 my_function

# 7. Commit
git add .
git commit -m "[CODE] Implement my_function"
```

### Cursor Rules

Project uses `.cursorrules` v2.1 with:
- ✅ Automatic Tor code detection
- ✅ Block if spec missing
- ✅ Mandatory Reality Check
- ✅ `.unwrap()` forbidden
- ✅ OPSEC validation

---

## 🚨 Troubleshooting

### Tor won't connect
```powershell
# Check process
Get-Process tor

# Test manually
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip

# Restart
tor
```

### Monero RPC unreachable
```powershell
# Check process
Get-Process monero-wallet-rpc

# Test
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}'

# Restart
.\scripts\setup-monero-testnet.ps1
```

### Tests failing
```powershell
# Check that Tor + Monero are running
.\scripts\metrics-dashboard.ps1

# Restart complete setup
.\scripts\setup-monero-testnet.ps1

# Clean and rebuild
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

MIT (to be defined as needed)

---

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/my-function`)
3. **ALWAYS create spec before code**
4. **ALWAYS perform Tor Reality Check**
5. Commit with standard format
6. Push and create PR

**Note:** PRs without validated Reality Check will be rejected.

---

## ⚠️ Disclaimer

**Educational project in development.**

- ❌ DO NOT use in production
- ❌ DO NOT use with real funds
- ✅ Testnet ONLY for now

**OPSEC:** Even on testnet, follow best practices for training purposes.