# Spec: export_multisig_info

## Objectif
Exporter les informations multisig du wallet pour synchronisation avec les autres participants (étape 3/6 du setup multisig)

## Préconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert
- [ ] `make_multisig` déjà appelé avec succès
- [ ] Wallet est maintenant en mode multisig
- [ ] **NOTE**: Cette opération doit être faite DEUX fois (sync round 1 et 2)

## Input
```rust
// Pas de paramètres
```

## Output
```rust
Result<ExportMultisigInfoResult, MoneroError>

struct ExportMultisigInfoResult {
    info: String,  // Informations multisig à partager avec autres participants
}
```

## Erreurs Possibles
- `MoneroError::RpcUnreachable` - RPC pas accessible
- `MoneroError::NotMultisig` - Wallet pas encore en mode multisig
- `MoneroError::WalletLocked` - Wallet verrouillé
- `MoneroError::InvalidResponse` - Réponse RPC invalide
- `MoneroError::RpcError` - Erreur Monero générique

## Flow Multisig avec Export/Import

```
Étape 1: prepare_multisig (3 wallets)
  Buyer:  MultisigV1_buyer
  Seller: MultisigV1_seller
  Arb:    MultisigV1_arb

Étape 2: make_multisig (3 wallets)
  Buyer:  address_shared, MultisigxV1_buyer
  Seller: address_shared, MultisigxV1_seller
  Arb:    address_shared, MultisigxV1_arb
  ✅ Tous ont même address_shared

Étape 3: export_multisig_info (3 wallets) - ROUND 1
  Buyer:  → export_info_buyer_r1
  Seller: → export_info_seller_r1
  Arb:    → export_info_arb_r1

Étape 4: import_multisig_info (3 wallets) - ROUND 1
  Buyer:  import [export_info_seller_r1, export_info_arb_r1]
  Seller: import [export_info_buyer_r1, export_info_arb_r1]
  Arb:    import [export_info_buyer_r1, export_info_seller_r1]

Étape 5: export_multisig_info (3 wallets) - ROUND 2
  Buyer:  → export_info_buyer_r2
  Seller: → export_info_seller_r2
  Arb:    → export_info_arb_r2

Étape 6: import_multisig_info (3 wallets) - ROUND 2
  Buyer:  import [export_info_seller_r2, export_info_arb_r2]
  Seller: import [export_info_buyer_r2, export_info_arb_r2]
  Arb:    import [export_info_buyer_r2, export_info_seller_r2]

✅ Tous les wallets maintenant synchronized et prêts pour transactions
```

## Dépendances
```toml
# Déjà dans workspace
tokio, reqwest, serde, serde_json, anyhow, thiserror, tracing
```

## Test de Validation
```powershell
# SETUP: 3 wallets avec make_multisig déjà fait

# 1. Export sur buyer
$body = '{"jsonrpc":"2.0","id":"0","method":"export_multisig_info"}'
$buyer_export = (Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body).result.info

Write-Output "Buyer export: $($buyer_export.Substring(0, 50))..."

# 2. Export sur seller
$seller_export = (Invoke-RestMethod -Uri "http://127.0.0.1:18083/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body).result.info

Write-Output "Seller export: $($seller_export.Substring(0, 50))..."

# 3. Export sur arbitre
$arb_export = (Invoke-RestMethod -Uri "http://127.0.0.1:18084/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body).result.info

Write-Output "Arb export: $($arb_export.Substring(0, 50))..."

# Expected:
# Chaque export retourne une string unique
# Format: commencé avec des caractères hexadécimaux
# Longueur: variable (100-2000 chars typiquement)
```

## Validation Stricte

### Format Export Info
- **Type**: String hexadécimale ou base64
- **Longueur**: 100-5000 caractères
- **Non vide**: Obligatoire
- **Unique**: Chaque wallet génère une info différente

### Préconditions Vérifiées
- Wallet doit être en mode multisig (vérifier avec `is_multisig()`)
- Wallet ne doit pas être locked

## Estimation
- Implémentation RPC: 20 min
- Tests: 15 min
- Reality Check: 10 min
- Total: ~45 min

## Status
- [ ] Spec créée
- [ ] Implémentation RPC
- [ ] Implémentation MultisigManager
- [ ] Tests unitaires
- [ ] Security theatre check
- [ ] Reality Check créé
- [ ] Reality Check validé

## Notes Importantes

### Synchronisation Rounds
Le multisig Monero nécessite **2 rounds** d'export/import:
1. **Round 1**: Après make_multisig
2. **Round 2**: Après premier import

### Pourquoi 2 Rounds?
Les wallets multisig doivent synchroniser leurs clés publiques en deux étapes pour établir un état cryptographique cohérent. C'est une exigence du protocole Monero multisig.

### Échange Out-of-Band
Les infos exportées doivent être échangées via un canal sécurisé:
- PGP-encrypted email
- Signal/Session messenger
- Tor hidden service (.onion)
- **JAMAIS** via HTTP non chiffré
