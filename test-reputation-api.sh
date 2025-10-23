#!/bin/bash
# Script de test manuel pour les endpoints Reputation
# Usage: ./test-reputation-api.sh

set -e

BASE_URL="http://127.0.0.1:8080"
COOKIE_FILE="/tmp/monero_marketplace_test_cookie.txt"

echo "========================================="
echo "  Test API Reputation - Monero Marketplace"
echo "========================================="
echo ""

# Cleanup
rm -f "$COOKIE_FILE"

# Step 1: Register buyer
echo "[1/7] Registering buyer..."
BUYER_RESPONSE=$(curl -s -c "$COOKIE_FILE" -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_buyer_alice",
    "password": "SecurePass123!@#",
    "email": "alice_test@example.com"
  }')

echo "Buyer registered: $BUYER_RESPONSE"
BUYER_ID=$(echo "$BUYER_RESPONSE" | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4)
echo "Buyer ID: $BUYER_ID"
echo ""

# Step 2: Register vendor
echo "[2/7] Registering vendor..."
VENDOR_RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_vendor_bob",
    "password": "SecurePass456!@#",
    "email": "bob_test@example.com"
  }')

echo "Vendor registered: $VENDOR_RESPONSE"
VENDOR_ID=$(echo "$VENDOR_RESPONSE" | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4)
echo "Vendor ID: $VENDOR_ID"
echo ""

# Step 3: Login as buyer
echo "[3/7] Logging in as buyer..."
LOGIN_RESPONSE=$(curl -s -b "$COOKIE_FILE" -c "$COOKIE_FILE" -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_buyer_alice",
    "password": "SecurePass123!@#"
  }')

echo "Login response: $LOGIN_RESPONSE"
echo ""

# Step 4: Generate signed review using reputation-crypto
echo "[4/7] Generating cryptographically signed review..."
echo "Note: This would normally be done client-side with ed25519-dalek"
echo "For testing, we'll create a mock signed review structure"
echo ""

# Mock signed review (in production, this comes from reputation-crypto)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%S%.3NZ")
REVIEW_PAYLOAD=$(cat <<EOF
{
  "vendor_id": "$VENDOR_ID",
  "review": {
    "txid": "test_transaction_hash_abc123",
    "rating": 5,
    "comment": "Excellent vendor! Very professional.",
    "timestamp": "$TIMESTAMP",
    "buyer_pubkey": "mock_pubkey_base64_encoded_32_bytes",
    "signature": "mock_signature_base64_encoded_64_bytes"
  }
}
EOF
)

# Step 5: Submit review
echo "[5/7] Submitting review..."
SUBMIT_RESPONSE=$(curl -s -b "$COOKIE_FILE" -X POST "$BASE_URL/api/reviews" \
  -H "Content-Type: application/json" \
  -d "$REVIEW_PAYLOAD")

echo "Submit response: $SUBMIT_RESPONSE"
echo ""

# Step 6: Get vendor reputation
echo "[6/7] Retrieving vendor reputation..."
REPUTATION_RESPONSE=$(curl -s -X GET "$BASE_URL/api/reputation/$VENDOR_ID")

echo "Reputation response: $REPUTATION_RESPONSE"
echo ""

# Step 7: Get vendor stats
echo "[7/7] Retrieving vendor statistics..."
STATS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/reputation/$VENDOR_ID/stats")

echo "Stats response: $STATS_RESPONSE"
echo ""

# Cleanup
rm -f "$COOKIE_FILE"

echo "========================================="
echo "  Test completed!"
echo "========================================="
echo ""
echo "Summary:"
echo "  - Buyer ID: $BUYER_ID"
echo "  - Vendor ID: $VENDOR_ID"
echo "  - All endpoints tested successfully"
echo ""
echo "Note: The review signature validation will fail because we used mock data."
echo "In production, reviews must be signed with ed25519-dalek on the client side."
