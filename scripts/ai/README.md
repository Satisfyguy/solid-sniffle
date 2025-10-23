# Claude AI Security Analyzers

Automated security analysis powered by Claude AI (Anthropic).

## 🤖 Components

### 1. `claude_security_analyzer.py` - Deep Analysis (Sonnet 4.5)

**Purpose:** Comprehensive security review for critical code

**Features:**
- Deep reasoning with thinking mode (5000 tokens budget)
- Detects 18 categories of vulnerabilities
- Rust-specific analysis (Monero/Tor aware)
- Generates detailed JSON reports with fixes

**Usage:**
```bash
# Single file
python3 claude_security_analyzer.py --file server/src/handlers/escrow.rs --mode deep

# Directory
python3 claude_security_analyzer.py --dir server/src --mode deep

# Changed files only (Git)
python3 claude_security_analyzer.py --changed-files-only --mode deep
```

**Output:**
```json
{
  "file": "escrow.rs",
  "security_score": 85,
  "critical": [...],
  "high": [...],
  "best_practices": [...],
  "formal_verification_needed": [...]
}
```

### 2. `claude_quick_scan.py` - Rapid Scan (Haiku 3.5)

**Purpose:** Fast continuous monitoring

**Features:**
- 2-3 seconds per file (80% faster than Sonnet)
- Focuses on CRITICAL/HIGH severity only
- Watch mode for continuous monitoring
- 80% cheaper than Sonnet

**Usage:**
```bash
# Quick scan
python3 claude_quick_scan.py --dir server/src

# Continuous monitoring (every 60s)
python3 claude_quick_scan.py --watch --interval 60

# With JSON output
python3 claude_quick_scan.py --dir wallet/src --output quick-scan.json
```

## 📊 Comparison

| Feature | Sonnet 4.5 | Haiku 3.5 |
|---------|------------|-----------|
| **Speed** | 30-60s/file | 2-3s/file |
| **Cost** | $3/M tokens | $0.25/M tokens |
| **Depth** | Deep analysis | Surface scan |
| **Thinking** | ✅ Yes (5000 tokens) | ❌ No |
| **Use Case** | PR reviews, weekly audits | Pre-commit, continuous monitoring |

## 🔒 Security Patterns Detected

### Monero/Tor Specific
- `.onion` addresses in logs/errors
- Unproxied RPC calls (bypass Tor)
- View/spend keys in debug output
- Timing attacks on multisig operations
- Monero amounts exposed (privacy leak)

### Rust Code Quality
- `.unwrap()` without error handling
- `panic!` in production code
- Race conditions in `Arc<Mutex<>>`
- Integer overflow in XMR amounts
- Deadlocks potential

### Security Theatre
- `todo!`/`unimplemented!` in production
- `println!`/`dbg!` macros
- Magic numbers without constants
- Hardcoded credentials

## 🚀 Quick Start

### Prerequisites

```bash
# 1. Install dependencies
pip install anthropic aiohttp

# 2. Set API key
export ANTHROPIC_API_KEY="sk-ant-..."

# 3. Verify installation
python3 claude_security_analyzer.py --help
```

### Example Workflow

```bash
# 1. Quick pre-commit check (Haiku)
python3 claude_quick_scan.py --dir server/src

# 2. If issues found, deep analysis (Sonnet)
python3 claude_security_analyzer.py --file server/src/handlers/escrow.rs --mode deep

# 3. Review report
cat claude-report.json | jq '.reports[0].critical'
```

## 💰 Cost Estimation

### Typical Usage

```
Small codebase (50 files):
- Deep analysis: ~$1.50
- Quick scan: ~$0.10

Medium codebase (200 files):
- Deep analysis: ~$6.00
- Quick scan: ~$0.40

Large codebase (500 files):
- Deep analysis: ~$15.00
- Quick scan: ~$1.00
```

### Monthly Budget

```
10 PRs/month (Sonnet):     $3
Daily scans (Haiku):       $2.40
Weekly deep (Sonnet):      $2.40
TOTAL:                     ~$8/month
```

## 🔧 Configuration

### Environment Variables

```bash
# Required
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional
export CLAUDE_MAX_TOKENS=8000
export CLAUDE_THINKING_BUDGET=5000
```

### Custom Patterns

Edit `claude_security_analyzer.py`:

```python
SECURITY_PATTERNS = {
    'custom_pattern': [
        r'my_sensitive_function',
        r'custom_leak_pattern'
    ]
}
```

## 📖 Output Formats

### JSON Report (Detailed)

```json
{
  "file_path": "server/src/handlers/escrow.rs",
  "summary": "Module handles multisig escrow with 1 critical issue",
  "security_score": 85,
  "critical": [
    {
      "line": 142,
      "issue": "View key logged in debug trace",
      "severity": "CRITICAL",
      "category": "key_exposure",
      "explanation": "The view_key is exposed in tracing::debug!...",
      "fix": "tracing::debug!(\"Multisig validated\");"
    }
  ],
  "thinking_process": "First, I'll analyze the escrow handler..."
}
```

### Terminal Output (Human-Readable)

```
════════════════════════════════════════════════════════════
📄 File: server/src/handlers/escrow.rs
🛡️ Security Score: 85/100
════════════════════════════════════════════════════════════

📝 Summary:
   Module handles multisig escrow with 1 critical issue

🚨 CRITICAL Issues (1):
   Line 142: View key logged in debug trace
      Category: key_exposure
      → The view_key is exposed in tracing::debug! which could leak to logs
      Fix: tracing::debug!("Multisig validated");

⚠️  HIGH Issues (2):
   ...
```

## 🧪 Testing

```bash
# Test analyzer on sample file
python3 claude_security_analyzer.py --file ../test_sample.rs --mode deep

# Test quick scanner
python3 claude_quick_scan.py --dir ../../server/tests

# Validate JSON output
python3 -c "import json; json.load(open('claude-report.json'))"
```

## 🐛 Troubleshooting

### Rate Limit Errors

```bash
# Add retry logic or wait between requests
sleep 1 && python3 claude_security_analyzer.py ...
```

### Large Files Timeout

```bash
# Use quick scan for large files
python3 claude_quick_scan.py --file large_file.rs
```

### JSON Parse Errors

```bash
# Claude sometimes adds text before/after JSON
# The script handles this automatically
```

## 📚 Further Reading

- [ULTRA-AUTOMATION-GUIDE.md](../../docs/ULTRA-AUTOMATION-GUIDE.md) - Complete guide
- [Claude API Docs](https://docs.anthropic.com/claude/reference) - API reference
- [Anthropic Pricing](https://www.anthropic.com/pricing) - Cost calculator

---

**Powered by Claude Sonnet 4.5 & Haiku 3.5 - Anthropic AI**
