#!/bin/bash
set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
BACKUP_DIR="/backups/database"
DB_PATH="/app/data/marketplace.db"
RETENTION_DAYS=30
GPG_RECIPIENT="backup-key@monero-marketplace.local" # GPG Key ID or email

# ============================================================================
# Pre-flight Checks
# ============================================================================
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 could not be found"
    exit 1
fi
if ! command -v gpg &> /dev/null; then
    echo "❌ gpg could not be found"
    exit 1
fi
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Database not found at $DB_PATH"
    exit 1
fi

# ============================================================================
# Backup Process
# ============================================================================
echo "🚀 Starting database backup..."

mkdir -p "$BACKUP_DIR"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
BACKUP_FILENAME="marketplace-db-${TIMESTAMP}"
TMP_BACKUP_PATH="/tmp/${BACKUP_FILENAME}.db"
FINAL_BACKUP_PATH="${BACKUP_DIR}/${BACKUP_FILENAME}.db.gz.gpg"
CHECKSUM_FILE="${BACKUP_DIR}/${BACKUP_FILENAME}.sha256"

# 1. Backup SQLCipher database
echo "  - Dumping database..."
sqlite3 "$DB_PATH" ".backup '$TMP_BACKUP_PATH'"

# 2. Verify integrity of the backup
echo "  - Verifying integrity..."
INTEGRITY_CHECK=$(sqlite3 "$TMP_BACKUP_PATH" "PRAGMA integrity_check;")
if [ "$INTEGRITY_CHECK" != "ok" ]; then
    echo "❌ Integrity check failed: $INTEGRITY_CHECK"
    rm -f "$TMP_BACKUP_PATH"
    exit 1
fi

# 3. Compress the backup
echo "  - Compressing..."
gzip -9 "$TMP_BACKUP_PATH"
TMP_BACKUP_PATH+=".gz" # Update path to .gz

# 4. Encrypt the backup
echo "  - Encrypting with GPG..."
gpg --encrypt --recipient "$GPG_RECIPIENT" --output "$FINAL_BACKUP_PATH" "$TMP_BACKUP_PATH"

# 5. Calculate SHA256 checksum
echo "  - Calculating checksum..."
sha256sum "$FINAL_BACKUP_PATH" > "$CHECKSUM_FILE"

# 6. Cleanup temporary files
echo "  - Cleaning up temporary files..."
rm -f "$TMP_BACKUP_PATH"

# 7. Cleanup old backups
echo "  - Cleaning up old backups (retention: ${RETENTION_DAYS} days)..."
find "$BACKUP_DIR" -type f -name "*.gpg" -mtime +"$RETENTION_DAYS" -exec rm -v {} \;
find "$BACKUP_DIR" -type f -name "*.sha256" -mtime +"$RETENTION_DAYS" -exec rm -v {} \;

echo "✅ Backup complete: $FINAL_BACKUP_PATH"
