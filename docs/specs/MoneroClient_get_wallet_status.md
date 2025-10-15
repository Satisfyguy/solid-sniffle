## Spec: MoneroClient::get_wallet_status

### Objectif
Récupérer le statut du wallet (multisig, balance, etc.)

### Préconditions
- [ ] Client Monero créé
- [ ] RPC Monero accessible

### Input
```rust
&self  // Client Monero
```

### Output
```rust
Result<WalletStatus, Error>
```

### Erreurs Possibles
- Error::MoneroRpc - Erreur RPC
- Error::Network - Erreur réseau
- Error::Serialization - Erreur parsing JSON

### Dépendances
```toml
reqwest = "0.11"
serde = "1.0"
```

### Test de Validation
```rust
let client = MoneroClient::new(config)?;
let status = client.get_wallet_status().await?;

assert!(status.balance >= 0);
assert!(status.unlocked_balance >= 0);
```

### Estimation
- Code: 10 min
- Test: 5 min
- Total: 15 min

### Status
- [x] Spec validée
- [x] Code écrit
- [x] Tests passent
- [x] Reality check fait