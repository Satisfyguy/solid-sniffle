#!/bin/bash
# Apply shipping migration to SQLCipher database
# This script applies the migration manually since diesel CLI doesn't work with encrypted DBs

DB_PATH="marketplace.db"
ENCRYPTION_KEY="1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724"

echo "🔒 Applying shipping migration to encrypted database..."
echo "📍 Database: $DB_PATH"

# Apply the migration using sqlite3 with pragma key
sqlite3 "$DB_PATH" <<EOF
PRAGMA key = '$ENCRYPTION_KEY';

-- Check if columns already exist
SELECT CASE
    WHEN COUNT(*) > 0 THEN 'Columns already exist, skipping migration'
    ELSE 'Applying migration...'
END as status
FROM pragma_table_info('orders')
WHERE name IN ('shipping_address', 'shipping_notes');

-- Apply migration (will fail silently if columns exist)
ALTER TABLE orders ADD COLUMN shipping_address TEXT;
ALTER TABLE orders ADD COLUMN shipping_notes TEXT;

-- Verify
SELECT '✅ Migration complete! Columns added:' as result;
SELECT name, type FROM pragma_table_info('orders') WHERE name IN ('shipping_address', 'shipping_notes');
EOF

echo ""
echo "✅ Done! Server can now be started."
