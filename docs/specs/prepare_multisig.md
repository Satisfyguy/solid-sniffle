## Spec: prepare_multisig

### Objectif
Appeler Monero RPC prepare_multisig pour obtenir multisig_info (étape 1/6 du setup multisig)

### Préconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert (testnet)
- [ ] Wallet PAS déjà en mode multisig
- [ ] Tor daemon running (connexions daemon via Tor recommandées)

### Input
```rust
rpc_url: String,  // "http://127.0.0.1:18082"
```

### Output
```rust
Result<MultisigInfo, MoneroError>

struct MultisigInfo {
    multisig_info: String,  // "MultisigV1..."
}

enum MoneroError {
    RpcUnreachable,
    AlreadyMultisig,
    WalletLocked,
    InvalidResponse(String),
    NetworkError(String),
}
```

### Erreurs Possibles
- MoneroError::RpcUnreachable - RPC pas accessible (pas lancé)
- MoneroError::AlreadyMultisig - Wallet déjà en mode multisig
- MoneroError::WalletLocked - Wallet verrouillé (password requis)
- MoneroError::InvalidResponse - Réponse JSON invalide
- MoneroError::NetworkError - Timeout ou erreur réseau

### Dépendances
```toml
# Déjà dans workspace
tokio, reqwest, serde, serde_json, anyhow, thiserror
```

### Test de Validation
```powershell
# 1. Lancer daemon testnet
cd C:\monero-dev\monero-x86_64-w64-mingw32-*
Start-Process .\monerod.exe -ArgumentList "--testnet","--detach"

# Attendre 10s
Start-Sleep 10

# 2. Créer wallet testnet (si pas déjà fait)
.\monero-wallet-cli.exe --testnet --generate-new-wallet buyer --password ""

# 3. Lancer RPC
Start-Process .\monero-wallet-rpc.exe -ArgumentList `
  "--testnet", `
  "--wallet-file","buyer", `
  "--password","""", `
  "--rpc-bind-ip","127.0.0.1", `
  "--rpc-bind-port","18082", `
  "--disable-rpc-login", `
  "--daemon-address","127.0.0.1:28081" `
  -WindowStyle Hidden

Start-Sleep 5

# 4. Tester RPC manuellement
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" `
  -Method Post `
  -ContentType "application/json" `
  -Body '{"jsonrpc":"2.0","id":"0","method":"prepare_multisig"}'

# Expected:
# result : @{multisig_info=MultisigV1KF...}

# 5. Tester code Rust
cargo test test_prepare_multisig
```

### Estimation
- Code: 30 min
- Test: 20 min
- Total: 50 min

### Status
- [x] Spec validée
- [ ] Code écrit
- [ ] Tests passent
- [ ] Reality check fait