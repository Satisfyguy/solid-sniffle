#!/bin/bash
set -euo pipefail

# ============================================================================
# Configuration & Arguments
# ============================================================================
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <environment>"
    echo "Example: $0 staging"
    exit 1
fi

ENVIRONMENT=$1
COMPOSE_FILE="docker-compose.yml"
if [ "$ENVIRONMENT" == "production" ]; then
    COMPOSE_FILE="docker-compose.prod.yml"
fi

# ============================================================================
# Script Logic
# =================================_==========================================
echo "↩️ Rolling back deployment in ${ENVIRONMENT}..."

# 1. Get previous image tag (second to last image)
PREVIOUS_VERSION=$(docker images --format "{{.Tag}}" "ghcr.io/your-repo/monero-marketplace" | grep -v "latest" | head -n 2 | tail -n 1)
if [ -z "$PREVIOUS_VERSION" ]; then
    echo "❌ Could not determine previous version. Manual intervention required."
    exit 1
fi
echo "  - Rolling back to version: ${PREVIOUS_VERSION}"

# 2. Stop current environment
echo "  - Stopping current services..."
docker-compose -f "$COMPOSE_FILE" stop

# 3. Restore database from the most recent backup
echo "  - Restoring database..."
LATEST_BACKUP=$(find /backups/database -name "*.gpg" -printf "%T@ %p\n" | sort -n | tail -1 | cut -d' ' -f2-)
if [ -z "$LATEST_BACKUP" ]; then
    echo "⚠️ Could not find a database backup to restore. Skipping."
else
    ./restore-database.sh "$LATEST_BACKUP"
fi

# 4. Start with previous image
echo "  - Starting services with previous version..."
# We need to override the image tag in the docker-compose file
IMAGE_TAG=$PREVIOUS_VERSION docker-compose -f "$COMPOSE_FILE" up -d

echo "✅ Rollback to ${PREVIOUS_VERSION} complete."
