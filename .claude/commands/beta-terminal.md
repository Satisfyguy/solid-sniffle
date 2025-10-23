# PROTOCOLE BETA TERMINAL - Validation Production Complète

Exécute les **6 agents spécialisés** en séquence pour valider la production-readiness d'un module ou commit.

## Agents lancés (ordre d'exécution):

1. **anti-hallucination-validator** - Vérifie que tout le code existe réellement
2. **production-ready-enforcer** - Scorecard 98 critères production
3. **monero-security-validator** - Patterns sécurité Monero + Tor
4. **reality-check-generator** - Génère Reality Checks pour fonctions réseau
5. **milestone-tracker** - Met à jour progression milestone
6. **htmx-template-generator** - (Si frontend) Génère templates HTMX

## Usage

```
/beta-terminal <dossier_ou_commit>
```

**Exemples:**
- `/beta-terminal reputation/` - Valide tout le module reputation
- `/beta-terminal HEAD` - Valide le dernier commit
- `/beta-terminal server/src/handlers/` - Valide les handlers

## Workflow

Pour chaque agent, tu dois:

### 1. Lancer l'agent avec Task tool
```
Task tool → subagent_type: "anti-hallucination-validator"
Prompt: "Validate all code in reputation/ module. Check:
- All functions exist at claimed lines
- All imports are valid
- All tests pass
- No invented APIs or methods"
```

### 2. Attendre le rapport complet de l'agent

### 3. Passer au suivant uniquement après rapport

### 4. Agréger tous les rapports dans un FINAL REPORT

## Output attendu

À la fin, tu dois générer:

```markdown
# 🚀 PROTOCOLE BETA TERMINAL - RAPPORT FINAL

**Module/Commit évalué:** reputation/ (commits 118d23b + 73c5fde)
**Date:** 2025-10-22
**Durée totale:** XX minutes

## ✅ AGENT 1: Anti-Hallucination Validator
- Affirmations vérifiées: XX
- Hallucinations détectées: 0
- Status: ✅ PASS

## ✅ AGENT 2: Production-Ready Enforcer
- Score: XX/100
- Blockers: X
- Status: ✅ PASS / ⚠️ WARNINGS

## ✅ AGENT 3: Monero Security Validator
- Patterns vérifiés: XX
- Vulnérabilités: 0
- Status: ✅ PASS

## ✅ AGENT 4: Reality Check Generator
- Fonctions réseau: X
- Reality Checks générés: X
- Status: ✅ PASS

## ✅ AGENT 5: Milestone Tracker
- Milestone actuel: X.Y
- Progression: XX% → YY%
- Status: ✅ UPDATED

## ✅ AGENT 6: HTMX Template Generator
- Templates générés: X
- Status: ⏭️ SKIPPED (backend only)

---

## 🎯 SCORE GLOBAL

**Production-Ready:** XX/100

### Catégories
- Security: XX/100
- Code Quality: XX/100
- Testing: XX/100
- Documentation: XX/100

### BLOCKERS CRITIQUES (X)
1. [Description] - XX min
2. [Description] - XX min

### ACTIONS IMMÉDIATES
1. **CRITIQUE** - [Action] (XX min)
2. **HAUTE** - [Action] (XX min)
3. **NORMALE** - [Action] (XX heures)

---

## 📊 SYNTHÈSE

**Statut:** ✅ PRODUCTION-READY / ⚠️ BLOCKERS / 🔴 NOT READY

**Timeline vers 98/100:** X-Y jours

**Prochaines étapes:**
1. [Step 1]
2. [Step 2]
3. [Step 3]
```

## Important

- **TOUJOURS lancer les 6 agents** (même si certains sont skipped)
- **NE PAS skip d'étapes** - Chaque agent apporte une validation unique
- **Agréger TOUS les rapports** dans le rapport final
- **Créer un fichier** `BETA-TERMINAL-{MODULE}-REPORT.md`
- **Commit le rapport** avec message clair

## Création du rapport

À la fin, tu dois:

1. Créer `BETA-TERMINAL-{MODULE}-REPORT.md`
2. Git add + commit:
```bash
git add BETA-TERMINAL-{MODULE}-REPORT.md
git commit --no-verify -m "docs: Beta Terminal Report - {Module} (Score XX/100)

Protocole Beta Terminal complet exécuté sur {module}

Agents lancés (6/6):
✅ Anti-Hallucination: 0 hallucinations
✅ Production-Ready: XX/100
✅ Monero Security: PASS
✅ Reality Checks: X generated
✅ Milestone Tracker: XX% → YY%
⏭️ HTMX Templates: SKIPPED

Blockers: X critiques, X hautes
Actions immédiates: X tâches (XX heures)

🤖 Protocole Beta Terminal v2.0
Co-Authored-By: Claude <noreply@anthropic.com>"
```

3. Push vers GitHub

---

**Protocole Beta Terminal v2.0**
"Six agents. Zero hallucination. Production-ready proof."
