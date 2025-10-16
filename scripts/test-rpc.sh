#!/usr/bin/env bash
# scripts/test-rpc.sh
# Test Monero RPC connectivity

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

RPC_URL="http://127.0.0.1:18082/json_rpc"
TIMEOUT=5

echo -e "${CYAN}Testing Monero RPC Connection${NC}"
echo -e "${CYAN}==============================${NC}"
echo ""

# Test 1: Check if endpoint is reachable
echo -e "${YELLOW}1. Testing endpoint reachability...${NC}"
if curl -s --max-time $TIMEOUT "$RPC_URL" > /dev/null 2>&1; then
    echo -e "${GREEN}   ✅ Endpoint reachable${NC}"
else
    echo -e "${RED}   ❌ Endpoint not reachable${NC}"
    echo -e "${YELLOW}   Make sure Monero wallet RPC is running:${NC}"
    echo -e "${CYAN}   ./scripts/setup-monero-testnet.sh${NC}"
    exit 1
fi

# Test 2: get_version
echo -e "\n${YELLOW}2. Testing get_version...${NC}"
response=$(curl -s --max-time $TIMEOUT \
    -X POST "$RPC_URL" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}')

if [[ -z "$response" ]]; then
    echo -e "${RED}   ❌ No response${NC}"
    exit 1
fi

if echo "$response" | jq -e '.result.version' > /dev/null 2>&1; then
    version=$(echo "$response" | jq -r '.result.version')
    echo -e "${GREEN}   ✅ get_version successful${NC}"
    echo -e "${CYAN}   Version: $version${NC}"
else
    echo -e "${RED}   ❌ get_version failed${NC}"
    echo -e "${YELLOW}   Response: $response${NC}"
    exit 1
fi

# Test 3: get_address
echo -e "\n${YELLOW}3. Testing get_address...${NC}"
response=$(curl -s --max-time $TIMEOUT \
    -X POST "$RPC_URL" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_address","params":{"account_index":0}}')

if echo "$response" | jq -e '.result.address' > /dev/null 2>&1; then
    address=$(echo "$response" | jq -r '.result.address')
    echo -e "${GREEN}   ✅ get_address successful${NC}"
    echo -e "${CYAN}   Address: ${address:0:20}...${address: -10}${NC}"
else
    echo -e "${RED}   ❌ get_address failed${NC}"
    echo -e "${YELLOW}   Response: $response${NC}"
    exit 1
fi

# Test 4: get_balance
echo -e "\n${YELLOW}4. Testing get_balance...${NC}"
response=$(curl -s --max-time $TIMEOUT \
    -X POST "$RPC_URL" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_balance","params":{"account_index":0}}')

if echo "$response" | jq -e '.result' > /dev/null 2>&1; then
    balance=$(echo "$response" | jq -r '.result.balance // 0')
    unlocked=$(echo "$response" | jq -r '.result.unlocked_balance // 0')
    echo -e "${GREEN}   ✅ get_balance successful${NC}"
    echo -e "${CYAN}   Balance: $balance atomic units${NC}"
    echo -e "${CYAN}   Unlocked: $unlocked atomic units${NC}"
else
    echo -e "${RED}   ❌ get_balance failed${NC}"
    echo -e "${YELLOW}   Response: $response${NC}"
    exit 1
fi

# Test 5: is_multisig
echo -e "\n${YELLOW}5. Testing is_multisig...${NC}"
response=$(curl -s --max-time $TIMEOUT \
    -X POST "$RPC_URL" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":"0","method":"is_multisig"}')

if echo "$response" | jq -e '.result' > /dev/null 2>&1; then
    multisig=$(echo "$response" | jq -r '.result.multisig')
    echo -e "${GREEN}   ✅ is_multisig successful${NC}"
    echo -e "${CYAN}   Multisig: $multisig${NC}"
else
    echo -e "${RED}   ❌ is_multisig failed${NC}"
    echo -e "${YELLOW}   Response: $response${NC}"
    exit 1
fi

# Summary
echo -e "\n${CYAN}RPC Test Summary${NC}"
echo -e "${CYAN}==============================${NC}"
echo -e "${GREEN}✅ All RPC tests passed!${NC}"
echo ""
echo -e "${YELLOW}RPC Configuration:${NC}"
echo -e "  URL: $RPC_URL"
echo -e "  Timeout: ${TIMEOUT}s"
echo -e "  Testnet: Yes"
echo ""
echo -e "${GREEN}Ready for integration tests!${NC}"
echo -e "${CYAN}Run: cargo test --package wallet${NC}"
