# Quick Start Guide - Monero Marketplace

## 🚀 Démarrage Rapide (5 minutes)

### 1. Vérifier l'Environnement
```powershell
# Vérifier l'état du projet
.\scripts\security-dashboard-basic.ps1

# Vérifier les alertes
.\scripts\security-alerts-basic.ps1
```

### 2. Comprendre le Score de Sécurité
- **90-100** : Excellent ✅
- **70-89** : Bon ✅  
- **50-69** : Moyen ⚠️
- **0-49** : Critique ❌

### 3. Règles Essentielles

#### ❌ Jamais Faire
```rust
// Unwrap sans contexte
let result = some_call().unwrap();

// Println en production
println!("Debug: {}", data);

// Magic numbers
let value = 1000000000000;

// Placeholders
// TODO: implement this
```

#### ✅ Toujours Faire
```rust
// Error handling avec contexte
let result = some_call()
    .context("Failed to perform operation")?;

// Logging structuré
tracing::info!("Debug: {}", data);

// Constantes nommées
const XMR_TO_ATOMIC: u64 = 1_000_000_000_000;
let value = XMR_TO_ATOMIC;

// Implémentation complète
pub fn my_function() -> Result<MyType> {
    // Logique complète avec tests
}
```

## 🛠️ Workflow Standard

### Créer une Nouvelle Fonction
```powershell
# 1. Créer la spec
.\scripts\new-spec.ps1 my_function

# 2. Implémenter le code
# (Cursor détecte automatiquement le mode Tor)

# 3. Reality Check (si fonction réseau)
.\scripts\auto-reality-check-tor.ps1 my_function

# 4. Valider
.\scripts\validate-reality-check-tor.ps1 my_function
```

### Avant Chaque Commit
```powershell
# Vérification automatique
.\scripts\pre-commit.ps1

# Si échec, corriger les problèmes
.\scripts\check-security-theatre-simple.ps1 -Verbose
```

## 🧅 Règles Tor Critiques

### RPC Monero - Localhost Uniquement
```rust
// ✅ CORRECT
let client = MoneroRpcClient::new("http://127.0.0.1:18082".to_string())?;

// ❌ INTERDIT
let client = MoneroRpcClient::new("http://0.0.0.0:18082".to_string())?;
```

### Appels Externes via Tor
```rust
use reqwest::Proxy;

async fn fetch_via_tor(url: &str) -> Result<String> {
    let proxy = Proxy::all("socks5h://127.0.0.1:9050")?;
    let client = reqwest::Client::builder().proxy(proxy).build()?;
    client.get(url).send().await?.text().await
}
```

### Jamais Logger de Données Sensibles
```rust
// ❌ INTERDIT
tracing::info!("Address: {}", onion_address);

// ✅ CORRECT
tracing::info!("User connected successfully");
```

## 🔧 Outils Essentiels

| Commande | Description |
|----------|-------------|
| `.\scripts\security-dashboard-basic.ps1` | Vue d'ensemble sécurité |
| `.\scripts\security-alerts-basic.ps1` | Vérifier les alertes |
| `.\scripts\check-security-theatre-simple.ps1` | Détection patterns interdits |
| `.\scripts\pre-commit.ps1` | Vérifications avant commit |

## 🚨 En Cas de Problème

### Security Theatre Détecté
```powershell
# Voir les détails
.\scripts\check-security-theatre-simple.ps1 -Verbose

# Corriger selon les recommandations
```

### Fonctions sans Specs
```powershell
# Créer les specs manquantes
.\scripts\new-spec.ps1 function_name
```

### Tests qui Échouent
```powershell
# Vérifier l'environnement
.\scripts\security-dashboard-basic.ps1
```

## 📚 Ressources

- [Guide Développeur Complet](DEVELOPER-GUIDE.md)
- [Formation Équipe](TEAM-TRAINING.md)
- [Prévention Security Theatre](SECURITY-THEATRE-PREVENTION.md)

## ⚠️ Rappel Important

**Ce projet traite de Monero et Tor - la sécurité n'est pas optionnelle.**

- ✅ Toujours suivre les règles OPSEC
- ✅ Jamais compromettre l'anonymat
- ✅ Toujours valider avec les reality checks
- ✅ Jamais ignorer les alertes de sécurité

**En cas de doute, demander avant de commiter !**
