# Ubuntu Setup Guide - Monero Marketplace

Complete guide for setting up the Monero Marketplace development environment on Ubuntu.

## Prerequisites

- Ubuntu 20.04+ (tested on 22.04 LTS)
- sudo privileges
- Internet connection

## Quick Start

```bash
# Clone repository
git clone <your-repo-url>
cd monero-marketplace

# Run automated setup
./scripts/ubuntu-setup.sh

# Verify installation
cargo --version
tor --version
```

## Manual Installation

### 1. Install System Dependencies

```bash
# Update package list
sudo apt update && sudo apt upgrade -y

# Install build essentials
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    wget \
    git \
    expect

# Install Tor
sudo apt install -y tor
sudo systemctl enable tor
sudo systemctl start tor

# Verify Tor is running
systemctl status tor
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org
```

### 2. Install Rust

```bash
# Install rustup (Rust toolchain manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Select option 1 (default installation)

# Load Rust environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version

# Install required components
rustup component add rustfmt clippy
```

### 3. Install Monero CLI (Testnet)

```bash
# Create Monero directory
mkdir -p ~/monero-testnet
cd ~/monero-testnet

# Download latest Monero CLI
wget https://downloads.getmonero.org/cli/linux64 -O monero-linux.tar.bz2

# Extract
tar -xjf monero-linux.tar.bz2
rm monero-linux.tar.bz2

# Find extracted directory
MONERO_DIR=$(find . -maxdepth 1 -type d -name "monero-*" | head -n 1)

# Add to PATH (optional)
echo "export PATH=\$PATH:$HOME/monero-testnet/$MONERO_DIR" >> ~/.bashrc
source ~/.bashrc

# Verify installation
monerod --version
```

### 4. Configure Git Hooks

```bash
cd ~/monero-marketplace

# Make all scripts executable
chmod +x scripts/*.sh

# Install pre-commit hook
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Test pre-commit hook
./scripts/pre-commit.sh
```

## Project Setup

### 1. Build the Project

```bash
# Build all workspace members
cargo build --workspace

# Run all tests
cargo test --workspace

# Run clippy (strict linting)
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --workspace
```

### 2. Start Monero Testnet

```bash
# Start testnet daemon
monerod --testnet --detach

# Wait for sync (or use stagenet for faster sync)
# Check status:
monerod --testnet status

# Create and start wallet RPC
./scripts/setup-monero-testnet.sh --wallet buyer

# Test RPC connection
./scripts/test-rpc.sh
```

### 3. Run Integration Tests

```bash
# Ensure Monero RPC is running on localhost:18082
curl -X POST http://127.0.0.1:18082/json_rpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}'

# Run wallet integration tests
cargo test --package wallet -- --nocapture

# Run specific test
cargo test --package wallet test_prepare_multisig -- --nocapture
```

## Security Configuration

### Tor Configuration

Default Tor configuration (`/etc/tor/torrc`) should include:

```
# SOCKS proxy for applications
SOCKSPort 127.0.0.1:9050

# Ensure no external access
SOCKSPolicy reject *
```

Restart Tor after changes:
```bash
sudo systemctl restart tor
```

### Monero RPC Security

**CRITICAL**: Monero RPC must ONLY bind to localhost:

```bash
# CORRECT (localhost only)
monero-wallet-rpc --rpc-bind-ip 127.0.0.1 --rpc-bind-port 18082

# NEVER DO THIS (public exposure)
monero-wallet-rpc --rpc-bind-ip 0.0.0.0  # ❌ DANGEROUS
```

### Firewall Configuration (Optional but Recommended)

```bash
# Allow only localhost access to Monero RPC
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow from 127.0.0.1 to 127.0.0.1 port 18082
sudo ufw enable

# Verify
sudo ufw status
```

## Development Workflow

### Before Each Commit

```bash
# Run pre-commit checks
./scripts/pre-commit.sh

# Or let git hook run automatically
git commit -m "your message"
```

### Security Checks

```bash
# Check for security theatre
./scripts/check-security-theatre.sh --verbose

# Security dashboard
./scripts/security-dashboard.sh

# Security alerts
./scripts/security-alerts.sh
```

### Reality Checks (Network Functions)

```bash
# Create reality check for function with network calls
./scripts/auto-reality-check-tor.sh function_name

# Validate reality check
./scripts/validate-reality-check-tor.sh function_name
```

## Troubleshooting

### Tor Not Working

```bash
# Check Tor status
systemctl status tor

# Check Tor logs
sudo journalctl -u tor -n 50

# Restart Tor
sudo systemctl restart tor

# Test Tor connection
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org
```

### Monero RPC Not Accessible

```bash
# Check if monerod is running
ps aux | grep monerod

# Check if wallet-rpc is running
ps aux | grep monero-wallet-rpc

# Kill stuck processes
pkill monerod
pkill monero-wallet-rpc

# Restart testnet
./scripts/setup-monero-testnet.sh
```

### Build Failures

```bash
# Clean build artifacts
cargo clean

# Update Rust
rustup update

# Rebuild
cargo build --workspace
```

### Permission Denied Errors

```bash
# Make all scripts executable
chmod +x scripts/*.sh

# Fix git hooks
chmod +x .git/hooks/pre-commit
```

## Environment Variables

Create `~/.bashrc` additions:

```bash
# Monero testnet path
export MONERO_TESTNET_PATH="$HOME/monero-testnet"
export PATH="$PATH:$MONERO_TESTNET_PATH/monero-x86_64-linux-gnu-v0.18.3.1"

# Rust cargo
export PATH="$PATH:$HOME/.cargo/bin"

# Development aliases
alias test-wallet="cargo test --package wallet -- --nocapture"
alias test-all="cargo test --workspace"
alias lint="cargo clippy --workspace -- -D warnings"
alias fmt="cargo fmt --workspace"
alias precommit="./scripts/pre-commit.sh"
```

Reload:
```bash
source ~/.bashrc
```

## Key Differences from Windows

| Windows | Ubuntu |
|---------|--------|
| `.\scripts\file.ps1` | `./scripts/file.sh` |
| `monerod.exe` | `monerod` |
| `C:\monero-dev` | `~/monero-testnet` |
| PowerShell | Bash |
| `Get-Process` | `ps aux \| grep` |
| `Stop-Process` | `pkill` |

## Next Steps

1. **Read project documentation**
   - [CLAUDE.md](CLAUDE.md) - Development guidelines
   - [docs/DEVELOPER-GUIDE.md](docs/DEVELOPER-GUIDE.md) - Detailed dev guide
   - [docs/SECURITY-THEATRE-PREVENTION.md](docs/SECURITY-THEATRE-PREVENTION.md)

2. **Run the test suite**
   ```bash
   cargo test --workspace
   ```

3. **Start developing**
   - Follow spec-driven development workflow
   - Run security checks before commits
   - Create Reality Checks for network functions

4. **Join the community** (if applicable)
   - Report issues
   - Contribute improvements

## OPSEC Reminders for Ubuntu

- Never expose Monero RPC publicly (bind to 127.0.0.1 ONLY)
- Always route external connections through Tor (SOCKS5 proxy)
- Never log sensitive data (.onion addresses, keys, IPs)
- Use generic User-Agent strings in HTTP requests
- Check firewall rules regularly

## Resources

- [Monero Documentation](https://www.getmonero.org/resources/developer-guides/)
- [Tor Project](https://www.torproject.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Project Specifications](docs/specs/)
- [Reality Checks](docs/reality-checks/)

---

**Status**: Ready for Ubuntu development ✅

**Last Updated**: 2025-10-16
