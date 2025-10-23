#!/bin/bash
set -euo pipefail

echo "🚀 Configuring UFW Firewall..."

# ============================================================================
# Configuration
# ============================================================================
SSH_PORT="22"
HTTPS_PORT="443"
PROMETHEUS_PORT="9090"
GRAFANA_PORT="3000"
BACKEND_PORT="8080"
MONERO_RPC_PORTS="18082,18083,18084"

# Internal IPs for monitoring access (replace with actual internal network ranges)
INTERNAL_IPS="192.168.1.0/24 10.0.0.0/8"

# ============================================================================
# Script Logic
# ============================================================================

# 1. Reset UFW to a clean state
echo "  - Resetting UFW..."
ufw --force reset

# 2. Set default policies
echo "  - Setting default policies: deny incoming, allow outgoing..."
ufw default deny incoming
ufw default allow outgoing

# 3. Allow SSH (rate-limited)
echo "  - Allowing SSH on port ${SSH_PORT} (rate-limited)..."
ufw limit "$SSH_PORT"/tcp

# 4. Allow HTTPS
echo "  - Allowing HTTPS on port ${HTTPS_PORT}..."
ufw allow "$HTTPS_PORT"/tcp

# 5. Allow Prometheus from internal IPs only
echo "  - Allowing Prometheus on port ${PROMETHEUS_PORT} from internal IPs..."
for ip in $INTERNAL_IPS; do
    ufw allow from "$ip" to any port "$PROMETHEUS_PORT" proto tcp
done

# 6. Allow Grafana from internal IPs only
echo "  - Allowing Grafana on port ${GRAFANA_PORT} from internal IPs..."
for ip in $INTERNAL_IPS; do
    ufw allow from "$ip" to any port "$GRAFANA_PORT" proto tcp
done

# 7. Deny direct access to backend application port
echo "  - Denying direct access to backend application on port ${BACKEND_PORT}..."
ufw deny "$BACKEND_PORT"/tcp

# 8. Deny direct access to Monero RPC ports
echo "  - Denying direct access to Monero RPC ports ${MONERO_RPC_PORTS}..."
ufw deny "$MONERO_RPC_PORTS"/tcp

# 9. Enable UFW
echo "  - Enabling UFW..."
ufw --force enable

echo "✅ UFW firewall configured successfully."
ufw status verbose
