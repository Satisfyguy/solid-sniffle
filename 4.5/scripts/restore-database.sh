#!/bin/bash
set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
DB_PATH="/app/data/marketplace.db"
DB_DIR="/app/data"
APP_CONTAINER="marketplace-server" # Docker container name

# ============================================================================
# Script Logic
# ============================================================================
if [ -z "$1" ]; then
    echo "Usage: $0 <path_to_backup_file.gpg>"
    exit 1
fi

BACKUP_FILE=$1

# 1. Pre-flight checks
if [ ! -f "$BACKUP_FILE" ]; then
    echo "❌ Backup file not found: $BACKUP_FILE"
    exit 1
fi
if ! command -v docker &> /dev/null; then
    echo "❌ docker command not found"
    exit 1
fi

# 2. User Confirmation
echo "⚠️ WARNING: This will overwrite the current database."
echo "The current database will be backed up to ${DB_PATH}.bak"
read -p "Are you sure you want to continue? (yes/no) " -r
echo
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo "Aborting."
    exit 1
fi

# 3. Stop application
echo "  - Stopping application server..."
docker stop "$APP_CONTAINER"

# 4. Backup current database
if [ -f "$DB_PATH" ]; then
    echo "  - Backing up current database to ${DB_PATH}.bak..."
    mv "$DB_PATH" "${DB_PATH}.bak"
fi

# 5. Decrypt and decompress
echo "  - Decrypting and decompressing backup..."
TMP_DB_PATH="/tmp/restored.db"
gpg --decrypt --output - "$BACKUP_FILE" | gunzip > "$TMP_DB_PATH"

# 6. Restore with sqlite3
echo "  - Restoring database..."
sqlite3 "$DB_PATH" ".restore '$TMP_DB_PATH'"

# 7. Verify integrity
echo "  - Verifying restored database integrity..."
INTEGRITY_CHECK=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check;")
if [ "$INTEGRITY_CHECK" != "ok" ]; then
    echo "❌ Integrity check failed after restore: $INTEGRITY_CHECK"
    echo "  - The restored database may be corrupt."
    echo "  - The old database is saved at ${DB_PATH}.bak"
    rm -f "$TMP_DB_PATH"
    exit 1
fi

echo "✅ Integrity check passed."

# 8. Cleanup temporary file
rm -f "$TMP_DB_PATH"

# 9. Restart application
echo "  - Restarting application server..."
docker start "$APP_CONTAINER"

echo "✅ Database restore complete."
