# Compilation sur Windows - Résolution des Problèmes

## Problème: Conflit avec Git's link.exe

### Symptôme
```
error: linking with `link.exe` failed: exit code: 1
link: extra operand
```

### Cause
Git for Windows installe son propre `link.exe` (outil Unix) dans `/usr/bin/link.exe`, qui entre en conflit avec le linker MSVC de Rust.

### Solutions

#### Solution 1: Utiliser PowerShell (RECOMMANDÉ)
Plutôt que Git Bash, utilisez PowerShell ou Windows Terminal:
```powershell
# Dans PowerShell
cargo build --workspace
cargo test --workspace
```

#### Solution 2: Renommer temporairement Git's link.exe
```bash
# Dans Git Bash
sudo mv /usr/bin/link.exe /usr/bin/link.exe.backup

# Compiler
cargo build --workspace

# Restaurer
sudo mv /usr/bin/link.exe.backup /usr/bin/link.exe
```

#### Solution 3: Modifier PATH temporairement
```bash
# Dans Git Bash
export PATH=$(echo $PATH | sed 's|/usr/bin:||g')
cargo build --workspace
```

#### Solution 4: Utiliser WSL2 (Pour développement long terme)
```bash
# Installer WSL2 + Ubuntu
wsl --install

# Dans WSL:
sudo apt update
sudo apt install -y build-essential curl git tor monero
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Cloner et compiler
git clone <repo>
cd monero-marketplace
cargo build --workspace
```

## Vérification de l'Installation

```powershell
# Vérifier Rust
rustc --version
cargo --version

# Vérifier targets installés
rustup target list --installed

# Vérifier compilation
cargo check --workspace
```

## État Actuel du Projet

**Version:** 0.1.0-alpha
**Statut:** Développement actif
**Target:** `x86_64-pc-windows-gnu` (évite conflits MSVC)

## Prochaines Étapes

1. Résoudre problème de compilation (PowerShell ou WSL2)
2. Compléter tests multisig end-to-end
3. Démarrer Phase 1 de la roadmap production

## Support

Si le problème persiste:
1. Vérifier que Rust est à jour: `rustup update`
2. Nettoyer le cache: `cargo clean`
3. Réinstaller le target: `rustup target remove x86_64-pc-windows-gnu && rustup target add x86_64-pc-windows-gnu`
