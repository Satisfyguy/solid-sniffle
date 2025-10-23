#!/bin/bash
set -euo pipefail

echo "🏥 Health Check - Monero Marketplace"
echo "===================================="

# Check each service
services=(
    "server:8080:/health"
    "prometheus:9090:/-/healthy"
    "grafana:3000:/api/health"
    "loki:3100:/ready"
    "alertmanager:9093:/-/healthy"
)

all_healthy=true

for service_info in "${services[@]}"; do
    IFS=':' read -r name port path <<< "$service_info"

    if curl -sf "http://localhost:${port}${path}" > /dev/null; then
        echo "✅ ${name} - HEALTHY"
    else
        echo "❌ ${name} - UNHEALTHY"
        all_healthy=false
    fi
done

# Check Monero wallets
for wallet in buyer vendor arbiter; do
    case $wallet in
        buyer)   port=18082 ;;
        vendor)  port=18083 ;;
        arbiter) port=18084 ;;
    esac

    if curl -sf "http://localhost:${port}/json_rpc" \
        -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}' \
        -H 'Content-Type: application/json' > /dev/null; then
        echo "✅ monero-wallet-rpc-${wallet} - HEALTHY"
    else
        echo "❌ monero-wallet-rpc-${wallet} - UNHEALTHY"
        all_healthy=false
    fi
done

echo ""
if $all_healthy; then
    echo "✅ All services healthy"
    exit 0
else
    echo "❌ Some services unhealthy"
    exit 1
fi
