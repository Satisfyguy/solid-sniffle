#!/bin/bash

# Test Rate Limiting (Patch 1)

set -e

BASE_URL="http://127.0.0.1:8080"
HEALTH_ENDPOINT="$BASE_URL/api/health"

echo "Testing Rate Limiting..."
echo "Sending 150 requests to $HEALTH_ENDPOINT"

success_count=0
ratelimit_count=0

for i in {1..150}; do
    response=$(curl -s -w "\n%{http_code}" $HEALTH_ENDPOINT)
    http_code=$(echo "$response" | tail -n 1)

    if [ "$http_code" == "200" ]; then
        success_count=$((success_count + 1))
    elif [ "$http_code" == "429" ]; then
        ratelimit_count=$((ratelimit_count + 1))
    fi

    # Print progress every 10 requests
    if [ $((i % 10)) -eq 0 ]; then
        echo "Progress: $i/150 | 200 OK: $success_count | 429 Rate Limited: $ratelimit_count"
    fi
done

echo ""
echo "Results:"
echo "  - 200 OK: $success_count"
echo "  - 429 Rate Limited: $ratelimit_count"

# Validation
if [ $ratelimit_count -gt 0 ]; then
    echo "✅ Rate limiting is ACTIVE (received $ratelimit_count rate limit responses)"
    exit 0
else
    echo "❌ Rate limiting is NOT working (expected some 429 responses after ~100 requests)"
    exit 1
fi
