#!/bin/bash
set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
BACKUP_DIR="/backups/wallets"
WALLET_BASE_DIR="/wallets"
WALLETS=("buyer" "vendor" "arbiter")
RETENTION_DAYS=90
GPG_RECIPIENT="backup-key@monero-marketplace.local" # GPG Key ID or email

# ============================================================================
# Pre-flight Checks
# ============================================================================
if ! command -v tar &> /dev/null; then
    echo "❌ tar could not be found"
    exit 1
fi
if ! command -v gpg &> /dev/null; then
    echo "❌ gpg could not be found"
    exit 1
fi

# ============================================================================
# Backup Process
# ============================================================================
echo "🚀 Starting wallet backups..."
mkdir -p "$BACKUP_DIR"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

for wallet in "${WALLETS[@]}"; do
    WALLET_PATH="${WALLET_BASE_DIR}/${wallet}"
    if [ ! -d "$WALLET_PATH" ]; then
        echo "⚠️ Wallet directory not found, skipping: $WALLET_PATH"
        continue
    fi

    echo "  - Backing up wallet: $wallet"
    BACKUP_UUID=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || uuidgen 2>/dev/null || echo "${RANDOM}${RANDOM}")
    BACKUP_FILENAME="wallet-${BACKUP_UUID}-${TIMESTAMP}"
    TAR_PATH="/tmp/${BACKUP_FILENAME}.tar.gz"
    FINAL_BACKUP_PATH="${BACKUP_DIR}/${BACKUP_FILENAME}.tar.gz.gpg"

    # 1. Create compressed archive
    echo "    - Archiving and compressing..."
    tar -czf "$TAR_PATH" -C "$WALLET_BASE_DIR" "$wallet"

    # 2. Encrypt the archive
    echo "    - Encrypting with GPG..."
    gpg --encrypt --recipient "$GPG_RECIPIENT" --output "$FINAL_BACKUP_PATH" "$TAR_PATH"

    # 3. Cleanup temporary file
    echo "    - Cleaning up temporary file..."
    rm -f "$TAR_PATH"

    echo "    -> Backup complete: $FINAL_BACKUP_PATH"
done

# ============================================================================
# Cleanup Old Backups
# ============================================================================
echo "  - Cleaning up old backups (retention: ${RETENTION_DAYS} days)..."
find "$BACKUP_DIR" -type f -name "*.gpg" -mtime +"$RETENTION_DAYS" -exec rm -v {} \;

echo "✅ All wallet backups complete."
