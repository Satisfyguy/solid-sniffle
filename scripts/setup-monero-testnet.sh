#!/usr/bin/env bash
# scripts/setup-monero-testnet.sh
# Complete Monero testnet setup for testing

# Default parameters
WALLET_NAME="buyer"
MONERO_PATH="$HOME/monero-testnet"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --wallet)
            WALLET_NAME="$2"
            shift 2
            ;;
        --path)
            MONERO_PATH="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--wallet NAME] [--path PATH]"
            exit 1
            ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔧 Setup Monero Testnet${NC}"
echo ""

# Find Monero binaries
if [[ ! -d "$MONERO_PATH" ]]; then
    echo -e "${YELLOW}Monero directory not found at $MONERO_PATH${NC}"
    echo -e "${YELLOW}Creating directory and downloading Monero CLI...${NC}"

    mkdir -p "$MONERO_PATH"
    cd "$MONERO_PATH" || exit 1

    # Download latest Monero CLI for Linux
    echo -e "${CYAN}Downloading Monero CLI (this may take a while)...${NC}"
    wget -q --show-progress https://downloads.getmonero.org/cli/linux64 -O monero-linux.tar.bz2

    echo -e "${CYAN}Extracting...${NC}"
    tar -xjf monero-linux.tar.bz2
    rm monero-linux.tar.bz2

    echo -e "${GREEN}✅ Monero CLI downloaded${NC}"
    cd - > /dev/null || exit 1
fi

# Find binaries
MONEROD=$(find "$MONERO_PATH" -name "monerod" -type f | head -n 1)
WALLET_CLI=$(find "$MONERO_PATH" -name "monero-wallet-cli" -type f | head -n 1)
WALLET_RPC=$(find "$MONERO_PATH" -name "monero-wallet-rpc" -type f | head -n 1)

if [[ -z "$MONEROD" ]]; then
    echo -e "${RED}❌ monerod not found in $MONERO_PATH${NC}"
    echo -e "${YELLOW}Run: ./scripts/setup-monero.sh${NC}"
    exit 1
fi

BIN_DIR=$(dirname "$MONEROD")
echo -e "${GREEN}Monero binaries: $BIN_DIR${NC}"
echo ""

# 1. Start testnet daemon (if not already running)
if pgrep -x "monerod" > /dev/null; then
    echo -e "${GREEN}1️⃣ Daemon already running ✅${NC}"
else
    echo -e "${YELLOW}1️⃣ Starting testnet daemon...${NC}"
    "$MONEROD" --testnet --detach --log-file "$MONERO_PATH/monerod.log"

    echo -e "${CYAN}   Waiting for synchronization (10s)...${NC}"
    sleep 10
    echo -e "${GREEN}   ✅ Daemon started${NC}"
fi
echo ""

# 2. Create wallet if not already done
WALLET_PATH="$BIN_DIR/$WALLET_NAME"
if [[ ! -f "$WALLET_PATH" ]]; then
    echo -e "${YELLOW}2️⃣ Creating testnet wallet: $WALLET_NAME${NC}"
    echo -e "${CYAN}   (Empty password for testing)${NC}"

    # Use wallet-cli with expect to automate password entry
    if command -v expect > /dev/null; then
        expect << EOF
spawn "$WALLET_CLI" --testnet --generate-new-wallet "$WALLET_PATH"
expect "Enter a new password for the wallet:"
send "\r"
expect "Confirm password:"
send "\r"
expect "Enter the wallet's language"
send "1\r"
expect eof
EOF
        echo -e "${GREEN}   ✅ Wallet created${NC}"
    else
        echo -e "${YELLOW}   ⚠️  'expect' not installed, manual wallet creation needed${NC}"
        echo -e "${CYAN}   Run: $WALLET_CLI --testnet --generate-new-wallet $WALLET_PATH${NC}"
    fi
else
    echo -e "${GREEN}2️⃣ Wallet already exists ✅${NC}"
fi
echo ""

# 3. Start wallet RPC (kill existing if running)
if pgrep -x "monero-wallet-rpc" > /dev/null; then
    echo -e "${YELLOW}3️⃣ Wallet RPC already running${NC}"
    echo -e "${CYAN}   Stopping to restart cleanly...${NC}"
    pkill -x monero-wallet-rpc
    sleep 2
fi

echo -e "${YELLOW}3️⃣ Starting wallet RPC: $WALLET_NAME${NC}"
"$WALLET_RPC" \
    --testnet \
    --wallet-file "$WALLET_PATH" \
    --password "" \
    --rpc-bind-ip 127.0.0.1 \
    --rpc-bind-port 18082 \
    --disable-rpc-login \
    --daemon-address 127.0.0.1:28081 \
    --log-file "$MONERO_PATH/wallet-rpc.log" \
    --detach

echo -e "${CYAN}   Waiting for RPC startup (5s)...${NC}"
sleep 5

# 4. Test RPC connection
echo -e "${YELLOW}4️⃣ Testing RPC connection...${NC}"
response=$(curl -s --max-time 5 \
    -X POST http://127.0.0.1:18082/json_rpc \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}')

if [[ -z "$response" ]]; then
    echo -e "${RED}   ❌ RPC not accessible${NC}"
    echo -e "${YELLOW}   Check logs at: $MONERO_PATH/wallet-rpc.log${NC}"
    exit 1
fi

echo -e "${GREEN}   ✅ RPC accessible${NC}"
version=$(echo "$response" | grep -oP '(?<="version":)\d+')
if [[ -n "$version" ]]; then
    echo -e "${CYAN}   Version: $version${NC}"
fi

echo ""
echo -e "${GREEN}✅ Setup Monero Testnet complete!${NC}"
echo ""
echo -e "${CYAN}📋 Summary:${NC}"
echo "  Daemon: testnet @ 127.0.0.1:28081"
echo "  Wallet: $WALLET_NAME (empty password)"
echo "  RPC: http://127.0.0.1:18082"
echo ""
echo -e "${CYAN}🧪 Next step:${NC}"
echo -e "${YELLOW}  cargo test --package wallet${NC}"
