#!/bin/bash
set -euo pipefail

echo "🚀 Setting up SOPS and Age for Secrets Management..."

# ============================================================================
# Configuration
# ============================================================================
SOPS_VERSION="3.8.1"
AGE_VERSION="1.1.1"
SECRETS_DIR="../security"
SECRETS_FILE="${SECRETS_DIR}/secrets.enc.yaml"
AGE_KEY_FILE="${SECRETS_DIR}/age.key"

# ============================================================================
# Script Logic
# ============================================================================
mkdir -p "$SECRETS_DIR"

# 1. Install SOPS
if ! command -v sops &> /dev/null; then
    echo "  - Installing SOPS v${SOPS_VERSION}..."
    curl -LO "https://github.com/getsops/sops/releases/download/v${SOPS_VERSION}/sops-v${SOPS_VERSION}.linux.amd64"
    sudo mv "sops-v${SOPS_VERSION}.linux.amd64" /usr/local/bin/sops
    sudo chmod +x /usr/local/bin/sops
else
    echo "  - SOPS already installed."
fi

# 2. Install Age
if ! command -v age &> /dev/null; then
    echo "  - Installing Age v${AGE_VERSION}..."
    curl -LO "https://github.com/FiloSottile/age/releases/download/v${AGE_VERSION}/age-v${AGE_VERSION}-linux-amd64.tar.gz"
    tar -xzf "age-v${AGE_VERSION}-linux-amd64.tar.gz"
    sudo mv age/age /usr/local/bin/age
    sudo mv age/age-keygen /usr/local/bin/age-keygen
    rm -rf age age-v${AGE_VERSION}-linux-amd64.tar.gz
else
    echo "  - Age already installed."
fi

# 3. Generate Age key
if [ ! -f "$AGE_KEY_FILE" ]; then
    echo "  - Generating Age key to ${AGE_KEY_FILE}..."
    age-keygen -o "$AGE_KEY_FILE"
    chmod 600 "$AGE_KEY_FILE"
    echo "    Public key: $(cat "$AGE_KEY_FILE" | grep -o 'age1[a-zA-Z0-9]*')"
else
    echo "  - Age key already exists at ${AGE_KEY_FILE}."
fi

# 4. Create secrets.yaml template
if [ ! -f "${SECRETS_FILE}.template" ]; then
    echo "  - Creating secrets template..."
    cat <<EOF > "${SECRETS_FILE}.template"
# This file contains encrypted secrets. DO NOT COMMIT PLAINTEXT SECRETS.
database_password: "your_db_password"
grafana_admin_password: "your_grafana_admin_password"
backup_gpg_passphrase: "your_backup_gpg_passphrase"
EOF
else
    echo "  - Secrets template already exists."
fi

# 5. Encrypt with SOPS
echo "  - Encrypting secrets with SOPS..."
# Ensure SOPS uses the generated age key
export SOPS_AGE_KEY_FILE="$AGE_KEY_FILE"
sops --encrypt --age "$(cat "$AGE_KEY_FILE" | grep -o 'age1[a-zA-Z0-9]*')" \
     --in-place "${SECRETS_FILE}.template"
mv "${SECRETS_FILE}.template" "$SECRETS_FILE"

# 6. Shred plaintext template (optional, but good practice)
# echo "  - Shredding plaintext template..."
# shred -u "${SECRETS_FILE}.template"

echo "✅ SOPS and Age setup complete. Secrets encrypted to ${SECRETS_FILE}."
echo "Remember to add the public key to your .sops.yaml config if using multiple keys."
