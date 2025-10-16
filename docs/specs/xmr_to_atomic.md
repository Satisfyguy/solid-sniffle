## Spec: xmr_to_atomic

### Objectif
Convertir un montant XMR en unités atomiques

### Préconditions
- [ ] Montant XMR valide (positif)
- [ ] Pas de dépassement de capacité

### Input
```rust
xmr: f64  // Montant en XMR
```

### Output
```rust
Result<u64, String>  // Unités atomiques ou erreur
```

### Erreurs Possibles
- "Invalid amount" - Montant négatif ou invalide
- "Overflow" - Dépassement de capacité

### Dépendances
```toml
// Aucune dépendance externe
```

### Test de Validation
```rust
assert_eq!(xmr_to_atomic(1.0).unwrap(), 1_000_000_000_000);
assert_eq!(xmr_to_atomic(0.5).unwrap(), 500_000_000_000);
assert!(xmr_to_atomic(-1.0).is_err());
```

### Estimation
- Code: 3 min
- Test: 2 min
- Total: 5 min

### Status
- [x] Spec validée
- [x] Code écrit
- [x] Tests passent
- [x] Reality check fait