#!/bin/bash
# Install latest Node.js LTS version
# Run with: sudo bash install-nodejs-latest.sh

set -e

echo "========================================="
echo "  Node.js Latest LTS Installation"
echo "========================================="
echo ""

# Check current version
if command -v node &> /dev/null; then
    CURRENT_VERSION=$(node --version)
    echo "Current Node.js version: $CURRENT_VERSION"
else
    echo "Node.js is not currently installed"
fi

echo ""
echo "Installing latest LTS version..."

# Remove old NodeSource repository if exists
echo "Cleaning up old repositories..."
sudo rm -f /etc/apt/sources.list.d/nodesource.list
sudo rm -f /etc/apt/keyrings/nodesource.gpg

# Download and setup NodeSource LTS repository
echo "Adding NodeSource LTS repository..."
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -

# Install Node.js
echo ""
echo "Installing Node.js..."
sudo apt-get install -y nodejs

# Verify installation
echo ""
echo "========================================="
echo "  Installation Complete!"
echo "========================================="
echo ""

NODE_VERSION=$(node --version)
NPM_VERSION=$(npm --version)

echo "Node.js version: $NODE_VERSION"
echo "npm version: $NPM_VERSION"

# Check if npm global directory needs fixing
NPM_PREFIX=$(npm config get prefix)
echo "npm prefix: $NPM_PREFIX"

# Optional: Setup npm global packages without sudo
echo ""
echo "Setting up npm global packages directory..."
mkdir -p ~/.npm-global
npm config set prefix '~/.npm-global'

# Add to PATH if not already there
if ! grep -q '.npm-global/bin' ~/.bashrc; then
    echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
    echo "Added npm global bin to PATH in ~/.bashrc"
    echo "Run: source ~/.bashrc"
fi

echo ""
echo "✅ Node.js installation complete!"
echo ""
echo "To verify:"
echo "  node --version"
echo "  npm --version"
echo ""
echo "To use the new PATH (for npm global packages):"
echo "  source ~/.bashrc"
