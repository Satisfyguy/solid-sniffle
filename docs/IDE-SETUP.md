# Configuration IDE - Monero Marketplace

## 🎯 Objectif
Ce document explique comment configurer votre environnement de développement IDE pour le projet Monero Marketplace avec toutes les fonctionnalités de sécurité et d'automatisation.

## 🛠️ Configuration VS Code

### Extensions Requises

Les extensions suivantes sont automatiquement installées via `scripts/setup-ide.ps1` :

1. **`rust-lang.rust-analyzer`** - Support Rust complet
2. **`vadimcn.vscode-lldb`** - Debugger LLDB pour Rust
3. **`ms-vscode.powershell`** - Support PowerShell
4. **`redhat.vscode-yaml`** - Support YAML
5. **`ms-vscode.vscode-json`** - Support JSON

### Configuration Automatique

```powershell
# Lancer la configuration automatique
.\scripts\setup-ide.ps1
```

### Tâches Disponibles

Accédez aux tâches via `Ctrl+Shift+P` > `Tasks: Run Task` :

- **Security Theatre Check** - Détection automatique du security theatre
- **Monero/Tor Patterns Check** - Vérification des patterns Monero/Tor
- **Security Dashboard** - Affichage du dashboard de sécurité
- **Security Alerts** - Vérification des alertes de sécurité
- **Pre-commit Check** - Exécution complète des vérifications pre-commit
- **Cargo Check** - Vérification de compilation
- **Cargo Clippy** - Analyse statique avec Clippy
- **Cargo Test** - Exécution des tests
- **New Spec** - Création d'une nouvelle spécification
- **Reality Check Tor** - Génération d'un reality check Tor

### Configuration de Debug

Le fichier `.vscode/launch.json` configure 3 configurations de debug :

1. **Debug CLI** - Debug de l'application CLI principale
2. **Debug Test Tool** - Debug de l'outil de test
3. **Debug Wallet Tests** - Debug des tests du wallet

### Paramètres Recommandés

Le fichier `.vscode/settings.json` configure :

- **Rust Analyzer** avec Clippy strict
- **PowerShell** avec analyse de script
- **Exclusions de fichiers** pour la sécurité
- **Formatage automatique** à la sauvegarde
- **Actions de code** automatiques

## 🔧 Configuration Manuelle

### Si VS Code n'est pas dans le PATH

```powershell
# Ajouter VS Code au PATH
$env:PATH += ";C:\Users\$env:USERNAME\AppData\Local\Programs\Microsoft VS Code\bin"

# Ou installer les extensions manuellement
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension ms-vscode.powershell
```

### Configuration Clippy

Le fichier `.cargo/config.toml` configure Clippy avec des règles strictes :

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-D", "warnings"]

[clippy]
all = true
warn = ["clippy::all"]
```

## 🚀 Workflow de Développement

### 1. Démarrage Rapide

```powershell
# 1. Configurer l'IDE
.\scripts\setup-ide.ps1

# 2. Ouvrir VS Code
code .

# 3. Vérifier la configuration
Ctrl+Shift+P > "Tasks: Run Task" > "Security Dashboard"
```

### 2. Développement Quotidien

1. **Avant de commiter** : `Ctrl+Shift+P` > `Tasks: Run Task` > `Pre-commit Check`
2. **Vérification sécurité** : `Ctrl+Shift+P` > `Tasks: Run Task` > `Security Theatre Check`
3. **Tests** : `Ctrl+Shift+P` > `Tasks: Run Task` > `Cargo Test`
4. **Debug** : `F5` pour lancer le debug

### 3. Création de Fonctionnalités

1. **Nouvelle spec** : `Ctrl+Shift+P` > `Tasks: Run Task` > `New Spec`
2. **Reality Check** : `Ctrl+Shift+P` > `Tasks: Run Task` > `Reality Check Tor`
3. **Développement** : Utiliser Rust Analyzer pour l'autocomplétion
4. **Tests** : `Ctrl+Shift+P` > `Tasks: Run Task` > `Cargo Test`

## 🛡️ Intégration Sécurité

### Vérifications Automatiques

- **Pre-commit hooks** exécutés automatiquement
- **Security theatre detection** intégrée
- **Monero/Tor patterns** vérifiés
- **Clippy strict** activé

### Alertes et Notifications

- **Security Dashboard** accessible via tâche
- **Security Alerts** avec notifications
- **Métriques** mises à jour automatiquement

## 📚 Ressources

- [Documentation Rust Analyzer](https://rust-analyzer.github.io/)
- [Documentation LLDB](https://lldb.llvm.org/)
- [Documentation PowerShell](https://docs.microsoft.com/en-us/powershell/)
- [Guide du Développeur](DEVELOPER-GUIDE.md)
- [Formation Équipe](TEAM-TRAINING.md)

## 🔧 Dépannage

### Problèmes Courants

1. **Extension LLDB non reconnue**
   ```powershell
   code --install-extension vadimcn.vscode-lldb --force
   ```

2. **PowerShell non reconnu**
   ```powershell
   code --install-extension ms-vscode.powershell --force
   ```

3. **Rust Analyzer ne fonctionne pas**
   ```powershell
   # Redémarrer VS Code
   # Vérifier que Rust est installé
   rustc --version
   ```

4. **Tâches ne s'exécutent pas**
   ```powershell
   # Vérifier que PowerShell est configuré
   Get-ExecutionPolicy
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

### Support

Pour toute question sur la configuration IDE :
- Consulter la [documentation VS Code](https://code.visualstudio.com/docs)
- Vérifier les [logs VS Code](https://code.visualstudio.com/docs/supporting/troubleshooting)
- Utiliser le [guide de dépannage Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
