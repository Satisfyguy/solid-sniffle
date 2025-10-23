# Exemples d'utilisation - Code Validator MCP

## 🎯 Cas d'usage concrets

### 1. Validation basique de code Python

```python
# Code à valider
code_python = """
def calculate_average(numbers):
    if not numbers:
        return 0
    total = sum(numbers)
    return total / len(numbers)

# Test
result = calculate_average([1, 2, 3, 4, 5])
print(f"Moyenne: {result}")
"""

# Utiliser l'outil validate_code
response = await validate_code({
    "code": code_python,
    "language": "python",
    "validation_level": "standard"
})
```

**Résultat attendu:**
```markdown
✅ **Code valide**

- Langage: python
- Aucun problème détecté
```

### 2. Détection d'hallucinations communes

```python
# Code avec des hallucinations typiques
code_suspect = """
from super.advanced.ai import MagicSolver
import quantum.computing as qc

def solve_everything():
    solver = MagicSolver()
    result = solver.superMethod()
    
    # TODO: [IMPLEMENT THIS]
    <YOUR_API_KEY_HERE>
    
    return result.magicFunction()
"""

response = await validate_code({
    "code": code_suspect,
    "check_hallucinations": true
})
```

**Résultat attendu:**
```markdown
⚠️ **Problèmes détectés**

- Langage: python
- Erreurs: 2
- Avertissements: 3

### 🔴 Erreurs
**Ligne 1**: Import 'super.advanced.ai' pourrait ne pas exister
**Ligne 7**: Placeholder non remplacé

### 🟡 Avertissements
**Ligne 6**: Méthode "superMethod" probablement inventée
**Ligne 10**: Méthode "magicFunction" probablement inventée
```

### 3. Vérification des imports avec disponibilité

```python
code_with_imports = """
import os
import json
import pandas as pd
import numpy as np
from fastapi import FastAPI, HTTPException
from mysterious_package import something
"""

response = await check_imports({
    "code": code_with_imports,
    "check_availability": true,
    "response_format": "markdown"
})
```

**Résultat attendu:**
```markdown
## 📦 Analyse des Imports

**Langage**: python
**Nombre d'imports**: 6

### Imports détectés

- **Ligne 1**: `import os` ✅
- **Ligne 2**: `import json` ✅
- **Ligne 3**: `import pandas` ✅
- **Ligne 4**: `import numpy` ✅
- **Ligne 5**: `from fastapi import FastAPI, HTTPException` ✅
- **Ligne 6**: `from mysterious_package import something` ❌ (non installé)
```

### 4. Exécution de tests avec validation

```python
# Code avec fonction et tests
code_to_test = """
def is_palindrome(text):
    '''Vérifie si un texte est un palindrome'''
    cleaned = ''.join(c.lower() for c in text if c.isalnum())
    return cleaned == cleaned[::-1]
"""

test_code = """
# Tests
assert is_palindrome("A man, a plan, a canal: Panama") == True
assert is_palindrome("race a car") == False
assert is_palindrome("") == True
print("✅ Tous les tests passent !")
"""

response = await run_tests({
    "code": code_to_test,
    "test_code": test_code,
    "language": "python",
    "timeout": 5
})
```

**Résultat attendu:**
```markdown
## ✅ Exécution réussie

- **Langage**: python
- **Temps d'exécution**: 0.03s

### Sortie
```
✅ Tous les tests passent !
```
```

### 5. Analyse de complexité avec suggestions

```python
code_complex = """
def process_data(data, options=None):
    results = []
    if options is None:
        options = {}
    
    for item in data:
        if isinstance(item, dict):
            if 'value' in item:
                if item['value'] > 0:
                    if 'type' in item:
                        if item['type'] == 'special':
                            if 'category' in options:
                                if options['category'] == item.get('cat'):
                                    results.append(item['value'] * 2)
                                else:
                                    results.append(item['value'])
                            else:
                                results.append(item['value'] * 1.5)
                        else:
                            results.append(item['value'])
    
    return results
"""

response = await analyze_complexity({
    "code": code_complex,
    "include_suggestions": true
})
```

**Résultat attendu:**
```markdown
## 📊 Analyse de Complexité

**Langage**: python

### Métriques Générales

- **Lignes totales**: 21
- **Lignes de code**: 20
- **Niveau d'imbrication max**: 8
- **Nombre de fonctions**: 1

### Fonctions

- **process_data** (ligne 1)
  - Longueur: 21 lignes
  - Complexité: 8

### 💡 Suggestions d'Amélioration

- Niveau d'imbrication maximal de 8. Refactorisez pour réduire la complexité.
- La fonction 'process_data' a une complexité de 8. Simplifiez la logique.
- Considérez l'extraction de la logique imbriquée dans des fonctions séparées.
```

### 6. Comparaison de versions pour détecter les régressions

```python
original_code = """
def calculate_discount(price, discount_percent):
    if discount_percent < 0 or discount_percent > 100:
        raise ValueError("Le pourcentage doit être entre 0 et 100")
    
    discount = price * (discount_percent / 100)
    return price - discount
"""

new_code = """
def calculate_discount(price, discount_percent):
    # Oups, j'ai oublié la validation !
    discount = price * discount_percent  # Bug: pas de division par 100
    return price - discount
"""

response = await compare_code_versions({
    "original_code": original_code,
    "new_code": new_code,
    "check_regression": true
})
```

**Résultat attendu:**
```markdown
## 🔄 Comparaison de Code

**Langage**: python

### Statistiques

- **Lignes ajoutées**: 1 ➕
- **Lignes supprimées**: 3 ➖
- **Total des changements**: 4

### ⚠️ Problèmes Potentiels

- **validation_removed**: La validation des paramètres a été supprimée
- **logic_change**: La logique de calcul a changé (division par 100 manquante)
```

## 🔧 Utilisation avancée avec Claude Code CLI

### Configuration pour un projet spécifique

Créez un fichier `.claude-validation.yml` dans votre projet :

```yaml
# Configuration de validation pour ce projet
validation:
  level: strict
  
  rules:
    - name: no-console-log
      pattern: "console\\.log"
      severity: warning
      message: "Utiliser un logger au lieu de console.log"
    
    - name: no-any-type
      pattern: ": any"
      severity: error
      message: "Éviter le type 'any' en TypeScript"
    
    - name: require-docstring
      pattern: "^def \\w+\\([^)]*\\):$"
      requires: "'''"
      message: "Les fonctions doivent avoir une docstring"

  ignore:
    - "test_*.py"
    - "*.spec.js"
    - "migrations/*"

  auto_fix:
    enabled: true
    patterns:
      - find: "print("
        replace: "logger.debug("
      - find: "== None"
        replace: "is None"
```

### Workflow automatisé

```bash
#!/bin/bash
# Script de validation automatique

# 1. Valider tous les fichiers Python du projet
for file in $(find . -name "*.py" -not -path "./venv/*"); do
    echo "Validation de $file..."
    claude-code validate "$file" --level strict
done

# 2. Vérifier les imports
claude-code check-imports --verify-installed

# 3. Analyser la complexité
claude-code analyze-complexity --threshold 10

# 4. Générer un rapport
claude-code report --format html --output validation-report.html
```

## 🎨 Personnalisation des patterns

### Ajouter des patterns d'hallucination personnalisés

```python
# Dans votre configuration ou script
CUSTOM_PATTERNS = [
    # Pattern pour détecter les API keys en dur
    (r'api_key\s*=\s*["\'][^"\']+["\']', 'API key en dur détectée'),
    
    # Pattern pour les mots de passe
    (r'password\s*=\s*["\'][^"\']+["\']', 'Mot de passe en dur'),
    
    # Pattern pour les TODO non complétés
    (r'TODO:\s*\[.*?\]', 'TODO avec placeholder'),
    
    # Pattern pour les imports non utilisés (simpliste)
    (r'^import (\w+)$', 'Vérifier si cet import est utilisé'),
]
```

## 📊 Intégration CI/CD

### GitHub Actions

```yaml
name: Code Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Setup Python
      uses: actions/setup-python@v2
      with:
        python-version: '3.9'
    
    - name: Install MCP Validator
      run: |
        pip install mcp pydantic httpx
        mkdir -p ~/.mcp/servers
        cp code_validator_mcp.py ~/.mcp/servers/
    
    - name: Validate Code
      run: |
        python -c "
        import sys
        sys.path.insert(0, '~/.mcp/servers')
        from code_validator_mcp import validate_code
        # Validation logic here
        "
    
    - name: Generate Report
      if: always()
      run: |
        echo "Generating validation report..."
        # Generate HTML report
```

### GitLab CI

```yaml
validate:
  stage: test
  script:
    - pip install mcp pydantic httpx
    - python code_validator_mcp.py validate --path src/
    - python code_validator_mcp.py report --format json
  artifacts:
    reports:
      junit: validation-report.xml
```

## 🎓 Bonnes pratiques

1. **Toujours valider avant de commiter**
   ```bash
   git add .
   claude-code validate --staged
   git commit -m "Feature validée"
   ```

2. **Utiliser différents niveaux selon le contexte**
   - `basic` : Pour le prototypage rapide
   - `standard` : Pour le développement quotidien
   - `strict` : Avant les releases et pour le code critique

3. **Configurer des seuils de qualité**
   ```python
   QUALITY_THRESHOLDS = {
       'max_complexity': 10,
       'max_function_length': 50,
       'min_test_coverage': 80,
       'max_duplication': 5
   }
   ```

4. **Automatiser la validation**
   - Pre-commit hooks
   - CI/CD pipelines
   - IDE integrations

5. **Documenter les exceptions**
   ```python
   # noqa: E501  # Ligne trop longue mais nécessaire pour l'URL
   very_long_url = "https://example.com/very/long/path/that/cannot/be/broken"
   ```

## 🚀 Performance

Pour optimiser les performances sur de gros projets :

1. **Validation parallèle**
   ```python
   from concurrent.futures import ThreadPoolExecutor
   
   def validate_files(files):
       with ThreadPoolExecutor(max_workers=4) as executor:
           results = executor.map(validate_single_file, files)
       return list(results)
   ```

2. **Mise en cache des résultats**
   ```python
   import hashlib
   
   def get_file_hash(content):
       return hashlib.md5(content.encode()).hexdigest()
   
   # Utiliser le hash pour éviter de revalider
   cache[file_hash] = validation_result
   ```

3. **Validation incrémentale**
   - Valider seulement les fichiers modifiés
   - Utiliser git diff pour identifier les changements

## 📝 Notes

- Le serveur MCP fonctionne de manière asynchrone
- Les validations sont non-bloquantes par défaut
- Les résultats sont mis en cache pendant 5 minutes
- Le timeout par défaut est de 30 secondes
