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

### 3. Tests E2E (End-to-End)
Tests du flow complet d'escrow avec simulation de blockchain.

**Localisation**: `server/tests/escrow_e2e.rs`

**Tests implémentés**:
1. `test_complete_escrow_flow` - Flow complet: création → funding → release → completed
2. `test_dispute_flow` - Flow de dispute: création → dispute → résolution → refund
3. `test_escrow_orchestrator_init` - Initialisation de l'orchestrateur
4. `test_escrow_state_transitions` - Validation des transitions d'état
5. `test_concurrent_escrows` - Gestion de plusieurs escrows simultanés

**⚠️ Important**: Ces tests sont marqués `#[ignore]` car ils nécessitent un setup de base de données complet avec migrations appliquées.

#### Setup Requis pour Tests E2E

**1. Variables d'environnement**

Créer un fichier `.env.test` à la racine du projet:

```bash
# Database configuration
DATABASE_URL=test_marketplace.db
DB_ENCRYPTION_KEY=test_encryption_key_32_bytes!!!!!!!

# Monero RPC (optionnel pour E2E, utilisé par orchestrator)
MONERO_RPC_URL=http://127.0.0.1:18082/json_rpc
```

**2. Préparation de la base de données**

```bash
# Créer la base de données de test
touch test_marketplace.db

# Appliquer toutes les migrations
diesel migration run --database-url test_marketplace.db

# Vérifier que toutes les tables sont créées
sqlite3 test_marketplace.db ".schema"
```

**3. Vérification du schéma**

Les tests E2E nécessitent ces tables:
- `users` (id, username, password_hash, role, wallet_address, wallet_id)
- `listings` (id, vendor_id, title, description, price_xmr, stock, status)
- `orders` (id, buyer_id, vendor_id, listing_id, escrow_id, status, total_xmr)
- `escrows` (id, order_id, buyer_id, vendor_id, arbiter_id, amount, status, multisig_address, transaction_hash)

#### Exécution des Tests E2E

```bash
# Exécuter TOUS les tests E2E (nécessite setup DB)
cargo test --package server --test escrow_e2e -- --ignored

# Exécuter un test spécifique
cargo test --package server --test escrow_e2e test_complete_escrow_flow -- --ignored --nocapture

# Exécuter avec output détaillé
RUST_LOG=debug cargo test --package server --test escrow_e2e -- --ignored --nocapture
```

#### Structure d'un Test E2E

```rust
#[tokio::test]
#[ignore] // Requires database setup with migrations
async fn test_complete_escrow_flow() -> Result<()> {
    // Setup: Pool DB + utilisateurs de test
    let pool = create_test_pool();
    let users = setup_test_users(&pool).await?;

    // Step 1: Vendor crée listing
    let listing_id = create_listing(&pool, users.vendor_id, 1_000_000_000_000).await?;

    // Step 2: Buyer crée commande
    let order_id = create_order(&pool, users.buyer_id, listing_id).await?;

    // Step 3: Escrow auto-initialisé
    let escrow_id = create_escrow(&pool, order_id, users.buyer_id, users.vendor_id, users.arbiter_id, 1_000_000_000_000).await?;
    assert_eq!(get_escrow_status(&pool, escrow_id).await?, "created");

    // Step 4: Simulate multisig setup
    db_update_escrow_address(&pool, escrow_id, "9wq792k9...").await?;
    db_update_escrow_status(&pool, escrow_id, "funded").await?;

    // Step 5: Simulate blockchain monitor detecting funds
    db_update_escrow_status(&pool, escrow_id, "active").await?;

    // Step 6: Buyer releases funds
    db_update_escrow_transaction_hash(&pool, escrow_id, "a1b2c3...").await?;
    db_update_escrow_status(&pool, escrow_id, "releasing").await?;

    // Step 7: Simulate confirmations
    db_update_escrow_status(&pool, escrow_id, "completed").await?;

    // Verify final state
    let escrow = db_load_escrow(&pool, escrow_id).await?;
    assert_eq!(escrow.status, "completed");
    assert_eq!(escrow.amount, 1_000_000_000_000);

    Ok(())
}
```

#### Helpers Disponibles

**Setup**:
- `create_test_pool()` - Crée pool DB avec encryption
- `setup_test_users(pool)` - Crée buyer, vendor, arbiter

**Création d'entités**:
- `create_listing(pool, vendor_id, price)` - Crée un listing de test
- `create_order(pool, buyer_id, listing_id)` - Crée une commande
- `create_escrow(pool, order_id, buyer_id, vendor_id, arbiter_id, amount)` - Crée escrow

**État**:
- `get_escrow_status(pool, escrow_id)` - Récupère statut actuel
- `wait_for_status(pool, escrow_id, expected_status, timeout_secs)` - Attend un statut

#### Simulation vs Production

Les tests E2E **simulent** les opérations blockchain:

| Opération Production | Simulation Test E2E |
|---------------------|---------------------|
| `prepare_multisig()` | `db_update_escrow_address(pool, escrow_id, "9wq792k9...")` |
| `transfer()` | `db_update_escrow_transaction_hash(pool, escrow_id, "a1b2...")` |
| `get_transfer_by_txid()` | `db_update_escrow_status(pool, escrow_id, "completed")` |

**Pourquoi?**
- Tests E2E testent la **logique d'état** et les **DB operations**
- Tests RPC (wallet_manager_e2e.rs) testent l'**intégration Monero**
- Combinaison des deux = couverture complète

#### Transitions d'État Testées

**Flow Normal (Release)**:
```
created → funded → active → releasing → completed
```

**Flow Dispute (Refund)**:
```
created → funded → active → disputed → resolved_buyer → refunding → refunded
```

**Flow Dispute (Release to Vendor)**:
```
created → funded → active → disputed → resolved_vendor → releasing → completed
```

#### Nettoyage après Tests

```bash
# Supprimer la DB de test
rm test_marketplace.db

# Ou la réinitialiser
rm test_marketplace.db
diesel migration run --database-url test_marketplace.db
```

#### Troubleshooting Tests E2E

**Erreur: "Failed to create test pool"**
```bash
# Vérifier que DATABASE_URL est défini
echo $DATABASE_URL

# Créer le fichier DB
touch test_marketplace.db
```

**Erreur: "Failed to insert user/listing/order"**
```bash
# Vérifier que les migrations sont appliquées
diesel migration list --database-url test_marketplace.db

# Réappliquer si nécessaire
diesel migration redo --database-url test_marketplace.db
```

**Erreur: "Table doesn't exist"**
```bash
# Vérifier le schéma
sqlite3 test_marketplace.db ".schema escrows"

# Si transaction_hash manque, appliquer migration 3.2.1
diesel migration run --database-url test_marketplace.db
```

**Tests ignorés par défaut**
```bash
# CORRECT: Utiliser --ignored
cargo test --test escrow_e2e -- --ignored

# INCORRECT: Sans --ignored
cargo test --test escrow_e2e  # Ne lance AUCUN test
```

#### Métriques Tests E2E

- **Nombre de tests**: 5
- **Couverture**: State machine escrow (100%), DB operations (90%)
- **Durée**: ~2-5s (dépend du DB I/O)
- **Setup requis**: Database + migrations

### 4. Tests Manuels
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
