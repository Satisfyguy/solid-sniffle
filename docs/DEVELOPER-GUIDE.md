# Guide Développeur - Monero Marketplace

## 🎯 Vue d'ensemble

Ce guide explique comment développer de manière sécurisée sur le projet Monero Marketplace en respectant les règles anti-security theatre et les bonnes pratiques de sécurité.

## 🚀 Quick Start

### Prérequis
- Windows 10/11 avec PowerShell 5.1+
- Rust 1.75+ (si disponible)
- Git configuré
- Compréhension des concepts Tor et Monero

### Setup Initial
```powershell
# 1. Cloner le projet
git clone <repo-url>
cd monero-marketplace

# 2. Vérifier l'environnement
.\scripts\security-dashboard-basic.ps1

# 3. Vérifier les alertes
.\scripts\security-alerts-basic.ps1
```

## 📋 Workflow de Développement

### 1. Avant de Commencer
```powershell
# Vérifier l'état du projet
.\scripts\security-dashboard-basic.ps1

# S'assurer qu'il n'y a pas d'alertes critiques
.\scripts\security-alerts-basic.ps1
```

### 2. Créer une Nouvelle Fonction
```powershell
# 1. Créer la spec
.\scripts\new-spec.ps1 my_function

# 2. Éditer la spec
code docs/specs/my_function.md

# 3. Implémenter la fonction
# (Cursor détectera automatiquement le mode Tor si applicable)

# 4. Reality Check (si fonction réseau)
.\scripts\auto-reality-check-tor.ps1 my_function

# 5. Valider
.\scripts\validate-reality-check-tor.ps1 my_function
```

### 3. Avant Commit
```powershell
# Le pre-commit hook s'exécute automatiquement, mais on peut le tester :
.\scripts\pre-commit.ps1
```

## 🛡️ Règles Anti-Security Theatre

### ❌ Interdictions Absolues

**1. Jamais d'unwrap() sans contexte :**
```rust
// ❌ INTERDIT
let result = some_call().unwrap();

// ✅ CORRECT
let result = some_call()
    .context("Failed to perform operation")?;
```

**2. Jamais de println! en production :**
```rust
// ❌ INTERDIT
println!("Debug info: {}", data);

// ✅ CORRECT
tracing::info!("Debug info: {}", data);
```

**3. Jamais de magic numbers :**
```rust
// ❌ INTERDIT
let value = 1000000000000;

// ✅ CORRECT
const XMR_TO_ATOMIC: u64 = 1_000_000_000_000;
let value = XMR_TO_ATOMIC;
```

**4. Jamais de placeholders :**
```rust
// ❌ INTERDIT
// TODO: implement this
// FIXME: this is broken
// HYPOTHÈSES: should work

// ✅ CORRECT
// Implémentation complète avec tests
```

**5. Jamais de credentials hardcodés :**
```rust
// ❌ INTERDIT
let password = "secret123";
let api_key = "abc123";

// ✅ CORRECT
let password = std::env::var("PASSWORD")
    .context("PASSWORD environment variable not set")?;
```

### ✅ Bonnes Pratiques

**1. Error Handling Robuste :**
```rust
use anyhow::{Context, Result};

pub async fn my_function() -> Result<MyReturnType> {
    let data = risky_operation()
        .await
        .context("Failed to perform risky operation")?;
    
    Ok(data)
}
```

**2. Logging Structuré :**
```rust
use tracing::{info, warn, error, debug};

pub async fn process_data(data: &str) -> Result<()> {
    info!("Processing data of length: {}", data.len());
    
    match process_internal(data).await {
        Ok(result) => {
            info!("Processing completed successfully");
            Ok(result)
        }
        Err(e) => {
            error!("Processing failed: {}", e);
            Err(e)
        }
    }
}
```

**3. Constantes Nommées :**
```rust
// Dans common/src/lib.rs
pub const MONERO_RPC_PORT: u16 = 18082;
pub const XMR_TO_ATOMIC: u64 = 1_000_000_000_000;
pub const MAX_MULTISIG_INFO_LEN: usize = 5000;

// Utilisation
use crate::MONERO_RPC_PORT;
let url = format!("http://127.0.0.1:{}", MONERO_RPC_PORT);
```

**4. Tests Complets :**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_my_function() -> Result<()> {
        // Arrange
        let input = "test_data";
        
        // Act
        let result = my_function(input).await?;
        
        // Assert
        assert!(result.is_valid());
        Ok(())
    }
}
```

## 🧅 Règles Tor Spécifiques

### OPSEC Critique

**1. RPC Monero - Localhost Uniquement :**
```rust
// ✅ CORRECT - RPC isolé
let client = MoneroRpcClient::new("http://127.0.0.1:18082".to_string())?;

// ❌ INTERDIT - RPC exposé publiquement
let client = MoneroRpcClient::new("http://0.0.0.0:18082".to_string())?;
```

**2. Tous les Appels Externes via Tor :**
```rust
use reqwest::Proxy;

async fn fetch_via_tor(url: &str) -> Result<String> {
    let proxy = Proxy::all("socks5h://127.0.0.1:9050")
        .context("Failed to configure Tor proxy")?;
    
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let response = client.get(url).send().await?;
    response.text().await.context("Failed to read response")
}
```

**3. Jamais Logger de Données Sensibles :**
```rust
// ❌ INTERDIT
tracing::info!("User address: {}", onion_address);
tracing::debug!("View key: {}", view_key);

// ✅ CORRECT
tracing::info!("User connected successfully");
tracing::debug!("Operation completed");
```

## 🔧 Outils de Développement

### Scripts Disponibles

| Script | Commande | Description |
|--------|----------|-------------|
| **Security Dashboard** | `.\scripts\security-dashboard-basic.ps1` | Vue d'ensemble sécurité |
| **Security Alerts** | `.\scripts\security-alerts-basic.ps1` | Vérifier les alertes |
| **Security Theatre Check** | `.\scripts\check-security-theatre-simple.ps1` | Détection patterns interdits |
| **New Spec** | `.\scripts\new-spec.ps1 <name>` | Créer nouvelle spécification |
| **Reality Check Tor** | `.\scripts\auto-reality-check-tor.ps1 <name>` | Générer reality check |
| **Validate RC** | `.\scripts\validate-reality-check-tor.ps1 <name>` | Valider reality check |
| **Pre-commit** | `.\scripts\pre-commit.ps1` | Vérifications avant commit |

### Validation Continue

**1. Vérification Manuelle :**
```powershell
# Vérifier l'état général
.\scripts\security-dashboard-basic.ps1

# Vérifier les alertes
.\scripts\security-alerts-basic.ps1

# Test complet
.\scripts\pre-commit.ps1
```

**2. Vérification Automatique :**
- Pre-commit hooks s'exécutent automatiquement
- GitHub Actions valident à chaque push
- Security audit hebdomadaire

## 📊 Métriques de Qualité

### Score de Sécurité
Le système calcule automatiquement un score de sécurité basé sur :
- **Unwraps** : -20 points par unwrap() détecté
- **TODOs** : -10 points si > 5 TODOs
- **Fonctions sans specs** : -15 points
- **Tests insuffisants** : -10 points
- **Reality checks manquants** : -5 points

### Niveaux de Score
- **90-100** : Excellent ✅
- **70-89** : Bon ✅
- **50-69** : Moyen ⚠️
- **0-49** : Critique ❌

## 🚨 Gestion des Erreurs

### Types d'Erreurs Courantes

**1. Security Theatre Détecté :**
```powershell
# Solution : Corriger le code
.\scripts\check-security-theatre-simple.ps1 -Verbose
# Suivre les recommandations affichées
```

**2. Fonctions sans Specs :**
```powershell
# Solution : Créer les specs manquantes
.\scripts\new-spec.ps1 function_name
```

**3. Tests qui Échouent :**
```powershell
# Solution : Vérifier l'environnement
.\scripts\security-dashboard-basic.ps1
# Corriger les problèmes identifiés
```

### Contournement Temporaire

**Exception Légitime :**
```powershell
# Ajouter dans .security-theatre-ignore
# Format: path_pattern:regex_pattern
cli/src/test_tool.rs:println!
```

**Justification Requise :**
- Expliquer pourquoi l'exception est nécessaire
- Limiter dans le temps
- Documenter la solution de contournement

## 📚 Ressources

### Documentation
- [Security Theatre Prevention](SECURITY-THEATRE-PREVENTION.md)
- [GitHub Actions](GITHUB-ACTIONS.md)
- [OPSEC Guidelines](OPSEC.md)
- [Monero Integration](MONERO-INTEGRATION.md)

### Liens Externes
- [Monero Documentation](https://www.getmonero.org/resources/developer-guides/)
- [Tor Project](https://www.torproject.org/)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Tracing Documentation](https://docs.rs/tracing/)

## 🤝 Contribution

### Processus de Contribution
1. **Fork** le repository
2. **Créer** une branche feature
3. **Suivre** le workflow de développement
4. **Tester** avec les scripts de validation
5. **Créer** une Pull Request
6. **Attendre** la validation automatique

### Code Review
- Tous les PRs sont automatiquement vérifiés
- Security theatre check obligatoire
- Reality check Tor pour les fonctions réseau
- Validation des specs et tests

---

## ⚠️ Rappel Important

**Ce projet traite de Monero et Tor - la sécurité n'est pas optionnelle.**

- ✅ Toujours suivre les règles OPSEC
- ✅ Jamais compromettre l'anonymat
- ✅ Toujours valider avec les reality checks
- ✅ Jamais ignorer les alertes de sécurité

**En cas de doute, demander avant de commiter !**
