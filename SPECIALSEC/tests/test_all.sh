#!/bin/bash

# SPECIALSEC - Script de test complet pour tous les patches
# Exécute tous les tests dans l'ordre

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}SPECIALSEC - Tests de Sécurité Backend${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""

# Check if server is running
if ! pgrep -f "target/release/server" > /dev/null; then
    echo -e "${YELLOW}⚠️  Serveur non démarré. Démarrage en cours...${NC}"
    cargo build --release
    ./target/release/server > /tmp/server.log 2>&1 &
    SERVER_PID=$!
    echo -e "${GREEN}✅ Serveur démarré (PID: $SERVER_PID)${NC}"
    sleep 3
else
    echo -e "${GREEN}✅ Serveur déjà en cours d'exécution${NC}"
fi

echo ""

# Test 1: Rate Limiting
echo -e "${YELLOW}[TEST 1/5] Rate Limiting...${NC}"
bash ./tests/test_rate_limiting.sh
echo -e "${GREEN}✅ Rate Limiting test passed${NC}"
echo ""

# Test 2: Escrow Authorization
echo -e "${YELLOW}[TEST 2/5] Escrow Authorization...${NC}"
bash ./tests/test_escrow_auth.sh
echo -e "${GREEN}✅ Escrow Authorization test passed${NC}"
echo ""

# Test 3: RPC URL Validation
echo -e "${YELLOW}[TEST 3/5] RPC URL Validation...${NC}"
bash ./tests/test_rpc_validation.sh
echo -e "${GREEN}✅ RPC URL Validation test passed${NC}"
echo ""

# Test 4: Credentials Security
echo -e "${YELLOW}[TEST 4/5] Credentials Security...${NC}"
bash ./tests/test_credentials.sh
echo -e "${GREEN}✅ Credentials Security test passed${NC}"
echo ""

# Test 5: Compilation & Security Audit
echo -e "${YELLOW}[TEST 5/5] Compilation & Security Audit...${NC}"
cargo test --workspace --lib
cargo audit
echo -e "${GREEN}✅ All tests passed${NC}"
echo ""

echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}✅ TOUS LES TESTS PASSÉS AVEC SUCCÈS${NC}"
echo -e "${GREEN}======================================${NC}"
