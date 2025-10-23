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
echo "🚀 Starting Blue/Green Deployment..."

# Determine current active environment
CURRENT_ACTIVE=$(grep "weight=100" "$UPSTREAM_CONF_PATH" | awk '{print $2}' | cut -d':' -f1)
if [ "$CURRENT_ACTIVE" == "server-blue" ]; then
    DEPLOY_TARGET="server-green"
    INACTIVE_TARGET="server-blue"
    DEPLOY_PORT="8081"
else
    DEPLOY_TARGET="server-blue"
    INACTIVE_TARGET="server-green"
    DEPLOY_PORT="8080"
fi

echo "  - Current active: ${CURRENT_ACTIVE}, Deploying to: ${DEPLOY_TARGET}"

# 1. Deploy to inactive environment
echo "  - Starting ${DEPLOY_TARGET} container..."
docker-compose -f ../docker/docker-compose.blue-green.yml up -d "$DEPLOY_TARGET"

# 2. Wait for health check
echo "  - Waiting for ${DEPLOY_TARGET} health check..."
for i in {1..30}; do
    if curl -f "http://localhost:${DEPLOY_PORT}/health" > /dev/null 2>&1; then
        echo "✅ ${DEPLOY_TARGET} healthy!"
        break
    fi
    echo "⏳ Attempt $i/30..."
    sleep 2
done

if ! curl -f "http://localhost:${DEPLOY_PORT}/health" > /dev/null 2>&1; then
    echo "❌ ${DEPLOY_TARGET} health check failed! Aborting deployment."
    ./rollback-blue-green.sh # Rollback to previous state
    exit 1
fi

# 3. Run smoke tests (placeholder - replace with actual tests)
echo "  - Running smoke tests on ${DEPLOY_TARGET}..."
# Add actual smoke tests here, e.g., curl specific endpoints, check database
sleep 5 # Simulate smoke tests
if [ "$DEPLOY_TARGET" == "server-green" ]; then
    echo "  - Smoke tests passed for green."
else
    echo "  - Smoke tests passed for blue."
fi

# 4. Switch traffic gradually (50/50 then 0/100)
echo "  - Switching traffic to 50/50..."
sed -i "s/server ${INACTIVE_TARGET}:8080 weight=100;/server ${INACTIVE_TARGET}:8080 weight=50;/" "$UPSTREAM_CONF_PATH"
sed -i "s/server ${DEPLOY_TARGET}:8080 weight=0;/server ${DEPLOY_TARGET}:8080 weight=50;/" "$UPSTREAM_CONF_PATH"
sudo nginx -s reload
sleep 60 # Monitor for 1 minute

echo "  - Switching traffic to 0/100..."
sed -i "s/server ${INACTIVE_TARGET}:8080 weight=50;/server ${INACTIVE_TARGET}:8080 weight=0;/" "$UPSTREAM_CONF_PATH"
sed -i "s/server ${DEPLOY_TARGET}:8080 weight=50;/server ${DEPLOY_TARGET}:8080 weight=100;/" "$UPSTREAM_CONF_PATH"
sudo nginx -s reload
sleep 300 # Monitor for 5 minutes

# 5. Check error rate (placeholder - integrate with Prometheus)
echo "  - Checking error rate..."
# In a real scenario, query Prometheus for error rates of the new deployment
# If error rate is high, trigger rollback

# 6. Stop old environment if successful
echo "  - Deployment successful. Stopping old environment (${INACTIVE_TARGET})..."
docker-compose -f ../docker/docker-compose.blue-green.yml stop "$INACTIVE_TARGET"
docker-compose -f ../docker/docker-compose.blue-green.yml rm -f "$INACTIVE_TARGET"

echo "✅ Blue/Green deployment complete. ${DEPLOY_TARGET} is now active."
