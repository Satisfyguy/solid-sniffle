#!/usr/bin/env python3
"""
Claude Quick Scan - Haiku 3.5 for Rapid Security Checks
Lightweight security scanner pour continuous monitoring

Usage:
    python claude_quick_scan.py --dir server/src
    python claude_quick_scan.py --watch  # Continuous monitoring
"""

import os
import sys
import json
import time
import asyncio
import argparse
from pathlib import Path
from typing import List, Dict
from dataclasses import dataclass

try:
    import anthropic
except ImportError:
    print("[ERROR] anthropic package not installed")
    print("Install with: pip install anthropic")
    sys.exit(1)


@dataclass
class QuickScanResult:
    """Résultat de scan rapide"""
    file_path: str
    issues_found: int
    critical_count: int
    high_count: int
    patterns_detected: List[str]
    requires_deep_analysis: bool


class ClaudeQuickScanner:
    """
    Scanner rapide avec Claude Haiku 3.5
    Optimisé pour la vitesse et le coût minimal
    """

    MODEL = 'claude-3-5-haiku-20250219'  # Rapide et économique

    # Patterns critiques à détecter rapidement
    CRITICAL_PATTERNS = {
        'tor_leak': [
            '.onion',
            'tracing::info!.*onion',
            'println!.*onion'
        ],
        'key_exposure': [
            'view_key.*to_string',
            'spend_key.*println',
            'private_key.*format!'
        ],
        'forbidden': [
            r'\.unwrap\(\)',
            'println!',
            'dbg!',
            'todo!',
            'panic!'
        ]
    }

    def __init__(self, api_key: str = None):
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY required")

        self.client = anthropic.Anthropic(api_key=self.api_key)
        self.scan_results: List[QuickScanResult] = []

    async def quick_scan_file(self, file_path: Path) -> QuickScanResult:
        """
        Scan rapide d'un fichier (~2-3 secondes)

        Args:
            file_path: Fichier à scanner

        Returns:
            QuickScanResult
        """
        code = file_path.read_text(encoding='utf-8')

        # Prompt minimal pour Haiku (rapide + économique)
        prompt = f"""
Scan rapide de sécurité pour ce code Rust.

Détecte UNIQUEMENT :
1. .onion dans logs (CRITIQUE)
2. Keys exposées (CRITIQUE)
3. .unwrap() (HIGH)
4. println!/dbg! (MEDIUM)
5. todo!/panic! (HIGH)

Code:
```rust
{code[:2000]}
```

Réponds JSON :
{{
    "critical": ["issue1", "issue2"],
    "high": ["issue3"],
    "requires_deep_analysis": true/false
}}
"""

        try:
            response = await asyncio.to_thread(
                self.client.messages.create,
                model=self.MODEL,
                max_tokens=500,  # Minimal pour réponse rapide
                messages=[{"role": "user", "content": prompt}]
            )

            result_text = response.content[0].text

            # Parse la réponse
            json_start = result_text.find('{')
            json_end = result_text.rfind('}') + 1
            json_str = result_text[json_start:json_end]

            data = json.loads(json_str)

            critical = data.get('critical', [])
            high = data.get('high', [])

            result = QuickScanResult(
                file_path=str(file_path),
                issues_found=len(critical) + len(high),
                critical_count=len(critical),
                high_count=len(high),
                patterns_detected=critical + high,
                requires_deep_analysis=data.get('requires_deep_analysis', False)
            )

            self.scan_results.append(result)
            return result

        except Exception as e:
            print(f"⚠️ Error scanning {file_path}: {e}")
            return QuickScanResult(
                file_path=str(file_path),
                issues_found=0,
                critical_count=0,
                high_count=0,
                patterns_detected=[],
                requires_deep_analysis=False
            )

    async def batch_scan(self, files: List[Path]) -> List[QuickScanResult]:
        """Scan multiple files en parallèle"""

        print(f"⚡ Quick scanning {len(files)} files with Claude Haiku...")

        # Scan en parallèle pour la vitesse
        tasks = [self.quick_scan_file(f) for f in files]
        results = await asyncio.gather(*tasks)

        return results

    def print_summary(self):
        """Affiche le résumé des scans"""

        total_files = len(self.scan_results)
        total_issues = sum(r.issues_found for r in self.scan_results)
        total_critical = sum(r.critical_count for r in self.scan_results)
        deep_analysis_needed = [
            r for r in self.scan_results if r.requires_deep_analysis
        ]

        print(f"\n{'='*60}")
        print(f"⚡ Quick Scan Summary (Claude Haiku)")
        print(f"{'='*60}\n")
        print(f"📁 Files scanned: {total_files}")
        print(f"⚠️  Total issues: {total_issues}")
        print(f"🚨 Critical issues: {total_critical}")
        print(f"🔬 Deep analysis needed: {len(deep_analysis_needed)}\n")

        if deep_analysis_needed:
            print(f"Files requiring deep analysis:")
            for result in deep_analysis_needed:
                print(f"  • {result.file_path} ({result.critical_count} critical)")

        print(f"{'='*60}\n")

    async def watch_mode(self, directory: Path, interval: int = 60):
        """
        Mode surveillance continue
        Scan toutes les X secondes
        """

        print(f"👀 Watching {directory} (interval: {interval}s)")
        print(f"Press Ctrl+C to stop\n")

        try:
            while True:
                rust_files = list(directory.rglob('*.rs'))
                await self.batch_scan(rust_files)
                self.print_summary()

                # Reset pour le prochain scan
                self.scan_results = []

                print(f"⏳ Waiting {interval}s before next scan...")
                await asyncio.sleep(interval)

        except KeyboardInterrupt:
            print("\n✅ Stopped monitoring")


async def main():
    parser = argparse.ArgumentParser(
        description='Claude Quick Scanner (Haiku) for rapid security checks'
    )
    parser.add_argument(
        '--dir',
        type=Path,
        default=Path('server/src'),
        help='Directory to scan'
    )
    parser.add_argument(
        '--watch',
        action='store_true',
        help='Continuous monitoring mode'
    )
    parser.add_argument(
        '--interval',
        type=int,
        default=60,
        help='Scan interval in seconds (watch mode)'
    )
    parser.add_argument(
        '--output',
        type=Path,
        help='Output JSON report'
    )

    args = parser.parse_args()

    # Initialisation
    try:
        scanner = ClaudeQuickScanner()
    except ValueError as e:
        print(f"❌ {e}")
        sys.exit(1)

    # Mode surveillance ou scan unique
    if args.watch:
        await scanner.watch_mode(args.dir, args.interval)
    else:
        rust_files = list(args.dir.rglob('*.rs'))
        await scanner.batch_scan(rust_files)
        scanner.print_summary()

        # Export si demandé
        if args.output:
            data = {
                'scanner': 'Claude Haiku 3.5',
                'timestamp': time.strftime('%Y-%m-%dT%H:%M:%SZ'),
                'results': [
                    {
                        'file': r.file_path,
                        'issues': r.issues_found,
                        'critical': r.critical_count,
                        'deep_analysis_needed': r.requires_deep_analysis
                    }
                    for r in scanner.scan_results
                ]
            }
            args.output.write_text(json.dumps(data, indent=2))
            print(f"✅ Report saved to {args.output}")

    # Exit code
    total_critical = sum(r.critical_count for r in scanner.scan_results)
    sys.exit(1 if total_critical > 0 else 0)


if __name__ == '__main__':
    asyncio.run(main())
