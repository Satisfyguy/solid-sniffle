#!/bin/bash
# Install IPFS (Kubo) for Monero Marketplace Reputation System
#
# This script installs IPFS CLI (Kubo) and configures it for local testing.
# For production, additional Tor configuration is required (see IPFS-SETUP.md)

set -e

IPFS_VERSION="v0.25.0"
IPFS_PLATFORM="linux-amd64"
IPFS_DIST_URL="https://dist.ipfs.tech/kubo/${IPFS_VERSION}/kubo_${IPFS_VERSION}_${IPFS_PLATFORM}.tar.gz"

echo "========================================"
echo "  IPFS Installation Script"
echo "  Monero Marketplace Reputation System"
echo "========================================"
echo ""

# Check if IPFS is already installed
if command -v ipfs &> /dev/null; then
    INSTALLED_VERSION=$(ipfs version -n)
    echo "✅ IPFS is already installed: $INSTALLED_VERSION"

    read -p "Do you want to reinstall? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
fi

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "[1/5] Downloading IPFS ${IPFS_VERSION}..."
wget -q --show-progress "$IPFS_DIST_URL" -O kubo.tar.gz

echo ""
echo "[2/5] Extracting archive..."
tar -xzf kubo.tar.gz

echo "[3/5] Installing IPFS binary..."
cd kubo
sudo bash install.sh

echo ""
echo "[4/5] Verifying installation..."
if ! command -v ipfs &> /dev/null; then
    echo "❌ IPFS installation failed"
    rm -rf "$TMP_DIR"
    exit 1
fi

INSTALLED_VERSION=$(ipfs version -n)
echo "✅ IPFS installed successfully: $INSTALLED_VERSION"

# Initialize IPFS if not already initialized
if [ ! -d "$HOME/.ipfs" ]; then
    echo ""
    echo "[5/5] Initializing IPFS repository..."
    ipfs init

    # Configure for local testing (API on localhost only)
    ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001
    ipfs config Addresses.Gateway /ip4/127.0.0.1/tcp/8080

    echo "✅ IPFS repository initialized at $HOME/.ipfs"
else
    echo ""
    echo "[5/5] IPFS repository already exists at $HOME/.ipfs"
fi

# Cleanup
rm -rf "$TMP_DIR"

echo ""
echo "========================================"
echo "  Installation Complete!"
echo "========================================"
echo ""
echo "Next steps:"
echo "  1. Start IPFS daemon:"
echo "     $ ipfs daemon"
echo ""
echo "  2. Test IPFS (in another terminal):"
echo "     $ ipfs swarm peers"
echo "     $ echo 'Hello IPFS!' | ipfs add"
echo ""
echo "  3. For production with Tor, see:"
echo "     reputation/docs/IPFS-SETUP.md"
echo ""
echo "Configuration:"
echo "  - API endpoint: http://127.0.0.1:5001"
echo "  - Gateway: http://127.0.0.1:8080"
echo "  - Config: $HOME/.ipfs/config"
echo ""
