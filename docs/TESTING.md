# 🧪 Testing Guide - Monero Marketplace Tor v2.0

Guide complet pour les tests du Monero Marketplace.

## 📋 Types de Tests

### 1. Tests Unitaires
Tests des fonctions individuelles dans chaque module.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_name(input);
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### 2. Tests d'Intégration
Tests des interactions entre modules.

```rust
// tests/integration.rs
use monero_marketplace_wallet::MoneroClient;
use monero_marketplace_common::types::MoneroConfig;

#[tokio::test]
async fn test_wallet_connection() {
    let config = MoneroConfig::default();
    let client = MoneroClient::new(config).unwrap();
    
    // Test avec Monero RPC réel
    let version = client.rpc().get_version().await;
    assert!(version.is_ok());
}
```

### 3. Tests Manuels
Tests avec Monero testnet réel.

## 🚀 Setup des Tests

### 1. Monero Testnet
```powershell
# Démarrer Monero testnet
.\scripts\start-testnet.ps1

# Vérifier que RPC répond
.\scripts\test-rpc.ps1
```

### 2. Wallet de Test
```bash
# Créer un wallet de test
monero-wallet-cli --testnet --generate-new-wallet test_wallet

# Déverrouiller le wallet
monero-wallet-cli --testnet --wallet-file test_wallet
```

## 🧪 Exécution des Tests

### Tests Unitaires
```bash
# Tous les tests
cargo test

# Tests d'un module spécifique
cargo test -p monero-marketplace-common

# Tests avec output détaillé
cargo test -- --nocapture

# Tests en parallèle
cargo test --jobs 4
```

### Tests d'Intégration
```bash
# Tests d'intégration uniquement
cargo test --test integration

# Tests avec Monero RPC
MONERO_RPC_URL=http://127.0.0.1:18082 cargo test --test integration
```

### Tests Manuels
```powershell
# Test de connexion RPC
.\scripts\test-rpc.ps1

# Test de multisig
cargo run --bin monero-marketplace -- multisig prepare
cargo run --bin monero-marketplace -- multisig check
```

## 📊 Couverture de Tests

### Installation de tarpaulin
```bash
cargo install cargo-tarpaulin
```

### Génération du rapport
```bash
# Couverture complète
cargo tarpaulin --out Html

# Couverture par module
cargo tarpaulin -p monero-marketplace-common --out Html
```

### Objectifs de Couverture
- **Minimum**: 60%
- **Recommandé**: 80%
- **Excellent**: 90%+

## 🔍 Tests Spécifiques

### Tests Monero RPC
```rust
#[tokio::test]
async fn test_get_version() {
    let config = MoneroConfig::default();
    let client = MoneroRpcClient::new(config).unwrap();
    
    let version = client.get_version().await.unwrap();
    assert!(!version.is_empty());
}

#[tokio::test]
async fn test_get_balance() {
    let config = MoneroConfig::default();
    let client = MoneroRpcClient::new(config).unwrap();
    
    let (balance, unlocked) = client.get_balance().await.unwrap();
    assert!(balance >= 0);
    assert!(unlocked >= 0);
}
```

### Tests Multisig
```rust
#[tokio::test]
async fn test_prepare_multisig() {
    let config = MoneroConfig::default();
    let rpc_client = MoneroRpcClient::new(config).unwrap();
    let multisig_manager = MultisigManager::new(rpc_client);
    
    let info = multisig_manager.prepare_multisig().await.unwrap();
    assert!(!info.info.is_empty());
}

#[tokio::test]
async fn test_multisig_flow() {
    // Test du flow complet multisig
    // 1. prepare_multisig
    // 2. make_multisig
    // 3. export_multisig_info
    // 4. import_multisig_info
    // 5. Vérifier is_multisig
}
```

### Tests d'Erreur
```rust
#[tokio::test]
async fn test_rpc_connection_error() {
    let config = MoneroConfig {
        rpc_url: "http://127.0.0.1:9999/json_rpc".to_string(),
        ..Default::default()
    };
    
    let client = MoneroRpcClient::new(config).unwrap();
    let result = client.get_version().await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::MoneroRpc(_) => {}, // Expected
        _ => panic!("Wrong error type"),
    }
}
```

## 🚨 Tests de Sécurité

### Tests d'Injection
```rust
#[tokio::test]
async fn test_sql_injection_prevention() {
    // Test avec des inputs malveillants
    let malicious_input = "'; DROP TABLE users; --";
    let result = process_input(malicious_input);
    assert!(result.is_err());
}
```

### Tests de Validation
```rust
#[test]
fn test_address_validation() {
    // Adresses valides
    assert!(validate_monero_address("4AdUndXHHZ6cFdRPAgP6zBFmZ1hBpiPsjCd1TqWLjokCLQcaQa4Yf8ZgWa61uB1DkHGrC1XqVjro7ykm5rF8YvP9aYTFjk").is_ok());
    
    // Adresses invalides
    assert!(validate_monero_address("invalid").is_err());
    assert!(validate_monero_address("").is_err());
}
```

## 📈 Métriques de Tests

### Collecte Automatique
```powershell
# Mise à jour des métriques après tests
.\scripts\update-metrics.ps1
```

### Métriques Trackées
- Nombre de tests
- Couverture de code
- Tests qui échouent
- Temps d'exécution

## 🔧 Configuration CI/CD

### GitHub Actions
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run tests
      run: cargo test
    - name: Generate coverage
      run: cargo tarpaulin --out Html
```

## 🚨 Troubleshooting

### Tests qui échouent
```bash
# Vérifier les logs détaillés
cargo test -- --nocapture

# Tests d'un module spécifique
cargo test -p monero-marketplace-wallet

# Nettoyer et recompiler
cargo clean
cargo test
```

### Monero RPC indisponible
```powershell
# Vérifier que Monero tourne
Get-Process monero*

# Redémarrer Monero
.\scripts\start-testnet.ps1
```

### Tests lents
```bash
# Tests en parallèle
cargo test --jobs 4

# Tests sans compilation
cargo test --no-run
```

## 📚 Bonnes Pratiques

### 1. Nommage des Tests
```rust
#[test]
fn test_function_name_with_valid_input() {
    // Test avec input valide
}

#[test]
fn test_function_name_with_invalid_input() {
    // Test avec input invalide
}

#[test]
fn test_function_name_returns_error_on_failure() {
    // Test de gestion d'erreur
}
```

### 2. Structure des Tests
```rust
#[test]
fn test_example() {
    // Arrange - Préparer les données
    let input = "test";
    let expected = "expected";
    
    // Act - Exécuter la fonction
    let result = function(input);
    
    // Assert - Vérifier le résultat
    assert_eq!(result, expected);
}
```

### 3. Tests Async
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## 🎯 Objectifs de Qualité

- **Couverture**: >80%
- **Tests unitaires**: 1 test par fonction publique
- **Tests d'intégration**: 1 test par workflow principal
- **Tests d'erreur**: 1 test par cas d'erreur
- **Temps d'exécution**: <30s pour tous les tests
