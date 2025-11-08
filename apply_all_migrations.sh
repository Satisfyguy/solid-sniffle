#!/bin/bash
set -e

DB_PATH="data/marketplace-dev.db"
ENCRYPTION_KEY="b9fa942239ade83c0be8e19dc9ede609116bc735ece6f01652be6554766f93db"
MIGRATIONS_DIR="server/migrations"

echo "🚀 Applying all migrations to a fresh, encrypted database..."

# 1. Delete existing database file
rm -f "$DB_PATH"
echo "🔥 Deleted existing database file."

# 2. Create a new, empty database file
sqlite3 "$DB_PATH" ".databases"
echo "✨ Created new, empty database file."

# 3. Apply all migrations in order
for migration in $(ls -d "$MIGRATIONS_DIR"/*/ | sort); do
    up_sql_file="${migration}up.sql"
    if [ -f "$up_sql_file" ]; then
        echo "Applying migration: $(basename "$migration")"
        sqlite3 "$DB_PATH" <<EOF
PRAGMA key = '$ENCRYPTION_KEY';
$(cat "$up_sql_file")
EOF
    fi
done

echo "✅ All migrations applied successfully!"
