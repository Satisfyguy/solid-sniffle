# GitHub Actions Workflows - Monero Marketplace

## 🎯 Vue d'ensemble

Ce projet utilise GitHub Actions pour l'intégration continue (CI/CD) avec un focus particulier sur la sécurité et la prévention du "security theatre". Quatre workflows principaux sont configurés pour garantir la qualité du code et la sécurité.

## 📋 Workflows Disponibles

### 1. **ci.yml** - Continuous Integration Principal

**Déclencheurs :**
- Push sur `main`, `develop`, `feature/*`
- Pull requests vers `main`, `develop`

**Jobs :**
- **security-check** : Vérifications de sécurité et qualité
- **build** : Compilation multi-plateforme (Windows, Ubuntu, macOS)
- **documentation** : Génération et déploiement de la documentation

**Fonctionnalités :**
- ✅ Détection automatique du security theatre
- ✅ Vérification Clippy avec warnings bloquants
- ✅ Tests unitaires et d'intégration
- ✅ Formatage automatique du code
- ✅ Génération de métriques
- ✅ Build multi-plateforme
- ✅ Documentation automatique

### 2. **security-audit.yml** - Audit de Sécurité Complet

**Déclencheurs :**
- Planifié : Chaque lundi à 2h du matin
- Push sur `main`
- Pull requests vers `main`
- Déclenchement manuel

**Fonctionnalités :**
- 🔍 **cargo-audit** : Vérification des vulnérabilités
- 🔍 **cargo-deny** : Vérification des licences
- 🔍 **semgrep** : Analyse statique de sécurité
- 🔍 **Security Theatre Check** : Détection des patterns interdits
- 🔍 **Clippy Security** : Lints de sécurité stricts
- 🔍 **Détection de secrets** : Scan des credentials hardcodés
- 📊 **Rapport automatique** : Commentaires sur les PR

### 3. **monero-integration.yml** - Tests d'Intégration Monero

**Déclencheurs :**
- Push sur `main`, `develop`
- Pull requests vers `main`, `develop`
- Déclenchement manuel

**Jobs :**
- **monero-testnet** : Tests avec Monero testnet
- **tor-integration** : Tests d'intégration Tor

**Fonctionnalités :**
- 🧪 **Setup Monero automatique** : Téléchargement et configuration
- 🧪 **Tests RPC** : Vérification des appels Monero
- 🧪 **Tests Multisig** : Validation des opérations multisig
- 🧅 **Tests Tor** : Vérification de la connectivité Tor
- 🧅 **Reality Checks** : Validation des checks Tor

### 4. **security-theatre.yml** - Détection Security Theatre

**Déclencheurs :**
- Push sur `main`, `develop`
- Pull requests vers `main`, `develop`

**Fonctionnalités :**
- 🎭 **Détection automatique** : Scan de tous les patterns interdits
- 🎭 **Tests d'intégration** : Vérification Monero + Tor
- 🎭 **Métriques** : Collecte et upload des métriques
- 🎭 **Rapports** : Génération de rapports de sécurité

## 🚀 Utilisation

### Déclencher un Workflow

```bash
# Déclencher manuellement (nécessite GitHub CLI)
gh workflow run security-audit.yml

# Ou via l'interface GitHub
# Actions > Security Audit > Run workflow
```

### Vérifier les Résultats

1. **Onglet Actions** : Voir tous les workflows
2. **Logs détaillés** : Cliquer sur un job pour voir les logs
3. **Artifacts** : Télécharger les rapports générés
4. **Commentaires PR** : Voir les rapports automatiques

### Configuration des Branches

```yaml
# Protection des branches (à configurer dans GitHub)
Branch Protection Rules:
  - Require status checks: ci.yml/security-check
  - Require up-to-date branches
  - Dismiss stale reviews
  - Require review from code owners
```

## 🔧 Configuration

### Variables d'Environnement

```yaml
# Définies dans chaque workflow
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUST_LOG: info
```

### Cache

```yaml
# Cache automatique des dépendances Rust
cache:
  cargo:
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

### Secrets (si nécessaire)

```yaml
# À configurer dans GitHub Settings > Secrets
secrets:
  MONERO_RPC_PASSWORD: # Pour les tests avec authentification
  TOR_BRIDGE_LINE: # Pour les tests Tor avec bridges
```

## 📊 Métriques et Rapports

### Métriques Collectées

- **Lines of Code** : Nombre de lignes de code
- **Functions** : Nombre de fonctions
- **Specs** : Nombre de spécifications
- **Unwraps** : Nombre d'utilisations de `.unwrap()`
- **TODOs** : Nombre de TODO restants
- **Test Coverage** : Estimation de la couverture

### Rapports Générés

- **Security Theatre Report** : Détails des patterns détectés
- **Security Audit Report** : Vulnérabilités et problèmes de sécurité
- **Code Metrics** : Métriques de qualité du code
- **Test Results** : Résultats des tests d'intégration

## 🛠️ Dépannage

### Workflow en Échec

1. **Vérifier les logs** : Cliquer sur le job en échec
2. **Identifier l'erreur** : Chercher les messages d'erreur
3. **Corriger localement** : Reproduire et corriger
4. **Re-trigger** : Push ou re-run du workflow

### Problèmes Courants

**Security Theatre détecté :**
```bash
# Corriger localement
pwsh -ExecutionPolicy Bypass -File "scripts/check-security-theatre-simple.ps1"

# Vérifier les exceptions
cat .security-theatre-ignore
```

**Tests Monero échouent :**
```bash
# Vérifier la connectivité
pwsh -ExecutionPolicy Bypass -File "scripts/test-rpc.ps1"

# Setup manuel
pwsh -ExecutionPolicy Bypass -File "scripts/setup-monero-testnet.ps1"
```

**Tests Tor échouent :**
```bash
# Vérifier Tor
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip

# Reality check
pwsh -ExecutionPolicy Bypass -File "scripts/validate-reality-check-tor.ps1" prepare_multisig
```

## 📈 Améliorations Futures

### Workflows à Ajouter

- **Performance Tests** : Benchmarks et tests de performance
- **Compatibility Tests** : Tests de compatibilité multi-versions
- **Release Automation** : Automatisation des releases
- **Dependency Updates** : Mise à jour automatique des dépendances

### Intégrations

- **Slack/Discord** : Notifications des échecs
- **Codecov** : Couverture de code
- **SonarQube** : Analyse de qualité
- **Dependabot** : Mise à jour des dépendances

## 🔒 Sécurité

### Bonnes Pratiques

- ✅ **Secrets** : Jamais de secrets hardcodés
- ✅ **Permissions** : Workflows avec permissions minimales
- ✅ **Validation** : Tous les inputs validés
- ✅ **Audit** : Logs d'audit complets
- ✅ **Isolation** : Tests dans des environnements isolés

### Monitoring

- 📊 **Métriques** : Suivi des tendances de qualité
- 🚨 **Alertes** : Notifications des échecs critiques
- 📋 **Rapports** : Rapports réguliers de sécurité
- 🔍 **Audit** : Audit périodique des workflows

---

## 📚 Ressources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Security Best Practices](https://docs.github.com/en/actions/security-guides)
- [Monero Integration Guide](docs/MONERO-INTEGRATION.md)
- [Tor Setup Guide](docs/TOR-SETUP.md)
