#!/usr/bin/env python3
"""
Claude Security Analyzer - Sonnet 4.5 Deep Analysis
AI-powered security analysis for Monero Marketplace Rust code

Usage:
    python claude_security_analyzer.py --file path/to/file.rs
    python claude_security_analyzer.py --dir server/src --mode deep
    python claude_security_analyzer.py --changed-files-only --post-to-pr
"""

import os
import sys
import json
import asyncio
import argparse
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict

try:
    import anthropic
except ImportError:
    print("[ERROR] anthropic package not installed")
    print("Install with: pip install anthropic")
    sys.exit(1)


@dataclass
class SecurityIssue:
    """Représente une issue de sécurité détectée"""
    line: int
    issue: str
    severity: str  # CRITICAL, HIGH, MEDIUM, LOW
    explanation: str
    fix: str
    category: str  # tor_leak, key_exposure, race_condition, etc.


@dataclass
class AnalysisReport:
    """Rapport d'analyse de sécurité"""
    file_path: str
    summary: str
    critical: List[SecurityIssue]
    high: List[SecurityIssue]
    medium: List[SecurityIssue]
    low: List[SecurityIssue]
    best_practices: List[str]
    security_score: int  # 0-100
    formal_verification_needed: List[str]
    thinking_process: str  # Claude's reasoning


class ClaudeSecurityAnalyzer:
    """
    Analyseur de sécurité alimenté par Claude Sonnet 4.5
    Optimisé pour détecter les vulnérabilités dans le code Rust
    """

    MODELS = {
        'deep': 'claude-sonnet-4-5-20250929',      # Analyse approfondie
        'quick': 'claude-3-5-haiku-20250219',      # Scans rapides
    }

    SECURITY_PATTERNS = {
        'tor_leaks': [
            r'\.onion',
            r'tracing::.*onion',
            r'log.*onion',
        ],
        'key_exposure': [
            r'view_key.*to_string',
            r'spend_key.*format',
            r'private_key.*println',
        ],
        'rpc_unsafe': [
            r'reqwest::.*without.*proxy',
            r'tcp.*connect.*18082',
        ],
        'forbidden_patterns': [
            r'\.unwrap\(\)',
            r'println!',
            r'dbg!',
            r'todo!',
            r'unimplemented!',
        ]
    }

    def __init__(self, api_key: Optional[str] = None):
        """
        Initialise l'analyseur Claude

        Args:
            api_key: Clé API Anthropic (ou via ANTHROPIC_API_KEY env var)
        """
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")

        if not self.api_key:
            raise ValueError(
                "ANTHROPIC_API_KEY not found. "
                "Set it via environment variable or pass as argument."
            )

        self.client = anthropic.Anthropic(api_key=self.api_key)
        self.reports: List[AnalysisReport] = []

    async def analyze_file(
        self,
        file_path: Path,
        mode: str = 'deep'
    ) -> AnalysisReport:
        """
        Analyse un fichier Rust avec Claude

        Args:
            file_path: Chemin vers le fichier .rs
            mode: 'deep' (Sonnet 4.5) ou 'quick' (Haiku)

        Returns:
            AnalysisReport avec les vulnérabilités détectées
        """
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        code = file_path.read_text(encoding='utf-8')
        model = self.MODELS.get(mode, self.MODELS['deep'])

        print(f"🔍 Analyzing {file_path.name} with {model}...")

        # Prompt optimisé pour Claude avec thinking mode
        prompt = self._build_security_prompt(str(file_path), code)

        try:
            response = await asyncio.to_thread(
                self.client.messages.create,
                model=model,
                max_tokens=8000 if mode == 'deep' else 4000,
                thinking={
                    "type": "enabled",
                    "budget_tokens": 5000  # Claude réfléchit avant de répondre
                } if mode == 'deep' else None,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )

            # Extraction du raisonnement et de la réponse
            thinking_content = ""
            analysis_content = ""

            for block in response.content:
                if block.type == "thinking":
                    thinking_content = block.thinking
                elif block.type == "text":
                    analysis_content = block.text

            # Parse la réponse JSON de Claude
            report = self._parse_claude_response(
                str(file_path),
                analysis_content,
                thinking_content
            )

            self.reports.append(report)
            return report

        except anthropic.APIError as e:
            print(f"❌ Claude API Error: {e}")
            raise

    def _build_security_prompt(self, file_path: str, code: str) -> str:
        """Construit le prompt optimisé pour Claude"""

        return f"""
<task>
Analyse de sécurité approfondie du module Rust suivant pour le **Monero Marketplace**.
Tu es un expert en sécurité Rust avec expertise en Monero, Tor, et cryptographie.
Utilise tes capacités de raisonnement pour détecter les vulnérabilités subtiles.
</task>

<security_focus>
**CRITIQUE - Sécurité Monero/Tor :**
1. 🧅 **Tor Leaks** : Adresses .onion dans logs/errors/tracing
2. 🔌 **RPC Bypass** : Appels non-proxifiés via SOCKS5 (127.0.0.1:9050)
3. 🔑 **Key Exposure** : View/spend keys dans logs/format/debug/Display
4. ⏱️ **Timing Attacks** : Operations multisig avec timing observable
5. 💰 **Amount Leaks** : Montants XMR exposés (privacy violation)

**Logique Escrow Multisig (2-of-3) :**
6. 🏁 **Race Conditions** : State transitions concurrentes
7. 🔢 **Integer Overflow** : Montants XMR (picomonero = u64)
8. ✅ **Validation** : multisig_info insuffisamment validé
9. 🔒 **Deadlocks** : Arc<Mutex<>> mal utilisé
10. 📈 **Monotonie** : État "signé" doit rester signé (invariant)

**Error Handling Rust :**
11. ⚠️ **.unwrap()** : Cachés ou sans justification
12. 🔥 **Panics** : Non documentés ou sans #[allow]
13. 📝 **Error Messages** : Données sensibles exposées
14. ↩️ **Result Propagation** : ? manquant ou mal utilisé

**Patterns Interdits (Security Theatre) :**
15. 🚫 println!/dbg!/eprintln! en production
16. 🚫 todo!/unimplemented!/unreachable!
17. 🚫 Magic numbers sans constantes
18. 🚫 Hardcoded credentials/secrets/keys
</security_focus>

<code_to_analyze>
**File:** {file_path}

```rust
{code}
```
</code_to_analyze>

<output_format>
Réponds UNIQUEMENT avec ce JSON (pas de markdown, pas d'explication avant/après) :

{{
    "summary": "Résumé exécutif en 2-3 phrases",
    "critical": [
        {{
            "line": 42,
            "issue": "Titre court de l'issue",
            "severity": "CRITICAL",
            "explanation": "Pourquoi c'est critique + impact exact",
            "fix": "Code Rust exact pour corriger (pas de pseudo-code)",
            "category": "tor_leak|key_exposure|race_condition|overflow|..."
        }}
    ],
    "high": [...],
    "medium": [...],
    "low": [...],
    "best_practices": [
        "Recommandation 1: ...",
        "Recommandation 2: ..."
    ],
    "security_score": 85,
    "formal_verification_needed": [
        "fonction_critique_1",
        "fonction_critique_2"
    ]
}}
</output_format>

<instructions>
1. **Réfléchis étape par étape** (thinking mode activé)
2. Lis TOUT le code avant de conclure
3. Cherche les vulnérabilités SUBTILES (pas juste les patterns évidents)
4. Priorise CRITICAL > HIGH > MEDIUM > LOW
5. Pour chaque issue : LINE EXACTE + FIX CONCRET
6. Si le code est sûr, dis-le (pas de faux positifs)
</instructions>

**Commence ton analyse maintenant.**
"""

    def _parse_claude_response(
        self,
        file_path: str,
        response_text: str,
        thinking: str
    ) -> AnalysisReport:
        """Parse la réponse JSON de Claude"""

        try:
            # Extraction du JSON (Claude peut ajouter du texte avant/après)
            json_start = response_text.find('{')
            json_end = response_text.rfind('}') + 1

            if json_start == -1 or json_end == 0:
                raise ValueError("No JSON found in Claude's response")

            json_str = response_text[json_start:json_end]
            data = json.loads(json_str)

            # Conversion des issues en SecurityIssue objects
            def parse_issues(issues_list: List[dict]) -> List[SecurityIssue]:
                return [
                    SecurityIssue(
                        line=issue['line'],
                        issue=issue['issue'],
                        severity=issue['severity'],
                        explanation=issue['explanation'],
                        fix=issue['fix'],
                        category=issue.get('category', 'unknown')
                    )
                    for issue in issues_list
                ]

            return AnalysisReport(
                file_path=file_path,
                summary=data['summary'],
                critical=parse_issues(data.get('critical', [])),
                high=parse_issues(data.get('high', [])),
                medium=parse_issues(data.get('medium', [])),
                low=parse_issues(data.get('low', [])),
                best_practices=data.get('best_practices', []),
                security_score=data.get('security_score', 0),
                formal_verification_needed=data.get('formal_verification_needed', []),
                thinking_process=thinking
            )

        except (json.JSONDecodeError, KeyError, ValueError) as e:
            print(f"❌ Error parsing Claude response: {e}")
            print(f"Response: {response_text[:500]}...")

            # Fallback: rapport vide
            return AnalysisReport(
                file_path=file_path,
                summary="Error parsing Claude response",
                critical=[], high=[], medium=[], low=[],
                best_practices=[],
                security_score=0,
                formal_verification_needed=[],
                thinking_process=thinking
            )

    def print_report(self, report: AnalysisReport) -> None:
        """Affiche un rapport de manière lisible"""

        print(f"\n{'='*80}")
        print(f"📄 File: {report.file_path}")
        print(f"🛡️ Security Score: {report.security_score}/100")
        print(f"{'='*80}\n")

        print(f"📝 Summary:")
        print(f"   {report.summary}\n")

        # Affichage par sévérité
        for severity, issues, emoji in [
            ('CRITICAL', report.critical, '🚨'),
            ('HIGH', report.high, '⚠️'),
            ('MEDIUM', report.medium, '⚡'),
            ('LOW', report.low, 'ℹ️'),
        ]:
            if issues:
                print(f"{emoji} {severity} Issues ({len(issues)}):")
                for issue in issues:
                    print(f"   Line {issue.line}: {issue.issue}")
                    print(f"      Category: {issue.category}")
                    print(f"      → {issue.explanation}")
                    print(f"      Fix: {issue.fix}\n")

        if report.best_practices:
            print(f"✅ Best Practices:")
            for bp in report.best_practices:
                print(f"   • {bp}")
            print()

        if report.formal_verification_needed:
            print(f"🔬 Formal Verification Recommended:")
            for func in report.formal_verification_needed:
                print(f"   • {func}")
            print()

    def export_json(self, output_path: Path) -> None:
        """Exporte tous les rapports en JSON"""

        data = {
            'timestamp': '2025-10-22T00:00:00Z',
            'analyzer': 'Claude Sonnet 4.5',
            'reports': [
                {
                    'file': r.file_path,
                    'summary': r.summary,
                    'security_score': r.security_score,
                    'critical_count': len(r.critical),
                    'high_count': len(r.high),
                    'medium_count': len(r.medium),
                    'low_count': len(r.low),
                    'issues': {
                        'critical': [asdict(i) for i in r.critical],
                        'high': [asdict(i) for i in r.high],
                        'medium': [asdict(i) for i in r.medium],
                        'low': [asdict(i) for i in r.low],
                    },
                    'best_practices': r.best_practices,
                    'formal_verification_needed': r.formal_verification_needed,
                }
                for r in self.reports
            ],
            'global_score': self._calculate_global_score(),
        }

        output_path.write_text(json.dumps(data, indent=2), encoding='utf-8')
        print(f"✅ Report exported to {output_path}")

    def _calculate_global_score(self) -> int:
        """Calcule le score global de sécurité"""

        if not self.reports:
            return 0

        total_score = sum(r.security_score for r in self.reports)
        avg_score = total_score / len(self.reports)

        # Pénalités pour issues critiques
        total_critical = sum(len(r.critical) for r in self.reports)
        total_high = sum(len(r.high) for r in self.reports)

        penalty = (total_critical * 10) + (total_high * 5)

        return max(0, int(avg_score - penalty))


async def main():
    """Point d'entrée CLI"""

    parser = argparse.ArgumentParser(
        description='Claude AI Security Analyzer for Monero Marketplace'
    )
    parser.add_argument(
        '--file',
        type=Path,
        help='Single Rust file to analyze'
    )
    parser.add_argument(
        '--dir',
        type=Path,
        help='Directory to analyze (recursive)'
    )
    parser.add_argument(
        '--mode',
        choices=['deep', 'quick'],
        default='deep',
        help='Analysis mode (deep=Sonnet 4.5, quick=Haiku)'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('claude-report.json'),
        help='Output JSON report path'
    )
    parser.add_argument(
        '--changed-files-only',
        action='store_true',
        help='Only analyze git changed files'
    )

    args = parser.parse_args()

    # Validation
    if not args.file and not args.dir and not args.changed_files_only:
        parser.error("Specify --file, --dir, or --changed-files-only")

    # Initialisation de l'analyseur
    try:
        analyzer = ClaudeSecurityAnalyzer()
    except ValueError as e:
        print(f"❌ {e}")
        sys.exit(1)

    # Collecte des fichiers à analyser
    files_to_analyze: List[Path] = []

    if args.file:
        files_to_analyze.append(args.file)

    if args.dir:
        files_to_analyze.extend(args.dir.rglob('*.rs'))

    if args.changed_files_only:
        # Git changed files
        import subprocess
        result = subprocess.run(
            ['git', 'diff', '--name-only', 'HEAD'],
            capture_output=True,
            text=True
        )
        changed_files = [
            Path(f) for f in result.stdout.strip().split('\n')
            if f.endswith('.rs')
        ]
        files_to_analyze.extend(changed_files)

    # Suppression des doublons
    files_to_analyze = list(set(files_to_analyze))

    print(f"🔍 Analyzing {len(files_to_analyze)} files with Claude {args.mode} mode\n")

    # Analyse de chaque fichier
    for file_path in files_to_analyze:
        try:
            report = await analyzer.analyze_file(file_path, mode=args.mode)
            analyzer.print_report(report)
        except Exception as e:
            print(f"❌ Error analyzing {file_path}: {e}")

    # Export du rapport global
    analyzer.export_json(args.output)

    # Score global
    global_score = analyzer._calculate_global_score()
    print(f"\n{'='*80}")
    print(f"🎯 Global Security Score: {global_score}/100")
    print(f"{'='*80}\n")

    # Exit code basé sur le score
    if global_score < 70:
        print("❌ Security score too low - FAIL")
        sys.exit(1)
    else:
        print("✅ Security check passed")
        sys.exit(0)


if __name__ == '__main__':
    asyncio.run(main())
