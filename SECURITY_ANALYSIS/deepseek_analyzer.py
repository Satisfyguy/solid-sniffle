#!/usr/bin/env python3
"""
DeepSeek Security Analyzer - DeepSeek-V3 (128K context)
Advanced security analysis for Rust codebases
"""

import asyncio
import json
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict, field
from enum import Enum
import httpx
from rich.console import Console
from rich.progress import Progress
import logging

from config import settings

# Configure logging
logging.basicConfig(level=getattr(logging, settings.LOG_LEVEL))
logger = logging.getLogger(__name__)

console = Console()

class Severity(str, Enum):
    CRITICAL = "CRITICAL"
    HIGH = "HIGH"
    MEDIUM = "MEDIUM"
    LOW = "LOW"
    INFO = "INFO"

@dataclass
class SecurityIssue:
    """Represents a security issue found during analysis"""
    file_path: str
    line: int
    code_snippet: str
    issue: str
    severity: Severity
    description: str
    recommendation: str
    cwe_id: Optional[str] = None
    owasp_category: Optional[str] = None

@dataclass
class AnalysisReport:
    """Container for security analysis results"""
    file_path: str
    issues: List[SecurityIssue] = field(default_factory=list)
    summary: Dict[str, int] = field(default_factory=dict)
    
    def add_issue(self, issue: SecurityIssue) -> None:
        """Add an issue to the report"""
        self.issues.append(issue)
        sev = issue.severity.value
        self.summary[sev] = self.summary.get(sev, 0) + 1

class DeepSeekAnalyzer:
    """Main analyzer class using DeepSeek-V3 API"""
    
    def __init__(self):
        self.client = httpx.AsyncClient(
            base_url=settings.DEEPSEEK_BASE_URL,
            headers={
                "Authorization": f"Bearer {settings.DEEPSEEK_API_KEY}",
                "Content-Type": "application/json"
            },
            timeout=settings.TIMEOUT
        )
        self.console = Console()
    
    async def analyze_code(self, file_path: Path) -> AnalysisReport:
        """Analyze a single Rust file"""
        report = AnalysisReport(file_path=str(file_path))
        
        try:
            code = file_path.read_text(encoding='utf-8')
            
            # Prepare the analysis prompt
            prompt = self._build_analysis_prompt(code)
            
            # Call DeepSeek API
            response = await self._call_deepseek_api(prompt)
            
            # Parse response and update report
            self._parse_response(response, report)
            
        except Exception as e:
            logger.error(f"Error analyzing {file_path}: {str(e)}")
            
        return report
    
    def _build_analysis_prompt(self, code: str) -> str:
        """Build the analysis prompt for DeepSeek"""
        return f"""
        [SYSTEM INSTRUCTIONS]
        You are a senior security engineer analyzing Rust code for vulnerabilities.
        Perform a thorough security analysis and report any issues found.
        
        [RULES]
        1. Focus on security issues, not style or performance
        2. Include code snippets for context
        3. Provide detailed remediation advice
        4. Rate severity using CVSS v3.1
        
        [CODE TO ANALYZE]
        ```rust
        {code}
        ```
        
        [OUTPUT FORMAT]
        Return a JSON array of issues with this structure:
        [
            {{
                "line": number,
                "severity": "CRITICAL|HIGH|MEDIUM|LOW|INFO",
                "issue": "short description",
                "description": "detailed explanation",
                "recommendation": "how to fix it",
                "cwe_id": "CWE-XXX"  // optional
            }}
        ]
        """
    
    async def _call_deepseek_api(self, prompt: str) -> Dict[str, Any]:
        """Make API call to DeepSeek"""
        payload = {
            "model": settings.DEEPSEEK_MODEL,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": settings.MAX_TOKENS,
            "temperature": settings.TEMPERATURE,
        }
        
        try:
            response = await self.client.post("/chat/completions", json=payload)
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            logger.error(f"API request failed: {e.response.text}")
            raise
    
    def _parse_response(self, response: Dict[str, Any], report: AnalysisReport) -> None:
        """Parse DeepSeek API response into report"""
        try:
            content = response['choices'][0]['message']['content']
            issues = json.loads(content)
            
            for issue in issues:
                security_issue = SecurityIssue(
                    file_path=report.file_path,
                    line=issue.get('line', 0),
                    code_snippet=issue.get('code_snippet', ''),
                    issue=issue['issue'],
                    severity=Severity(issue['severity']),
                    description=issue['description'],
                    recommendation=issue['recommendation'],
                    cwe_id=issue.get('cwe_id')
                )
                report.add_issue(security_issue)
                
        except (json.JSONDecodeError, KeyError) as e:
            logger.error(f"Failed to parse API response: {e}")

async def main():
    """Main entry point"""
    if not settings.DEEPSEEK_API_KEY:
        console.print("[red]Error: DEEPSEEK_API_KEY environment variable not set[/red]")
        return
    
    analyzer = DeepSeekAnalyzer()
    
    # Example: Analyze a test file
    test_file = Path("src/main.rs")  # Update with actual path
    if test_file.exists():
        report = await analyzer.analyze_code(test_file)
        
        # Print summary
        console.print("\n[bold]Security Analysis Summary:[/bold]")
        for severity, count in report.summary.items():
            console.print(f"- {severity}: {count}")
    else:
        console.print(f"[yellow]File not found: {test_file}[/yellow]")

if __name__ == "__main__":
    asyncio.run(main())
