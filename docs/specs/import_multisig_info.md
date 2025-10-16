# Spec: import_multisig_info

## Objectif
Importer les informations multisig des autres participants pour synchroniser le wallet (étape 4/6 du setup multisig)

## Préconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert et en mode multisig
- [ ] `export_multisig_info` appelé sur tous les wallets
- [ ] Infos exportées récupérées des autres participants (via canal sécurisé)
- [ ] **NOTE**: Cette opération doit être faite DEUX fois (sync round 1 et 2)

## Input
```rust
infos: Vec<String>  // Infos exportées des AUTRES participants (N-1 infos)
```

## Output
```rust
Result<ImportMultisigInfoResult, MoneroError>

struct ImportMultisigInfoResult {
    n_outputs: u64,  // Nombre d'outputs importés
}
```

## Erreurs Possibles
- `MoneroError::RpcUnreachable` - RPC pas accessible
- `MoneroError::NotMultisig` - Wallet pas en mode multisig
- `MoneroError::WalletLocked` - Wallet verrouillé
- `MoneroError::ValidationError` - Infos invalides ou incompatibles
- `MoneroError::RpcError` - Erreur Monero (ex: infos déjà importées)
- `MoneroError::InvalidResponse` - Réponse RPC invalide

## Flow Complet Export/Import

```
ROUND 1:
========
1. Buyer exports  → info_buyer_r1
2. Seller exports → info_seller_r1
3. Arb exports    → info_arb_r1

4. Buyer imports  [info_seller_r1, info_arb_r1]    → n_outputs: X
5. Seller imports [info_buyer_r1, info_arb_r1]     → n_outputs: X
6. Arb imports    [info_buyer_r1, info_seller_r1]  → n_outputs: X

ROUND 2:
========
7. Buyer exports  → info_buyer_r2
8. Seller exports → info_seller_r2
9. Arb exports    → info_arb_r2

10. Buyer imports  [info_seller_r2, info_arb_r2]    → n_outputs: Y
11. Seller imports [info_buyer_r2, info_arb_r2]     → n_outputs: Y
12. Arb imports    [info_buyer_r2, info_seller_r2]  → n_outputs: Y

✅ SYNCHRONISATION COMPLÈTE
```

## Dépendances
```toml
# Déjà dans workspace
tokio, reqwest, serde, serde_json, anyhow, thiserror, tracing
```

## Test de Validation
```powershell
# SETUP: 3 wallets avec export_multisig_info fait (round 1)

# Récupérer exports (voir export_multisig_info.md)
$buyer_export = "..." # De export précédent
$seller_export = "..." # De export précédent
$arb_export = "..." # De export précédent

# 1. Import sur buyer (importe seller + arb)
$body = @{
  jsonrpc = "2.0"
  id = "0"
  method = "import_multisig_info"
  params = @{
    info = @($seller_export, $arb_export)
  }
} | ConvertTo-Json -Depth 10

$buyer_import = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body

Write-Output "Buyer imported $($buyer_import.result.n_outputs) outputs"

# 2. Import sur seller (importe buyer + arb)
$body = @{
  jsonrpc = "2.0"
  id = "0"
  method = "import_multisig_info"
  params = @{
    info = @($buyer_export, $arb_export)
  }
} | ConvertTo-Json -Depth 10

$seller_import = Invoke-RestMethod -Uri "http://127.0.0.1:18083/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body

Write-Output "Seller imported $($seller_import.result.n_outputs) outputs"

# 3. Import sur arbitre (importe buyer + seller)
$body = @{
  jsonrpc = "2.0"
  id = "0"
  method = "import_multisig_info"
  params = @{
    info = @($buyer_export, $seller_export)
  }
} | ConvertTo-Json -Depth 10

$arb_import = Invoke-RestMethod -Uri "http://127.0.0.1:18084/json_rpc" `
  -Method Post -ContentType "application/json" -Body $body

Write-Output "Arb imported $($arb_import.result.n_outputs) outputs"

# Expected:
# Chaque import retourne n_outputs >= 0
# Pas d'erreurs
# Prêt pour round 2 (export → import à nouveau)
```

## Validation Stricte

### Validation Input
- **Nombre d'infos**: Exactement N-1 (pour 3 participants: 2 infos)
- **Format**: Chaque info non vide, longueur > 100 chars
- **Unicité**: Pas de doublons
- **Compatibilité**: Infos doivent provenir du même setup multisig

### Validation Output
- **n_outputs**: >= 0 (peut être 0 si pas encore de transactions)
- **Succès RPC**: Pas d'erreurs

### Préconditions
- Wallet en mode multisig (vérifier avec `is_multisig()`)
- Export fait avant import (pour chaque round)
- Infos proviennent des autres participants (pas de self-import)

## Estimation
- Implémentation RPC: 25 min
- Tests: 20 min
- Reality Check: 10 min
- Total: ~55 min

## Status
- [ ] Spec créée
- [ ] Implémentation RPC
- [ ] Implémentation MultisigManager
- [ ] Tests unitaires
- [ ] Security theatre check
- [ ] Reality Check créé
- [ ] Reality Check validé

## Notes Importantes

### Ordre des Opérations
1. **Tous exportent** (export_multisig_info)
2. **Échanger infos** via canal sécurisé
3. **Tous importent** (import_multisig_info)
4. **Répéter 1-3** pour round 2

### Erreurs Communes
- **"Already imported"**: Infos déjà importées, passer au round suivant
- **"Invalid info"**: Info corrompue ou pas du bon setup
- **"Wrong number of infos"**: Doit être exactement N-1

### Synchronisation
Après les 2 rounds, tous les wallets doivent être synchronisés:
- Même balance
- Même nombre d'outputs
- Prêts pour créer/signer transactions multisig

### Sécurité Échange
Les infos exportées contiennent des données cryptographiques sensibles:
- **JAMAIS** envoyer par email non chiffré
- **JAMAIS** poster sur forum public
- **TOUJOURS** utiliser chiffrement end-to-end (PGP, Signal, Tor)
- **VÉRIFIER** identité des participants (clés PGP, fingerprints)
