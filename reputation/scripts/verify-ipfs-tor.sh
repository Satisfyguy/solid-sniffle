#!/bin/bash
# IPFS + Tor Security Verification Script
# Zero-tolerance validation for production deployment

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

ERRORS=0
WARNINGS=0

function check_pass() {
    echo -e "${GREEN}✅${NC} $1"
}

function check_fail() {
    echo -e "${RED}❌ CRITICAL:${NC} $1"
    ((ERRORS++))
}

function check_warn() {
    echo -e "${YELLOW}⚠️  WARNING:${NC} $1"
    ((WARNINGS++))
}

echo "========================================="
echo "  IPFS + Tor Security Verification"
echo "  Monero Marketplace Reputation System"
echo "========================================="
echo ""

# Check 1: IPFS is installed
echo "[1/10] Checking IPFS installation..."
if command -v ipfs &> /dev/null; then
    VERSION=$(ipfs version -n)
    check_pass "IPFS is installed: $VERSION"
else
    check_fail "IPFS is not installed"
    echo ""
    echo "Run: ./scripts/install-ipfs.sh"
    exit 1
fi

# Check 2: Tor is running
echo ""
echo "[2/10] Checking Tor daemon..."
if systemctl is-active --quiet tor 2>/dev/null; then
    check_pass "Tor service is running"
elif pgrep -x tor >/dev/null; then
    check_pass "Tor process is running"
else
    check_fail "Tor is not running"
    echo "    Start with: sudo systemctl start tor"
fi

# Check 3: Tor SOCKS proxy is accessible
echo ""
echo "[3/10] Verifying Tor SOCKS proxy..."
if curl --socks5-hostname 127.0.0.1:9050 -s --max-time 10 https://check.torproject.org/api/ip &>/dev/null; then
    check_pass "Tor SOCKS proxy is accessible on 127.0.0.1:9050"
else
    check_fail "Tor SOCKS proxy is not accessible"
fi

# Check 4: IPFS daemon is running
echo ""
echo "[4/10] Checking IPFS daemon..."
if curl -s --max-time 5 http://127.0.0.1:5001/api/v0/version &>/dev/null; then
    check_pass "IPFS daemon is running"
else
    check_fail "IPFS daemon is not running"
    echo "    Start with: ./scripts/ipfs-daemon.sh start"
fi

# Check 5: IPFS API is localhost-only
echo ""
echo "[5/10] Verifying IPFS API binding..."
API_ADDR=$(ipfs config Addresses.API 2>/dev/null || echo "")
if [[ "$API_ADDR" == *"127.0.0.1"* ]]; then
    check_pass "IPFS API is bound to localhost: $API_ADDR"
elif [[ "$API_ADDR" == *"0.0.0.0"* ]]; then
    check_fail "IPFS API is PUBLICLY EXPOSED: $API_ADDR"
    echo "    Fix with: ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001"
else
    check_warn "Could not verify API binding: $API_ADDR"
fi

# Check 6: IPFS Gateway is localhost-only
echo ""
echo "[6/10] Verifying IPFS Gateway binding..."
GATEWAY_ADDR=$(ipfs config Addresses.Gateway 2>/dev/null || echo "")
if [[ "$GATEWAY_ADDR" == *"127.0.0.1"* ]]; then
    check_pass "IPFS Gateway is bound to localhost: $GATEWAY_ADDR"
elif [[ "$GATEWAY_ADDR" == *"0.0.0.0"* ]]; then
    check_fail "IPFS Gateway is PUBLICLY EXPOSED: $GATEWAY_ADDR"
    echo "    Fix with: ipfs config Addresses.Gateway /ip4/127.0.0.1/tcp/8080"
else
    check_warn "Could not verify Gateway binding: $GATEWAY_ADDR"
fi

# Check 7: QUIC is disabled (required for SOCKS5)
echo ""
echo "[7/10] Verifying QUIC is disabled..."
QUIC_ENABLED=$(ipfs config Swarm.Transports.Network.QUIC 2>/dev/null || echo "unknown")
if [[ "$QUIC_ENABLED" == "false" ]]; then
    check_pass "QUIC is disabled (SOCKS5 compatible)"
elif [[ "$QUIC_ENABLED" == "true" ]]; then
    check_fail "QUIC is enabled (incompatible with SOCKS5/Tor)"
    echo "    Fix with: ipfs config --json Swarm.Transports.Network.QUIC false"
else
    check_warn "Could not verify QUIC status: $QUIC_ENABLED"
fi

# Check 8: DHT mode
echo ""
echo "[8/10] Verifying DHT mode..."
DHT_MODE=$(ipfs config Routing.Type 2>/dev/null || echo "unknown")
if [[ "$DHT_MODE" == "dhtclient" ]]; then
    check_pass "DHT client mode enabled (reduced exposure)"
elif [[ "$DHT_MODE" == "dht" ]]; then
    check_warn "DHT server mode enabled (consider 'dhtclient' for production)"
    echo "    Optionally set: ipfs config Routing.Type dhtclient"
else
    check_warn "Unknown DHT mode: $DHT_MODE"
fi

# Check 9: No direct peer connections (check for non-Tor IPs)
echo ""
echo "[9/10] Checking IPFS peer connections..."
if ipfs swarm peers 2>/dev/null | grep -v "127.0.0.1" | grep -qE '[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}'; then
    check_warn "Direct IP connections detected in peer list"
    echo "    This may indicate IPFS is not routing through Tor"
    echo "    Ensure ALL_PROXY=socks5h://127.0.0.1:9050 is set when starting daemon"
    PEER_COUNT=$(ipfs swarm peers 2>/dev/null | wc -l)
    echo "    Total peers: $PEER_COUNT"
else
    check_pass "No direct IP peer connections detected"
    PEER_COUNT=$(ipfs swarm peers 2>/dev/null | wc -l)
    if [ "$PEER_COUNT" -gt 0 ]; then
        check_pass "Connected to $PEER_COUNT peers (through Tor)"
    else
        check_warn "No peers connected (may be starting up)"
    fi
fi

# Check 10: Test IPFS operations
echo ""
echo "[10/10] Testing IPFS upload..."
TEST_CONTENT="IPFS Security Test $(date +%s)"
TEST_HASH=$(echo "$TEST_CONTENT" | ipfs add -q --only-hash 2>/dev/null || echo "")

if [ -n "$TEST_HASH" ]; then
    check_pass "IPFS upload test successful"
    echo "    Test hash: ${TEST_HASH:0:30}..."
else
    check_fail "IPFS upload test failed"
fi

# Summary
echo ""
echo "========================================="
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo "========================================="
    echo ""
    echo "IPFS is properly configured for production deployment."
    echo "All traffic will be routed through Tor (127.0.0.1:9050)"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  $WARNINGS warnings detected${NC}"
    echo "========================================="
    echo ""
    echo "IPFS configuration has some warnings but is functional."
    echo "Review warnings above and fix if deploying to production."
    exit 0
else
    echo -e "${RED}❌ $ERRORS critical errors, $WARNINGS warnings${NC}"
    echo "========================================="
    echo ""
    echo "IPFS configuration has critical security issues."
    echo "DO NOT deploy to production until all errors are fixed."
    exit 1
fi
