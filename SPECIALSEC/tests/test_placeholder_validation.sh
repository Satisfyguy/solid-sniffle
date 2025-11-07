#!/bin/bash
# Test script for placeholder validation
# This script validates that production builds reject placeholder values

set -e

echo "=================================================="
echo "Testing Placeholder Validation"
echo "=================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASS=0
FAIL=0

# Function to test placeholder detection
test_placeholder() {
    local var_name=$1
    local test_value=$2
    local should_fail=$3
    local test_desc=$4

    echo "[TEST] $test_desc"
    echo "   Variable: $var_name"
    echo "   Value: ${test_value:0:50}..."

    # Export the variable
    export "$var_name"="$test_value"

    # Try to run the server in release mode (it should panic if placeholder)
    timeout 5s cargo run --release --package server > /tmp/placeholder_test.log 2>&1 || true

    # Check the log for panic message
    if grep -q "placeholder pattern" /tmp/placeholder_test.log; then
        if [ "$should_fail" = "yes" ]; then
            echo -e "${GREEN}✅ PASS${NC}: Correctly rejected placeholder"
            ((PASS++))
        else
            echo -e "${RED}❌ FAIL${NC}: False positive - rejected valid value"
            ((FAIL++))
        fi
    else
        if [ "$should_fail" = "yes" ]; then
            echo -e "${RED}❌ FAIL${NC}: Failed to detect placeholder"
            ((FAIL++))
        else
            echo -e "${GREEN}✅ PASS${NC}: Correctly accepted valid value"
            ((PASS++))
        fi
    fi

    # Cleanup
    unset "$var_name"
    echo ""
}

echo "=================================================="
echo "Test Group 1: Placeholder Detection"
echo "=================================================="
echo ""

# Test 1: Detect "your-xxx-here" pattern
test_placeholder "DB_ENCRYPTION_KEY" "your-64-char-hex-key-here" "yes" "Detect 'your-xxx-here' pattern"

# Test 2: Detect "changeme" pattern
test_placeholder "SESSION_SECRET_KEY" "changeme_this_is_not_secure" "yes" "Detect 'changeme' pattern"

# Test 3: Detect "example" pattern
test_placeholder "JWT_SECRET" "example-jwt-secret-key-value" "yes" "Detect 'example' pattern"

# Test 4: Detect "placeholder" pattern
test_placeholder "ARBITER_PUBKEY" "placeholder_arbiter_public_key" "yes" "Detect 'placeholder' pattern"

echo "=================================================="
echo "Test Group 2: Valid Values (Should NOT Panic)"
echo "=================================================="
echo ""

# Test 5: Accept legitimate hex
test_placeholder "DB_ENCRYPTION_KEY" "8dca8a38790f2ce50422553309fa4f756dfd50d7c67a0aba2009d688b64ea811" "no" "Accept legitimate hex key"

# Test 6: Accept legitimate base64
test_placeholder "SESSION_SECRET_KEY" "dK3mN8pQvL2xYwZ6tB9jR4sA7fH1gE5nC0uI2oM3kP8vT6qX9rW1lJ4hD7yS0bF=" "no" "Accept legitimate base64"

# Test 7: Accept legitimate public key
test_placeholder "ARBITER_PUBKEY" "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456" "no" "Accept legitimate public key"

echo "=================================================="
echo "Test Group 3: Case Insensitivity"
echo "=================================================="
echo ""

# Test 8: Detect uppercase placeholder
test_placeholder "DB_ENCRYPTION_KEY" "YOUR-SECRET-KEY-HERE" "yes" "Detect uppercase placeholder"

# Test 9: Detect mixed case placeholder
test_placeholder "SESSION_SECRET_KEY" "ChangeMe-Secret" "yes" "Detect mixed case placeholder"

echo "=================================================="
echo "Test Group 4: Edge Cases"
echo "=================================================="
echo ""

# Test 10: Empty value (should not panic - handled elsewhere)
test_placeholder "DB_ENCRYPTION_KEY" "" "no" "Handle empty value gracefully"

# Test 11: Very short value (should not panic - handled elsewhere)
test_placeholder "JWT_SECRET" "abc" "no" "Handle short value gracefully"

echo "=================================================="
echo "Test Results Summary"
echo "=================================================="
echo ""
echo -e "${GREEN}Passed: $PASS${NC}"
echo -e "${RED}Failed: $FAIL${NC}"
echo "Total: $((PASS + FAIL))"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed${NC}"
    exit 1
fi
