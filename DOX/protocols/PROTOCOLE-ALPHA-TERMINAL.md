# Protocole Alpha Terminal

**Version:** 1.0
**Date de Création:** 2025-10-20
**Statut:** 🟢 Actif

---

## 🎯 Objectif

Le **Protocole Alpha Terminal** est une méthodologie systématique de vérification et documentation du progrès projet après chaque commit significatif. Il garantit:

1. **Zéro Hallucination** - Toutes les affirmations sont vérifiées ligne par ligne dans le code
2. **Production-Ready** - Chaque avancement est évalué selon les standards de production
3. **Documentation Synchronisée** - PLAN-COMPLET.md reste toujours à jour avec la réalité du code
4. **Actions Immédiates** - Identification claire des prochaines tâches prioritaires

---

## 🔥 Déclenchement

**Commande utilisateur:**
```
Active le protocole Alpha Terminal
```

ou

```
/alpha-terminal
```

**Moment d'utilisation:**
- Après chaque commit significatif (nouvelles fonctionnalités, intégrations majeures)
- Après résolution de bloqueurs critiques
- Avant les reviews de milestone
- Sur demande pour audit du progrès

---

## 📋 Checklist du Protocole (7 Étapes)

### ✅ Étape 1: Identification du Dernier Commit

**Actions:**
```bash
git log --oneline -1
git show --stat HEAD
```

**Vérifier:**
- [ ] Hash du commit
- [ ] Message de commit
- [ ] Fichiers modifiés
- [ ] Lines added/deleted

**Output attendu:**
```
Commit: abc1234
Message: feat: Implement XYZ feature
Files: 5 changed, +234/-12
```

---

### ✅ Étape 2: Vérification Anti-Hallucination

**Méthodologie:** Lire directement les fichiers + grep + comptage

#### 2.1 Vérifier les Affirmations du Commit

Pour chaque affirmation dans le commit message:

**Template de vérification:**
```markdown
### Affirmation: "Feature X implémentée"

**Méthode de vérification:**
- [ ] Read du fichier concerné
- [ ] Grep des fonctions clés
- [ ] Comptage des appels/lignes
- [ ] Validation syntaxique

**Preuve:**
- Fichier: path/to/file.rs:line
- Code snippet vérifié: `function_name()`
- Comptage: X occurrences trouvées

**Verdict:** ✅ VÉRIFIÉ | ❌ NON VÉRIFIÉ | ⚠️ PARTIEL
```

#### 2.2 Vérifications Standard Obligatoires

**Pour CHAQUE commit contenant du code serveur:**

1. **Vérifier Zero Unwrap/Expect dans handlers/**
```bash
grep -rn "\.unwrap()\|\.expect(" server/src/handlers/*.rs | wc -l
# Attendu: 0
```

2. **Vérifier Présence Tests E2E**
```bash
find server/tests -name "*integration.rs" -o -name "*e2e.rs" | wc -l
grep -r "#\[test\]" server/tests/ | wc -l
```

3. **Vérifier Security Theatre**
```bash
./scripts/check-security-theatre.sh
# Note: violations dans tests/ sont acceptables
```

4. **Vérifier Intégration si applicable**
```bash
# Tracer la chaîne d'appels
grep -n "function_name" server/src/**/*.rs
```

---

### ✅ Étape 3: Évaluation Production-Ready

**Utiliser le production-ready skill si disponible:**
```
@production-ready skill activated
```

**Scorecard à remplir:**

| Catégorie | Score /100 | Preuve Code | Issues |
|-----------|------------|-------------|--------|
| Security Hardening | | | |
| Input Validation | | | |
| Error Handling | | | |
| Authorization | | | |
| Integration | | | |
| State Management | | | |
| Database Security | | | |
| Code Quality | | | |

**Score Global:** XX/100

**Blockers Identifiés:**
- [ ] Blocker 1: Description + fichier:ligne
- [ ] Blocker 2: Description + fichier:ligne

---

### ✅ Étape 4: Mise à Jour Métriques

**Comptage Obligatoire:**

```bash
# 1. Lines of Code
find server/src -name "*.rs" -type f | xargs wc -l | tail -1
# Output: X,XXX total

# 2. Fichiers Rust
find . -name "*.rs" -type f | wc -l
# Output: XX fichiers

# 3. Tests
grep -r "#\[test\]\|#\[tokio::test\]" server/ | wc -l
# Output: XX tests

# 4. API Endpoints (si applicable)
grep -r "#\[get\]\|#\[post\]\|#\[put\]\|#\[delete\]" server/src/handlers/ | wc -l
# Output: XX endpoints

# 5. Security Theatre Violations
./scripts/check-security-theatre.sh 2>&1 | grep "issues"
# Output: XX issues
```

**Tableau de Synthèse:**
```markdown
| Métrique | Valeur Actuelle | Évolution | Vérifié |
|----------|-----------------|-----------|---------|
| LOC server/src | X,XXX | +YYY | ✅ |
| Fichiers Rust | XX | +Y | ✅ |
| Tests E2E | XX | +Y | ✅ |
| API Endpoints | XX/YY (ZZ%) | +Y | ✅ |
| Security Theatre (prod) | XX | -Y | ✅ |
```

---

### ✅ Étape 5: Mise à Jour PLAN-COMPLET.md

**Sections à mettre à jour:**

#### 5.1 Header (Lignes 1-15)
```markdown
**Version:** X.Y → X.Y+1
**Dernière Mise à Jour:** 2025-10-XX → 2025-10-YY
**Statut Actuel:** Milestone X.Y à ZZ% → Milestone X.Y à ZZ+N%
```

#### 5.2 Nouvelle Section Avancées (Après ligne ~80)
```markdown
**🚀 NOUVEAUTÉS (2025-10-YY - Commit XXXXXXX):**

**🔴 BLOQUEUR CRITIQUE #N RÉSOLU - Titre:**
- ✅ Accomplissement 1 avec preuve (fichier:ligne)
- ✅ Accomplissement 2 avec preuve (fichier:ligne)
- ✅ Accomplissement 3 avec preuve (fichier:ligne)

**📊 Statistiques Codebase (Vérifiées Anti-Hallucination):**
- **Total LOC server/src:** X,XXX lignes (+YY% vs avant)
- **Fichier principal:** path/to/file.rs: XXX lignes
- **API Endpoints Actifs:** XX/YY (ZZ%)
- **Tests E2E:** XX tests
- **Security Theatre Production:** XX violations

**🔒 Vérification Production-Ready (XX/100):**
[Insérer scorecard]
```

#### 5.3 Milestones (Ligne ~320)
```markdown
- [x] **Milestone X.Y (ZZ% ✅):** Feature A complète ✅ **NOUVEAU**
- [x] **Milestone X.Y (ZZ% ✅):** Feature B complète ✅ **NOUVEAU**
- [ ] **Milestone X.Y (Restant N%):** Feature C en cours ⚠️
```

#### 5.4 Section Vérification Anti-Hallucination (Avant ## Next Review)
```markdown
## 🔍 Vérification Anti-Hallucination (2025-10-YY)

**Méthodologie:** Lecture directe + grep + comptage

### ✅ Affirmation 1: Titre
**Claim:** "Description"
**Preuve:** fichier:ligne
**Comptage:** X occurrences
**Verdict:** ✅ VÉRIFIÉ

[Répéter pour chaque affirmation]

### 📊 Résumé
| Affirmation | Méthode | Lignes Vérifiées | Verdict |
|-------------|---------|------------------|---------|
| Feature A | Read + Grep | file.rs:XX-YY | ✅ |
```

#### 5.5 Changelog (Avant fin de document)
```markdown
| X.Y+1 | 2025-10-YY | Milestone X.Y → ZZ% - Résumé commit | Claude |
```

---

### ✅ Étape 6: Identification des Actions Immédiates

**Template TACHES-IMMEDIATES:**

```markdown
# Tâches Immédiates - Post Commit XXXXXXX

**Date:** 2025-10-YY
**Contexte:** [Résumé du commit et accomplissements]

---

## 🔴 PRIORITÉ CRITIQUE (Blockers)

### Task 1: [Titre du Blocker]
**Temps estimé:** X heures
**Impact:** Description de l'impact

**Commandes:**
```bash
# Étape 1
command1

# Étape 2
command2
```

**Fichiers à modifier:**
- [ ] path/to/file1.rs (lignes XX-YY)
- [ ] path/to/file2.rs (lignes XX-YY)

**Critères de succès:**
- [ ] Critère 1
- [ ] Critère 2

---

## 🟡 PRIORITÉ HAUTE (Quick Wins)

### Task 2: [Titre]
**Temps estimé:** XX minutes ⚡
**Impact:** +XX% milestone

[Même structure que Task 1]

---

## 🟢 PRIORITÉ NORMALE (Améliorations)

### Task 3: [Titre]
**Temps estimé:** X-Y jours

[Même structure]

---

## 📊 Résumé

**Total tasks:** X
**Temps estimé total:** X-Y jours
**Impact milestone:** +XX%

**Ordre d'exécution recommandé:**
1. Task 1 (CRITIQUE)
2. Task 2 (Quick Win)
3. Task 3 (Amélioration)
```

---

### ✅ Étape 7: Git Commit de la Documentation

**Créer commit documentation:**

```bash
git add PLAN-COMPLET.md
git add TACHES-IMMEDIATES.md  # si créé

git commit --no-verify -m "docs: Update PLAN-COMPLET.md - Milestone X.Y → ZZ% (Protocole Alpha Terminal)

Version X.Y+1 - Vérification anti-hallucination complète

Changements principaux:
- Mise à jour statut: YY% → ZZ% (Milestone X.Y)
- Ajout section vérification anti-hallucination
- Metrics actualisées: X,XXX LOC, XX/YY endpoints
- Documentation commit XXXXXXX
- Score production-ready: XX/100

Nouveautés vérifiées (2025-10-YY):
✅ Feature A - Preuve fichier:ligne
✅ Feature B - Preuve fichier:ligne
✅ Feature C - Preuve fichier:ligne

Vérification anti-hallucination:
- Méthodologie: Read + Grep + comptage direct
- X affirmations vérifiées ligne par ligne
- Aucune hallucination détectée

Production-Ready Scorecard (XX/100):
[Liste des scores]

Prochaine étape: [Description]

🤖 Generated with Claude Code (Protocole Alpha Terminal)
Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 📊 Output Attendu

À la fin du protocole, l'utilisateur reçoit:

### 1. **Rapport de Vérification**
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
- [LISTE DES BLOCKERS]

### Actions Immédiates
1. Task 1 (CRITIQUE) - X heures
2. Task 2 (HAUTE) - XX min
3. Task 3 (NORMALE) - X jours

**Timeline estimée:** X-Y jours pour atteindre 100%
```

### 2. **Fichiers Mis à Jour**
- ✅ PLAN-COMPLET.md (version X.Y+1)
- ✅ TACHES-IMMEDIATES.md (si nécessaire)
- ✅ Git commit documentation créé

### 3. **Prochaines Étapes Claires**
Liste priorisée des tâches avec:
- Temps estimé
- Commandes exactes
- Fichiers à modifier
- Critères de succès

---

## 🔧 Customisation du Protocole

### Variables Configurables

```bash
# Dans .env ou config
ALPHA_TERMINAL_VERBOSE=true          # Mode verbeux avec tous les détails
ALPHA_TERMINAL_AUTO_COMMIT=false     # Auto-commit PLAN-COMPLET.md
ALPHA_TERMINAL_CREATE_TASKS=true     # Créer TACHES-IMMEDIATES.md
ALPHA_TERMINAL_PRODUCTION_SKILL=true # Activer production-ready skill
```

### Seuils de Qualité

```bash
# Seuils pour validation
MIN_PRODUCTION_SCORE=95              # Score minimum pour "production-ready"
MAX_SECURITY_THEATRE_PROD=0          # Violations max dans production code
MIN_TEST_COVERAGE=75                 # Coverage minimum attendu
MAX_UNWRAP_HANDLERS=0                # Unwrap max dans handlers/
```

---

## 📚 Références

**Fichiers liés:**
- `.claude/commands/alpha-terminal.md` - Slash command pour activation rapide
- `CLAUDE.md` - Documentation développeur avec référence au protocole
- `PLAN-COMPLET.md` - Document mis à jour par le protocole
- `TACHES-IMMEDIATES.md` - Output des actions immédiates
- `.cursorrules` - Règles de développement (production-ready standards)

**Scripts utilisés:**
- `./scripts/check-security-theatre.sh` - Détection violations
- `./scripts/security-dashboard.sh` - Dashboard sécurité

**Skills requis:**
- `production-ready` - Évaluation standards de production

---

## 🎓 Exemple Complet

### Input Utilisateur
```
Active le protocole Alpha Terminal
```

### Exécution (7 étapes)
```
[Étape 1] Identification commit: abc1234 "feat: Orders API integration"
[Étape 2] Vérification anti-hallucination: 5 affirmations
  ✅ Affirmation 1: orders.rs:416 vérifié
  ✅ Affirmation 2: escrow.rs:346 vérifié
  ✅ Affirmation 3: 0 unwrap handlers/ vérifié
  ✅ Affirmation 4: 8 appels RPC vérifié
  ✅ Affirmation 5: 15 tests E2E vérifié
[Étape 3] Production-ready: Score 98/100
[Étape 4] Métriques: 4,855 LOC, 14/20 endpoints
[Étape 5] PLAN-COMPLET.md mis à jour: v2.6, 95%
[Étape 6] Actions immédiates: 3 tasks identifiées
[Étape 7] Commit doc: 63c574d créé
```

### Output
```markdown
## ✅ PROTOCOLE ALPHA TERMINAL - RAPPORT COMPLET

**Commit:** abc1234 - "feat: Orders API integration"
**Vérification:** ✅ 5/5 affirmations (0 hallucinations)
**Production-Ready:** 98/100 ✅
**Milestone:** 85% → 95% (+10%)

### Actions Immédiates (3 tasks)
1. [HAUTE] Corriger 30 violations tests (1h) ⚡
2. [NORMALE] E2E test escrow flow (3h)
3. [BASSE] Documentation API (2h)

**Timeline:** 2-3 jours → Milestone 100%
**Fichiers:** PLAN-COMPLET.md v2.6 ✅
```

---

## 🚀 Avantages du Protocole

1. **Garantie Qualité**
   - Zéro hallucination tolérée
   - Production-ready vérifié systématiquement
   - Documentation toujours synchronisée

2. **Traçabilité**
   - Chaque affirmation prouvée avec fichier:ligne
   - Historique complet dans PLAN-COMPLET.md
   - Git commits documentation structurés

3. **Efficacité**
   - Actions immédiates claires et priorisées
   - Temps estimés réalistes
   - Commandes exactes fournies

4. **Prédictibilité**
   - Timeline milestone actualisée
   - Blockers identifiés tôt
   - Progrès mesurable (+XX%)

---

## ✅ Checklist Utilisateur

**Avant d'activer le protocole:**
- [ ] Commit(s) récent(s) pushed
- [ ] Code compilé sans erreurs
- [ ] Tests passing (si applicable)

**Pendant l'exécution:**
- [ ] Laisser Claude exécuter les 7 étapes
- [ ] Reviewer le rapport final
- [ ] Valider les actions immédiates

**Après le protocole:**
- [ ] PLAN-COMPLET.md à jour
- [ ] Prochaines tâches claires
- [ ] Timeline milestone connue

---

**Protocole Alpha Terminal - v1.0**
**"Zero hallucination. Real progress. Clear next steps."**
