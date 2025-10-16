# Spec: make_multisig

## Objectif
Créer wallet multisig 2-of-3 en combinant les multisig_info des 3 participants (buyer, seller, arbitre)

## Préconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert (ex: buyer)
- [ ] `prepare_multisig` déjà appelé sur les 3 wallets
- [ ] On a récupéré les 3 `multisig_info` strings
- [ ] Wallet PAS encore en mode multisig finalisé

## Input
```rust
threshold: u32,              // 2 (2-of-3 multisig)
multisig_info: Vec<String>,  // [seller_info, arb_info] (2 autres participants)
```

## Output
```rust
Result<MakeMultisigResult, MoneroError>

struct MakeMultisigResult {
    address: String,      // Adresse multisig partagée (commence par "5" sur testnet)
    multisig_info: String, // Info pour étape suivante (export/import)
}
```

## Erreurs Possibles
- `MoneroError::RpcUnreachable` - RPC pas accessible
- `MoneroError::AlreadyMultisig` - Wallet déjà finalisé en multisig
- `MoneroError::ValidationError` - multisig_info invalides
- `MoneroError::RpcError` - Erreur Monero (ex: threshold invalide)
- `MoneroError::WalletLocked` - Wallet verrouillé
- `MoneroError::WalletBusy` - Autre opération en cours

## Flow Multisig 2-of-3
```
Buyer Wallet          Seller Wallet         Arbitre Wallet
    |                      |                      |
    | prepare_multisig     | prepare_multisig     | prepare_multisig
    |-------------------->|-------------------->|
    |  info_buyer          |  info_seller         |  info_arb
    |                      |                      |
    |                      |                      |
    | make_multisig(2, [info_seller, info_arb])   |
    |<--------------------------------------------|
    |  → address_multisig  |                      |
    |  → info_buyer_2      |                      |
    |                      |                      |
    |                      | make_multisig(2, [info_buyer, info_arb])
    |                      |<---------------------|
    |                      |  → address_multisig  |
    |                      |  → info_seller_2     |
    |                      |                      |
    |                      |                      | make_multisig(2, [info_buyer, info_seller])
    |                      |                      |<----
    |                      |                      |  → address_multisig
    |                      |                      |  → info_arb_2
    |                      |                      |
    | ✅ Tous ont même address_multisig           |
```

## Dépendances
```toml
# Déjà dans workspace
tokio, reqwest, serde, serde_json, anyhow, thiserror, tracing
```

## Test de Validation
```powershell
# SETUP: 3 wallets testnet avec prepare_multisig fait

# 1. Lancer 3 RPC instances (ports différents)
# Buyer sur 18082
Start-Process monero-wallet-rpc -ArgumentList `
  "--testnet","--wallet-file","buyer","--password","""", `
  "--rpc-bind-ip","127.0.0.1","--rpc-bind-port","18082", `
  "--disable-rpc-login" -WindowStyle Hidden

# Seller sur 18083
Start-Process monero-wallet-rpc -ArgumentList `
  "--testnet","--wallet-file","seller","--password","""", `
  "--rpc-bind-ip","127.0.0.1","--rpc-bind-port","18083", `
  "--disable-rpc-login" -WindowStyle Hidden

# Arbitre sur 18084
Start-Process monero-wallet-rpc -ArgumentList `
  "--testnet","--wallet-file","arb","--password","""", `
  "--rpc-bind-ip","127.0.0.1","--rpc-bind-port","18084", `
  "--disable-rpc-login" -WindowStyle Hidden

# 2. Préparer multisig sur chaque wallet
$buyer_info = (Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" `
  -Method Post -ContentType "application/json" `
  -Body '{"jsonrpc":"2.0","id":"0","method":"prepare_multisig"}').result.multisig_info

$seller_info = (Invoke-RestMethod -Uri "http://127.0.0.1:18083/json_rpc" `
  -Method Post -ContentType "application/json" `
  -Body '{"jsonrpc":"2.0","id":"0","method":"prepare_multisig"}').result.multisig_info

$arb_info = (Invoke-RestMethod -Uri "http://127.0.0.1:18084/json_rpc" `
  -Method Post -ContentType "application/json" `
  -Body '{"jsonrpc":"2.0","id":"0","method":"prepare_multisig"}').result.multisig_info

# 3. Make multisig sur buyer (avec infos seller + arb)
$body = @{
  jsonrpc = "2.0"
  id = "0"
  method = "make_multisig"
  params = @{
    threshold = 2
    multisig_info = @($seller_info, $arb_info)
  }
} | ConvertTo-Json -Depth 10

Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body

# Expected:
# result : @{
#   address = "5..."  (adresse testnet multisig)
#   multisig_info = "MultisigxV1..."
# }

# 4. Répéter pour seller et arb

# 5. Vérifier que les 3 ont même adresse
```

## Estimation
- Implémentation RPC: 30 min
- Tests: 20 min
- Reality Check: 15 min
- Total: ~1h

## Status
- [ ] Spec créée
- [ ] Implémentation RPC
- [ ] Implémentation MultisigManager
- [ ] Implémentation MoneroClient
- [ ] Tests unitaires
- [ ] Security theatre check
- [ ] Reality Check créé
- [ ] Reality Check validé
