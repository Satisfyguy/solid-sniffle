#!/bin/bash
set -euo pipefail

echo "🚀 Starting Monero Marketplace Docker Stack..."

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker daemon not running"
    exit 1
fi

# Create required directories
mkdir -p ../../data ../../logs ../../wallets/{buyer,vendor,arbiter} ../monitoring/{alerts,grafana}

# Pull latest images
echo "📦 Pulling images..."
docker-compose -f ../docker/docker-compose.yml pull

# Start services in order
echo "🏗️  Starting infrastructure..."
docker-compose -f ../docker/docker-compose.yml up -d monero-wallet-rpc-buyer monero-wallet-rpc-vendor monero-wallet-rpc-arbiter
sleep 10

echo "📊 Starting monitoring..."
docker-compose -f ../docker/docker-compose.yml up -d prometheus grafana loki promtail alertmanager
sleep 5

echo "🌐 Starting application server..."
docker-compose -f ../docker/docker-compose.yml up -d server

# Wait for health checks
echo "🏥 Waiting for health checks..."
for i in {1..30}; do
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ Application healthy!"
        break
    fi
    echo "⏳ Attempt $i/30..."
    sleep 2
done

# Display status
echo ""
echo "📋 Service Status:"
docker-compose -f ../docker/docker-compose.yml ps

echo ""
echo "🔗 URLs:"
echo "  - Application: http://localhost:8080"
echo "  - Prometheus:  http://localhost:9090"
echo "  - Grafana:     http://localhost:3000 (admin/admin123_CHANGE_ME)"
echo "  - Alertmanager: http://localhost:9093"

echo ""
echo "📝 Logs:"
echo "  docker-compose -f ../docker/docker-compose.yml logs -f server"
