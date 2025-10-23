## Spec: MoneroClient::new

### Objectif
Créer un client Monero de haut niveau avec RPC et multisig

### Préconditions
- [ ] Configuration Monero fournie
- [ ] RPC client créé avec succès

### Input
```rust
config: MoneroConfig
```

### Output
```rust
Result<MoneroClient, Error>
```

### Erreurs Possibles
- Error::Config - Configuration invalide
- Error::MoneroRpc - Erreur création RPC client

### Dépendances
```toml
monero-marketplace-common = { path = "../common" }
```

### Test de Validation
```rust
let config = MoneroConfig::default();
let client = MoneroClient::new(config)?;

// Client créé avec succès
assert!(client.is_ok());
```

### Estimation
- Code: 5 min
- Test: 5 min
- Total: 10 min

### Status
- [x] Spec validée
- [x] Code écrit
- [x] Tests passent
- [x] Reality check fait