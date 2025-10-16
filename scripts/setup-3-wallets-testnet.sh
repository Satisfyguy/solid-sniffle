#!/usr/bin/env bash
# scripts/setup-3-wallets-testnet.sh
# Configure 3 Monero wallets for 2-of-3 multisig testing

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${CYAN}"
cat << "EOF"
╔═══════════════════════════════════════╗
║  Monero 3-Wallet Testnet Setup      ║
║  2-of-3 Multisig Configuration       ║
╚═══════════════════════════════════════╝
EOF
echo -e "${NC}"

# Configuration
MONERO_DIR="$HOME/monero-testnet"
WALLETS_DIR="$HOME/monero-testnet-wallets"
BASE_RPC_PORT=18082

# Find Monero binaries
MONEROD=$(find "$MONERO_DIR" -name "monerod" -type f | head -n 1)
WALLET_RPC=$(find "$MONERO_DIR" -name "monero-wallet-rpc" -type f | head -n 1)
WALLET_CLI=$(find "$MONERO_DIR" -name "monero-wallet-cli" -type f | head -n 1)

if [[ -z "$MONEROD" ]] || [[ -z "$WALLET_RPC" ]] || [[ -z "$WALLET_CLI" ]]; then
    echo -e "${RED}❌ Monero binaries not found in $MONERO_DIR${NC}"
    echo -e "${YELLOW}💡 Run ./scripts/ubuntu-setup.sh first${NC}"
    exit 1
fi

MONERO_BIN_DIR=$(dirname "$MONEROD")
echo -e "${GREEN}✅ Monero binaries found: $MONERO_BIN_DIR${NC}\n"

# Function to print step
print_step() {
    echo -e "\n${BLUE}═══ $1 ═══${NC}\n"
}

# Function to check if process is running
is_running() {
    pgrep -f "$1" > /dev/null
}

# Step 1: Create wallets directory
print_step "Step 1: Creating wallets directory"
mkdir -p "$WALLETS_DIR"/{wallet1,wallet2,wallet3}
echo -e "${GREEN}✅ Wallets directory created: $WALLETS_DIR${NC}"

# Step 2: Check monerod
print_step "Step 2: Checking monerod"
if is_running "monerod.*--testnet"; then
    echo -e "${GREEN}✅ monerod is already running${NC}"
else
    echo -e "${YELLOW}⚠️  monerod is not running${NC}"
    echo -e "${CYAN}Starting monerod in background...${NC}"

    "$MONEROD" \
        --testnet \
        --detach \
        --data-dir "$MONERO_DIR/testnet-data" \
        --log-file "$MONERO_DIR/monerod.log"

    echo -e "${CYAN}Waiting for monerod to start (10 seconds)...${NC}"
    sleep 10

    if is_running "monerod.*--testnet"; then
        echo -e "${GREEN}✅ monerod started successfully${NC}"
    else
        echo -e "${RED}❌ Failed to start monerod${NC}"
        exit 1
    fi
fi

# Step 3: Create 3 wallets
print_step "Step 3: Creating 3 test wallets"

for i in 1 2 3; do
    WALLET_DIR="$WALLETS_DIR/wallet$i"
    WALLET_FILE="$WALLET_DIR/wallet$i"

    echo -e "${CYAN}Creating wallet $i...${NC}"

    if [[ -f "$WALLET_FILE" ]]; then
        echo -e "${YELLOW}⚠️  Wallet $i already exists, skipping creation${NC}"
    else
        # Create wallet using wallet-cli in non-interactive mode
        expect << EOF
set timeout 30
spawn "$WALLET_CLI" --testnet --generate-new-wallet "$WALLET_FILE" --password "" --mnemonic-language English
expect "Generated new wallet:"
expect "View key:"
expect eof
EOF

        if [[ -f "$WALLET_FILE" ]]; then
            echo -e "${GREEN}✅ Wallet $i created${NC}"
        else
            echo -e "${RED}❌ Failed to create wallet $i${NC}"
            exit 1
        fi
    fi
done

# Step 4: Start wallet RPC servers
print_step "Step 4: Starting wallet RPC servers"

for i in 1 2 3; do
    WALLET_FILE="$WALLETS_DIR/wallet$i/wallet$i"
    RPC_PORT=$((BASE_RPC_PORT + i - 1))

    echo -e "${CYAN}Starting wallet RPC $i on port $RPC_PORT...${NC}"

    # Kill existing RPC if running
    if is_running "monero-wallet-rpc.*$RPC_PORT"; then
        echo -e "${YELLOW}⚠️  Killing existing RPC on port $RPC_PORT${NC}"
        pkill -f "monero-wallet-rpc.*$RPC_PORT" || true
        sleep 2
    fi

    # Start RPC
    "$WALLET_RPC" \
        --testnet \
        --wallet-file "$WALLET_FILE" \
        --password "" \
        --rpc-bind-port "$RPC_PORT" \
        --disable-rpc-login \
        --daemon-address localhost:28081 \
        --trusted-daemon \
        --log-file "$WALLETS_DIR/wallet$i/rpc.log" \
        --detach &

    # Wait for RPC to start
    echo -e "${CYAN}Waiting for RPC to start...${NC}"
    for attempt in {1..10}; do
        if curl -s "http://127.0.0.1:$RPC_PORT/json_rpc" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Wallet RPC $i started on port $RPC_PORT${NC}"
            break
        fi

        if [[ $attempt -eq 10 ]]; then
            echo -e "${RED}❌ Failed to start wallet RPC $i${NC}"
            exit 1
        fi

        sleep 2
    done
done

# Step 5: Display information
print_step "Setup Complete! 🎉"

echo -e "${GREEN}✅ 3 wallets created${NC}"
echo -e "${GREEN}✅ 3 wallet RPC servers running${NC}"

echo -e "\n${CYAN}══════════════════════════════════════${NC}"
echo -e "${CYAN}  Wallet Information${NC}"
echo -e "${CYAN}══════════════════════════════════════${NC}\n"

echo -e "${YELLOW}Wallet 1:${NC}"
echo -e "  Directory: $WALLETS_DIR/wallet1"
echo -e "  RPC Port: 18082"
echo -e "  RPC URL: http://127.0.0.1:18082/json_rpc"

echo -e "\n${YELLOW}Wallet 2:${NC}"
echo -e "  Directory: $WALLETS_DIR/wallet2"
echo -e "  RPC Port: 18083"
echo -e "  RPC URL: http://127.0.0.1:18083/json_rpc"

echo -e "\n${YELLOW}Wallet 3:${NC}"
echo -e "  Directory: $WALLETS_DIR/wallet3"
echo -e "  RPC Port: 18084"
echo -e "  RPC URL: http://127.0.0.1:18084/json_rpc"

echo -e "\n${CYAN}══════════════════════════════════════${NC}"
echo -e "${CYAN}  Next Steps${NC}"
echo -e "${CYAN}══════════════════════════════════════${NC}\n"

echo -e "1. ${YELLOW}Test wallet connections:${NC}"
echo -e "   ${GREEN}cargo run --bin monero-marketplace -- --rpc-url http://127.0.0.1:18082/json_rpc test${NC}"
echo -e "   ${GREEN}cargo run --bin monero-marketplace -- --rpc-url http://127.0.0.1:18083/json_rpc test${NC}"
echo -e "   ${GREEN}cargo run --bin monero-marketplace -- --rpc-url http://127.0.0.1:18084/json_rpc test${NC}"

echo -e "\n2. ${YELLOW}Prepare multisig (step 1/6):${NC}"
echo -e "   ${GREEN}cargo run --bin monero-marketplace -- --rpc-url http://127.0.0.1:18082/json_rpc multisig prepare${NC}"
echo -e "   ${GREEN}# Repeat for wallet 2 and 3...${NC}"

echo -e "\n3. ${YELLOW}Make multisig (step 2/6):${NC}"
echo -e "   ${GREEN}cargo run --bin monero-marketplace -- --rpc-url http://127.0.0.1:18082/json_rpc multisig make -t 2 -i <info2> -i <info3>${NC}"
echo -e "   ${GREEN}# Repeat for wallet 2 and 3...${NC}"

echo -e "\n4. ${YELLOW}Stop all services:${NC}"
echo -e "   ${GREEN}pkill -f monero-wallet-rpc${NC}"
echo -e "   ${GREEN}pkill -f monerod${NC}"

echo -e "\n5. ${YELLOW}View logs:${NC}"
echo -e "   ${GREEN}tail -f $WALLETS_DIR/wallet1/rpc.log${NC}"
echo -e "   ${GREEN}tail -f $MONERO_DIR/monerod.log${NC}"

echo -e "\n${CYAN}══════════════════════════════════════${NC}"
echo -e "${GREEN}Happy testing! 🦀🔐${NC}"
echo -e "${CYAN}══════════════════════════════════════${NC}\n"
