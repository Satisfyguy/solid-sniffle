# 🏗️ Proposition de Restructuration du Projet

**Date:** 2025-10-26  
**Version Actuelle:** v4.0  
**Objectif:** Structure Rust professionnelle et maintenable

---

## 📊 ANALYSE DE LA STRUCTURE ACTUELLE

### ❌ Problèmes Identifiés

1. **Racine encombrée** - 80+ fichiers markdown à la racine
2. **Dossiers temporaires** - `4.5/`, `4.s/`, `archive/`, `venv/`
3. **Binaires dans git** - `buyer`, `linux64`, `monero-x86_64-linux-gnu-v0.18.4.3/`
4. **Logs dans git** - `*.log`, `build.log`, `server.log`
5. **Fichiers de build** - `mingw-temp.zip`, `go-ipfs_v0.24.0_linux-amd64.tar.gz`
6. **Scripts éparpillés** - Scripts à la racine ET dans `scripts/`
7. **Documentation fragmentée** - Docs à la racine ET dans `docs/`

### ✅ Structure Actuelle (Bonne)

```
monero.marketplace/
├── server/          ✅ Crate principal
├── wallet/          ✅ Crate wallet
├── common/          ✅ Crate commun
├── cli/             ✅ Crate CLI
├── reputation/      ✅ Module reputation
├── templates/       ✅ Templates Tera
├── static/          ✅ Assets frontend
└── scripts/         ✅ Scripts utilitaires
```

---

## 🎯 STRUCTURE PROPOSÉE (Standard Rust)

```
monero-marketplace/
│
├── 📦 WORKSPACE CRATES
│   ├── crates/
│   │   ├── server/              # Serveur web principal
│   │   ├── wallet/              # Client Monero RPC
│   │   ├── common/              # Types & erreurs partagés
│   │   ├── cli/                 # Outils CLI
│   │   └── reputation/          # Module réputation
│   │       ├── common/
│   │       ├── crypto/
│   │       └── wasm/
│
├── 📁 ASSETS & RESOURCES
│   ├── static/                  # CSS, JS, images
│   │   ├── css/
│   │   ├── js/
│   │   └── wasm/
│   ├── templates/               # Templates Tera
│   │   ├── auth/
│   │   ├── listings/
│   │   ├── orders/
│   │   └── partials/
│   └── migrations/              # Migrations DB (consolidées)
│
├── 📚 DOCUMENTATION
│   ├── docs/
│   │   ├── architecture/        # Design docs
│   │   ├── api/                 # API specs
│   │   ├── deployment/          # Guides déploiement
│   │   ├── development/         # Guides dev
│   │   ├── security/            # Audits & security
│   │   └── milestones/          # Rapports milestones
│   └── README.md
│
├── 🔧 SCRIPTS & TOOLS
│   ├── scripts/
│   │   ├── dev/                 # Scripts développement
│   │   ├── deploy/              # Scripts déploiement
│   │   ├── test/                # Scripts de test
│   │   └── setup/               # Scripts d'installation
│   └── tools/                   # Outils custom (Python, etc.)
│
├── 🐳 INFRASTRUCTURE
│   ├── .github/
│   │   └── workflows/           # CI/CD
│   ├── docker/                  # Dockerfiles
│   │   ├── server/
│   │   ├── monero/
│   │   └── ipfs/
│   └── deploy/                  # Configs déploiement
│       ├── nginx/
│       ├── systemd/
│       └── terraform/
│
├── 🧪 TESTS
│   ├── tests/                   # Tests d'intégration workspace
│   │   ├── e2e/
│   │   ├── integration/
│   │   └── fixtures/
│   └── benches/                 # Benchmarks
│
├── 📋 CONFIG FILES (Racine)
│   ├── Cargo.toml               # Workspace root
│   ├── Cargo.lock
│   ├── .gitignore
│   ├── .env.example
│   ├── rust-toolchain.toml
│   ├── rustfmt.toml
│   └── clippy.toml
│
└── 📖 DOCS RACINE (Essentiels uniquement)
    ├── README.md                # Guide principal
    ├── CHANGELOG.md             # Historique versions
    ├── CONTRIBUTING.md          # Guide contribution
    ├── LICENSE
    └── SECURITY.md              # Security policy
```

---

## 🔄 PLAN DE MIGRATION

### Phase 1: Nettoyage (30 min)

#### 1.1 Supprimer fichiers temporaires
```bash
# Binaires et archives
rm -rf buyer linux64 mingw-temp.zip
rm -rf monero-x86_64-linux-gnu-v0.18.4.3/
rm -rf go-ipfs/ go-ipfs_v0.24.0_linux-amd64.tar.gz

# Logs
rm -f *.log server*.log build.log ipfs.log

# Dossiers temporaires
rm -rf 4.5/ 4.s/ archive/ venv/ node_modules/

# Fichiers de test
rm -f test.txt cookies.txt buyer.* ma_requette.json

# Python temporaire
rm -f code_validator_mcp.py main.py models.py requirements.txt
```

#### 1.2 Mettre à jour .gitignore
```gitignore
# Binaires
buyer
vendor
arbiter
linux64
*.exe

# Logs
*.log
server*.log

# Archives
*.tar.gz
*.zip

# Python
venv/
__pycache__/
*.pyc

# Node
node_modules/

# Monero
monero-*/
*.keys
*.address.txt

# IPFS
go-ipfs/

# Build
target/
*.db
*.db-*
```

### Phase 2: Réorganiser Documentation (45 min)

#### 2.1 Créer structure docs/
```bash
mkdir -p docs/{architecture,api,deployment,development,security,milestones}
```

#### 2.2 Déplacer fichiers
```bash
# Architecture
mv PLAN-COMPLET.md docs/architecture/
mv ROADMAP.md docs/architecture/
mv PHASE-*.md docs/architecture/

# Milestones
mv MILESTONE-*.md docs/milestones/
mv BETA-TERMINAL-*.md docs/milestones/
mv ALPHA-TERMINAL-*.md docs/milestones/
mv COMPLETION-*.md docs/milestones/
mv SUCCESS-*.md docs/milestones/

# Security
mv SECURITY.md docs/security/
mv SECURITY_*.md docs/security/
mv AUDIT-*.md docs/security/
mv NON-CUSTODIAL-*.md docs/security/

# Development
mv DEV_TESTING.md docs/development/
mv QUICK-START-*.md docs/development/
mv MIGRATION-*.md docs/development/
mv REFACTORING_*.md docs/development/

# Deployment
mv STAGING-DEPLOYMENT-REPORT.md docs/deployment/
mv PRODUCTION-READY-*.md docs/deployment/

# Protocols
mv PROTOCOLE-*.md docs/development/protocols/

# Guides
mv CLAUDE.md docs/development/
mv GEMINI*.md docs/development/
```

#### 2.3 Garder à la racine (essentiels)
```bash
# Garder uniquement
README.md
CHANGELOG.md (créer)
CONTRIBUTING.md (créer)
LICENSE
SECURITY.md (lien vers docs/security/)
TAF.md (TODO list - temporaire)
```

### Phase 3: Réorganiser Crates (1h)

#### 3.1 Créer structure crates/
```bash
mkdir -p crates
mv server crates/
mv wallet crates/
mv common crates/
mv cli crates/
mv reputation crates/
```

#### 3.2 Mettre à jour Cargo.toml racine
```toml
[workspace]
members = [
    "crates/server",
    "crates/wallet",
    "crates/common",
    "crates/cli",
    "crates/reputation/common",
    "crates/reputation/crypto",
    "crates/reputation/wasm",
]
resolver = "2"

[workspace.package]
version = "4.0.0"
edition = "2021"
authors = ["Monero Marketplace Team"]
license = "MIT"
repository = "https://github.com/Satisfyguy/solid-sniffle"

[workspace.dependencies]
# Versions partagées
tokio = { version = "1.48", features = ["full"] }
actix-web = "4.11"
serde = { version = "1.0", features = ["derive"] }
# ... etc
```

#### 3.3 Mettre à jour chemins dans Cargo.toml des crates
```toml
# crates/server/Cargo.toml
[dependencies]
monero-marketplace-common = { path = "../common" }
monero-marketplace-wallet = { path = "../wallet" }
reputation-common = { path = "../reputation/common" }
```

### Phase 4: Réorganiser Scripts (30 min)

```bash
mkdir -p scripts/{dev,deploy,test,setup}

# Dev
mv start-server.sh scripts/dev/
mv setup_dev_env.sh scripts/dev/
mv install-*.sh scripts/setup/

# Test
mv test-*.sh scripts/test/
mv beta-terminal-protocol*.sh scripts/test/

# Deploy
mv market-prod.sh scripts/deploy/
mv build.sh scripts/deploy/

# Nettoyer scripts/ existant
# Organiser par catégorie
```

### Phase 5: Infrastructure (30 min)

```bash
mkdir -p deploy/{nginx,systemd,terraform}
mkdir -p docker/{server,monero,ipfs}

# Déplacer configs existantes
mv 4.5/nginx/ deploy/nginx/ 2>/dev/null || true
mv 4.5/docker/ docker/ 2>/dev/null || true
```

### Phase 6: Migrations DB (15 min)

```bash
# Consolider migrations
mkdir -p migrations
cp crates/server/migrations/* migrations/ 2>/dev/null || true
cp crates/reputation/migrations/* migrations/ 2>/dev/null || true

# Renommer avec timestamps
# 001_initial_schema.sql
# 002_add_reputation.sql
# etc.
```

---

## ✅ VALIDATION POST-MIGRATION

### Checklist

```bash
# 1. Build workspace
cargo build --workspace
# ✅ Doit compiler sans erreurs

# 2. Tests
cargo test --workspace
# ✅ Tous les tests passent

# 3. Clippy
cargo clippy --workspace --all-targets -- -D warnings
# ✅ 0 erreurs

# 4. Chemins templates
# ✅ Vérifier que templates/ est accessible

# 5. Static assets
# ✅ Vérifier que static/ est accessible

# 6. Scripts
chmod +x scripts/**/*.sh
# ✅ Tous exécutables

# 7. Documentation
# ✅ Liens dans README.md mis à jour

# 8. Git
git status
# ✅ Pas de fichiers perdus
```

---

## 📝 FICHIERS À CRÉER

### 1. CHANGELOG.md
```markdown
# Changelog

## [4.0.0] - 2025-10-26

### Added
- Frontend complet avec design noir brutal
- Système de commandes opérationnel
- Notifications WebSocket temps réel
- Module réputation avec signatures cryptographiques

### Changed
- Restructuration complète du projet
- Organisation en workspace Rust standard

### Fixed
- 14 erreurs Clippy corrigées
- 38 violations security theatre justifiées
```

### 2. CONTRIBUTING.md
```markdown
# Guide de Contribution

## Structure du Projet
- `crates/` - Crates Rust
- `docs/` - Documentation
- `scripts/` - Scripts utilitaires
- `static/` - Assets frontend
- `templates/` - Templates Tera

## Développement
```bash
# Setup
./scripts/setup/setup_dev_env.sh

# Run
./scripts/dev/start-server.sh

# Test
cargo test --workspace
```

## Standards
- Rust 2021 edition
- Clippy strict mode
- 0 security theatre
```

### 3. rust-toolchain.toml
```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
profile = "default"
```

### 4. rustfmt.toml
```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
```

### 5. clippy.toml
```toml
cognitive-complexity-threshold = 30
```

---

## 🎯 BÉNÉFICES

### Avant
```
❌ 80+ fichiers markdown à la racine
❌ Binaires dans git
❌ Logs dans git
❌ Structure confuse
❌ Difficile à naviguer
```

### Après
```
✅ 5 fichiers essentiels à la racine
✅ Documentation organisée
✅ Structure Rust standard
✅ Facile à maintenir
✅ Prêt pour contribution open-source
```

---

## ⏱️ TEMPS TOTAL ESTIMÉ

| Phase | Durée | Risque |
|-------|-------|--------|
| Phase 1: Nettoyage | 30 min | 🟢 Faible |
| Phase 2: Documentation | 45 min | 🟢 Faible |
| Phase 3: Crates | 1h | 🟡 Moyen |
| Phase 4: Scripts | 30 min | 🟢 Faible |
| Phase 5: Infrastructure | 30 min | 🟢 Faible |
| Phase 6: Migrations | 15 min | 🟢 Faible |
| **TOTAL** | **3h30** | 🟢 **Faible** |

---

## 🚀 RECOMMANDATION

**✅ OUI, on peut réorganiser sans risque!**

**Approche recommandée:**
1. Créer une branche `restructure`
2. Faire la migration phase par phase
3. Tester après chaque phase
4. Merger quand tout fonctionne

**Commande pour démarrer:**
```bash
git checkout -b restructure
git add -A
git commit -m "checkpoint: avant restructuration"
```

Voulez-vous que je commence la restructuration?
