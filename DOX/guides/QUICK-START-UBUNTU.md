# 🚀 Quick Start - Ubuntu Installation

**Phase 1: Claude AI Ultra-Automation**
**Platform:** Ubuntu 20.04+ / Debian 11+
**Time:** 5 minutes

---

## ✅ Prerequisites Check

```bash
# 1. Check Python version (need 3.11+)
python3 --version
# Expected: Python 3.11.x or higher

# 2. Check pip
pip3 --version

# 3. Verify you're in project root
pwd
# Should show: /path/to/monero-marketplace
```

---

## 📦 Installation (3 steps)

### Step 1: Install Python Dependencies

```bash
# Install anthropic + dependencies
pip3 install -r requirements.txt

# Or manually:
pip3 install anthropic aiohttp

# Verify installation
python3 -c "import anthropic; print('✅ anthropic installed')"
```

### Step 2: Set API Key

```bash
# Get your API key from: https://console.anthropic.com/

# Option A: Current session
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Option B: Permanent (recommended)
echo 'export ANTHROPIC_API_KEY="sk-ant-api03-..."' >> ~/.bashrc
source ~/.bashrc

# Verify
echo $ANTHROPIC_API_KEY
```

### Step 3: Test Installation

```bash
# Run automated tests
./scripts/test-automation-setup.sh

# Expected output:
# ✅ Passed: 18
# ❌ Failed: 0
```

---

## 🎯 First Security Audit (5 minutes)

### Quick Test

```bash
# Test on a small file
python3 scripts/ai/claude_security_analyzer.py \
  --file server/src/lib.rs \
  --mode deep

# Output: JSON report with security score
```

### Full Audit

```bash
# Run complete security audit
./scripts/audit-master.sh --full

# This will:
# ✅ Run Claude AI analysis (Sonnet + Haiku)
# ✅ Check security theatre patterns
# ✅ Verify Monero/Tor security
# ✅ Run Clippy + cargo-audit
# ✅ Execute tests
# ✅ Generate global security score

# Expected time: 5-10 minutes
```

### Quick Scan (30 seconds)

```bash
# Fast scan with Haiku
python3 scripts/ai/claude_quick_scan.py --dir server/src

# Output: Critical/High issues only
```

---

## 📊 Check Results

```bash
# View latest report
jq . $(ls -t docs/security-reports/audit-master-*.json | head -1)

# Get global score
jq -r '.global_score' $(ls -t docs/security-reports/audit-master-*.json | head -1)

# Count critical issues
jq '[.reports[].issues.critical | length] | add // 0' docs/security-reports/claude-*.json 2>/dev/null
```

---

## 🔄 CI/CD Setup (GitHub Actions)

### Add Secret to GitHub

1. Go to: `https://github.com/YOUR_USERNAME/monero-marketplace/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `ANTHROPIC_API_KEY`
4. Value: `sk-ant-api03-...`
5. Save

### Test Workflow

```bash
# Commit Phase 1 files
git add .
git commit -m "feat: Add Claude AI Ultra-Automation (Phase 1)"

# Push to trigger workflows
git push

# Check Actions tab on GitHub
# ✅ claude-security-review.yml will run on next PR
# ✅ claude-daily-scan.yml will run daily at 2 AM UTC
```

---

## 🛠️ Common Ubuntu Tasks

### Install System Dependencies (if needed)

```bash
# Update package list
sudo apt update

# Install Python 3.11 (if not present)
sudo apt install python3.11 python3.11-venv python3-pip

# Install jq (for JSON parsing)
sudo apt install jq

# Install git (if not present)
sudo apt install git
```

### Create Virtual Environment (optional but recommended)

```bash
# Create venv
python3 -m venv venv

# Activate
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Use scripts
python scripts/ai/claude_security_analyzer.py --help

# Deactivate when done
deactivate
```

---

## 📝 Daily Workflow

### Pre-Commit Workflow

```bash
# 1. Make changes to .rs files
vim server/src/handlers/escrow.rs

# 2. Quick scan before commit
python3 scripts/ai/claude_quick_scan.py --changed-files-only

# 3. If issues found, fix them

# 4. Commit
git add .
git commit -m "fix: Address security issues in escrow handler"
```

### Weekly Audit

```bash
# Every Monday (or on-demand)
./scripts/audit-master.sh --full

# Review report
cat docs/security-reports/audit-master-*.json | jq '.global_score'

# Aim for 85+ score
```

---

## 🐛 Troubleshooting

### Issue: `anthropic` not found

```bash
# Verify installation
pip3 list | grep anthropic

# Reinstall if needed
pip3 install --upgrade anthropic

# Check Python path
which python3
```

### Issue: API key not set

```bash
# Check variable
echo $ANTHROPIC_API_KEY

# Set temporarily
export ANTHROPIC_API_KEY="sk-ant-..."

# Set permanently
echo 'export ANTHROPIC_API_KEY="sk-ant-..."' >> ~/.bashrc
source ~/.bashrc
```

### Issue: Permission denied

```bash
# Make scripts executable
chmod +x scripts/audit-master.sh
chmod +x scripts/ai/*.py
chmod +x scripts/test-automation-setup.sh
```

### Issue: Bash script errors

```bash
# Check bash version (need 4.0+)
bash --version

# Test syntax
bash -n scripts/audit-master.sh

# Run with verbose mode
bash -x scripts/audit-master.sh --quick
```

---

## 📚 Next Steps

1. **Read Documentation**
   - [ULTRA-AUTOMATION-GUIDE.md](docs/ULTRA-AUTOMATION-GUIDE.md) - Complete guide
   - [scripts/ai/README.md](scripts/ai/README.md) - AI tools reference

2. **Run First Audit**
   ```bash
   ./scripts/audit-master.sh --full
   ```

3. **Integrate with Git Hooks** (optional)
   ```bash
   # Add to .git/hooks/pre-commit
   #!/bin/bash
   python3 scripts/ai/claude_quick_scan.py --changed-files-only || exit 1
   ```

4. **Monitor GitHub Actions**
   - Check PR reviews
   - Review daily scan reports
   - Address issues promptly

---

## 💰 Cost Tracking

### Check Current Month Usage

```bash
# Count API calls (approximate)
ls docs/security-reports/claude-*.json | wc -l

# Estimate cost
# Sonnet: ~$0.05-0.10 per file
# Haiku: ~$0.002 per file
```

### Optimize Costs

```bash
# Use Haiku for quick checks
python3 scripts/ai/claude_quick_scan.py --dir server/src

# Use Sonnet only for deep analysis
python3 scripts/ai/claude_security_analyzer.py \
  --file critical_file.rs \
  --mode deep
```

---

## ✅ Success Checklist

- [ ] Python 3.11+ installed
- [ ] `pip install -r requirements.txt` done
- [ ] `ANTHROPIC_API_KEY` set
- [ ] `./scripts/test-automation-setup.sh` passes (18/18)
- [ ] First audit completed: `./scripts/audit-master.sh --full`
- [ ] Global security score ≥ 70
- [ ] GitHub secret added
- [ ] CI/CD workflows tested

---

## 🆘 Getting Help

- **Documentation:** [ULTRA-AUTOMATION-GUIDE.md](docs/ULTRA-AUTOMATION-GUIDE.md)
- **GitHub Issues:** Create issue with `ubuntu` label
- **Test Output:** Share `./scripts/test-automation-setup.sh` output

---

**Platform:** Ubuntu 20.04+ / Debian 11+
**Last Updated:** 2025-10-22
**Status:** Production Ready ✅
