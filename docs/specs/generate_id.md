## Spec: generate_id

### Objectif
Générer un ID unique basé sur le timestamp système

### Préconditions
- [ ] Système accessible
- [ ] Time système valide

### Input
```rust
// Aucun paramètre
```

### Output
```rust
String  // ID unique hexadécimal
```

### Erreurs Possibles
- Panic si time système invalide (très rare)

### Dépendances
```toml
std::collections::hash_map::DefaultHasher
```

### Test de Validation
```rust
let id1 = generate_id();
let id2 = generate_id();

assert!(!id1.is_empty());
assert!(id1 != id2);  // IDs différents
assert!(id1.len() > 10);  // Longueur raisonnable
```

### Estimation
- Code: 2 min
- Test: 2 min
- Total: 4 min

### Status
- [x] Spec validée
- [x] Code écrit
- [x] Tests passent
- [x] Reality check fait