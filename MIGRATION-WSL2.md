# Guide de Migration vers WSL2/Ubuntu

## Pourquoi migrer ?

Ce projet **Monero Marketplace** est conçu pour Linux :
- Scripts Bash natifs
- Intégration Tor optimisée pour Linux
- Configuration Cargo pour Linux
- Monero RPC testé sur Linux

**Résultat actuel sur Windows :** Erreurs de compilation (conflit linker MSVC/MinGW)

---

## Étape 1 : Installer WSL2 et Ubuntu

### 1.1 Ouvrir PowerShell en Administrateur

Clic droit sur "Démarrer" → "Windows PowerShell (Admin)"

### 1.2 Installer WSL2 avec Ubuntu

```powershell
# Installation complète WSL2 + Ubuntu
wsl --install

# OU si WSL est déjà installé :
wsl --install -d Ubuntu-22.04

# Vérifier la version WSL
wsl --status
```

### 1.3 Redémarrer Windows

**Important :** Le redémarrage est obligatoire après l'installation.

---

## Étape 2 : Configurer Ubuntu

### 2.1 Lancer Ubuntu

- Cherchez "Ubuntu" dans le menu Démarrer
- Première ouverture : créez votre utilisateur/mot de passe

### 2.2 Mettre à jour le système

```bash
sudo apt update && sudo apt upgrade -y
```

---

## Étape 3 : Copier le projet dans WSL2

### Option A : Cloner depuis Git (Recommandé)

```bash
# Dans Ubuntu WSL2
cd ~
git clone <votre-repo-url> monero-marketplace
cd monero-marketplace
```

### Option B : Copier depuis Windows

```bash
# Accéder au disque C: depuis WSL
cd /mnt/c/Users/Lenovo/monero-marketplace

# Copier vers le home Ubuntu
cp -r /mnt/c/Users/Lenovo/monero-marketplace ~/monero-marketplace
cd ~/monero-marketplace
```

**⚠️ Important :** Travailler dans `~/monero-marketplace` (Linux filesystem) est **beaucoup plus rapide** que dans `/mnt/c/` (Windows filesystem).

---

## Étape 4 : Installation automatique

### 4.1 Rendre le script exécutable

```bash
cd ~/monero-marketplace
chmod +x scripts/ubuntu-setup.sh
```

### 4.2 Exécuter le script d'installation

```bash
./scripts/ubuntu-setup.sh
```

**Ce script installe automatiquement :**
- ✅ Build tools (gcc, pkg-config, libssl-dev)
- ✅ Tor daemon
- ✅ Rust toolchain (rustup, rustc, cargo)
- ✅ Monero CLI (testnet)
- ✅ Git hooks (pre-commit)
- ✅ Compile le projet

**Durée estimée :** 5-10 minutes

---

## Étape 5 : Configuration de l'environnement

### 5.1 Ajouter au ~/.bashrc

```bash
# Ouvrir le fichier
nano ~/.bashrc

# Ajouter à la fin :
# Monero Marketplace Development
export MONERO_TESTNET_PATH="$HOME/monero-testnet"
export PATH="$PATH:$HOME/monero-testnet/monero-x86_64-linux-gnu-v0.18.x.x" # ajuster version

# Rust
export PATH="$PATH:$HOME/.cargo/bin"

# Aliases
alias test-wallet="cargo test --package wallet -- --nocapture"
alias test-all="cargo test --workspace"
alias lint="cargo clippy --workspace -- -D warnings"
alias fmt="cargo fmt --workspace"
alias precommit="./scripts/pre-commit.sh"
```

### 5.2 Recharger la configuration

```bash
source ~/.bashrc
```

---

## Étape 6 : Vérifications

### 6.1 Vérifier Rust

```bash
rustc --version
cargo --version
```

### 6.2 Vérifier Tor

```bash
systemctl status tor
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org
```

### 6.3 Vérifier Monero

```bash
monerod --version
monero-wallet-cli --version
```

### 6.4 Compiler le projet

```bash
cd ~/monero-marketplace
cargo build --workspace
```

**Résultat attendu :** `Finished dev [unoptimized + debuginfo] target(s) in X.XXs`

### 6.5 Lancer les tests

```bash
# Tests unitaires
cargo test --workspace --lib

# Tests d'intégration (nécessite Monero RPC)
./scripts/setup-monero-testnet.sh
cargo test --package wallet
```

---

## Étape 7 : Workflow de développement

### 7.1 Accéder au projet

```bash
# Depuis n'importe où
cd ~/monero-marketplace
```

### 7.2 Édition de code

**Option A : VS Code avec WSL Extension**
```bash
# Depuis WSL, ouvrir VS Code
code .
```
VS Code s'ouvrira et se connectera automatiquement à WSL2.

**Option B : Éditeur dans WSL**
```bash
# Installer vim/nano
sudo apt install vim nano

# Éditer
vim common/src/lib.rs
```

### 7.3 Commandes quotidiennes

```bash
# Développement
cargo check --workspace          # Vérification rapide
cargo build --workspace          # Compilation
cargo test --workspace           # Tests
cargo fmt --workspace            # Formatage
cargo clippy --workspace         # Linting

# Sécurité
./scripts/pre-commit.sh          # Toutes les vérifications
./scripts/check-security-theatre.sh  # Détection security theatre

# Git
git add .
git commit -m "feat: nouvelle fonctionnalité"
# Le pre-commit hook s'exécute automatiquement
```

---

## Étape 8 : Configuration Git (si nécessaire)

```bash
# Configuration utilisateur
git config --global user.name "Votre Nom"
git config --global user.email "votre@email.com"

# Vérifier
git config --list
```

---

## Avantages de WSL2

✅ **Performance :** Compilation Rust 2-3x plus rapide
✅ **Compatibilité :** Scripts bash natifs fonctionnent directement
✅ **Tor :** Daemon Tor intégré au système
✅ **Monero :** Binaires Linux officiels
✅ **Tooling :** Tous les outils Linux disponibles
✅ **Isolation :** Environnement de dev séparé de Windows

---

## Accès aux fichiers

### Depuis Windows Explorer

Accédez à : `\\wsl$\Ubuntu\home\<votre-user>\monero-marketplace`

Ou tapez dans la barre d'adresse : `\\wsl$`

### Depuis WSL vers Windows

```bash
# Accéder au disque C:
cd /mnt/c/Users/Lenovo/

# Accéder au disque D:
cd /mnt/d/
```

---

## Résolution de problèmes

### WSL ne démarre pas

```powershell
# PowerShell Admin
wsl --shutdown
wsl --update
```

### Tor ne démarre pas

```bash
sudo systemctl restart tor
sudo systemctl status tor
```

### Rust n'est pas dans le PATH

```bash
source ~/.cargo/env
# Ou ajouter à ~/.bashrc
```

### Compilation lente

```bash
# Vérifier que vous êtes dans le filesystem Linux
pwd
# Doit afficher /home/<user>/... et NON /mnt/c/...

# Si dans /mnt/c/, copier vers ~
cp -r /mnt/c/Users/Lenovo/monero-marketplace ~/
```

---

## Documentation supplémentaire

- `UBUNTU-SETUP.md` - Guide détaillé Ubuntu
- `CLAUDE.md` - Règles de développement
- `docs/DEVELOPER-GUIDE.md` - Guide développeur complet
- `scripts/README.md` - Documentation des scripts

---

## Support

Si vous rencontrez des problèmes :

1. Vérifier les logs : `journalctl -u tor -n 50`
2. Consulter UBUNTU-SETUP.md
3. Lire les messages d'erreur complets
4. Vérifier que vous êtes bien dans le filesystem WSL (`pwd`)

---

**Prêt à commencer ? Exécutez :**

```bash
cd ~/monero-marketplace
./scripts/ubuntu-setup.sh
```

🦀🔐 **Happy coding on Linux!**
