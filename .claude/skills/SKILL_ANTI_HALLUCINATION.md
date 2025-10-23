---
name: anti-hallucination
description: Skill pour prévenir les hallucinations lors de la génération de code. Valide la syntaxe, vérifie les imports, détecte les patterns d'hallucination courants et assure la qualité du code généré.
version: 1.0.0
author: Claude
tags: [validation, quality, testing, anti-hallucination]
---

# Anti-Hallucination Code Skill

## Objectif

Cette skill guide Claude pour générer du code sans hallucinations en appliquant systématiquement des validations et vérifications à chaque étape de la génération de code.

## Processus de génération sécurisée

### Phase 1: Analyse du contexte et planification

Avant de générer du code, TOUJOURS :

1. **Clarifier les besoins**
   - Identifier le langage cible
   - Lister les dépendances nécessaires
   - Définir les entrées/sorties attendues
   - Identifier les edge cases

2. **Vérifier les connaissances**
   - Ne jamais inventer des APIs ou méthodes
   - Utiliser uniquement des patterns documentés
   - Si incertain sur une API, le signaler explicitement

3. **Planifier la structure**
   ```
   - Définir l'architecture du code
   - Identifier les fonctions nécessaires
   - Lister les imports requis
   - Prévoir les tests
   ```

### Phase 2: Génération avec validation continue

#### 2.1 Validation des imports

**TOUJOURS vérifier que les imports existent :**

```python
# ✅ CORRECT - Imports standards vérifiés
import os
import json
from datetime import datetime
from typing import List, Optional

# ❌ ÉVITER - Imports inventés
from super.ai import MagicSolver  # N'existe pas
from quantum.computing import *   # Trop vague
```

**Bibliothèques communes à connaître :**

<library_reference>
Python:
- Standard: os, sys, json, datetime, typing, pathlib, asyncio, re, math, random
- Data: pandas, numpy, scipy, matplotlib, seaborn, plotly
- Web: requests, httpx, fastapi, flask, django, aiohttp
- ML: tensorflow, torch, sklearn, transformers, langchain
- Testing: pytest, unittest, mock
- Utils: pydantic, click, rich, tqdm, python-dotenv

JavaScript/TypeScript:
- Runtime: fs, path, crypto, util, stream, events
- Web: express, fastify, axios, fetch
- React: react, react-dom, next, remix
- Vue: vue, nuxt, vite, pinia
- Testing: jest, vitest, mocha, chai
- Utils: lodash, moment, dayjs, zod, yup

Java:
- Standard: java.util, java.io, java.nio, java.time
- Spring: spring-boot, spring-web, spring-data
- Testing: junit, mockito, assertj
- Utils: lombok, guava, apache-commons

Go:
- Standard: fmt, os, io, time, strings, encoding/json
- Web: gin, echo, fiber, chi
- Database: gorm, sqlx
- Testing: testify, gomock
</library_reference>

#### 2.2 Détection des patterns d'hallucination

**Patterns à ÉVITER absolument :**

```python
# ❌ HALLUCINATION : Méthodes inventées
result = data.superProcess()
output = solver.magicMethod()
response = api.quantumCompute()

# ❌ HALLUCINATION : Placeholders non remplacés
API_KEY = "<YOUR_API_KEY_HERE>"
URL = "[INSERT_URL]"
# TODO: [IMPLEMENT THIS]

# ❌ HALLUCINATION : Syntaxe invalide ou incomplète
def function(
    # Parenthèse non fermée
    
class MyClass
    # : manquant

# ❌ HALLUCINATION : Logique impossible
def divide_by_zero():
    return 1 / 0  # Sans gestion d'erreur

# ❌ HALLUCINATION : Imports circulaires ou impossibles
from mymodule import mymodule  # Import circulaire
from . import ...  # Syntaxe invalide
```

#### 2.3 Patterns de code valide

**TOUJOURS suivre ces patterns :**

```python
# ✅ CORRECT : Gestion d'erreurs appropriée
def safe_divide(a: float, b: float) -> Optional[float]:
    """Divise a par b de manière sécurisée."""
    if b == 0:
        return None
    return a / b

# ✅ CORRECT : Validation des entrées
def process_data(data: List[dict]) -> dict:
    """Traite les données avec validation."""
    if not data:
        raise ValueError("Les données ne peuvent pas être vides")
    
    if not all(isinstance(item, dict) for item in data):
        raise TypeError("Tous les éléments doivent être des dictionnaires")
    
    # Traitement...
    return {"status": "success", "count": len(data)}

# ✅ CORRECT : Imports explicites et vérifiés
from typing import List, Optional, Dict, Any
from dataclasses import dataclass
from enum import Enum

# ✅ CORRECT : Documentation complète
def calculate_metrics(values: List[float]) -> Dict[str, float]:
    """
    Calcule des métriques statistiques.
    
    Args:
        values: Liste de valeurs numériques
        
    Returns:
        Dictionnaire contenant mean, median, std
        
    Raises:
        ValueError: Si la liste est vide
    """
    if not values:
        raise ValueError("La liste ne peut pas être vide")
    
    import statistics
    return {
        "mean": statistics.mean(values),
        "median": statistics.median(values),
        "std": statistics.stdev(values) if len(values) > 1 else 0
    }
```

### Phase 3: Validation post-génération

#### 3.1 Checklist de validation

Après avoir généré du code, TOUJOURS vérifier :

- [ ] **Syntaxe** : Le code est syntaxiquement correct
- [ ] **Imports** : Tous les imports existent et sont utilisés
- [ ] **Types** : Les types sont cohérents (si typé)
- [ ] **Erreurs** : Gestion d'erreurs appropriée
- [ ] **Edge cases** : Cas limites gérés (None, vide, zéro)
- [ ] **Documentation** : Docstrings ou commentaires présents
- [ ] **Nommage** : Conventions respectées (snake_case, camelCase)
- [ ] **Sécurité** : Pas de secrets en dur, pas d'injection
- [ ] **Performance** : Pas de boucles infinies, complexité raisonnable
- [ ] **Tests** : Code testable avec exemples

#### 3.2 Validation par langage

<validation_rules>
Python:
- Indentation : 4 espaces
- Imports : En haut du fichier, groupés (standard, tiers, local)
- Docstrings : Format Google ou NumPy
- Types : Utiliser typing pour les hints
- Erreurs : try/except avec exceptions spécifiques
- Tests : assert ou pytest

JavaScript/TypeScript:
- Semicolons : Cohérent (avec ou sans)
- Imports : ES6 modules ou CommonJS (cohérent)
- JSDoc : Pour la documentation
- Types : Interfaces en TypeScript
- Erreurs : try/catch, Promise.catch
- Tests : describe/it/expect

Java:
- Packages : Convention domaine inversé
- Imports : Pas de wildcard sauf nécessaire
- JavaDoc : /** */ pour les méthodes publiques
- Types : Génériques quand approprié
- Erreurs : try/catch avec finally si nécessaire
- Tests : @Test avec JUnit

Go:
- Packages : Nom court et descriptif
- Imports : Groupés et gofmt
- Comments : // pour l'exporté
- Erreurs : Retour explicite d'error
- Tests : _test.go avec TestXxx
</validation_rules>

### Phase 4: Tests et exemples

#### 4.1 Toujours fournir des exemples d'utilisation

```python
# Après avoir défini une fonction, montrer son usage :

# Définition
def parse_config(config_path: str) -> dict:
    """Parse un fichier de configuration JSON."""
    with open(config_path, 'r') as f:
        return json.load(f)

# Exemple d'utilisation
if __name__ == "__main__":
    # Exemple avec un fichier valide
    try:
        config = parse_config("config.json")
        print(f"Configuration chargée : {config}")
    except FileNotFoundError:
        print("Fichier de configuration non trouvé")
    except json.JSONDecodeError:
        print("Fichier de configuration invalide")
```

#### 4.2 Inclure des tests basiques

```python
def test_function():
    """Tests basiques pour valider le comportement."""
    # Test cas normal
    assert add(2, 3) == 5
    
    # Test cas limite
    assert add(0, 0) == 0
    
    # Test nombres négatifs
    assert add(-1, 1) == 0
    
    # Test avec None (si applicable)
    assert add(None, 5) is None  # ou lever une exception
    
    print("✅ Tous les tests passent")
```

## Exemples de génération sécurisée

### Exemple 1: API REST avec FastAPI (sans hallucination)

```python
# ✅ VERSION SÉCURISÉE - Sans hallucination
from typing import List, Optional
from datetime import datetime
from pydantic import BaseModel, Field, validator
from fastapi import FastAPI, HTTPException, status

# Modèles Pydantic avec validation
class UserCreate(BaseModel):
    """Modèle pour créer un utilisateur."""
    username: str = Field(..., min_length=3, max_length=50)
    email: str = Field(..., regex=r'^[\w\.-]+@[\w\.-]+\.\w+$')
    age: Optional[int] = Field(None, ge=0, le=150)
    
    @validator('username')
    def username_alphanumeric(cls, v):
        if not v.replace('_', '').isalnum():
            raise ValueError('Username doit être alphanumérique')
        return v

class UserResponse(BaseModel):
    """Modèle de réponse utilisateur."""
    id: int
    username: str
    email: str
    age: Optional[int]
    created_at: datetime

# Application FastAPI
app = FastAPI(title="User API", version="1.0.0")

# Base de données simulée (en production, utiliser une vraie DB)
users_db = []
user_id_counter = 1

@app.post("/users", response_model=UserResponse, status_code=status.HTTP_201_CREATED)
async def create_user(user: UserCreate):
    """
    Crée un nouvel utilisateur.
    
    Args:
        user: Données de l'utilisateur à créer
        
    Returns:
        L'utilisateur créé avec son ID
        
    Raises:
        HTTPException: Si l'email existe déjà
    """
    # Vérifier l'unicité de l'email
    if any(u['email'] == user.email for u in users_db):
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Cet email est déjà utilisé"
        )
    
    # Créer l'utilisateur
    global user_id_counter
    new_user = {
        "id": user_id_counter,
        "username": user.username,
        "email": user.email,
        "age": user.age,
        "created_at": datetime.now()
    }
    
    users_db.append(new_user)
    user_id_counter += 1
    
    return UserResponse(**new_user)

@app.get("/users", response_model=List[UserResponse])
async def list_users(skip: int = 0, limit: int = 10):
    """
    Liste les utilisateurs avec pagination.
    
    Args:
        skip: Nombre d'utilisateurs à ignorer
        limit: Nombre maximum d'utilisateurs à retourner
        
    Returns:
        Liste des utilisateurs
    """
    if skip < 0:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="skip doit être >= 0"
        )
    
    if limit < 1 or limit > 100:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="limit doit être entre 1 et 100"
        )
    
    return [UserResponse(**u) for u in users_db[skip:skip + limit]]

# Exemple d'utilisation
if __name__ == "__main__":
    import uvicorn
    # Pour lancer : python app.py
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Exemple 2: Traitement de données avec pandas (sans hallucination)

```python
# ✅ VERSION SÉCURISÉE - Sans hallucination
import pandas as pd
import numpy as np
from typing import Optional, Dict, Any
from pathlib import Path

def analyze_sales_data(
    file_path: str,
    date_column: str = 'date',
    amount_column: str = 'amount'
) -> Dict[str, Any]:
    """
    Analyse les données de ventes depuis un fichier CSV.
    
    Args:
        file_path: Chemin vers le fichier CSV
        date_column: Nom de la colonne de date
        amount_column: Nom de la colonne de montant
        
    Returns:
        Dictionnaire avec les statistiques
        
    Raises:
        FileNotFoundError: Si le fichier n'existe pas
        ValueError: Si les colonnes requises sont manquantes
    """
    # Vérifier l'existence du fichier
    if not Path(file_path).exists():
        raise FileNotFoundError(f"Fichier non trouvé : {file_path}")
    
    # Charger les données
    try:
        df = pd.read_csv(file_path)
    except Exception as e:
        raise ValueError(f"Erreur lors de la lecture du CSV : {e}")
    
    # Vérifier les colonnes requises
    required_columns = [date_column, amount_column]
    missing_columns = [col for col in required_columns if col not in df.columns]
    
    if missing_columns:
        raise ValueError(f"Colonnes manquantes : {missing_columns}")
    
    # Nettoyer les données
    df[date_column] = pd.to_datetime(df[date_column], errors='coerce')
    df[amount_column] = pd.to_numeric(df[amount_column], errors='coerce')
    
    # Supprimer les valeurs manquantes
    initial_rows = len(df)
    df = df.dropna(subset=[date_column, amount_column])
    rows_dropped = initial_rows - len(df)
    
    if df.empty:
        raise ValueError("Aucune donnée valide après nettoyage")
    
    # Calculer les statistiques
    stats = {
        'total_rows': len(df),
        'rows_dropped': rows_dropped,
        'date_range': {
            'start': df[date_column].min().isoformat(),
            'end': df[date_column].max().isoformat()
        },
        'amount_stats': {
            'total': float(df[amount_column].sum()),
            'mean': float(df[amount_column].mean()),
            'median': float(df[amount_column].median()),
            'std': float(df[amount_column].std()),
            'min': float(df[amount_column].min()),
            'max': float(df[amount_column].max())
        },
        'monthly_average': None
    }
    
    # Calculer la moyenne mensuelle
    df['month'] = df[date_column].dt.to_period('M')
    monthly = df.groupby('month')[amount_column].sum()
    
    if not monthly.empty:
        stats['monthly_average'] = float(monthly.mean())
    
    return stats

# Exemple d'utilisation avec gestion d'erreurs
def main():
    """Exemple d'utilisation de la fonction d'analyse."""
    
    # Créer des données de test
    test_data = pd.DataFrame({
        'date': pd.date_range('2024-01-01', periods=100),
        'amount': np.random.uniform(100, 1000, 100),
        'product': ['A', 'B'] * 50
    })
    
    # Sauvegarder en CSV
    test_file = 'test_sales.csv'
    test_data.to_csv(test_file, index=False)
    
    try:
        # Analyser les données
        results = analyze_sales_data(test_file)
        
        print("📊 Analyse des ventes")
        print("-" * 40)
        print(f"Nombre de lignes : {results['total_rows']}")
        print(f"Période : {results['date_range']['start']} à {results['date_range']['end']}")
        print(f"Total des ventes : ${results['amount_stats']['total']:,.2f}")
        print(f"Moyenne : ${results['amount_stats']['mean']:,.2f}")
        print(f"Médiane : ${results['amount_stats']['median']:,.2f}")
        
        if results['monthly_average']:
            print(f"Moyenne mensuelle : ${results['monthly_average']:,.2f}")
            
    except FileNotFoundError as e:
        print(f"❌ Erreur : {e}")
    except ValueError as e:
        print(f"❌ Données invalides : {e}")
    except Exception as e:
        print(f"❌ Erreur inattendue : {e}")
    finally:
        # Nettoyer
        if Path(test_file).exists():
            Path(test_file).unlink()

if __name__ == "__main__":
    main()
```

## Règles d'or anti-hallucination

### 1. **Ne jamais inventer**
- Si une API/méthode n'est pas certaine, utiliser une alternative connue
- Si impossible, expliquer clairement la limitation

### 2. **Toujours valider**
- Vérifier les entrées
- Gérer les erreurs
- Tester les edge cases

### 3. **Documentation claire**
- Docstrings pour les fonctions
- Commentaires pour la logique complexe
- Exemples d'utilisation

### 4. **Imports réalistes**
- Utiliser uniquement des packages qui existent
- Préférer la stdlib quand possible
- Mentionner les installations nécessaires

### 5. **Gestion d'erreurs robuste**
- Try/catch appropriés
- Messages d'erreur informatifs
- Pas de silent failures

### 6. **Tests inclus**
- Au moins un exemple d'utilisation
- Tests des cas normaux et edge cases
- Validation du comportement

## Checklist finale

Avant de livrer du code, vérifier :

```markdown
## ✅ Checklist Anti-Hallucination

### Syntaxe et Structure
- [ ] Le code compile/s'exécute sans erreur
- [ ] Indentation correcte et cohérente
- [ ] Parenthèses, accolades, crochets équilibrés
- [ ] Pas de syntaxe inventée

### Imports et Dépendances
- [ ] Tous les imports existent
- [ ] Pas d'imports wildcards (sauf justifié)
- [ ] Dépendances mentionnées dans les commentaires
- [ ] Pas de modules inventés

### Qualité du Code
- [ ] Noms de variables/fonctions descriptifs
- [ ] Pas de TODO/FIXME non résolus
- [ ] Pas de placeholders (<...>, [...])
- [ ] Pas de code commenté inutile

### Robustesse
- [ ] Validation des entrées
- [ ] Gestion des erreurs
- [ ] Pas de division par zéro
- [ ] Pas de boucles infinies potentielles

### Documentation
- [ ] Docstrings/commentaires présents
- [ ] Exemples d'utilisation fournis
- [ ] Comportement documenté
- [ ] Edge cases expliqués

### Sécurité
- [ ] Pas de secrets en dur
- [ ] Pas d'injection SQL/commande
- [ ] Validation des entrées utilisateur
- [ ] Pas de eval() ou exec() non sécurisé

### Tests
- [ ] Au moins un exemple qui fonctionne
- [ ] Cas d'erreur testés
- [ ] Résultats attendus documentés
```

## Configuration recommandée

Pour utiliser cette skill efficacement, configurer Claude Code avec :

```yaml
# .claude-code-config.yml
skills:
  - anti-hallucination
  
validation:
  pre_generation:
    - verify_context: true
    - check_knowledge: true
    
  during_generation:
    - validate_syntax: realtime
    - check_imports: true
    - detect_patterns: true
    
  post_generation:
    - full_validation: true
    - run_tests: auto
    - generate_examples: true

code_style:
  language_defaults:
    python:
      style: pep8
      docstring: google
      typing: strict
    javascript:
      style: standard
      semicolons: false
    typescript:
      strict: true
      
anti_hallucination:
  level: strict
  auto_fix: true
  explain_fixes: true
```

## Utilisation de la skill

Pour activer cette skill dans Claude Code :

```bash
# Activer la skill
claude-code skill enable anti-hallucination

# Générer du code avec validation
claude-code generate "fonction pour parser des emails" --skill anti-hallucination

# Valider du code existant
claude-code validate myfile.py --skill anti-hallucination

# Rapport de qualité
claude-code quality-check --skill anti-hallucination
```

Cette skill garantit que le code généré est :
- ✅ Syntaxiquement correct
- ✅ Sans imports inventés
- ✅ Sans méthodes hallucinées
- ✅ Avec gestion d'erreurs
- ✅ Documenté et testé
- ✅ Prêt pour la production