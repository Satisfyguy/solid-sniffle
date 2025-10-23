#!/bin/bash
set -euo pipefail

echo "🛑 Stopping Monero Marketplace..."

# Graceful shutdown
docker-compose -f ../docker/docker-compose.yml stop -t 30

# Remove containers
docker-compose -f ../docker/docker-compose.yml down

echo "✅ Stopped"
