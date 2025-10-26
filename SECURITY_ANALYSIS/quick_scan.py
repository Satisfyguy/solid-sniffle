#!/usr/bin/env python3
"""
Quick Security Scanner - Fast security checks using pattern matching
"""

import re
import sys
import os
from pathlib import Path
from typing import List, Dict, Pattern, Optional, Tuple
from dataclasses import dataclass
from rich.console import Console
import logging

# Ajout du répertoire parent au path pour les imports
sys.path.append(str(Path(__file__).parent.absolute()))

# Configuration du logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Détermination du répertoire racine du projet
PROJECT_ROOT = Path(__file__).parent.parent.absolute()

# Configuration par défaut
class Settings:
    RUST_EXTENSIONS = ['.rs']
    IGNORE_DIRS = ['target', '.git', 'venv', '__pycache__']
    LOG_LEVEL = 'INFO'

settings = Settings()

# Configure logging
logging.basicConfig(level=getattr(logging, settings.LOG_LEVEL))
logger = logging.getLogger(__name__)

console = Console()

@dataclass
class SecurityPattern:
    name: str
    pattern: str
    severity: str
    description: str
    recommendation: str
    compiled: Pattern = None
    
    def __post_init__(self):
        self.compiled = re.compile(self.pattern, re.MULTILINE)

class QuickScanner:
    """Fast security scanner using pattern matching"""
    
    def __init__(self, root_dir: Optional[Path] = None):
        self.root_dir = root_dir or PROJECT_ROOT
        self.patterns = self._load_security_patterns()
        self.console = Console()
        
        # Vérification du répertoire racine
        if not self.root_dir.exists():
            raise FileNotFoundError(f"Répertoire non trouvé: {self.root_dir}")
            
        logger.info(f"Analyse du répertoire: {self.root_dir}")
    
    def _load_security_patterns(self) -> List[SecurityPattern]:
        """Load security patterns to scan for"""
        return [
            SecurityPattern(
                name="Hardcoded Credentials",
                pattern=r'(?i)(password|secret|api[_-]?key|token|bearer)\s*[=:]\s*["\'][^\s"\']+["\']',
                severity="HIGH",
                description="Hardcoded credentials in source code",
                recommendation="Use environment variables or a secure secret management system"
            ),
            SecurityPattern(
                name="Dangerous Functions",
                pattern=r'(?i)(system\(|exec\(|eval\(|unserialize\()',
                severity="CRITICAL",
                description="Potentially dangerous function call",
                recommendation="Use safer alternatives and validate all inputs"
            ),
            # Add more patterns as needed
        ]
    
    def _is_valid_file(self, file_path: Path) -> bool:
        """Vérifie si un fichier doit être analysé"""
        # Ignore les fichiers dans les dossiers à ignorer
        if any(part in settings.IGNORE_DIRS for part in file_path.parts):
            return False
            
        # Vérifie l'extension du fichier
        if file_path.suffix.lower() not in settings.RUST_EXTENSIONS:
            return False
            
        return True

    def find_rust_files(self) -> List[Path]:
        """Trouve tous les fichiers Rust dans le répertoire racine"""
        rust_files = []
        for ext in settings.RUST_EXTENSIONS:
            rust_files.extend(self.root_dir.rglob(f'*{ext}'))
        return [f for f in rust_files if self._is_valid_file(f)]

    async def scan_file(self, file_path: Path) -> List[Dict]:
        """Scan a single file for security issues"""
        issues = []
        
        try:
            # Utilisation du chemin absolu pour éviter les problèmes
            abs_path = file_path.absolute()
            logger.debug(f"Analyse du fichier: {abs_path}")
            content = abs_path.read_text(encoding='utf-8')
            
            for pattern in self.patterns:
                for match in pattern.compiled.finditer(content):
                    # Get line number
                    line_num = content[:match.start()].count('\n') + 1
                    
                    # Get line content
                    lines = content.split('\n')
                    line_content = lines[line_num - 1] if line_num <= len(lines) else ""
                    
                    issues.append({
                        'file': str(file_path),
                        'line': line_num,
                        'pattern': pattern.name,
                        'severity': pattern.severity,
                        'match': match.group(0),
                        'line_content': line_content.strip(),
                        'description': pattern.description,
                        'recommendation': pattern.recommendation
                    })
                    
        except Exception as e:
            logger.error(f"Error scanning {file_path}: {str(e)}")
            
        return issues

async def main():
    """Point d'entrée principal pour le scan rapide"""
    import argparse
    
    # Configuration des arguments en ligne de commande
    parser = argparse.ArgumentParser(description='Scanner de sécurité pour le code Rust')
    parser.add_argument('--file', type=str, help='Fichier spécifique à analyser')
    parser.add_argument('--dir', type=str, help='Répertoire à analyser', default='.')
    parser.add_argument('--verbose', '-v', action='store_true', help='Afficher plus de détails')
    args = parser.parse_args()
    
    # Configuration du niveau de log
    if args.verbose:
        logger.setLevel(logging.DEBUG)
    else:
        logger.setLevel(logging.INFO)
    
    # Initialisation du scanner
    scanner = QuickScanner(root_dir=Path(args.dir).resolve())
    console = Console()
    
    # Recherche des fichiers à analyser
    if args.file:
        target_file = Path(args.file).resolve()
        if not target_file.exists():
            console.print(f"[red]Erreur: Le fichier {target_file} n'existe pas[/red]")
            return 1
        rust_files = [target_file]
    else:
        console.print(f"[bold]Recherche des fichiers Rust dans {scanner.root_dir}...[/bold]")
        rust_files = scanner.find_rust_files()
        
        if not rust_files:
            console.print("[yellow]Aucun fichier Rust trouvé à analyser[/yellow]")
            return 0
            
        console.print(f"[green]Trouvé {len(rust_files)} fichiers Rust à analyser[/green]")
    
    if not rust_files:
        console.print("[yellow]No Rust files found to scan[/yellow]")
        return
    
    all_issues = []
    
    # Scan each file
    for file_path in rust_files:
        issues = await scanner.scan_file(file_path)
        all_issues.extend(issues)
    
    # Print results
    if not all_issues:
        console.print("\n[bold green]✅ Aucune vulnérabilité de sécurité trouvée ![/bold green]")
        return 0
    
    # Affichage du résumé
    console.print("\n[bold]Résumé de l'analyse:[/bold]")
    console.print(f"- Fichiers analysés: {len(rust_files)}")
    console.print(f"- Problèmes trouvés: {len(all_issues)}")
    
    # Retourne le code de sortie approprié
    return 1 if any(issue['severity'] in ['CRITICAL', 'HIGH'] for issue in all_issues) else 0
    
    # Group by severity
    by_severity = {}
    for issue in all_issues:
        by_severity.setdefault(issue['severity'], []).append(issue)
    
    # Print summary
    console.print("\n[bold]Scan Results:[/bold]")
    for severity, issues in sorted(by_severity.items()):
        color = {
            "CRITICAL": "red",
            "HIGH": "bright_red",
            "MEDIUM": "yellow",
            "LOW": "blue"
        }.get(severity, "white")
        
        console.print(f"\n[{color}]{severity} ({len(issues)})[/{color}]")
        for issue in issues[:5]:  # Show first 5 of each
            console.print(f"  {issue['file']}:{issue['line']} - {issue['pattern']}")
        
        if len(issues) > 5:
            console.print(f"  ... and {len(issues) - 5} more")

if __name__ == "__main__":
    import asyncio
    import sys
    
    try:
        sys.exit(asyncio.run(main()))
    except KeyboardInterrupt:
        print("\nAnalyse interrompue par l'utilisateur")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Erreur lors de l'analyse: {str(e)}")
        sys.exit(1)
