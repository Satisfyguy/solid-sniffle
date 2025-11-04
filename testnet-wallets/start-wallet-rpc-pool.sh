#!/bin/bash
# Start 3 wallet-rpc instances for non-custodial escrow

# Ports: 18082 (buyer), 18083 (vendor), 18084 (arbiter)

# Kill existing instances
pkill -9 -f monero-wallet-rpc 2>/dev/null

# Start buyer wallet-rpc
monero-wallet-rpc --testnet --rpc-bind-port 18082 --rpc-bind-ip 127.0.0.1 \
  --wallet-dir ./buyer --disable-rpc-login --offline --confirm-external-bind --detach \
  --log-file buyer-rpc.log

# Start vendor wallet-rpc  
monero-wallet-rpc --testnet --rpc-bind-port 18083 --rpc-bind-ip 127.0.0.1 \
  --wallet-dir ./vendor --disable-rpc-login --offline --confirm-external-bind --detach \
  --log-file vendor-rpc.log

# Start arbiter wallet-rpc
monero-wallet-rpc --testnet --rpc-bind-port 18084 --rpc-bind-ip 127.0.0.1 \
  --wallet-dir ./arbiter --disable-rpc-login --offline --confirm-external-bind --detach \
  --log-file arbiter-rpc.log

sleep 2
echo "✅ Started 3 wallet-rpc instances:"
echo "   - Buyer:   http://127.0.0.1:18082"
echo "   - Vendor:  http://127.0.0.1:18083"
echo "   - Arbiter: http://127.0.0.1:18084"

ps aux | grep monero-wallet-rpc | grep -v grep
