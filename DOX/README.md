# DOX - Documentation Centralisée

Ce dossier contient toute la documentation du projet Monero Marketplace, organisée par catégories.

## Structure

### 📋 `/protocols/`
Protocoles officiels de développement et validation:
- `PROTOCOLE-ALPHA-TERMINAL.md` - Vérification anti-hallucination post-commit
- `PROTOCOLE-BETA-TERMINAL.md` - Validation production-ready
- Autres protocoles de workflow

### 📚 `/guides/`
Guides d'installation, configuration et utilisation:
- Quick-start guides (Ubuntu, reputation system, etc.)
- Setup et migration guides
- Instructions Gemini/AI
- Commandes de référence
- Guides de test manuels

### 📊 `/sessions/`
Récapitulatifs et résumés de sessions de développement:
- Session summaries
- Rapport de sessions spécifiques
- Refactoring summaries

### 🎯 `/phases/`
Plans de développement et roadmap:
- `PLAN-COMPLET.md` - Plan de développement global (**FICHIER PRINCIPAL**)
- Phase-specific plans (Phase 1, 3-4, 5)
- Plans pour frontend/reputation
- Roadmap général
- Milestone verifications

### 📝 `/reports/`
Rapports Alpha/Beta Terminal, completions, fixes:
- Rapports Alpha Terminal
- Rapports Beta Terminal
- Completion reports (REP-3-4, implementation, etc.)
- Success reports
- Status reports (ETAT-FLOW, fichiers modifiés)
- Production-ready fixes
- Healthchecks, images, reputation, timeout implementations
- Clippy reports

### 🔒 `/audits/`
Audits de sécurité et validations:
- `AUDIT-V3.md`
- `ANTI-SECURITY-THEATRE-IMPLEMENTATION.md`
- `SECURITY.md` / `SECURITY_VALIDATION.md`
- Correction scores

### 🔄 `/migration/`
Documentation de migrations architecturales:
- Design migrations (NEXUS, custodial/non-custodial)
- Frontend migrations
- Refactoring documentation
- Restructure proposals
- Non-custodial analysis et certification

### 🧪 `/testing/`
Documentation de tests et bugs:
- Test flow guides
- Bug reports
- Dev testing documentation

### 🎨 `/frontend/`
Documentation spécifique frontend:
- NEXUS components inventory
- Torrc corrections
- Tera syntax issues
- Frontend simple guides

## Fichiers à la Racine du Projet

Seuls 2 fichiers Markdown restent à la racine du projet (requis par les scripts):

- **`README.md`** - Vue d'ensemble du projet (référencé par scripts d'audit)
- **`CLAUDE.md`** - Instructions pour Claude Code (référencé par check-environment.sh)

## Navigation Rapide

### Documents les Plus Consultés

1. **Plan principal**: [`phases/PLAN-COMPLET.md`](phases/PLAN-COMPLET.md)
2. **Protocole Alpha Terminal**: [`protocols/PROTOCOLE-ALPHA-TERMINAL.md`](protocols/PROTOCOLE-ALPHA-TERMINAL.md)
3. **Guide démarrage rapide**: [`guides/DEMARRAGE_RAPIDE.md`](guides/DEMARRAGE_RAPIDE.md)
4. **État du marketplace**: [`reports/ETAT-FLOW-MARKETPLACE.md`](reports/ETAT-FLOW-MARKETPLACE.md)

### Par Tâche

- **Installation/Setup** → `/guides/`
- **Développement d'une feature** → `/phases/PLAN-COMPLET.md`
- **Post-commit** → `/protocols/PROTOCOLE-ALPHA-TERMINAL.md`
- **Audit sécurité** → `/audits/`
- **Debug bug** → `/testing/`
- **Migration architecture** → `/migration/`

## Statistiques

- **Total fichiers**: 90 fichiers Markdown
- **Protocols**: 4 fichiers
- **Guides**: 22 fichiers
- **Sessions**: 6 fichiers
- **Phases**: 8 fichiers
- **Reports**: 31 fichiers
- **Audits**: 5 fichiers
- **Migration**: 7 fichiers
- **Testing**: 2 fichiers
- **Frontend**: 5 fichiers

## Maintenance

Les scripts suivants ont été mis à jour pour pointer vers les nouveaux chemins:
- ✅ `CLAUDE.md` - Références mises à jour
- ✅ `.claude/commands/alpha-terminal.md` - Git add commands mis à jour
- ✅ `.claude/agents/milestone-tracker.md` - Grep commands mis à jour
- ✅ `scripts/check-environment.sh` - Fonctionne normalement (pas de changement requis)

### Migration Effectuée

La migration a été effectuée le 2025-10-27 avec les scripts:
- `scripts/migrate-md-to-dox.sh` - Script intelligent de catégorisation
- `scripts/bulk-move-md.sh` - Migration en masse

90 fichiers Markdown ont été déplacés de la racine vers DOX/.

---

**Note**: Cette organisation permet de garder la racine du projet propre tout en maintenant toute la documentation accessible et bien organisée.
