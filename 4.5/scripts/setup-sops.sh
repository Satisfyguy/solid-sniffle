#!/bin/bash
set -euo pipefail

# =============================================================================
# SOPS + Age Setup Script
# =============================================================================
# This script installs SOPS and Age for secret management
# Usage: ./setup-sops.sh

echo "🔐 Setting up SOPS + Age for secret management..."

# Check if running on Ubuntu/Debian
if ! command -v apt &> /dev/null; then
    echo "❌ This script requires Ubuntu/Debian (apt package manager)"
    exit 1
fi

# Install Age
echo "  - Installing Age..."
if ! command -v age &> /dev/null; then
    sudo apt update
    sudo apt install -y age
    echo "    ✅ Age installed"
else
    echo "    ✅ Age already installed"
fi

# Install SOPS
echo "  - Installing SOPS..."
if ! command -v sops &> /dev/null; then
    SOPS_VERSION="3.8.1"
    wget "https://github.com/mozilla/sops/releases/download/v${SOPS_VERSION}/sops-v${SOPS_VERSION}.linux.amd64" -O /tmp/sops
    chmod +x /tmp/sops
    sudo mv /tmp/sops /usr/local/bin/sops
    echo "    ✅ SOPS installed"
else
    echo "    ✅ SOPS already installed"
fi

# Generate Age key if not exists
if [ ! -f "../security/age.key" ]; then
    echo "  - Generating Age key..."
    age-keygen -o "../security/age.key"
    chmod 600 "../security/age.key"

    # Extract public key
    AGE_PUBLIC_KEY=$(grep "# public key:" "../security/age.key" | awk '{print $4}')

    # Create .sops.yaml
    cat > "../.sops.yaml" <<EOF
creation_rules:
  - age: ${AGE_PUBLIC_KEY}
EOF

    echo "    ✅ Age key generated at: 4.5/security/age.key"
    echo "    📋 Public key: ${AGE_PUBLIC_KEY}"
    echo "    ⚠️  BACKUP age.key SECURELY - you cannot decrypt secrets without it!"
else
    echo "    ✅ Age key already exists"
fi

echo ""
echo "✅ SOPS + Age setup complete!"
echo ""
echo "Next steps:"
echo "  1. Edit secrets: sops 4.5/security/secrets.enc.yaml"
echo "  2. The file will be encrypted automatically on save"
echo "  3. To decrypt: sops --decrypt 4.5/security/secrets.enc.yaml"
echo ""
echo "⚠️  IMPORTANT: Backup 4.5/security/age.key in a secure location!"
