#!/bin/bash
set -euo pipefail

# ============================================================================
# Configuration & Arguments
# ============================================================================
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <environment> <version>"
    echo "Example: $0 staging 123abc456"
    exit 1
fi

ENVIRONMENT=$1
VERSION=$2
COMPOSE_FILE="docker-compose.yml"
# Use prod override if in production
if [ "$ENVIRONMENT" == "production" ]; then
    COMPOSE_FILE="docker-compose.prod.yml"
fi

# ============================================================================
# Script Logic
# ============================================================================
echo "🚀 Deploying version ${VERSION} to ${ENVIRONMENT}..."

# 1. Backup database before deploy
echo "  - Backing up database..."
./backup-database.sh

# 2. Pull new Docker image
echo "  - Pulling new image: ghcr.io/your-repo/monero-marketplace:${VERSION}"
docker pull "ghcr.io/your-repo/monero-marketplace:${VERSION}"

# 3. Rolling update
echo "  - Performing rolling update..."
docker-compose -f "$COMPOSE_FILE" up -d --no-deps server

# 4. Health check
echo "  - Waiting for health check..."
sleep 10 # Give it a moment to start
if ! ./docker-health-check.sh; then
    echo "❌ Health check failed!"
    echo "  - Initiating rollback..."
    ./rollback.sh "$ENVIRONMENT"
    exit 1
fi

echo "✅ Deployment of ${VERSION} to ${ENVIRONMENT} successful."
