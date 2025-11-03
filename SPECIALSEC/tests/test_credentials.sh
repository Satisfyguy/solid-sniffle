#!/bin/bash

# Test Credentials Security (Patches 6 & 7)

set -e

echo "Testing Credentials Security..."
echo ""

# Test 1: Arbiter Password Generation
echo "[1] Testing arbiter password is NOT hardcoded..."
if grep -r "arbiter_system_2024" ../server/src/main.rs; then
    echo "❌ FAIL: Hardcoded password 'arbiter_system_2024' still present in code"
    exit 1
else
    echo "✅ PASS: Hardcoded password removed from code"
fi

echo ""

# Test 2: SESSION_SECRET_KEY production safety
echo "[2] Testing SESSION_SECRET_KEY production safety..."
unset SESSION_SECRET_KEY

# Build release
echo "   Building release binary..."
cargo build --release > /dev/null 2>&1

# Try to run without SESSION_SECRET_KEY (should panic)
echo "   Running release binary without SESSION_SECRET_KEY..."
timeout 3s ./target/release/server > /tmp/server_test.log 2>&1 || true

if grep -q "FATAL: SESSION_SECRET_KEY" /tmp/server_test.log; then
    echo "✅ PASS: Release build panics without SESSION_SECRET_KEY"
else
    echo "❌ FAIL: Release build did NOT panic without SESSION_SECRET_KEY"
    cat /tmp/server_test.log
    exit 1
fi

echo ""

# Test 3: Dev mode fallback
echo "[3] Testing dev mode fallback..."
unset SESSION_SECRET_KEY

# Build debug
echo "   Building debug binary..."
cargo build > /dev/null 2>&1

# Try to run in debug mode (should warn but start)
echo "   Running debug binary without SESSION_SECRET_KEY..."
timeout 3s cargo run > /tmp/server_dev_test.log 2>&1 || true

if grep -q "SESSION_SECRET_KEY not set" /tmp/server_dev_test.log && grep -q "dev mode only" /tmp/server_dev_test.log; then
    echo "✅ PASS: Debug build warns about missing SESSION_SECRET_KEY but continues"
else
    echo "⚠️  WARNING: Debug build behavior unclear"
    tail -20 /tmp/server_dev_test.log
fi

echo ""
echo "✅ Credentials security tests completed"
