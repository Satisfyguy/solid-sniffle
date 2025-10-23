#!/bin/bash
set -euo pipefail

# ============================================================================
# Script Logic
# ============================================================================
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <wallet_name> <path_to_backup_file.gpg>"
    echo "Example: $0 buyer /backups/wallets/wallet-buyer-2025-10-21.tar.gz.gpg"
    exit 1
fi

WALLET_NAME=$1
BACKUP_FILE=$2
WALLET_DIR="/wallets/${WALLET_NAME}"
CONTAINER_NAME="monero-wallet-rpc-${WALLET_NAME}"

# 1. Pre-flight checks
if [[ ! " buyer vendor arbiter " =~ " ${WALLET_NAME} " ]]; then
    echo "❌ Invalid wallet name. Must be one of: buyer, vendor, arbiter."
    exit 1
fi
if [ ! -f "$BACKUP_FILE" ]; then
    echo "❌ Backup file not found: $BACKUP_FILE"
    exit 1
fi
if ! command -v docker &> /dev/null; then
    echo "❌ docker command not found"
    exit 1
fi

# 2. User Confirmation
echo "⚠️ WARNING: This will overwrite the current wallet files for '${WALLET_NAME}'."
read -p "Are you sure you want to continue? (yes/no) " -r
echo
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo "Aborting."
    exit 1
fi

# 3. Stop wallet RPC
echo "  - Stopping wallet RPC container: $CONTAINER_NAME..."
docker stop "$CONTAINER_NAME"

# 4. Backup current wallet files (if they exist)
if [ -d "$WALLET_DIR" ]; then
    echo "  - Backing up current wallet files to ${WALLET_DIR}.bak..."
    mv "$WALLET_DIR" "${WALLET_DIR}.bak"
fi
mkdir -p "$WALLET_DIR"

# 5. Decrypt and extract
echo "  - Decrypting and extracting backup..."
gpg --decrypt --output - "$BACKUP_FILE" | tar -xzf - -C "/"

# 6. Verify wallet files
# A simple check to see if the main wallet file and its keys file exist.
echo "  - Verifying wallet files..."
if [ ! -f "${WALLET_DIR}/${WALLET_NAME}" ] || [ ! -f "${WALLET_DIR}/${WALLET_NAME}.keys" ]; then
    echo "❌ Verification failed: Wallet files not found after restore."
    echo "  - The old wallet is saved at ${WALLET_DIR}.bak"
    exit 1
fi
echo "✅ Verification successful."

# 7. Restart wallet RPC
echo "  - Restarting wallet RPC container..."
docker start "$CONTAINER_NAME"

echo "✅ Wallet '${WALLET_NAME}' restore complete."
