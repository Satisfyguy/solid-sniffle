# 🚀 Guide des commandes Rust + Cursor + Claude

## 📦 Gestion de projet

### Créer un nouveau projet
```bash
# Projet binaire (application)
cargo new mon_projet

# Projet bibliothèque
cargo new ma_lib --lib

# Projet avec template optimisé
cp -r ~/rust-template mon_nouveau_projet
cd mon_nouveau_projet
```

### Structure d'un projet
```bash
# Voir l'arborescence du projet
tree -L 2

# Initialiser Git si pas déjà fait
git init
```

---

## 🔧 Commandes de développement

### Build & Run
```bash
# Compiler en mode debug
cargo build

# Compiler en mode release (optimisé)
cargo build --release

# Compiler et exécuter
cargo run

# Exécuter avec arguments
cargo run -- --arg1 value1

# Exécuter la version release
cargo run --release
```

### Vérification du code
```bash
# Vérifier sans compiler complètement (rapide)
cargo check

# Formater le code
cargo fmt

# Vérifier le formatage sans modifier
cargo fmt -- --check

# Linter avec Clippy
cargo clippy

# Clippy strict (recommandé)
cargo clippy -- -D warnings

# Voir tous les warnings Clippy
cargo clippy -- -W clippy::all
```

### Tests
```bash
# Lancer tous les tests
cargo test

# Tests avec affichage console
cargo test -- --nocapture

# Test spécifique
cargo test nom_du_test

# Tests avec verbosité
cargo test -- --show-output

# Tests en parallèle limité
cargo test -- --test-threads=1

# Benchmarks (si configurés)
cargo bench
```

### Documentation
```bash
# Générer et ouvrir la doc
cargo doc --open

# Doc avec dépendances
cargo doc --open --no-deps

# Vérifier les liens de doc
cargo doc --no-deps
```

---

## 🤖 Alias avec Claude (installés par le script)

### Analyse automatique
```bash
# Check + résumé IA des warnings
cargoc

# Tests + explication IA
cargot

# Run + analyse d'erreurs IA
cargor

# Build release + résumé IA
cargob

# Commandes de qualité
rustdoc    # Ouvre la documentation
rustfmt    # Vérifie le formatage
rustclip   # Clippy strict
```

### Exemples d'utilisation
```bash
# Workflow typique
cargoc          # Vérifier les erreurs
rustfmt         # Formater le code
rustclip        # Linter
cargot          # Tester
cargor          # Exécuter

# Debug rapide
cargoc && cargot    # Check et test en une fois
```

---

## 📚 Gestion des dépendances

### Ajouter des dépendances
```bash
# Ajouter une dépendance
cargo add serde

# Ajouter avec features
cargo add tokio --features full

# Ajouter en dev-dependency
cargo add --dev criterion

# Ajouter une version spécifique
cargo add serde --version 1.0.195
```

### Mettre à jour
```bash
# Voir les dépendances obsolètes
cargo outdated

# Mettre à jour toutes les dépendances
cargo update

# Mettre à jour une dépendance spécifique
cargo update -p serde
```

### Supprimer
```bash
# Supprimer une dépendance
cargo rm serde
```

### Informations
```bash
# Voir l'arbre des dépendances
cargo tree

# Voir les dépendances limitées
cargo tree --depth 1

# Chercher une dépendance
cargo tree -i tokio
```

---

## 🔍 Surveillance en temps réel

### Watch (auto-recompilation)
```bash
# Recompiler à chaque changement
cargo watch

# Watch + run
cargo watch -x run

# Watch + check + test
cargo watch -x check -x test

# Watch + clear terminal
cargo watch -c -x run

# Watch avec exécution custom
cargo watch -s "clear && cargo run"
```

---

## 🐛 Debugging

### Avec backtrace
```bash
# Activer le backtrace complet
RUST_BACKTRACE=1 cargo run

# Backtrace ultra-détaillé
RUST_BACKTRACE=full cargo run

# Permanentiser (déjà dans .bashrc via le script)
export RUST_BACKTRACE=1
```

### Avec GDB/LLDB
```bash
# Build avec symboles debug
cargo build

# Lancer avec gdb
rust-gdb target/debug/mon_projet

# Lancer avec lldb
rust-lldb target/debug/mon_projet
```

---

## 🧹 Nettoyage

```bash
# Nettoyer les fichiers de build
cargo clean

# Nettoyer cache global Cargo
cargo cache -r all

# Voir l'espace disque utilisé
du -sh target/
```

---

## 🔐 Bonnes pratiques

### Avant chaque commit
```bash
# Pipeline qualité complète
cargo fmt && \
cargo clippy -- -D warnings && \
cargo test && \
cargo doc --no-deps

# Ou version courte avec Claude
rustfmt && rustclip && cargot
```

### Workflow quotidien recommandé
```bash
# 1. Ouvrir le projet dans Cursor
cd mon_projet
cursor .

# 2. Vérifier l'état
cargoc

# 3. Développer avec watch
cargo watch -x check -x test

# 4. Avant de pousser
cargo fmt
cargo clippy -- -D warnings
cargot
git add .
git commit -m "feat: nouvelle fonctionnalité"
git push
```

---

## 💡 Commandes Claude Code directes

### Utiliser Claude en ligne de commande
```bash
# Poser une question
claude "Comment implémenter un trait Iterator en Rust?"

# Analyser un fichier
claude "Explique-moi ce code" < src/main.rs

# Déboguer une erreur
cargo build 2>&1 | claude "Explique cette erreur et propose une solution"

# Review de code
cat src/lib.rs | claude "Fais une review de ce code Rust"

# Générer des tests
cat src/main.rs | claude "Génère des tests unitaires pour ces fonctions"
```

### Workflows avancés
```bash
# Analyse de performance
cargo build --release && \
time target/release/mon_projet | \
claude "Analyse ces performances et suggère des optimisations"

# Documentation automatique
cargo doc --no-deps && \
claude "Génère un README.md pour ce projet Rust"

# Refactoring
cat src/old_code.rs | \
claude "Refactorise ce code en Rust idiomatique"
```

---

## 🎯 Astuces Cursor + Claude

### Dans Cursor
- **Ctrl+L** : Ouvrir le chat Claude
- **Ctrl+K** : Édition inline avec Claude
- **Ctrl+I** : Composer avec Claude

### Prompts utiles dans Cursor
```
"Explique-moi ce fichier Cargo.toml"
"Ajoute des tests pour cette fonction"
"Optimise cette boucle for"
"Convertis cette fonction synchrone en async"
"Ajoute de la documentation Rustdoc"
"Corrige les warnings Clippy"
"Implémente ce trait pour ma struct"
```

---

## 📊 Monitoring & Profiling

```bash
# Mesurer le temps de compilation
cargo build --timings

# Profiler avec perf (Linux)
cargo build --release
perf record target/release/mon_projet
perf report

# Analyser la taille binaire
cargo bloat --release

# Vérifier les dépendances inutilisées
cargo +nightly udeps
```

---

## 🚀 Déploiement

```bash
# Build release optimisé
cargo build --release

# Strip le binaire (réduire la taille)
strip target/release/mon_projet

# Cross-compilation (exemple Linux → Windows)
cargo build --release --target x86_64-pc-windows-gnu

# Créer un package
cargo package

# Publier sur crates.io
cargo publish
```

---

## 🆘 Aide rapide

```bash
# Aide Cargo
cargo --help
cargo <commande> --help

# Aide Rustc
rustc --help

# Version des outils
rustc --version
cargo --version
rustfmt --version
clippy-driver --version

# Mettre à jour Rust
rustup update

# Lister les toolchains installées
rustup show
```

---

## 📝 Checklist projet professionnel

- [ ] `cargo fmt` configuré et appliqué
- [ ] `cargo clippy` sans warnings
- [ ] Tests avec couverture > 80% (`cargo tarpaulin`)
- [ ] Documentation Rustdoc complète
- [ ] CI/CD configuré (GitHub Actions)
- [ ] `Cargo.toml` avec métadonnées complètes
- [ ] `README.md` avec exemples
- [ ] Licence définie
- [ ] CHANGELOG.md à jour

---

## 🎓 Ressources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Exercices interactifs
- [Awesome Rust](https://github.com/rust-unofficial/awesome-rust) - Liste de ressources

**🔥 Astuce finale** : Lance `cargo watch -c -x check -x test` dans un terminal et code dans Cursor avec Claude en assistant. Tu auras un feedback instantané + IA pour t'aider !