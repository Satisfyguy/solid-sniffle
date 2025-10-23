# Migration Windows → Ubuntu

Guide rapide pour migrer le projet Monero Marketplace de Windows vers Ubuntu.

## Résumé des Changements

### ✅ Ce qui NE change PAS

- **Code Rust** - 100% identique (multi-plateforme)
- **Architecture** - Workspace structure inchangée
- **Tests** - Mêmes tests d'intégration
- **Logique métier** - Monero multisig, Tor, RPC
- **Documentation** - Specs, Reality Checks conservées
- **Git workflow** - Même stratégie de branches

### 🔄 Ce qui change

| Aspect | Windows | Ubuntu |
|--------|---------|--------|
| **Scripts** | `.ps1` (PowerShell) | `.sh` (Bash) |
| **Tor** | Service Windows | `systemctl` daemon |
| **Monero** | `.exe` binaires | ELF binaires |
| **Paths** | `C:\monero-dev` | `~/monero-testnet` |
| **Commandes** | `.\scripts\file.ps1` | `./scripts/file.sh` |
| **Process mgmt** | `Get-Process`, `Stop-Process` | `ps`, `pkill` |

## Migration Rapide (5 étapes)

### 1. Cloner le Repository

```bash
# Sur Ubuntu
git clone <your-repo-url>
cd monero-marketplace
```

### 2. Lancer le Setup Automatique

```bash
# Rend tous les scripts exécutables
chmod +x scripts/*.sh

# Installation complète automatique
./scripts/ubuntu-setup.sh
```

Ce script installe:
- Build tools (gcc, make, pkg-config)
- Tor daemon
- Rust toolchain
- Monero CLI
- Git hooks
- Compile le projet

**Durée**: ~10-15 minutes (selon connexion internet)

### 3. Configurer l'Environnement

Ajouter à `~/.bashrc`:

```bash
# Monero Marketplace Development
export MONERO_TESTNET_PATH="$HOME/monero-testnet"
export PATH="$PATH:$MONERO_TESTNET_PATH/monero-x86_64-linux-gnu-v0.18.3.1"

# Rust
export PATH="$PATH:$HOME/.cargo/bin"

# Aliases pratiques
alias test-wallet="cargo test --package wallet -- --nocapture"
alias test-all="cargo test --workspace"
alias lint="cargo clippy --workspace -- -D warnings"
alias fmt="cargo fmt --workspace"
alias precommit="./scripts/pre-commit.sh"
```

Puis recharger:
```bash
source ~/.bashrc
```

### 4. Démarrer Monero Testnet

```bash
# Setup wallet testnet + RPC
./scripts/setup-monero-testnet.sh

# Tester la connexion
./scripts/test-rpc.sh
```

### 5. Vérifier l'Installation

```bash
# Check complet de l'environnement
./scripts/check-environment.sh

# Build du projet
cargo build --workspace

# Tests
cargo test --workspace
```

## Tableau de Correspondance des Scripts

### Scripts Critiques

| Windows | Ubuntu | Description |
|---------|--------|-------------|
| `.\scripts\ubuntu-setup.sh` | `./scripts/ubuntu-setup.sh` | **NOUVEAU** - Setup auto Ubuntu |
| `.\scripts\pre-commit.ps1` | `./scripts/pre-commit.sh` | Pre-commit checks |
| `.\scripts\check-security-theatre-simple.ps1` | `./scripts/check-security-theatre.sh` | Détection security theatre |
| `.\scripts\setup-monero-testnet.ps1` | `./scripts/setup-monero-testnet.sh` | Setup Monero testnet |
| `.\scripts\test-rpc.ps1` | `./scripts/test-rpc.sh` | Test RPC connectivity |
| `.\scripts\check-environment.sh` | `./scripts/check-environment.sh` | **NOUVEAU** - Verify environment |

### Scripts à Convertir (si besoin)

Scripts PowerShell qui n'ont PAS encore d'équivalent Bash:
- `security-dashboard.ps1` → `security-dashboard.sh` (à créer)
- `security-alerts.ps1` → `security-alerts.sh` (à créer)
- `new-spec.ps1` → `new-spec.sh` (à créer)
- `auto-reality-check-tor.ps1` → `auto-reality-check-tor.sh` (à créer)
- `validate-reality-check-tor.ps1` → `validate-reality-check-tor.sh` (à créer)

## Commandes Équivalentes

### Gestion des Processus

```bash
# Windows PowerShell
Get-Process monerod
Stop-Process -Name monerod -Force

# Ubuntu Bash
ps aux | grep monerod
pgrep monerod
pkill monerod
pkill -9 monerod  # Force kill
```

### Services

```bash
# Windows
Start-Process monerod -ArgumentList "--testnet","--detach"

# Ubuntu
systemctl start tor
systemctl status tor
monerod --testnet --detach
```

### Tests de Fichiers/Dossiers

```bash
# Windows PowerShell
Test-Path "file.txt"
if (Test-Path "file.txt") { ... }

# Ubuntu Bash
[[ -f "file.txt" ]]
if [[ -f "file.txt" ]]; then ... fi
[[ -d "directory" ]]  # Pour dossiers
```

### Réseau

```bash
# Windows
Invoke-RestMethod -Uri "http://..." -Method Post

# Ubuntu
curl -X POST http://...
curl -s --max-time 5 -X POST http://...
```

## Installation Tor (Détails)

### Installation

```bash
sudo apt update
sudo apt install -y tor

# Démarrer et activer au boot
sudo systemctl start tor
sudo systemctl enable tor

# Vérifier status
systemctl status tor
```

### Configuration (`/etc/tor/torrc`)

```
# SOCKS proxy
SOCKSPort 127.0.0.1:9050

# Sécurité
SOCKSPolicy reject *
```

### Test Tor

```bash
# Test SOCKS proxy
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org

# Should return: "Congratulations. This browser is configured to use Tor."
```

## Installation Monero (Détails)

### Téléchargement

```bash
mkdir -p ~/monero-testnet
cd ~/monero-testnet

# Download latest
wget https://downloads.getmonero.org/cli/linux64 -O monero-linux.tar.bz2

# Extract
tar -xjf monero-linux.tar.bz2
rm monero-linux.tar.bz2

# Find binaries
find . -name "monerod"
```

### Démarrage Testnet

```bash
# Daemon
monerod --testnet --detach

# Wallet RPC
monero-wallet-rpc \
  --testnet \
  --wallet-file buyer \
  --password "" \
  --rpc-bind-ip 127.0.0.1 \
  --rpc-bind-port 18082 \
  --disable-rpc-login \
  --daemon-address 127.0.0.1:28081 \
  --detach
```

## Différences de Développement

### Build

```bash
# Identique sur Windows et Ubuntu
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --workspace
```

### Git Hooks

```bash
# Windows
.git\hooks\pre-commit  (fichier texte)

# Ubuntu
.git/hooks/pre-commit  (script exécutable)
chmod +x .git/hooks/pre-commit

# Ou symlink
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
```

## Avantages Ubuntu

### Performance

- **Compilation Rust** - Souvent 20-30% plus rapide
- **Synchronisation Monero** - I/O disk plus rapide
- **Tor** - Daemon natif, plus stable

### Outils

- **Native CLI tools** - `grep`, `sed`, `awk`, `jq` intégrés
- **systemd** - Gestion services simplifiée
- **Package manager** - `apt install` vs téléchargements manuels

### OPSEC

- **Firewall natif** - `ufw` simplifié
- **Meilleure isolation** - Containers, VMs natives
- **Outils réseau** - `netstat`, `ss`, `iptables` natifs

### Production

- **Environnement similaire** - Production probablement Linux
- **Docker** - Déploiement simplifié
- **CI/CD** - GitHub Actions/GitLab CI natifs

## Pièges Courants

### 1. Permissions Scripts

**Problème**: `Permission denied` lors de l'exécution

**Solution**:
```bash
chmod +x scripts/*.sh
chmod +x .git/hooks/pre-commit
```

### 2. Tor Non Démarré

**Problème**: `curl: (7) Failed to connect to 127.0.0.1 port 9050`

**Solution**:
```bash
sudo systemctl start tor
systemctl status tor
```

### 3. Monero RPC Non Accessible

**Problème**: Tests RPC échouent

**Solution**:
```bash
# Vérifier processus
ps aux | grep monero

# Redémarrer
pkill monerod
pkill monero-wallet-rpc
./scripts/setup-monero-testnet.sh
```

### 4. Path Non Trouvé

**Problème**: `command not found: monerod`

**Solution**:
```bash
# Ajouter à PATH dans ~/.bashrc
export PATH="$PATH:$HOME/monero-testnet/monero-x86_64-linux-gnu-v0.18.3.1"
source ~/.bashrc
```

### 5. Git Hooks Non Exécutables

**Problème**: Pre-commit hook ne s'exécute pas

**Solution**:
```bash
chmod +x .git/hooks/pre-commit
# Ou recréer symlink
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
```

## Checklist Post-Migration

- [ ] Ubuntu installé (20.04+)
- [ ] Script `ubuntu-setup.sh` exécuté
- [ ] Tor daemon running (`systemctl status tor`)
- [ ] Rust installé (`rustc --version`)
- [ ] Monero CLI installé (`monerod --version`)
- [ ] Projet compile (`cargo build --workspace`)
- [ ] Tests passent (`cargo test --workspace`)
- [ ] Pre-commit hook fonctionne (`./scripts/pre-commit.sh`)
- [ ] RPC Monero accessible (`./scripts/test-rpc.sh`)
- [ ] Environment check OK (`./scripts/check-environment.sh`)
- [ ] `~/.bashrc` configuré avec aliases
- [ ] Git hooks installés

## Prochaines Étapes

1. **Lire la doc mise à jour**
   - [CLAUDE.md](CLAUDE.md) - Maintenant avec commandes Ubuntu
   - [UBUNTU-SETUP.md](UBUNTU-SETUP.md) - Guide détaillé Ubuntu

2. **Tester le workflow complet**
   ```bash
   # Créer une branche test
   git checkout -b test-ubuntu-migration

   # Faire un changement mineur
   echo "// Test Ubuntu" >> common/src/lib.rs

   # Commit (pre-commit hook devrait s'exécuter)
   git add .
   git commit -m "Test Ubuntu migration"
   ```

3. **Créer les scripts manquants** (si besoin)
   - Convertir scripts PowerShell restants en Bash
   - Voir section "Scripts à Convertir"

4. **Documenter vos découvertes**
   - Ajouter vos notes dans ce fichier
   - Créer des issues pour problèmes rencontrés

## Support

### Problème avec le Setup

```bash
# Vérifier l'environnement
./scripts/check-environment.sh

# Logs détaillés
./scripts/ubuntu-setup.sh 2>&1 | tee setup.log
```

### Problème avec Monero

```bash
# Logs Monero daemon
tail -f ~/monero-testnet/monerod.log

# Logs wallet RPC
tail -f ~/monero-testnet/wallet-rpc.log
```

### Problème avec Tor

```bash
# Logs Tor
sudo journalctl -u tor -n 100 -f

# Restart Tor
sudo systemctl restart tor
```

## Ressources

- [UBUNTU-SETUP.md](UBUNTU-SETUP.md) - Guide détaillé setup Ubuntu
- [scripts/README.md](scripts/README.md) - Documentation scripts
- [CLAUDE.md](CLAUDE.md) - Guidelines de développement (màj Ubuntu)

---

**Migration Status**: ✅ Ready for Ubuntu

**Scripts Created**:
- ✅ `ubuntu-setup.sh` - Setup automatique
- ✅ `pre-commit.sh` - Pre-commit checks
- ✅ `check-security-theatre.sh` - Security theatre detection
- ✅ `setup-monero-testnet.sh` - Monero testnet setup
- ✅ `test-rpc.sh` - RPC connectivity test
- ✅ `check-environment.sh` - Environment verification

**Documentation Updated**:
- ✅ `CLAUDE.md` - Commandes PowerShell → Bash
- ✅ `UBUNTU-SETUP.md` - Guide installation Ubuntu
- ✅ `scripts/README.md` - Documentation scripts
- ✅ `MIGRATION-UBUNTU.md` - Ce guide

**Last Updated**: 2025-10-16
