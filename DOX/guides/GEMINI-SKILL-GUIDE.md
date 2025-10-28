# Guide d'Installation et d'Utilisation - Skill Gemini CLI

## 🎯 Skill: market-prod

**Description**: Assistant IA pour le plan de transformation "Production-Ready" du Monero Marketplace.

## 📥 Installation

### Option 1: Script automatique (Recommandé)
```bash
# Dans votre terminal Ubuntu/WSL
cd /mnt/c/Users/Lenovo/monero-marketplace
chmod +x install-gemini-skill.sh
./install-gemini-skill.sh
```

### Option 2: Installation manuelle
```bash
# 1. Installer Gemini CLI (si pas déjà fait)
npm install -g @google/gemini-cli
# OU
pip install gemini-cli

# 2. Créer le répertoire des skills
mkdir -p ~/.gemini-cli/skills

# 3. Copier la skill
cp .claude/skills/market-prod.yaml ~/.gemini-cli/skills/

# 4. Vérifier l'installation
gemini-cli --list-skills
```

## 🚀 Utilisation

### Commandes de base
```bash
# Utiliser la skill
gemini-cli market-prod "votre question"

# Exemples
gemini-cli market-prod "Comment implémenter la Phase 1 du plan?"
gemini-cli market-prod "Quels sont les quality gates pour la Phase 2?"
gemini-cli market-prod "Comment configurer les feature flags?"
gemini-cli market-prod "Quelle est la stratégie de testing pour la production?"
```

### Questions recommandées
```bash
# Phase 1 - Configuration Multi-Environnement
gemini-cli market-prod "Détaille la Phase 1: Configuration Multi-Environnement"

# Phase 2 - WalletManager Production
gemini-cli market-prod "Comment transformer le WalletManager pour la production?"

# Phase 3 - WebSocket Production
gemini-cli market-prod "Quelle est la stratégie pour implémenter les WebSockets?"

# Quality Gates
gemini-cli market-prod "Quels sont tous les quality gates à respecter?"

# Testing
gemini-cli market-prod "Comment implémenter les tests d'intégration réels?"
```

## 📋 Contenu de la Skill

La skill contient le **plan de transformation complet** avec :

### 🎯 5 Phases de Développement
1. **Phase 1**: Configuration Multi-Environnement
2. **Phase 2**: WalletManager Production
3. **Phase 3**: WebSocket Production
4. **Phase 4**: Encryption Key Management
5. **Phase 5**: Monitoring et Observabilité

### 🚦 Quality Gates
- Pre-commit hooks
- CI/CD Pipeline
- Production Readiness
- Tests d'intégration réels
- Security audit

### 📊 Métriques de Performance
- Latence cibles
- Throughput attendu
- Disponibilité requise
- Métriques de succès

### 🔄 Processus de Développement
- Workflow Git
- Code Review
- Deployment
- Incident Response

## 🎯 Paradigme "Production-First"

La skill implémente le nouveau paradigme :
- ✅ Coder DIRECTEMENT pour la production
- ✅ Feature flags pour basculer entre testnet/mainnet
- ✅ Configuration multi-environnement
- ✅ Tests contre services RÉELS
- ✅ Code = Production code dès le premier commit

## 🔧 Intégration avec le Projet

### Structure des fichiers
```
.claude/skills/
├── market-prod.yaml          # Skill principale
├── check-security-theatre.yaml  # Skill existante
└── productionskill.yaml      # Skill vide (à supprimer)

install-gemini-skill.sh       # Script d'installation
GEMINI-SKILL-GUIDE.md         # Ce guide
```

### Workflow recommandé
1. **Poser une question** à la skill
2. **Implémenter** selon les directives
3. **Respecter** les quality gates
4. **Tester** avec les critères définis
5. **Valider** avant commit

## 🚨 Troubleshooting

### Problème: "gemini-cli command not found"
```bash
# Vérifier l'installation
which gemini-cli
npm list -g @google/gemini-cli

# Réinstaller si nécessaire
npm install -g @google/gemini-cli
```

### Problème: "Skill not found"
```bash
# Vérifier le répertoire des skills
ls -la ~/.gemini-cli/skills/

# Réinstaller la skill
./install-gemini-skill.sh
```

### Problème: "Permission denied"
```bash
# Rendre le script exécutable
chmod +x install-gemini-skill.sh
chmod +x ~/.gemini-cli/skills/market-prod.yaml
```

## 📚 Ressources

- **Plan complet**: Contenu dans la skill
- **Documentation projet**: `docs/` directory
- **Scripts**: `scripts/` directory
- **Configuration**: `.cursorrules` file

## 🎯 Prochaines Étapes

1. **Installer** la skill
2. **Tester** avec une question simple
3. **Commencer** la Phase 1 du plan
4. **Respecter** les quality gates
5. **Itérer** selon le processus défini

---

**Note**: Cette skill est votre **source de vérité unique** pour le plan de transformation. Toute implémentation doit être cohérente avec ce document.
