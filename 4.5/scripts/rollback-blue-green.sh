#!/bin/bash
set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
NGINX_CONF_PATH="/etc/nginx/nginx.conf"
UPSTREAM_CONF_PATH="/etc/nginx/upstream.conf"

# ============================================================================
# Script Logic
# ============================================================================
echo "↩️ Initiating Blue/Green Rollback..."

# Determine current active environment
CURRENT_ACTIVE=$(grep "weight=100" "$UPSTREAM_CONF_PATH" | awk '{print $2}' | cut -d':' -f1)
if [ "$CURRENT_ACTIVE" == "server-blue" ]; then
    ROLLBACK_TARGET="server-green"
    FAILED_TARGET="server-blue"
else
    ROLLBACK_TARGET="server-blue"
    FAILED_TARGET="server-green"
fi

echo "  - Current active: ${CURRENT_ACTIVE}, Rolling back to: ${ROLLBACK_TARGET}"

# 1. Switch all traffic back to the rollback target
echo "  - Switching all traffic back to ${ROLLBACK_TARGET}..."
sed -i "s/server ${FAILED_TARGET}:8080 weight=.*;/server ${FAILED_TARGET}:8080 weight=0;/" "$UPSTREAM_CONF_PATH"
sed -i "s/server ${ROLLBACK_TARGET}:8080 weight=.*;/server ${ROLLBACK_TARGET}:8080 weight=100;/" "$UPSTREAM_CONF_PATH"
sudo nginx -s reload

# 2. Stop the failed environment
echo "  - Stopping failed environment (${FAILED_TARGET})..."
docker-compose -f ../docker/docker-compose.blue-green.yml stop "$FAILED_TARGET"
docker-compose -f ../docker/docker-compose.blue-green.yml rm -f "$FAILED_TARGET"

echo "✅ Blue/Green Rollback complete. ${ROLLBACK_TARGET} is now active."
