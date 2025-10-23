---
description: Execute Protocole Alpha Terminal - Vérification anti-hallucination et mise à jour PLAN-COMPLET.md après commit
---

# Slash Command: /alpha-terminal

Execute the **Protocole Alpha Terminal** workflow:

1. ✅ Identify last commit (git log)
2. ✅ Anti-hallucination verification (line-by-line code reading)
3. ✅ Production-ready evaluation (98/100 scorecard)
4. ✅ Metrics update (LOC, endpoints, tests)
5. ✅ Update PLAN-COMPLET.md (version, milestones, changelog)
6. ✅ Identify immediate actions (TACHES-IMMEDIATES.md)
7. ✅ Create documentation commit

## Full Protocol Documentation

See: [PROTOCOLE-ALPHA-TERMINAL.md](../../PROTOCOLE-ALPHA-TERMINAL.md)

## Your Task

Execute all 7 steps of the Protocole Alpha Terminal systematically:

### Step 1: Identify Last Commit
```bash
git log --oneline -1
git show --stat HEAD
```
Report: commit hash, message, files changed

### Step 2: Anti-Hallucination Verification
For each claim in the commit message:
- Read the actual file
- Grep for key functions
- Count occurrences
- Validate syntax

**Standard checks (MANDATORY):**
```bash
# 1. Zero unwrap in production
grep -rn "\.unwrap()\|\.expect(" server/src/handlers/*.rs | wc -l
# Expected: 0

# 2. E2E tests count
grep -r "#\[test\]\|#\[tokio::test\]" server/tests/ | wc -l

# 3. Security theatre
./scripts/check-security-theatre.sh

# 4. Trace integration chain (if applicable)
grep -n "function_name" server/src/**/*.rs
```

### Step 3: Production-Ready Evaluation
Activate production-ready skill and fill scorecard:

| Category | Score /100 | Code Proof | Issues |
|----------|------------|------------|--------|
| Security Hardening | | | |
| Input Validation | | | |
| Error Handling | | | |
| Authorization | | | |
| Integration | | | |
| State Management | | | |
| Database Security | | | |
| Code Quality | | | |

**Overall Score:** XX/100

### Step 4: Update Metrics
```bash
# LOC
find server/src -name "*.rs" -type f | xargs wc -l | tail -1

# Rust files
find . -name "*.rs" -type f | wc -l

# Tests
grep -r "#\[test\]\|#\[tokio::test\]" server/ | wc -l

# API endpoints
grep -r "#\[get\]\|#\[post\]\|#\[put\]\|#\[delete\]" server/src/handlers/ | wc -l

# Security theatre
./scripts/check-security-theatre.sh 2>&1 | grep "issues"
```

Create metrics table.

### Step 5: Update PLAN-COMPLET.md
Edit the following sections:

1. **Header (lines 1-15):** Update version, date, milestone %
2. **New section (after line ~80):** Add "NOUVEAUTÉS (2025-10-YY)" with proofs
3. **Milestones (line ~320):** Mark completed tasks
4. **Anti-hallucination section (before ## Next Review):** Add verification report
5. **Changelog (before EOF):** Add new version entry

### Step 6: Identify Immediate Actions
Create or update TACHES-IMMEDIATES.md with:
- 🔴 CRITICAL tasks (blockers)
- 🟡 HIGH tasks (quick wins)
- 🟢 NORMAL tasks (improvements)

For each task:
- Estimated time
- Impact on milestone
- Exact commands
- Files to modify
- Success criteria

### Step 7: Git Commit Documentation
```bash
git add PLAN-COMPLET.md
git commit --no-verify -m "docs: Update PLAN-COMPLET.md - Milestone X.Y → ZZ% (Protocole Alpha Terminal)

Version X.Y+1 - Vérification anti-hallucination complète

Changements principaux:
- Mise à jour statut: YY% → ZZ%
- Ajout section vérification anti-hallucination
- Metrics actualisées: X,XXX LOC, XX endpoints
- Documentation commit XXXXXXX
- Score production-ready: XX/100

Nouveautés vérifiées (2025-10-YY):
✅ [List verified features with proofs]

Vérification anti-hallucination:
- Méthodologie: Read + Grep + comptage direct
- X affirmations vérifiées ligne par ligne
- Aucune hallucination détectée

Production-Ready Scorecard (XX/100):
[Scores]

Prochaine étape: [Next milestone]

🤖 Generated with Claude Code (Protocole Alpha Terminal)
Co-Authored-By: Claude <noreply@anthropic.com>"
```

## Expected Output

Provide the user with:

### 1. Verification Report
```markdown
## ✅ PROTOCOLE ALPHA TERMINAL - RAPPORT

**Commit vérifié:** abc1234
**Date:** 2025-10-YY
**Durée:** X minutes

### Résumé Vérification
- ✅ X affirmations vérifiées (0 hallucinations)
- ✅ Score production-ready: XX/100
- ✅ Métriques actualisées
- ✅ PLAN-COMPLET.md synchronisé

### Milestone Progress
- Avant: YY%
- Après: ZZ%
- Progrès: +N%

### Blockers Identifiés
[LIST]

### Actions Immédiates
1. Task 1 (CRITIQUE) - X heures
2. Task 2 (HAUTE) - XX min
3. Task 3 (NORMALE) - X jours

**Timeline estimée:** X-Y jours pour 100%
```

### 2. Updated Files
- ✅ PLAN-COMPLET.md (version X.Y+1)
- ✅ TACHES-IMMEDIATES.md (if needed)
- ✅ Git commit created

### 3. Clear Next Steps
Prioritized task list with exact commands

## Important Notes

- **NEVER skip verification steps** - Each claim must be proven with file:line
- **ALWAYS count actual occurrences** - Use grep, wc, find
- **ALWAYS update version number** - Increment PLAN-COMPLET.md version
- **ALWAYS create git commit** - Document the documentation update

**Protocole Alpha Terminal v1.0**
"Zero hallucination. Real progress. Clear next steps."
