#!/usr/bin/env python3
"""
Monero RPC Prometheus Exporter
Exports wallet metrics from Monero wallet RPC to Prometheus format
"""

import json
import time
import requests
from prometheus_client import start_http_server, Gauge, Counter
import os
import logging

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Prometheus metrics
WALLET_BALANCE = Gauge('monero_wallet_balance_piconero', 'Wallet balance in piconero', ['wallet_name'])
WALLET_UNLOCKED_BALANCE = Gauge('monero_wallet_unlocked_balance_piconero', 'Unlocked balance in piconero', ['wallet_name'])
WALLET_HEIGHT = Gauge('monero_wallet_height', 'Wallet sync height', ['wallet_name'])
WALLET_NUM_UNSPENT_OUTPUTS = Gauge('monero_wallet_num_unspent_outputs', 'Number of unspent outputs', ['wallet_name'])
WALLET_RPC_CALLS_TOTAL = Counter('monero_wallet_rpc_calls_total', 'Total RPC calls', ['wallet_name', 'method', 'status'])
WALLET_RPC_ERRORS_TOTAL = Counter('monero_wallet_rpc_errors_total', 'Total RPC errors', ['wallet_name', 'method'])

class MoneroRPCClient:
    def __init__(self, host, port, wallet_name):
        self.url = f"http://{host}:{port}/json_rpc"
        self.wallet_name = wallet_name

    def call_rpc(self, method, params=None):
        """Make JSON-RPC call to Monero wallet RPC.

        Args:
            method (str): The RPC method name (e.g., 'get_balance', 'get_height')
            params (dict, optional): RPC method parameters. Defaults to empty dict.

        Returns:
            dict: The 'result' field from RPC response, or None on error.

        Side Effects:
            - Updates Prometheus counters for RPC calls and errors
            - Logs errors to stderr
        """
        payload = {
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params or {}
        }

        try:
            response = requests.post(self.url, json=payload, timeout=30)
            WALLET_RPC_CALLS_TOTAL.labels(wallet_name=self.wallet_name, method=method, status='success').inc()

            if response.status_code == 200:
                result = response.json()
                if 'result' in result:
                    return result['result']
                elif 'error' in result:
                    logger.error(f"RPC error for {method}: {result['error']}")
                    WALLET_RPC_ERRORS_TOTAL.labels(wallet_name=self.wallet_name, method=method).inc()
                    return None
            else:
                logger.error(f"HTTP error {response.status_code} for {method}")
                WALLET_RPC_ERRORS_TOTAL.labels(wallet_name=self.wallet_name, method=method).inc()
                return None
        except Exception as e:
            logger.error(f"Exception calling {method}: {type(e).__name__}")
            WALLET_RPC_ERRORS_TOTAL.labels(wallet_name=self.wallet_name, method=method).inc()
            WALLET_RPC_CALLS_TOTAL.labels(wallet_name=self.wallet_name, method=method, status='error').inc()
            return None

    def get_balance(self):
        """Get wallet balance and update Prometheus metrics.

        Calls the Monero RPC get_balance method and updates:
        - monero_wallet_balance_piconero
        - monero_wallet_unlocked_balance_piconero
        - monero_wallet_num_unspent_outputs

        Returns:
            None (updates metrics as side effect)
        """
        result = self.call_rpc("get_balance")
        if result:
            balance = result.get('balance', 0)
            unlocked_balance = result.get('unlocked_balance', 0)
            num_unspent_outputs = result.get('num_unspent_outputs', 0)

            WALLET_BALANCE.labels(wallet_name=self.wallet_name).set(balance)
            WALLET_UNLOCKED_BALANCE.labels(wallet_name=self.wallet_name).set(unlocked_balance)
            WALLET_NUM_UNSPENT_OUTPUTS.labels(wallet_name=self.wallet_name).set(num_unspent_outputs)

            logger.info(f"{self.wallet_name}: Balance={balance/1e12:.8f} XMR, Unlocked={unlocked_balance/1e12:.8f} XMR")

    def get_height(self):
        """Get wallet blockchain sync height and update Prometheus metrics.

        Calls the Monero RPC get_height method and updates:
        - monero_wallet_height

        Returns:
            None (updates metrics as side effect)
        """
        result = self.call_rpc("get_height")
        if result:
            height = result.get('height', 0)
            WALLET_HEIGHT.labels(wallet_name=self.wallet_name).set(height)
            logger.info(f"{self.wallet_name}: Height={height}")

def main():
    # Configuration from environment variables
    EXPORTER_PORT = int(os.getenv('EXPORTER_PORT', 9101))
    POLL_INTERVAL = int(os.getenv('POLL_INTERVAL', 30))

    # Wallet configurations (read from environment)
    wallets = []

    # Buyer wallet
    if os.getenv('BUYER_RPC_HOST'):
        wallets.append(MoneroRPCClient(
            host=os.getenv('BUYER_RPC_HOST', 'monero-wallet-rpc-buyer'),
            port=int(os.getenv('BUYER_RPC_PORT', 18082)),
            wallet_name='buyer'
        ))

    # Vendor wallet
    if os.getenv('VENDOR_RPC_HOST'):
        wallets.append(MoneroRPCClient(
            host=os.getenv('VENDOR_RPC_HOST', 'monero-wallet-rpc-vendor'),
            port=int(os.getenv('VENDOR_RPC_PORT', 18083)),
            wallet_name='vendor'
        ))

    # Arbiter wallet
    if os.getenv('ARBITER_RPC_HOST'):
        wallets.append(MoneroRPCClient(
            host=os.getenv('ARBITER_RPC_HOST', 'monero-wallet-rpc-arbiter'),
            port=int(os.getenv('ARBITER_RPC_PORT', 18084)),
            wallet_name='arbiter'
        ))

    if not wallets:
        logger.warning("No wallets configured, using defaults")
        wallets = [
            MoneroRPCClient('monero-wallet-rpc-buyer', 18082, 'buyer'),
            MoneroRPCClient('monero-wallet-rpc-vendor', 18083, 'vendor'),
            MoneroRPCClient('monero-wallet-rpc-arbiter', 18084, 'arbiter'),
        ]

    # Start Prometheus HTTP server
    start_http_server(EXPORTER_PORT)
    logger.info(f"Monero exporter started on port {EXPORTER_PORT}")
    logger.info(f"Polling interval: {POLL_INTERVAL}s")
    logger.info(f"Monitoring {len(wallets)} wallets")

    # Main loop
    while True:
        for wallet in wallets:
            try:
                wallet.get_balance()
                wallet.get_height()
            except Exception as e:
                logger.error(f"Error polling {wallet.wallet_name}: {type(e).__name__}")

        time.sleep(POLL_INTERVAL)

if __name__ == '__main__':
    main()
