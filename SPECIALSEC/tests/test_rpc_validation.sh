#!/bin/bash

# Test RPC URL Validation (Patch 5)

set -e

BASE_URL="http://127.0.0.1:8080"
REGISTER_ENDPOINT="$BASE_URL/api/escrow/register-wallet-rpc"

echo "Testing RPC URL Validation..."
echo ""

# Test 1: Public URL (should be REJECTED)
echo "[1] Testing public URL (should be rejected)..."
response=$(curl -s -w "\n%{http_code}" -X POST $REGISTER_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{
    "rpc_url": "http://attacker.com:18082/json_rpc",
    "role": "buyer"
  }')

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" == "400" ]; then
    echo "✅ Public URL correctly REJECTED (HTTP 400)"
    echo "   Response: $body"
else
    echo "❌ Public URL NOT rejected (got HTTP $http_code, expected 400)"
    echo "   Response: $body"
fi

echo ""

# Test 2: Localhost URL (should be ACCEPTED - but will fail on other validation)
echo "[2] Testing localhost URL (should pass URL validation)..."
response=$(curl -s -w "\n%{http_code}" -X POST $REGISTER_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{
    "rpc_url": "http://127.0.0.1:18082/json_rpc",
    "role": "buyer"
  }')

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" != "400" ] || [[ "$body" != *"rpc_url_must_be_local_or_onion"* ]]; then
    echo "✅ Localhost URL passed URL validation (HTTP $http_code)"
    echo "   May fail on other validation (auth, csrf, etc.) - expected"
else
    echo "❌ Localhost URL rejected on URL validation (should pass)"
    echo "   Response: $body"
fi

echo ""

# Test 3: .onion URL (should be ACCEPTED on URL validation)
echo "[3] Testing .onion URL (should pass URL validation)..."
response=$(curl -s -w "\n%{http_code}" -X POST $REGISTER_ENDPOINT \
  -H "Content-Type: application/json" \
  -d '{
    "rpc_url": "http://abc123xyz.onion:18082/json_rpc",
    "role": "buyer"
  }')

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" != "400" ] || [[ "$body" != *"rpc_url_must_be_local_or_onion"* ]]; then
    echo "✅ .onion URL passed URL validation (HTTP $http_code)"
else
    echo "❌ .onion URL rejected on URL validation (should pass)"
    echo "   Response: $body"
fi

echo ""
echo "✅ RPC URL validation tests completed"
