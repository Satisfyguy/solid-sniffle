## Spec: get_wallet_info

### Objectif
Récupère les informations complètes du wallet Monero (balance, statut multisig, version, etc.)

### Preconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert et deverrouille
- [ ] Daemon testnet synchronise

### Input
```rust
// Aucun parametre requis
```

### Output
```rust
Result<WalletInfo, Error>
```

### Erreurs Possibles
- Error::MoneroRpc - Erreur de communication avec le wallet RPC
- Error::Network - Erreur de reseau ou timeout
- Error::Serialization - Erreur de parsing de la reponse JSON

### Dependances
```toml
[dependencies]
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

### Test de Validation (PowerShell)
```powershell
# Setup
.\scripts\start-testnet.ps1

# Test manuel
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" 
  -Method Post -ContentType "application/json" 
  -Body '{"jsonrpc":"2.0","id":"0","method":"get_balance"}'

# Expected output:
# result : @{balance=0; unlocked_balance=0}
```

### Estimation
- Code: 15 min
- Test: 10 min
- Total: 25 min

### Status
- [x] Spec validee
- [x] Code ecrit
- [x] Tests passent
- [ ] Reality check fait
