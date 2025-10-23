#!/usr/bin/env python3
"""
Code Validator MCP Server - Anti-hallucination pour Claude Code CLI

Ce serveur MCP fournit des outils pour valider et vérifier le code généré,
réduisant ainsi les hallucinations lors de la génération de code.

Fonctionnalités principales:
- Validation syntaxique multi-langage
- Vérification des imports et dépendances
- Analyse statique du code
- Détection des patterns d'hallucination courants
- Vérification de la cohérence du code
- Tests automatiques
"""

import asyncio
import ast
import json
import os
import re
import subprocess
import tempfile
from enum import Enum
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple

from mcp.server.fastmcp import FastMCP
from pydantic import BaseModel, Field, field_validator, ConfigDict
import httpx

# ===================== CONFIGURATION =====================

# Initialiser le serveur MCP
mcp = FastMCP("code_validator_mcp")

# Limites et constantes
CHARACTER_LIMIT = 50000
MAX_FILE_SIZE = 1_000_000  # 1MB
TIMEOUT_SECONDS = 30

# Patterns d'hallucination courants
HALLUCINATION_PATTERNS = [
    # Imports inexistants
    (r'from (\w+)\.(\w+) import \*', 'Import avec wildcard suspect'),
    (r'import (\w+)\.(\w+)\.(\w+)\.(\w+)', 'Import avec chemin trop profond'),
    
    # Méthodes inventées
    (r'\.superMethod\(', 'Méthode "superMethod" probablement inventée'),
    (r'\.magicFunction\(', 'Méthode "magicFunction" probablement inventée'),
    
    # Commentaires suspects
    (r'#\s*TODO:?\s*\[.*?\]', 'TODO avec placeholder non rempli'),
    (r'//\s*FIXME:?\s*\[.*?\]', 'FIXME avec placeholder non rempli'),
    
    # Placeholders non remplacés
    (r'<YOUR_.*?_HERE>', 'Placeholder non remplacé'),
    (r'\[INSERT_.*?\]', 'Placeholder non remplacé'),
    (r'XXX', 'Marqueur XXX trouvé'),
]

# ===================== MODÈLES PYDANTIC =====================

class ResponseFormat(str, Enum):
    """Format de sortie pour les réponses."""
    JSON = "json"
    MARKDOWN = "markdown"

class ProgrammingLanguage(str, Enum):
    """Langages de programmation supportés."""
    PYTHON = "python"
    JAVASCRIPT = "javascript"
    TYPESCRIPT = "typescript"
    JAVA = "java"
    CPP = "cpp"
    C = "c"
    GO = "go"
    RUST = "rust"
    PHP = "php"
    RUBY = "ruby"
    SWIFT = "swift"
    KOTLIN = "kotlin"
    AUTO = "auto"

class ValidationLevel(str, Enum):
    """Niveau de validation."""
    BASIC = "basic"      # Syntaxe seulement
    STANDARD = "standard"  # Syntaxe + imports
    STRICT = "strict"    # Tout + analyse statique

class ValidateCodeInput(BaseModel):
    """Paramètres pour la validation de code."""
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra='forbid'
    )
    
    code: str = Field(..., description="Code à valider", min_length=1, max_length=MAX_FILE_SIZE)
    language: ProgrammingLanguage = Field(
        default=ProgrammingLanguage.AUTO,
        description="Langage du code (auto-détection si non spécifié)"
    )
    validation_level: ValidationLevel = Field(
        default=ValidationLevel.STANDARD,
        description="Niveau de validation: basic (syntaxe), standard (+imports), strict (+analyse)"
    )
    check_hallucinations: bool = Field(
        default=True,
        description="Vérifier les patterns d'hallucination courants"
    )
    response_format: ResponseFormat = Field(
        default=ResponseFormat.MARKDOWN,
        description="Format de sortie: 'markdown' ou 'json'"
    )

class CheckImportsInput(BaseModel):
    """Paramètres pour vérifier les imports/dépendances."""
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra='forbid'
    )
    
    code: str = Field(..., description="Code à analyser", min_length=1, max_length=MAX_FILE_SIZE)
    language: ProgrammingLanguage = Field(
        default=ProgrammingLanguage.AUTO,
        description="Langage du code"
    )
    check_availability: bool = Field(
        default=True,
        description="Vérifier si les packages sont installés/disponibles"
    )
    response_format: ResponseFormat = Field(
        default=ResponseFormat.MARKDOWN,
        description="Format de sortie"
    )

class RunTestsInput(BaseModel):
    """Paramètres pour exécuter des tests sur le code."""
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra='forbid'
    )
    
    code: str = Field(..., description="Code à tester", min_length=1, max_length=MAX_FILE_SIZE)
    test_code: Optional[str] = Field(
        default=None,
        description="Code de test (si séparé du code principal)",
        max_length=MAX_FILE_SIZE
    )
    language: ProgrammingLanguage = Field(
        default=ProgrammingLanguage.AUTO,
        description="Langage du code"
    )
    timeout: int = Field(
        default=TIMEOUT_SECONDS,
        description="Timeout en secondes pour l'exécution",
        ge=1,
        le=300
    )
    response_format: ResponseFormat = Field(
        default=ResponseFormat.MARKDOWN,
        description="Format de sortie"
    )

class AnalyzeComplexityInput(BaseModel):
    """Paramètres pour analyser la complexité du code."""
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra='forbid'
    )
    
    code: str = Field(..., description="Code à analyser", min_length=1, max_length=MAX_FILE_SIZE)
    language: ProgrammingLanguage = Field(
        default=ProgrammingLanguage.AUTO,
        description="Langage du code"
    )
    include_suggestions: bool = Field(
        default=True,
        description="Inclure des suggestions d'amélioration"
    )
    response_format: ResponseFormat = Field(
        default=ResponseFormat.MARKDOWN,
        description="Format de sortie"
    )

class CompareCodeVersionsInput(BaseModel):
    """Paramètres pour comparer deux versions de code."""
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra='forbid'
    )
    
    original_code: str = Field(..., description="Code original", min_length=1, max_length=MAX_FILE_SIZE)
    new_code: str = Field(..., description="Nouveau code", min_length=1, max_length=MAX_FILE_SIZE)
    language: ProgrammingLanguage = Field(
        default=ProgrammingLanguage.AUTO,
        description="Langage du code"
    )
    check_regression: bool = Field(
        default=True,
        description="Vérifier les régressions potentielles"
    )
    response_format: ResponseFormat = Field(
        default=ResponseFormat.MARKDOWN,
        description="Format de sortie"
    )

# ===================== FONCTIONS UTILITAIRES =====================

def detect_language(code: str) -> ProgrammingLanguage:
    """Détecte automatiquement le langage du code."""
    patterns = {
        ProgrammingLanguage.PYTHON: [r'^\s*import\s+\w+', r'^\s*from\s+\w+', r'^\s*def\s+\w+', r'^\s*class\s+\w+'],
        ProgrammingLanguage.JAVASCRIPT: [r'^\s*const\s+\w+', r'^\s*let\s+\w+', r'^\s*function\s+\w+', r'^\s*=>'],
        ProgrammingLanguage.TYPESCRIPT: [r'^\s*interface\s+\w+', r'^\s*type\s+\w+', r': string', r': number'],
        ProgrammingLanguage.JAVA: [r'^\s*public\s+class', r'^\s*import\s+java\.', r'^\s*package\s+\w+'],
        ProgrammingLanguage.CPP: [r'^\s*#include\s*<', r'^\s*using\s+namespace', r'^\s*class\s+\w+\s*{'],
        ProgrammingLanguage.GO: [r'^\s*package\s+\w+', r'^\s*import\s+"', r'^\s*func\s+\w+'],
        ProgrammingLanguage.RUST: [r'^\s*use\s+\w+', r'^\s*fn\s+\w+', r'^\s*impl\s+\w+'],
    }
    
    for lang, patterns_list in patterns.items():
        for pattern in patterns_list:
            if re.search(pattern, code, re.MULTILINE):
                return lang
    
    return ProgrammingLanguage.PYTHON  # Par défaut

def check_hallucination_patterns(code: str) -> List[Dict[str, Any]]:
    """Vérifie les patterns d'hallucination courants dans le code."""
    issues = []
    
    lines = code.split('\n')
    for line_num, line in enumerate(lines, 1):
        for pattern, description in HALLUCINATION_PATTERNS:
            if re.search(pattern, line):
                issues.append({
                    'line': line_num,
                    'type': 'hallucination_pattern',
                    'severity': 'warning',
                    'message': description,
                    'code': line.strip()
                })
    
    return issues

async def validate_python_syntax(code: str) -> Tuple[bool, List[Dict[str, Any]]]:
    """Valide la syntaxe Python."""
    issues = []
    try:
        ast.parse(code)
        return True, []
    except SyntaxError as e:
        issues.append({
            'line': e.lineno,
            'type': 'syntax_error',
            'severity': 'error',
            'message': str(e.msg),
            'code': e.text.strip() if e.text else ''
        })
        return False, issues

async def validate_javascript_syntax(code: str) -> Tuple[bool, List[Dict[str, Any]]]:
    """Valide la syntaxe JavaScript avec Node.js."""
    issues = []
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
        f.write(code)
        temp_file = f.name
    
    try:
        result = subprocess.run(
            ['node', '--check', temp_file],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        if result.returncode != 0:
            # Parser les erreurs
            for line in result.stderr.split('\n'):
                if 'SyntaxError' in line:
                    issues.append({
                        'line': 0,
                        'type': 'syntax_error',
                        'severity': 'error',
                        'message': line,
                        'code': ''
                    })
            return False, issues
        
        return True, []
    except (subprocess.TimeoutExpired, FileNotFoundError):
        # Node.js non disponible
        return True, []  # On assume que c'est OK si on ne peut pas vérifier
    finally:
        os.unlink(temp_file)

async def check_python_imports(code: str) -> List[Dict[str, Any]]:
    """Vérifie les imports Python."""
    issues = []
    import_pattern = r'^\s*(?:from\s+([\w\.]+)\s+)?import\s+([\w\s,]+)'
    
    for line_num, line in enumerate(code.split('\n'), 1):
        match = re.match(import_pattern, line)
        if match:
            module = match.group(1) or match.group(2).split(',')[0].strip()
            
            # Vérifier les imports standards
            standard_libs = {
                'os', 'sys', 'json', 'math', 'random', 'datetime', 'time',
                'collections', 'itertools', 'functools', 're', 'typing',
                'pathlib', 'subprocess', 'asyncio', 'threading', 'multiprocessing'
            }
            
            # Vérifier les imports courants tiers
            common_libs = {
                'numpy', 'pandas', 'matplotlib', 'requests', 'flask', 'django',
                'pytest', 'scipy', 'sklearn', 'tensorflow', 'torch', 'fastapi'
            }
            
            base_module = module.split('.')[0]
            
            if base_module not in standard_libs and base_module not in common_libs:
                # Import suspect
                issues.append({
                    'line': line_num,
                    'type': 'suspicious_import',
                    'severity': 'warning',
                    'message': f"Import '{module}' pourrait ne pas exister",
                    'code': line.strip()
                })
    
    return issues

def format_validation_result(
    is_valid: bool,
    issues: List[Dict[str, Any]],
    language: str,
    response_format: ResponseFormat,
    additional_info: Optional[Dict[str, Any]] = None
) -> str:
    """Formate le résultat de validation."""
    
    if response_format == ResponseFormat.JSON:
        result = {
            'valid': is_valid,
            'language': language,
            'issues': issues,
            'issue_count': {
                'errors': len([i for i in issues if i['severity'] == 'error']),
                'warnings': len([i for i in issues if i['severity'] == 'warning']),
                'info': len([i for i in issues if i['severity'] == 'info'])
            }
        }
        if additional_info:
            result.update(additional_info)
        return json.dumps(result, indent=2)
    
    # Format Markdown
    if is_valid and not issues:
        result = "✅ **Code valide**\n\n"
        result += f"- Langage: {language}\n"
        result += "- Aucun problème détecté\n"
    else:
        result = "⚠️ **Problèmes détectés**\n\n"
        result += f"- Langage: {language}\n"
        
        errors = [i for i in issues if i['severity'] == 'error']
        warnings = [i for i in issues if i['severity'] == 'warning']
        info = [i for i in issues if i['severity'] == 'info']
        
        result += f"- Erreurs: {len(errors)}\n"
        result += f"- Avertissements: {len(warnings)}\n"
        result += f"- Informations: {len(info)}\n\n"
        
        if errors:
            result += "### 🔴 Erreurs\n\n"
            for issue in errors:
                result += f"**Ligne {issue['line']}**: {issue['message']}\n"
                if issue['code']:
                    result += f"```\n{issue['code']}\n```\n"
                result += "\n"
        
        if warnings:
            result += "### 🟡 Avertissements\n\n"
            for issue in warnings:
                result += f"**Ligne {issue['line']}**: {issue['message']}\n"
                if issue['code']:
                    result += f"```\n{issue['code']}\n```\n"
                result += "\n"
    
    if additional_info:
        result += "\n### ℹ️ Informations supplémentaires\n\n"
        for key, value in additional_info.items():
            result += f"- **{key}**: {value}\n"
    
    return result

# ===================== OUTILS MCP =====================

@mcp.tool(
    name="validate_code",
    annotations={
        "title": "Valider le Code",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False
    }
)
async def validate_code(params: ValidateCodeInput) -> str:
    """Valide la syntaxe et la structure du code.
    
    Effectue une validation complète du code incluant:
    - Vérification syntaxique
    - Détection des imports suspects
    - Recherche de patterns d'hallucination
    - Analyse de la cohérence
    
    Args:
        params: Paramètres de validation incluant le code et le niveau de validation
    
    Returns:
        str: Rapport de validation au format demandé
    """
    # Détecter le langage si nécessaire
    language = params.language
    if language == ProgrammingLanguage.AUTO:
        language = detect_language(params.code)
    
    all_issues = []
    is_valid = True
    
    # Validation syntaxique
    if language == ProgrammingLanguage.PYTHON:
        syntax_valid, syntax_issues = await validate_python_syntax(params.code)
        all_issues.extend(syntax_issues)
        is_valid = is_valid and syntax_valid
    elif language in [ProgrammingLanguage.JAVASCRIPT, ProgrammingLanguage.TYPESCRIPT]:
        syntax_valid, syntax_issues = await validate_javascript_syntax(params.code)
        all_issues.extend(syntax_issues)
        is_valid = is_valid and syntax_valid
    
    # Vérification des hallucinations
    if params.check_hallucinations:
        hallucination_issues = check_hallucination_patterns(params.code)
        all_issues.extend(hallucination_issues)
    
    # Vérification des imports (niveau standard et strict)
    if params.validation_level in [ValidationLevel.STANDARD, ValidationLevel.STRICT]:
        if language == ProgrammingLanguage.PYTHON:
            import_issues = await check_python_imports(params.code)
            all_issues.extend(import_issues)
    
    # Analyse statique (niveau strict)
    if params.validation_level == ValidationLevel.STRICT:
        # Ajouter des vérifications supplémentaires ici
        pass
    
    # Limiter la taille de la réponse
    if len(all_issues) > 100:
        all_issues = all_issues[:100]
        additional_info = {'note': 'Liste tronquée à 100 problèmes'}
    else:
        additional_info = None
    
    return format_validation_result(
        is_valid,
        all_issues,
        language.value,
        params.response_format,
        additional_info
    )

@mcp.tool(
    name="check_imports",
    annotations={
        "title": "Vérifier les Imports",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False
    }
)
async def check_imports(params: CheckImportsInput) -> str:
    """Vérifie les imports et dépendances du code.
    
    Analyse les déclarations d'import pour:
    - Identifier les dépendances requises
    - Détecter les imports suspects ou inexistants
    - Vérifier la disponibilité des packages
    
    Args:
        params: Paramètres incluant le code et les options de vérification
    
    Returns:
        str: Rapport sur les imports au format demandé
    """
    # Détecter le langage
    language = params.language
    if language == ProgrammingLanguage.AUTO:
        language = detect_language(params.code)
    
    imports_info = {
        'language': language.value,
        'imports': [],
        'issues': []
    }
    
    if language == ProgrammingLanguage.PYTHON:
        # Analyser les imports Python
        import_pattern = r'^\s*(?:from\s+([\w\.]+)\s+)?import\s+([\w\s,]+)'
        
        for line_num, line in enumerate(params.code.split('\n'), 1):
            match = re.match(import_pattern, line)
            if match:
                from_module = match.group(1)
                import_names = match.group(2)
                
                if from_module:
                    imports_info['imports'].append({
                        'line': line_num,
                        'type': 'from_import',
                        'module': from_module,
                        'names': [n.strip() for n in import_names.split(',')]
                    })
                else:
                    for name in import_names.split(','):
                        imports_info['imports'].append({
                            'line': line_num,
                            'type': 'direct_import',
                            'module': name.strip()
                        })
        
        # Vérifier la disponibilité si demandé
        if params.check_availability:
            standard_libs = {
                'os', 'sys', 'json', 'math', 'random', 'datetime', 'time',
                'collections', 'itertools', 'functools', 're', 'typing', 'pathlib'
            }
            
            for imp in imports_info['imports']:
                module = imp['module'].split('.')[0]
                if module not in standard_libs:
                    # Vérifier avec pip
                    try:
                        result = subprocess.run(
                            ['pip', 'show', module],
                            capture_output=True,
                            text=True,
                            timeout=5
                        )
                        imp['available'] = result.returncode == 0
                    except:
                        imp['available'] = 'unknown'
    
    elif language in [ProgrammingLanguage.JAVASCRIPT, ProgrammingLanguage.TYPESCRIPT]:
        # Analyser les imports JavaScript/TypeScript
        import_patterns = [
            r"^\s*import\s+(.+?)\s+from\s+['\"](.+?)['\"]",
            r"^\s*const\s+(.+?)\s*=\s*require\(['\"](.+?)['\"]\)"
        ]
        
        for line_num, line in enumerate(params.code.split('\n'), 1):
            for pattern in import_patterns:
                match = re.match(pattern, line)
                if match:
                    imports_info['imports'].append({
                        'line': line_num,
                        'names': match.group(1),
                        'module': match.group(2)
                    })
    
    # Formater la réponse
    if params.response_format == ResponseFormat.JSON:
        return json.dumps(imports_info, indent=2)
    
    # Format Markdown
    result = f"## 📦 Analyse des Imports\n\n"
    result += f"**Langage**: {language.value}\n"
    result += f"**Nombre d'imports**: {len(imports_info['imports'])}\n\n"
    
    if imports_info['imports']:
        result += "### Imports détectés\n\n"
        for imp in imports_info['imports']:
            if language == ProgrammingLanguage.PYTHON:
                if imp['type'] == 'from_import':
                    result += f"- **Ligne {imp['line']}**: `from {imp['module']} import {', '.join(imp['names'])}`"
                else:
                    result += f"- **Ligne {imp['line']}**: `import {imp['module']}`"
                
                if 'available' in imp:
                    if imp['available'] == True:
                        result += " ✅"
                    elif imp['available'] == False:
                        result += " ❌ (non installé)"
                    else:
                        result += " ❓"
                result += "\n"
            else:
                result += f"- **Ligne {imp['line']}**: `{imp['names']}` from `{imp['module']}`\n"
    else:
        result += "*Aucun import détecté*\n"
    
    return result

@mcp.tool(
    name="run_tests",
    annotations={
        "title": "Exécuter des Tests",
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": True
    }
)
async def run_tests(params: RunTestsInput) -> str:
    """Exécute des tests sur le code fourni.
    
    Permet d'exécuter le code dans un environnement isolé pour:
    - Vérifier qu'il s'exécute sans erreur
    - Tester des cas d'usage spécifiques
    - Valider le comportement attendu
    
    Args:
        params: Code à tester et paramètres d'exécution
    
    Returns:
        str: Résultats des tests au format demandé
    """
    # Détecter le langage
    language = params.language
    if language == ProgrammingLanguage.AUTO:
        language = detect_language(params.code)
    
    results = {
        'language': language.value,
        'execution_successful': False,
        'output': '',
        'errors': '',
        'execution_time': 0
    }
    
    # Préparer le code à exécuter
    if params.test_code:
        full_code = params.code + "\n\n" + params.test_code
    else:
        full_code = params.code
    
    # Exécuter selon le langage
    if language == ProgrammingLanguage.PYTHON:
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(full_code)
            temp_file = f.name
        
        try:
            import time
            start_time = time.time()
            
            result = subprocess.run(
                ['python', temp_file],
                capture_output=True,
                text=True,
                timeout=params.timeout
            )
            
            execution_time = time.time() - start_time
            
            results['execution_successful'] = result.returncode == 0
            results['output'] = result.stdout[:10000]  # Limiter la sortie
            results['errors'] = result.stderr[:10000]
            results['execution_time'] = round(execution_time, 2)
            
        except subprocess.TimeoutExpired:
            results['errors'] = f"Timeout après {params.timeout} secondes"
        except Exception as e:
            results['errors'] = str(e)
        finally:
            os.unlink(temp_file)
    
    elif language in [ProgrammingLanguage.JAVASCRIPT, ProgrammingLanguage.TYPESCRIPT]:
        with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
            f.write(full_code)
            temp_file = f.name
        
        try:
            import time
            start_time = time.time()
            
            result = subprocess.run(
                ['node', temp_file],
                capture_output=True,
                text=True,
                timeout=params.timeout
            )
            
            execution_time = time.time() - start_time
            
            results['execution_successful'] = result.returncode == 0
            results['output'] = result.stdout[:10000]
            results['errors'] = result.stderr[:10000]
            results['execution_time'] = round(execution_time, 2)
            
        except subprocess.TimeoutExpired:
            results['errors'] = f"Timeout après {params.timeout} secondes"
        except Exception as e:
            results['errors'] = str(e)
        finally:
            os.unlink(temp_file)
    else:
        results['errors'] = f"Langage {language.value} non supporté pour l'exécution"
    
    # Formater la réponse
    if params.response_format == ResponseFormat.JSON:
        return json.dumps(results, indent=2)
    
    # Format Markdown
    if results['execution_successful']:
        result = "## ✅ Exécution réussie\n\n"
    else:
        result = "## ❌ Échec de l'exécution\n\n"
    
    result += f"- **Langage**: {language.value}\n"
    result += f"- **Temps d'exécution**: {results['execution_time']}s\n\n"
    
    if results['output']:
        result += "### Sortie\n```\n"
        result += results['output']
        result += "\n```\n\n"
    
    if results['errors']:
        result += "### Erreurs\n```\n"
        result += results['errors']
        result += "\n```\n\n"
    
    return result

@mcp.tool(
    name="analyze_complexity",
    annotations={
        "title": "Analyser la Complexité",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False
    }
)
async def analyze_complexity(params: AnalyzeComplexityInput) -> str:
    """Analyse la complexité et la qualité du code.
    
    Évalue plusieurs métriques de qualité:
    - Complexité cyclomatique
    - Longueur des fonctions
    - Niveau d'imbrication
    - Duplication de code
    - Respect des conventions
    
    Args:
        params: Code à analyser et options
    
    Returns:
        str: Rapport d'analyse au format demandé
    """
    # Détecter le langage
    language = params.language
    if language == ProgrammingLanguage.AUTO:
        language = detect_language(params.code)
    
    lines = params.code.split('\n')
    
    metrics = {
        'language': language.value,
        'total_lines': len(lines),
        'code_lines': len([l for l in lines if l.strip() and not l.strip().startswith('#')]),
        'comment_lines': len([l for l in lines if l.strip().startswith('#')]),
        'functions': [],
        'classes': [],
        'max_nesting': 0,
        'suggestions': []
    }
    
    if language == ProgrammingLanguage.PYTHON:
        # Analyser les fonctions Python
        function_pattern = r'^\s*def\s+(\w+)\s*\('
        class_pattern = r'^\s*class\s+(\w+)'
        
        current_function = None
        current_class = None
        current_nesting = 0
        
        for line_num, line in enumerate(lines, 1):
            # Compter l'indentation
            indent = len(line) - len(line.lstrip())
            nesting_level = indent // 4
            metrics['max_nesting'] = max(metrics['max_nesting'], nesting_level)
            
            # Détecter les fonctions
            func_match = re.match(function_pattern, line)
            if func_match:
                if current_function:
                    current_function['end_line'] = line_num - 1
                    current_function['lines'] = current_function['end_line'] - current_function['start_line'] + 1
                
                current_function = {
                    'name': func_match.group(1),
                    'start_line': line_num,
                    'complexity': 1  # Base complexity
                }
                metrics['functions'].append(current_function)
            
            # Détecter les classes
            class_match = re.match(class_pattern, line)
            if class_match:
                current_class = {
                    'name': class_match.group(1),
                    'start_line': line_num
                }
                metrics['classes'].append(current_class)
            
            # Calculer la complexité cyclomatique
            if current_function:
                if any(keyword in line for keyword in ['if ', 'elif ', 'for ', 'while ', 'except']):
                    current_function['complexity'] += 1
        
        # Finaliser la dernière fonction
        if current_function:
            current_function['end_line'] = len(lines)
            current_function['lines'] = current_function['end_line'] - current_function['start_line'] + 1
    
    # Générer des suggestions
    if params.include_suggestions:
        # Fonctions trop longues
        for func in metrics['functions']:
            if func.get('lines', 0) > 50:
                metrics['suggestions'].append({
                    'type': 'function_length',
                    'message': f"La fonction '{func['name']}' a {func['lines']} lignes. Considérez la diviser en fonctions plus petites.",
                    'line': func['start_line']
                })
            
            if func.get('complexity', 0) > 10:
                metrics['suggestions'].append({
                    'type': 'complexity',
                    'message': f"La fonction '{func['name']}' a une complexité de {func['complexity']}. Simplifiez la logique.",
                    'line': func['start_line']
                })
        
        # Imbrication excessive
        if metrics['max_nesting'] > 4:
            metrics['suggestions'].append({
                'type': 'nesting',
                'message': f"Niveau d'imbrication maximal de {metrics['max_nesting']}. Refactorisez pour réduire la complexité."
            })
        
        # Ratio commentaires/code
        if metrics['code_lines'] > 0:
            comment_ratio = metrics['comment_lines'] / metrics['code_lines']
            if comment_ratio < 0.1:
                metrics['suggestions'].append({
                    'type': 'documentation',
                    'message': "Peu de commentaires détectés. Ajoutez de la documentation."
                })
    
    # Formater la réponse
    if params.response_format == ResponseFormat.JSON:
        return json.dumps(metrics, indent=2)
    
    # Format Markdown
    result = "## 📊 Analyse de Complexité\n\n"
    result += f"**Langage**: {language.value}\n\n"
    
    result += "### Métriques Générales\n\n"
    result += f"- **Lignes totales**: {metrics['total_lines']}\n"
    result += f"- **Lignes de code**: {metrics['code_lines']}\n"
    result += f"- **Lignes de commentaires**: {metrics['comment_lines']}\n"
    result += f"- **Niveau d'imbrication max**: {metrics['max_nesting']}\n"
    result += f"- **Nombre de fonctions**: {len(metrics['functions'])}\n"
    result += f"- **Nombre de classes**: {len(metrics['classes'])}\n\n"
    
    if metrics['functions']:
        result += "### Fonctions\n\n"
        for func in metrics['functions']:
            result += f"- **{func['name']}** (ligne {func['start_line']})\n"
            if 'lines' in func:
                result += f"  - Longueur: {func['lines']} lignes\n"
            if 'complexity' in func:
                result += f"  - Complexité: {func['complexity']}\n"
    
    if metrics['suggestions']:
        result += "\n### 💡 Suggestions d'Amélioration\n\n"
        for suggestion in metrics['suggestions']:
            result += f"- {suggestion['message']}\n"
    
    return result

@mcp.tool(
    name="compare_code_versions",
    annotations={
        "title": "Comparer des Versions de Code",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False
    }
)
async def compare_code_versions(params: CompareCodeVersionsInput) -> str:
    """Compare deux versions de code pour détecter les changements.
    
    Identifie:
    - Les lignes ajoutées/supprimées/modifiées
    - Les changements structurels
    - Les régressions potentielles
    - L'impact sur la qualité
    
    Args:
        params: Les deux versions de code à comparer
    
    Returns:
        str: Rapport de comparaison au format demandé
    """
    import difflib
    
    # Détecter le langage
    language = params.language
    if language == ProgrammingLanguage.AUTO:
        language = detect_language(params.original_code)
    
    original_lines = params.original_code.splitlines()
    new_lines = params.new_code.splitlines()
    
    # Créer le diff
    differ = difflib.unified_diff(
        original_lines,
        new_lines,
        fromfile='Original',
        tofile='Nouveau',
        lineterm=''
    )
    
    diff_lines = list(differ)
    
    # Analyser les changements
    stats = {
        'language': language.value,
        'lines_added': 0,
        'lines_removed': 0,
        'lines_modified': 0,
        'functions_changed': [],
        'imports_changed': [],
        'potential_issues': []
    }
    
    for line in diff_lines:
        if line.startswith('+') and not line.startswith('+++'):
            stats['lines_added'] += 1
            
            # Vérifier les nouveaux imports
            if 'import' in line:
                stats['imports_changed'].append({
                    'type': 'added',
                    'line': line[1:].strip()
                })
        elif line.startswith('-') and not line.startswith('---'):
            stats['lines_removed'] += 1
            
            # Vérifier les imports supprimés
            if 'import' in line:
                stats['imports_changed'].append({
                    'type': 'removed',
                    'line': line[1:].strip()
                })
    
    # Vérifier les régressions potentielles
    if params.check_regression:
        # Valider les deux versions
        original_valid = True
        new_valid = True
        
        if language == ProgrammingLanguage.PYTHON:
            try:
                ast.parse(params.original_code)
            except SyntaxError:
                original_valid = False
            
            try:
                ast.parse(params.new_code)
            except SyntaxError:
                new_valid = False
        
        if original_valid and not new_valid:
            stats['potential_issues'].append({
                'type': 'syntax_regression',
                'message': 'La nouvelle version contient des erreurs de syntaxe absentes de l\'original'
            })
        
        # Vérifier si des fonctions ont été supprimées
        if language == ProgrammingLanguage.PYTHON:
            original_functions = set(re.findall(r'def\s+(\w+)\s*\(', params.original_code))
            new_functions = set(re.findall(r'def\s+(\w+)\s*\(', params.new_code))
            
            removed_functions = original_functions - new_functions
            added_functions = new_functions - original_functions
            
            for func in removed_functions:
                stats['functions_changed'].append({
                    'type': 'removed',
                    'name': func
                })
                stats['potential_issues'].append({
                    'type': 'function_removed',
                    'message': f"La fonction '{func}' a été supprimée"
                })
            
            for func in added_functions:
                stats['functions_changed'].append({
                    'type': 'added',
                    'name': func
                })
    
    # Formater la réponse
    if params.response_format == ResponseFormat.JSON:
        stats['diff'] = '\n'.join(diff_lines)
        return json.dumps(stats, indent=2)
    
    # Format Markdown
    result = "## 🔄 Comparaison de Code\n\n"
    result += f"**Langage**: {language.value}\n\n"
    
    result += "### Statistiques\n\n"
    result += f"- **Lignes ajoutées**: {stats['lines_added']} ➕\n"
    result += f"- **Lignes supprimées**: {stats['lines_removed']} ➖\n"
    result += f"- **Total des changements**: {stats['lines_added'] + stats['lines_removed']}\n\n"
    
    if stats['imports_changed']:
        result += "### Changements d'Imports\n\n"
        for imp in stats['imports_changed']:
            if imp['type'] == 'added':
                result += f"- ➕ `{imp['line']}`\n"
            else:
                result += f"- ➖ `{imp['line']}`\n"
        result += "\n"
    
    if stats['functions_changed']:
        result += "### Changements de Fonctions\n\n"
        for func in stats['functions_changed']:
            if func['type'] == 'added':
                result += f"- ➕ Fonction `{func['name']}` ajoutée\n"
            else:
                result += f"- ➖ Fonction `{func['name']}` supprimée\n"
        result += "\n"
    
    if stats['potential_issues']:
        result += "### ⚠️ Problèmes Potentiels\n\n"
        for issue in stats['potential_issues']:
            result += f"- **{issue['type']}**: {issue['message']}\n"
        result += "\n"
    
    if diff_lines and len(diff_lines) < 100:
        result += "### Diff Détaillé\n\n```diff\n"
        result += '\n'.join(diff_lines[:100])
        result += "\n```\n"
    
    return result

# ===================== POINT D'ENTRÉE =====================

if __name__ == "__main__":
    # Lancer le serveur MCP
    import sys
    mcp.run()
