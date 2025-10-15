# 🚀 Setup Guide - Monero Marketplace Tor v2.0

Guide de configuration pour le développement du Monero Marketplace.

## 📋 Prérequis

### 1. Rust
```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Vérifier l'installation
rustc --version
cargo --version
```

### 2. Monero (Testnet)
```bash
# Windows
# Télécharger depuis https://www.getmonero.org/downloads/
# Extraire dans C:\monero\

# Linux
wget https://downloads.getmonero.org/cli/linux64
tar -xzf linux64
sudo mv monero-* /opt/monero/
```

### 3. PowerShell (Windows)
```powershell
# Vérifier la version
$PSVersionTable.PSVersion

# Si < 5.1, installer PowerShell Core
# https://github.com/PowerShell/PowerShell/releases
```

## 🛠️ Configuration Initiale

### 1. Cloner le Projet
```bash
git clone <repository-url>
cd monero-marketplace
```

### 2. Setup Monero Testnet
```powershell
# Lancer le script de setup
.\scripts\setup-monero.ps1

# Vérifier que Monero est installé
Get-Process monero* -ErrorAction SilentlyContinue
```

### 3. Démarrer Monero Testnet
```powershell
# Lancer daemon + wallet RPC
.\scripts\start-testnet.ps1

# Vérifier que RPC répond
.\scripts\test-rpc.ps1
```

### 4. Vérifier le Projet
```bash
# Compiler le projet
cargo check

# Lancer les tests
cargo test

# Vérifier les métriques
.\scripts\update-metrics.ps1
```

## 🧪 Premier Test

### 1. Créer une Spec
```powershell
# Créer une spec pour une fonction test
.\scripts\new-spec.ps1 test_function
```

### 2. Éditer la Spec
Ouvrir `docs/specs/test_function.md` et compléter:
- Objectif: "Fonction de test qui retourne Ok(())"
- Input: "Aucun paramètre"
- Output: "Result<(), Error>"

### 3. Générer le Code
Demander à Cursor:
```
Génère le code pour test_function selon la spec dans docs/specs/test_function.md
```

### 4. Reality Check
```powershell
# Générer le reality check
.\scripts\reality-check.ps1 test_function

# Compléter docs/reality-checks/test_function-YYYY-MM-DD.md
```

### 5. Commit
```powershell
# Vérifications pré-commit
.\scripts\pre-commit.ps1

# Si tout est OK, commiter
git add .
git commit -m "[CODE] Implement test_function

- Fonction de test simple
- Retourne Ok(())
- Tests unitaires inclus

Tested: ✅
Reality check: ✅"
```

## 🔧 Configuration Cursor

### 1. Activer les Règles
- Ouvrir Cursor
- Aller dans Settings > Rules
- Activer `.cursorrules`

### 2. Vérifier l'Automation
Cursor devrait maintenant:
- ✅ Vérifier que les specs existent avant génération
- ✅ Vérifier que le projet compile
- ✅ Auto-formater après génération
- ✅ Lancer clippy
- ✅ Mettre à jour les métriques

## 📊 Métriques & Monitoring

### Dashboard Local
```powershell
# Lancer le serveur de métriques
python -m http.server 8080 --directory docs/metrics

# Ouvrir http://localhost:8080
```

### Métriques Collectées
- Lines of Code
- Nombre de fonctions
- Nombre de specs
- Nombre d'unwraps
- Nombre de TODOs
- Couverture de tests (estimation)

## 🚨 Troubleshooting

### Monero RPC ne répond pas
```powershell
# Vérifier que monero-wallet-rpc tourne
Get-Process monero-wallet-rpc

# Redémarrer si nécessaire
.\scripts\start-testnet.ps1
```

### Projet ne compile pas
```bash
# Nettoyer et recompiler
cargo clean
cargo check

# Vérifier les dépendances
cargo tree
```

### Cursor ne respecte pas les règles
1. Vérifier que `.cursorrules` est à la racine
2. Redémarrer Cursor
3. Vérifier les logs Cursor

### Scripts PowerShell ne fonctionnent pas
```powershell
# Vérifier la politique d'exécution
Get-ExecutionPolicy

# Si nécessaire, autoriser l'exécution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## 📚 Ressources

- [Monero Documentation](https://www.getmonero.org/resources/developer-guides/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cursor Documentation](https://cursor.sh/docs)

## 🆘 Support

En cas de problème:
1. Vérifier les logs dans `docs/metrics/`
2. Lancer `.\scripts\update-metrics.ps1`
3. Consulter les reality checks dans `docs/reality-checks/`
4. Créer une issue avec les détails
