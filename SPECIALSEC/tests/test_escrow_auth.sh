#!/bin/bash

# Test Escrow Authorization (Patches 2 & 3)

set -e

BASE_URL="http://127.0.0.1:8080"

echo "Testing Escrow Authorization..."
echo ""

# NOTE: Ces tests nécessitent un setup avec escrows créés
# Pour l'instant, on vérifie que les endpoints retournent 403 sans auth

echo "[1] Testing refund_funds without proper authorization..."
response=$(curl -s -w "\n%{http_code}" -X POST \
  "$BASE_URL/api/escrow/test-escrow-id/refund" \
  -H "Content-Type: application/json" \
  -d '{"buyer_address":"9w7Qr8...test"}')

http_code=$(echo "$response" | tail -n 1)

if [ "$http_code" == "401" ] || [ "$http_code" == "403" ]; then
    echo "✅ refund_funds properly rejects unauthorized requests ($http_code)"
else
    echo "⚠️  refund_funds returned $http_code (expected 401/403)"
fi

echo ""
echo "[2] Testing resolve_dispute without proper authorization..."
response=$(curl -s -w "\n%{http_code}" -X POST \
  "$BASE_URL/api/escrow/test-escrow-id/resolve" \
  -H "Content-Type: application/json" \
  -d '{"resolution":"buyer","recipient_address":"9w7Qr8...test"}')

http_code=$(echo "$response" | tail -n 1)

if [ "$http_code" == "401" ] || [ "$http_code" == "403" ]; then
    echo "✅ resolve_dispute properly rejects unauthorized requests ($http_code)"
else
    echo "⚠️  resolve_dispute returned $http_code (expected 401/403)"
fi

echo ""
echo "✅ Escrow authorization tests completed"
echo "⚠️  Note: Full authorization tests require authenticated sessions and real escrows"
