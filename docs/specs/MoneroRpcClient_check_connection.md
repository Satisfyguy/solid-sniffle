## Spec: MoneroRpcClient::check_connection

### Objectif
Vérifier que le RPC Monero est accessible et répond

### Préconditions
- [ ] Client RPC créé
- [ ] RPC Monero tourne sur localhost:18082

### Input
```rust
&self  // Client RPC
```

### Output
```rust
Result<(), MoneroError>
```

### Erreurs Possibles
- MoneroError::RpcUnreachable - RPC pas accessible
- MoneroError::NetworkError - Erreur réseau/timeout

### Dépendances
```toml
reqwest = "0.11"
```

### Test de Validation
```rust
let client = MoneroRpcClient::new("http://127.0.0.1:18082".to_string())?;

// Test avec RPC accessible
let result = client.check_connection().await;
assert!(result.is_ok());

// Test avec RPC inaccessible
let client = MoneroRpcClient::new("http://127.0.0.1:19999".to_string())?;
let result = client.check_connection().await;
assert!(result.is_err());
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