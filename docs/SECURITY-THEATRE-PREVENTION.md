# Security Theatre Prevention System

## 🎯 Objectif

Ce document décrit le système automatique de prévention du "security theatre" implémenté dans le projet Monero Marketplace. Le système détecte et bloque automatiquement les patterns de code qui donnent une fausse impression de sécurité sans apporter de réelle protection.

## 🚨 Qu'est-ce que le Security Theatre?

Le security theatre désigne des pratiques qui semblent améliorer la sécurité mais n'apportent en réalité aucune protection réelle. Dans le contexte du développement logiciel, cela inclut:

- **Asserts inutiles** : `assert!(true)` qui ne testent rien
- **Placeholders** : Commentaires `// TODO` ou `// FIXME` laissés en production
- **Suppositions** : Code basé sur des hypothèses non validées
- **Commentaires vagues** : `// ERREUR POSSIBLE` sans explication
- **Code mort** : `unimplemented!()` ou `todo!()` en production
- **Credentials hardcodés** : Mots de passe ou clés en dur
- **Magic numbers** : Valeurs numériques sans constantes explicites

## 🔧 Système de Détection

### Script Principal

**Fichier:** `scripts/check-security-theatre.ps1`

Le script scanne automatiquement tous les fichiers Rust du projet et détecte:

#### Patterns Détectés

1. **Asserts inutiles**
   ```rust
   assert!(true)           // ❌ Détecté
   assert!(false)          // ❌ Détecté
   assert!(1 == 1)         // ❌ Détecté
   ```

2. **Placeholders**
   ```rust
   // Placeholder          // ❌ Détecté
   // TODO                 // ❌ Détecté
   // FIXME                // ❌ Détecté
   // XXX                  // ❌ Détecté
   // HACK                 // ❌ Détecté
   ```

3. **Suppositions**
   ```rust
   // should work          // ❌ Détecté
   // probably works       // ❌ Détecté
   // assume this works    // ❌ Détecté
   ```

4. **Hypothèses non validées**
   ```rust
   // HYPOTHÈSES           // ❌ Détecté
   // À VALIDER            // ❌ Détecté
   // TO BE VALIDATED      // ❌ Détecté
   ```

5. **Commentaires vagues**
   ```rust
   // ERREUR POSSIBLE      // ❌ Détecté
   // À IMPLÉMENTER        // ❌ Détecté
   // NOT IMPLEMENTED      // ❌ Détecté
   ```

6. **Code mort**
   ```rust
   unimplemented!()        // ❌ Détecté
   todo!()                 // ❌ Détecté
   panic!()                // ❌ Détecté
   ```

7. **Credentials hardcodés**
   ```rust
   password = "secret"     // ❌ Détecté
   secret = "key"          // ❌ Détecté
   api_key = "token"       // ❌ Détecté
   ```

8. **Magic numbers**
   ```rust
   1000000000000           // ❌ Détecté (sans commentaire)
   0x12345678             // ❌ Détecté (sans commentaire)
   ```

9. **Patterns interdits**
   ```rust
   .unwrap()              // ❌ Détecté
   println!()             // ❌ Détecté
   dbg!()                 // ❌ Détecté
   ```

### Utilisation

```powershell
# Scan complet
.\scripts\check-security-theatre.ps1

# Scan avec détails
.\scripts\check-security-theatre.ps1 -Verbose

# Scan d'un dossier spécifique
.\scripts\check-security-theatre.ps1 -Path "wallet/src"
```

## 🛡️ Configuration des Exceptions

### Fichier d'Exceptions

**Fichier:** `.security-theatre-ignore`

Format: `path_pattern:regex_pattern`

### Exemples d'Exceptions

```bash
# Tests peuvent utiliser expect() avec message clair
**/tests/*.rs:expect\(".*"\)

# CLI test tool peut utiliser println
cli/src/test_tool.rs:println!

# Documentation peut contenir des placeholders
docs/**/*.md://\s*Placeholder

# Constantes cryptographiques légitimes
**/*.rs:0x[0-9a-fA-F]{8,}\s*//\s*[A-Z_]+
```

### Ajouter une Exception

1. Ouvrir `.security-theatre-ignore`
2. Ajouter la ligne: `path_pattern:regex_pattern`
3. Tester: `.\scripts\check-security-theatre.ps1 -Verbose`

## 🔄 Intégration au Workflow

### Pre-commit Hook

Le système est intégré au hook Git pre-commit:

**Fichier:** `.git/hooks/pre-commit`

```bash
# Exécute automatiquement avant chaque commit
git commit -m "message"
# → Lance check-security-theatre.ps1
# → Bloque le commit si détection
```

### Script Pre-commit

**Fichier:** `scripts/pre-commit.ps1`

Le script inclut maintenant l'étape 8:

```powershell
# 8. Check Security Theatre
Write-Host "`n8. Checking for security theatre..." -ForegroundColor Yellow
& ".\scripts\check-security-theatre.ps1"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Security theatre detected!" -ForegroundColor Red
    $errors++
}
```

### Configuration Clippy

**Fichier:** `.cargo/config.toml`

Configuration stricte de Clippy pour détecter:

```toml
[clippy]
deny = [
    "clippy::todo",           # Prevent todo!() macros
    "clippy::unimplemented",  # Prevent unimplemented!() macros
    "clippy::panic",          # Prevent panic!() macros
    "clippy::unwrap_used",    # Prevent .unwrap() usage
    "clippy::print_stdout",   # Prevent println!() in production
    "clippy::dbg_macro",      # Prevent dbg!() macros
    # ... et bien d'autres
]
```

## 📊 Rapport de Détection

### Exemple de Sortie

```
🔍 Security Theatre Detection
=============================

📁 Scanning 15 Rust files...

📊 Security Theatre Report
=========================

❌ Security theatre detected: 3 issues

📋 Issues by Category:
  Asserts inutiles: 1
  Placeholders: 2

🚨 Top Issues:
  wallet/src/rpc.rs:45 - Asserts inutiles
    assert!(true)
  wallet/src/client.rs:23 - Placeholders
    // TODO: implement this
  wallet/src/multisig.rs:67 - Placeholders
    // FIXME: add validation

💡 Recommendations:
  1. Replace .unwrap() with proper error handling
  2. Remove placeholder comments and implement real code
  3. Replace assumptions with validated logic
  4. Use constants instead of magic numbers
  5. Remove hardcoded credentials

❌ COMMIT BLOCKED - Fix security theatre issues first
```

## 🚀 Workflow de Développement

### 1. Développement Normal

```powershell
# Écrire du code
# Tester localement
cargo test

# Commit (déclenche automatiquement les checks)
git add .
git commit -m "Add new feature"
# → Pre-commit hook lance check-security-theatre.ps1
# → Bloque si détection
```

### 2. Contournement Temporaire

Si vous devez temporairement contourner une détection:

1. **Ajouter une exception** dans `.security-theatre-ignore`
2. **Justifier** dans le message de commit
3. **Créer une issue** pour corriger plus tard

```bash
# Exemple d'exception temporaire
wallet/src/special_case.rs:unimplemented!\s*\(\s*"Feature.*"\s*\)
```

### 3. Correction des Issues

```powershell
# 1. Identifier les issues
.\scripts\check-security-theatre.ps1 -Verbose

# 2. Corriger le code
# Remplacer assert!(true) par un vrai test
# Remplacer // TODO par du code réel
# Remplacer .unwrap() par proper error handling

# 3. Vérifier la correction
.\scripts\check-security-theatre.ps1

# 4. Commit
git add .
git commit -m "Fix security theatre issues"
```

## 🔧 Configuration Avancée

### Variables d'Environnement

```powershell
# Désactiver temporairement (développement uniquement)
$env:SECURITY_THEATRE_CHECK = "false"

# Changer le fichier d'exceptions
$env:SECURITY_THEATRE_IGNORE = ".custom-ignore"
```

### Intégration CI/CD

```yaml
# .github/workflows/security.yml
name: Security Theatre Check
on: [push, pull_request]
jobs:
  security-theatre:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - name: Check Security Theatre
        run: |
          powershell -ExecutionPolicy Bypass -File "scripts/check-security-theatre.ps1"
```

## 📚 Bonnes Pratiques

### ✅ À Faire

1. **Utiliser des constantes** au lieu de magic numbers
   ```rust
   const XMR_TO_ATOMIC: u64 = 1_000_000_000_000;
   ```

2. **Gérer les erreurs proprement**
   ```rust
   let result = risky_call()
       .context("Failed to perform risky operation")?;
   ```

3. **Implémenter du code réel** au lieu de placeholders
   ```rust
   // Au lieu de: // TODO: implement validation
   fn validate_input(input: &str) -> Result<(), ValidationError> {
       // Code réel ici
   }
   ```

4. **Documenter les suppositions**
   ```rust
   // SÉCURITÉ: Cette fonction assume que l'input est déjà validé
   // car elle est appelée uniquement après validate_input()
   fn process_validated_input(input: &str) -> Result<(), ProcessingError> {
       // Code ici
   }
   ```

### ❌ À Éviter

1. **Asserts inutiles**
   ```rust
   assert!(true);  // ❌ Ne teste rien
   ```

2. **Placeholders en production**
   ```rust
   // TODO: implement this  // ❌ En production
   ```

3. **Suppositions non documentées**
   ```rust
   // should work  // ❌ Basé sur une supposition
   ```

4. **Credentials hardcodés**
   ```rust
   let password = "secret123";  // ❌ En dur
   ```

## 🆘 Dépannage

### Problèmes Courants

1. **"PowerShell not found"**
   ```bash
   # Installer PowerShell Core
   # Ou utiliser le fallback Unix
   ```

2. **"Too many false positives"**
   ```bash
   # Ajouter des exceptions dans .security-theatre-ignore
   # Ajuster les patterns dans le script
   ```

3. **"Hook not working"**
   ```bash
   # Vérifier les permissions
   chmod +x .git/hooks/pre-commit
   
   # Tester manuellement
   .git/hooks/pre-commit
   ```

### Support

- **Issues GitHub** : Pour signaler des bugs
- **Documentation** : Ce fichier et les commentaires dans le code
- **Tests** : `cargo test` pour vérifier le fonctionnement

## 📈 Métriques

Le système track automatiquement:

- Nombre d'issues détectées par catégorie
- Fichiers les plus problématiques
- Tendances dans le temps
- Taux de correction

Voir `scripts/metrics-dashboard.ps1` pour le dashboard complet.

---

**Dernière mise à jour:** 2024-12-08  
**Version:** 1.0  
**Mainteneur:** Monero Marketplace Team
