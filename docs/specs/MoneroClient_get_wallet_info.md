## Spec: MoneroClient::get_wallet_info

### Objectif
Récupérer les informations complètes du wallet Monero

### Préconditions
- [ ] Client Monero créé
- [ ] RPC Monero accessible

### Input
```rust
&self  // Client Monero
```

### Output
```rust
Result<WalletInfo, Error>
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
let info = client.get_wallet_info().await?;

assert!(!info.version.is_empty());
assert!(info.balance >= 0);
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