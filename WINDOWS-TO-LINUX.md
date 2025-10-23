# Migration Windows → Linux (WSL2)

## Problème actuel

❌ **Erreur de compilation sur Windows :**
```
error: linking with `link.exe` failed: exit code: 1
error: could not compile `windows_x86_64_msvc` (build script)
```

**Cause :** Ce projet est conçu pour Linux. La configuration Cargo cible `x86_64-unknown-linux-gnu` mais vous êtes sur Windows avec MinGW, créant un conflit de linker.

---

## Solution : WSL2 + Ubuntu

### Méthode Rapide (5 minutes)

#### 1. Installer WSL2 + Ubuntu

**Option A : Script automatique (Recommandé)**
```powershell
# PowerShell en Administrateur
cd C:\Users\Lenovo\monero-marketplace
.\install-wsl2.ps1
```

**Option B : Commande manuelle**
```powershell
# PowerShell en Administrateur
wsl --install
# Ou spécifiquement Ubuntu :
wsl --install -d Ubuntu-22.04
```

#### 2. Redémarrer Windows

**Obligatoire après installation WSL2**

#### 3. Configurer Ubuntu

Lancez "Ubuntu" depuis le menu Démarrer :

```bash
# Créer user/password (première fois)
# Puis mettre à jour :
sudo apt update && sudo apt upgrade -y
```

#### 4. Copier et installer le projet

```bash
# Copier depuis Windows vers Ubuntu
cp -r /mnt/c/Users/Lenovo/monero-marketplace ~/monero-marketplace
cd ~/monero-marketplace

# Installer tout automatiquement
chmod +x scripts/ubuntu-setup.sh
./scripts/ubuntu-setup.sh
```

**Le script installe :**
- Build tools (gcc, libssl-dev, etc.)
- Tor daemon
- Rust (rustup, cargo, clippy)
- Monero CLI (testnet)
- Git hooks
- Compile le projet ✅

**Durée : 5-10 minutes**

---

## Vérification

```bash
# Après installation
cd ~/monero-marketplace

# Compiler
cargo build --workspace
# ✅ Devrait compiler sans erreur

# Tester
cargo test --workspace --lib
# ✅ Tests unitaires passent

# Vérifier Tor
systemctl status tor
# ✅ Tor actif

# Vérifier Monero
monero-wallet-cli --version
# ✅ Monero installé
```

---

## Développement quotidien

### Ouvrir le projet

**Option A : VS Code (Recommandé)**
```bash
# Depuis Ubuntu WSL
cd ~/monero-marketplace
code .
```
VS Code s'ouvre et se connecte à WSL2 automatiquement.

**Option B : Terminal Ubuntu**
```bash
# Depuis n'importe où
cd ~/monero-marketplace
```

### Commandes

```bash
# Développement
cargo check              # Vérification rapide
cargo build              # Compilation
cargo test               # Tests
cargo fmt                # Format
cargo clippy             # Lint

# Sécurité
./scripts/pre-commit.sh  # Toutes vérifications

# Git (comme d'habitude)
git add .
git commit -m "message"
git push
```

### Accéder aux fichiers depuis Windows

Explorateur Windows → `\\wsl$\Ubuntu\home\<user>\monero-marketplace`

---

## Pourquoi WSL2 ?

✅ **Performance** : Compilation 2-3x plus rapide
✅ **Compatibilité** : Tous les scripts bash fonctionnent
✅ **Tor** : Daemon intégré au système
✅ **Monero** : Binaires Linux officiels
✅ **Tooling** : Accès à tous les outils Linux
✅ **Pas de dual-boot** : Reste dans Windows

---

## Guides détaillés

- `MIGRATION-WSL2.md` - Guide complet étape par étape
- `UBUNTU-SETUP.md` - Configuration Ubuntu détaillée
- `CLAUDE.md` - Règles de développement
- `scripts/README.md` - Documentation scripts

---

## TL;DR

```powershell
# 1. PowerShell Admin
wsl --install

# 2. Redémarrer Windows

# 3. Lancer Ubuntu, puis :
cp -r /mnt/c/Users/Lenovo/monero-marketplace ~/monero-marketplace
cd ~/monero-marketplace
chmod +x scripts/ubuntu-setup.sh
./scripts/ubuntu-setup.sh

# 4. Profit!
cargo build --workspace  # ✅ Compile
```

---

🦀🔐 **C'est parti pour Linux !**
