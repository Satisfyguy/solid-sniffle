#!/bin/bash
set -euo pipefail

echo "🚀 Starting Backup/Restore Test..."
echo "===================================="

# ============================================================================
# Configuration
# ============================================================================
TEST_DIR="/tmp/backup-test"
DB_NAME="test-marketplace.db"
DB_PATH="${TEST_DIR}/${DB_NAME}"
BACKUP_DIR="${TEST_DIR}/backups"
GPG_RECIPIENT="backup-key@monero-marketplace.local" # Must match backup script

# ============================================================================
# Setup
# ============================================================================
echo "  - Setting up test environment in ${TEST_DIR}..."
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR" "$BACKUP_DIR"

# Create a dummy GPG key for testing if it doesn't exist
if ! gpg --list-keys "$GPG_RECIPIENT" > /dev/null 2>&1; then
    echo "  - Creating dummy GPG key for testing..."
    gpg --batch --passphrase '' --quick-gen-key "$GPG_RECIPIENT" default default
fi

# ============================================================================
# 1. Create Test Database
# ============================================================================
echo "1. Creating test database..."
sqlite3 "$DB_PATH" "CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT);"
sqlite3 "$DB_PATH" "INSERT INTO test (data) VALUES ('hello world');"
INITIAL_DATA=$(sqlite3 "$DB_PATH" "SELECT data FROM test WHERE id = 1;")
echo "   - Initial data: '$INITIAL_DATA'"

# ============================================================================
# 2. Run Backup
# ============================================================================
echo "2. Running backup script..."
# We need to temporarily override the paths used by the backup script
DB_PATH_ORIG=$DB_PATH \
BACKUP_DIR_ORIG=$BACKUP_DIR \
GPG_RECIPIENT_ORIG=$GPG_RECIPIENT \
    sed -e "s|^DB_PATH=.*|DB_PATH=\"${DB_PATH}\"|" \
        -e "s|^BACKUP_DIR=.*|BACKUP_DIR=\"${BACKUP_DIR}\"|" \
        -e "s|^GPG_RECIPIENT=.*|GPG_RECIPIENT=\"${GPG_RECIPIENT}\"|" \
        ./backup-database.sh > /dev/null

BACKUP_FILE=$(find "$BACKUP_DIR" -name "*.gpg" | head -n 1)
if [ -z "$BACKUP_FILE" ]; then
    echo "❌ Backup file not created."
    exit 1
fi
echo "   - Backup created: $(basename "$BACKUP_FILE")"

# ============================================================================
# 3. Corrupt Database
# ============================================================================
echo "3. Corrupting database..."
dd if=/dev/urandom of="$DB_PATH" bs=1k count=1 conv=notrunc
CORRUPTION_CHECK=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check;")
if [ "$CORRUPTION_CHECK" == "ok" ]; then
    echo "❌ Database corruption failed. Test is invalid."
    exit 1
fi
echo "   - Corruption confirmed: $CORRUPTION_CHECK"

# ============================================================================
# 4. Restore from Backup
# ============================================================================
echo "4. Restoring from backup..."
# Temporarily modify and run the restore script
# Note: This is a simplified version, as the original script has interactive prompts
# and docker commands. We will simulate the core restore logic.
TMP_RESTORED_DB="/tmp/test-restored.db"
gpg --decrypt --output - "$BACKUP_FILE" | gunzip > "$TMP_RESTORED_DB"
rm "$DB_PATH"
sqlite3 "$DB_PATH" ".restore '$TMP_RESTORED_DB'"
rm "$TMP_RESTORED_DB"
echo "   - Restore command executed."

# ============================================================================
# 5. Verify Integrity and Data
# ============================================================================
echo "5. Verifying restored database..."
FINAL_INTEGRITY_CHECK=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check;")
if [ "$FINAL_INTEGRITY_CHECK" != "ok" ]; then
    echo "❌ Integrity check failed after restore: $FINAL_INTEGRITY_CHECK"
    exit 1
fi
echo "   - Integrity check: OK"

FINAL_DATA=$(sqlite3 "$DB_PATH" "SELECT data FROM test WHERE id = 1;")
echo "   - Restored data: '$FINAL_DATA'"
if [ "$INITIAL_DATA" != "$FINAL_DATA" ]; then
    echo "❌ Data mismatch after restore!"
    exit 1
fi
echo "   - Data verification: OK"

# ============================================================================
# Cleanup
# ============================================================================
echo "6. Cleaning up..."
rm -rf "$TEST_DIR"

echo ""
echo "✅ Backup and restore test completed successfully!"
