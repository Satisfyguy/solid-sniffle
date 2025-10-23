## Spec: MoneroRpcClient::new

### Objectif
Créer un nouveau client RPC Monero avec validation OPSEC (localhost uniquement)

### Préconditions
- [ ] URL RPC fournie
- [ ] URL doit être localhost uniquement (OPSEC)

### Input
```rust
url: String  // "http://127.0.0.1:18082"
```

### Output
```rust
Result<MoneroRpcClient, MoneroError>
```

### Erreurs Possibles
- MoneroError::InvalidResponse - URL non-localhost (violation OPSEC)
- MoneroError::NetworkError - Erreur création client HTTP

### Dépendances
```toml
reqwest = "0.11"
```

### Test de Validation
```rust
// Test OPSEC - URLs publiques rejetées
let result = MoneroRpcClient::new("http://0.0.0.0:18082".to_string());
assert!(result.is_err());

let result = MoneroRpcClient::new("http://192.168.1.10:18082".to_string());
assert!(result.is_err());

// Localhost OK
let result = MoneroRpcClient::new("http://127.0.0.1:18082".to_string());
assert!(result.is_ok());
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