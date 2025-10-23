# Code Validator MCP - Anti-hallucination pour Claude Code CLI

Un serveur MCP (Model Context Protocol) conçu pour réduire drastiquement les hallucinations lors de la génération de code avec Claude Code CLI.

## 🎯 Objectif

Ce serveur MCP fournit des outils de validation et de vérification en temps réel pour s'assurer que le code généré par Claude est:
- **Syntaxiquement correct** : Validation de la syntaxe pour plusieurs langages
- **Sans hallucinations** : Détection des patterns d'hallucination courants
- **Avec des imports valides** : Vérification que les modules importés existent
- **Testable** : Exécution de tests pour valider le comportement
- **De qualité** : Analyse de complexité et suggestions d'amélioration

## ✨ Fonctionnalités

### 🔍 Outils disponibles

1. **`validate_code`** - Validation complète du code
   - Vérification syntaxique
   - Détection des patterns d'hallucination
   - Analyse des imports suspects
   - Support multi-langage (Python, JavaScript, TypeScript, etc.)

2. **`check_imports`** - Vérification des imports/dépendances
   - Liste tous les imports détectés
   - Vérifie si les packages sont installés
   - Identifie les imports suspects

3. **`run_tests`** - Exécution de tests
   - Exécute le code dans un environnement isolé
   - Support des tests unitaires
   - Timeout configurable

4. **`analyze_complexity`** - Analyse de la qualité
   - Complexité cyclomatique
   - Longueur des fonctions
   - Niveau d'imbrication
   - Suggestions d'amélioration

5. **`compare_code_versions`** - Comparaison de versions
   - Détecte les changements entre deux versions
   - Identifie les régressions potentielles
   - Analyse l'impact sur la qualité

## 📦 Installation

### Prérequis

```bash
# Python 3.8+
python --version

# Installer les dépendances
pip install mcp pydantic httpx
```

### Installation du serveur MCP

1. **Télécharger le serveur**:
```bash
# Créer un dossier pour les serveurs MCP
mkdir -p ~/.mcp/servers
cd ~/.mcp/servers

# Copier le fichier du serveur
cp /path/to/code_validator_mcp.py .

# Rendre le fichier exécutable
chmod +x code_validator_mcp.py
```

2. **Configurer Claude Desktop**:

Ajoutez cette configuration dans `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "code-validator": {
      "command": "python",
      "args": ["/home/YOUR_USER/.mcp/servers/code_validator_mcp.py"],
      "env": {}
    }
  }
}
```

3. **Pour Claude Code CLI**:

Créez ou modifiez `~/.config/claude-code/config.json`:

```json
{
  "mcpServers": {
    "code-validator": {
      "command": "python",
      "args": ["/home/YOUR_USER/.mcp/servers/code_validator_mcp.py"],
      "transport": "stdio"
    }
  },
  "autoValidation": true,
  "validationLevel": "strict"
}
```

## 🚀 Utilisation avec Claude Code CLI

### Configuration recommandée

Créez un fichier `.claude-code-rules` dans votre projet:

```yaml
# Règles de validation pour Claude Code CLI
validation:
  enabled: true
  level: strict
  
  # Validation automatique avant chaque génération
  pre_generation:
    - check_context: true
    - verify_imports: true
  
  # Validation après génération
  post_generation:
    - validate_code: true
    - run_tests: auto
    - check_hallucinations: true

# Patterns d'hallucination personnalisés
hallucination_patterns:
  - pattern: "TODO: implement"
    severity: error
    message: "Implementation manquante"
  
  - pattern: "// FIXME"
    severity: warning
    message: "Code à corriger"

# Configuration des tests
testing:
  auto_test: true
  timeout: 30
  coverage_threshold: 80
```

### Exemples d'utilisation

#### 1. Génération de code avec validation automatique

```bash
# Claude Code CLI utilisera automatiquement le serveur MCP
claude-code generate "Créer une API REST avec FastAPI pour gérer des utilisateurs"

# Le serveur validera automatiquement:
# - La syntaxe Python
# - Les imports FastAPI
# - La structure du code
# - L'absence d'hallucinations
```

#### 2. Validation manuelle d'un fichier

```python
# Dans Claude ou Claude Code CLI
await validate_code({
    "code": open("app.py").read(),
    "language": "python",
    "validation_level": "strict",
    "check_hallucinations": true
})
```

#### 3. Workflow complet avec tests

```bash
# 1. Générer le code
claude-code generate "Fonction pour calculer la suite de Fibonacci"

# 2. Le serveur MCP valide automatiquement
# 3. Génération des tests
claude-code test generate

# 4. Exécution des tests via MCP
# 5. Rapport de validation complet
```

## 🛡️ Patterns d'hallucination détectés

Le serveur détecte automatiquement:

1. **Imports inexistants**
   - `from mysterious.module import *`
   - Chemins d'import trop profonds

2. **Méthodes inventées**
   - `.superMethod()`, `.magicFunction()`
   - Méthodes non standard sur des objets connus

3. **Placeholders non remplacés**
   - `<YOUR_API_KEY_HERE>`
   - `[INSERT_VALUE]`
   - `TODO: implement`

4. **Syntaxe invalide**
   - Erreurs de syntaxe
   - Indentation incorrecte
   - Parenthèses non fermées

5. **Dépendances manquantes**
   - Modules non installés
   - Versions incompatibles

## 📊 Format des réponses

Le serveur supporte deux formats de sortie:

### Format Markdown (par défaut)
```markdown
## ✅ Code valide

- Langage: python
- Aucun problème détecté

### Métriques
- Lignes de code: 150
- Complexité: Faible
- Couverture de tests: 85%
```

### Format JSON
```json
{
  "valid": true,
  "language": "python",
  "issues": [],
  "metrics": {
    "lines": 150,
    "complexity": "low",
    "test_coverage": 85
  }
}
```

## 🔧 Configuration avancée

### Variables d'environnement

```bash
# Niveau de validation par défaut
export MCP_VALIDATION_LEVEL=strict

# Timeout pour l'exécution des tests
export MCP_TEST_TIMEOUT=60

# Activation des logs détaillés
export MCP_DEBUG=true
```

### Personnalisation des règles

Créez un fichier `.mcp-rules.json`:

```json
{
  "custom_patterns": [
    {
      "pattern": "console\\.log",
      "severity": "warning",
      "message": "Utiliser un logger plutôt que console.log"
    }
  ],
  "ignored_imports": [
    "internal_module",
    "legacy_code"
  ],
  "max_complexity": 10,
  "max_function_length": 50
}
```

## 🤝 Intégration avec d'autres outils

### VS Code
```json
{
  "claude-code.validation.enabled": true,
  "claude-code.validation.server": "code-validator",
  "claude-code.validation.autoFix": true
}
```

### Pre-commit hooks
```yaml
repos:
  - repo: local
    hooks:
      - id: claude-code-validate
        name: Validate with Claude Code MCP
        entry: python ~/.mcp/servers/code_validator_mcp.py validate
        language: system
        files: \.(py|js|ts)$
```

## 📈 Métriques et monitoring

Le serveur peut générer des rapports de qualité:

```bash
# Générer un rapport de qualité du projet
claude-code quality-report --format=html --output=report.html

# Métriques en temps réel
claude-code monitor --dashboard
```

## 🐛 Dépannage

### Le serveur ne se lance pas
```bash
# Vérifier les logs
cat ~/.claude-code/logs/mcp-server.log

# Tester manuellement
python ~/.mcp/servers/code_validator_mcp.py --test
```

### Validation trop stricte
```bash
# Ajuster le niveau de validation
export MCP_VALIDATION_LEVEL=standard
```

### Imports non détectés
```bash
# Mettre à jour la liste des packages
pip list --format=json > ~/.mcp/packages.json
```

## 🚦 Niveaux de validation

1. **`basic`** - Validation syntaxique uniquement
2. **`standard`** - Syntaxe + imports (par défaut)
3. **`strict`** - Tout + analyse statique + tests

## 📝 License

MIT

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à:
- Ajouter de nouveaux patterns d'hallucination
- Supporter de nouveaux langages
- Améliorer les algorithmes de détection

## 📞 Support

Pour toute question ou problème:
- Créez une issue sur GitHub
- Consultez la documentation complète
- Contactez le support Claude
