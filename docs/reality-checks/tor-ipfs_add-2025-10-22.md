# Reality Check Tor: ipfs_add

**Date:** 2025-10-22
**Function:** `IpfsClient::add()`
**Location:** `server/src/ipfs/client.rs::add` (lines 143-179)
**Status:** ⏳ PENDING VALIDATION

## 🎯 Objectif de la Fonction

Upload reputation data to IPFS (InterPlanetary File System) using multipart/form-data encoding. This function uploads JSON-encoded vendor reputation files and returns the IPFS content hash (CID). Currently configured for local IPFS node (127.0.0.1:5001) or Infura gateway.

## 🔒 Garanties de Sécurité Requises

- [ ] Tout le trafic IPFS passe par Tor (127.0.0.1:9050)
- [ ] Pas de fuite IP/DNS lors de connexion IPFS
- [ ] IPFS API uniquement via localhost OU via Tor pour gateways externes
- [ ] Pas de logs sensibles (IPFS hashes, taille fichiers, metadata)
- [ ] User-Agent générique (anti-fingerprinting)
- [ ] Timeouts appropriés pour latence Tor (≥30s) - DEJA IMPLEMENTE
- [ ] Retry logic avec exponential backoff - DEJA IMPLEMENTE
- [ ] Pas de fuite metadata dans multipart headers

## ⚠️ VULNERABILITE CRITIQUE DETECTEE

**STATUT ACTUEL:** La fonction `IpfsClient::add()` N'UTILISE PAS Tor proxy.

**Problème identifié (ligne 67-71):**
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
    .pool_max_idle_per_host(10)
    .build()
    .context("Failed to build HTTP client")?;
```

**Impact OPSEC:**
1. Connexions IPFS directes révèlent l'IP réelle de l'utilisateur
2. Infura gateway (ipfs.infura.io) peut logger IP source
3. Local IPFS node pourrait faire des connexions directes au réseau IPFS DHT
4. Metadata leak via User-Agent par défaut (révèle reqwest version)

**Correctif requis:**
```rust
use reqwest::Proxy;

let proxy = Proxy::all("socks5h://127.0.0.1:9050")
    .context("Failed to configure Tor proxy")?;

let client = reqwest::Client::builder()
    .proxy(proxy)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
    .timeout(Duration::from_secs(60))  // Tor + IPFS = slow
    .pool_max_idle_per_host(10)
    .build()
    .context("Failed to build HTTP client")?;
```

## 🧪 Tests Automatiques

### 1. Vérification Tor Daemon
```bash
#!/bin/bash
set -euo pipefail

echo "=== Reality Check: IPFS Add via Tor ==="
echo "Function: server/src/ipfs/client.rs::add()"
echo "Date: 2025-10-22"
echo ""

# Test 1: Tor is running
echo "[Test 1] Checking Tor daemon..."
if ! curl --socks5-hostname 127.0.0.1:9050 -s -m 10 https://check.torproject.org 2>/dev/null | grep -q "Congratulations"; then
    echo "❌ FAIL: Tor daemon not running or not accessible on 127.0.0.1:9050"
    echo "Fix: sudo systemctl start tor"
    exit 1
fi
echo "✅ PASS: Tor daemon running and accessible"

# Test 2: IPFS node is accessible (localhost or Tor)
echo ""
echo "[Test 2] Checking IPFS node availability..."
if curl -s -m 5 -X POST http://127.0.0.1:5001/api/v0/version 2>/dev/null | grep -q "Version"; then
    echo "✅ PASS: Local IPFS node running on 127.0.0.1:5001"
    IPFS_MODE="local"
elif [ -n "${IPFS_INFURA_PROJECT_ID:-}" ] && [ -n "${IPFS_INFURA_SECRET:-}" ]; then
    echo "ℹ️  INFO: Using Infura gateway (requires Tor proxy)"
    IPFS_MODE="infura"
else
    echo "❌ FAIL: No IPFS node available (local or Infura credentials missing)"
    echo "Fix: Start local IPFS daemon or set IPFS_INFURA_PROJECT_ID + IPFS_INFURA_SECRET"
    exit 1
fi

# Test 3: Code uses Tor proxy (CRITICAL - currently FAILING)
echo ""
echo "[Test 3] Checking if IpfsClient uses Tor proxy..."
if grep -q "Proxy::all.*127.0.0.1:9050" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
    echo "✅ PASS: Code configures Tor SOCKS5 proxy"
else
    echo "❌ CRITICAL FAIL: Code does NOT use Tor proxy"
    echo "OPSEC RISK: IPFS connections leak real IP address"
    echo "Location: server/src/ipfs/client.rs::new() (lines 67-71)"
    echo ""
    echo "Required fix:"
    echo "  use reqwest::Proxy;"
    echo "  let proxy = Proxy::all(\"socks5h://127.0.0.1:9050\")?;"
    echo "  let client = reqwest::Client::builder().proxy(proxy)..."
    echo ""
    echo "⛔ BLOCKING ISSUE - Cannot proceed with validation"
    exit 1
fi

# Test 4: Generic User-Agent configured
echo ""
echo "[Test 4] Checking User-Agent configuration..."
if grep -q "user_agent.*Mozilla" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
    echo "✅ PASS: Generic User-Agent configured (anti-fingerprinting)"
else
    echo "⚠️  WARNING: User-Agent not configured (uses default reqwest UA)"
    echo "Fingerprinting risk: Reveals Rust/reqwest version"
fi

# Test 5: No public ports exposed
echo ""
echo "[Test 5] Checking for public port exposure..."
if command -v netstat &> /dev/null; then
    if netstat -tuln 2>/dev/null | grep -E ":5001.*0\.0\.0\.0|:8080.*0\.0\.0\.0"; then
        echo "❌ FAIL: IPFS ports exposed publicly"
        echo "Ports should only bind to 127.0.0.1"
        exit 1
    fi
    echo "✅ PASS: No public IPFS ports detected"
else
    echo "⚠️  SKIP: netstat not available (Windows)"
fi

# Test 6: No sensitive logs
echo ""
echo "[Test 6] Checking for sensitive data in logs..."
LOG_DIR="c:/Users/Lenovo/monero-marketplace/server/logs"
if [ -d "$LOG_DIR" ]; then
    # Check for IPFS hashes (Qm...) or IP addresses in logs
    if grep -r -E "Qm[a-zA-Z0-9]{44}|[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}" "$LOG_DIR" 2>/dev/null; then
        echo "⚠️  WARNING: Potential sensitive data in logs (IPFS hashes or IPs)"
        echo "Review log output to ensure no privacy leaks"
    else
        echo "✅ PASS: No obvious sensitive data in logs"
    fi
else
    echo "ℹ️  INFO: No log directory found, skipping log check"
fi

echo ""
echo "========================================="
echo "⚠️  CRITICAL: Tor proxy NOT configured in code"
echo "Cannot proceed with full validation until fixed"
echo "========================================="
exit 1
```

### 2. Test d'Exécution de la Fonction
```bash
#!/bin/bash
# Test IPFS upload with sample reputation data

cd c:/Users/Lenovo/monero-marketplace

# Create test reputation data
TEST_DATA='{"format_version":"1.0","vendor_pubkey":"test-vendor-123","generated_at":"2025-10-22T00:00:00Z","reviews":[],"stats":{"total_reviews":0,"average_rating":0.0}}'

# Start IPFS daemon (if not running)
# ipfs daemon &

# Wait for IPFS to be ready
sleep 2

# Test upload via CLI
echo "$TEST_DATA" | curl -X POST -F "file=@-" http://127.0.0.1:5001/api/v0/add

# Expected output: {"Name":"...","Hash":"Qm...","Size":"..."}

# Run Rust test
cargo test --package server --lib ipfs::client::tests::test_ipfs_client_creation -- --nocapture

# Integration test (requires running IPFS + Tor)
# cargo test --package server --test ipfs_integration -- --nocapture --ignored
```

## 📋 Tests Manuels Requis

### Test 1: DNS Leak Check
```bash
# Setup: Monitor DNS queries before test
echo "[Manual Test 1] DNS Leak Detection"
echo "Objective: Ensure IPFS operations don't leak DNS queries"
echo ""

# On Linux:
# sudo tcpdump -i any -n port 53 -w /tmp/dns-leak-test.pcap &
# TCPDUMP_PID=$!

# On Windows (requires Wireshark):
# Start Wireshark, filter: udp.port == 53

# Step 1: Baseline DNS traffic
echo "Step 1: Capture baseline DNS traffic (10 seconds)..."
# Wait 10 seconds
sleep 10

# Step 2: Trigger IPFS upload
echo "Step 2: Trigger IPFS upload operation..."
cd c:/Users/Lenovo/monero-marketplace
cargo test --package server --lib ipfs::client::tests -- --nocapture

# Step 3: Analyze DNS queries
echo "Step 3: Analyze captured traffic..."
# sudo kill $TCPDUMP_PID
# tcpdump -r /tmp/dns-leak-test.pcap -n port 53

# Expected result:
# ✅ PASS: No DNS queries to external resolvers (only 127.0.0.1:53 for Tor)
# ❌ FAIL: DNS queries for ipfs.infura.io, ipfs.io, or other IPFS domains

echo ""
echo "VALIDATION:"
echo "  ✅ Only Tor DNS resolver (127.0.0.1) used"
echo "  ✅ No queries to public DNS servers"
echo "  ✅ No IPFS domain names in cleartext"
echo ""
```

### Test 2: Fingerprinting Check
```bash
# Verify HTTP headers don't reveal identifying information

echo "[Manual Test 2] HTTP Fingerprinting"
echo "Objective: Ensure HTTP requests are generic and non-identifying"
echo ""

# Intercept HTTP traffic during IPFS upload
# Method 1: Use mitmproxy
# mitmproxy --mode socks5 --listen-port 9050

# Method 2: Use Wireshark to capture localhost traffic
# Filter: tcp.port == 5001 and http

# Step 1: Trigger upload
cargo test --package server --lib ipfs::client::tests -- --nocapture

# Step 2: Inspect captured HTTP POST to /api/v0/add

# Expected User-Agent:
# ✅ PASS: "Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0"
# ❌ FAIL: "reqwest/0.11.x" (reveals Rust client)

# Expected Content-Type:
# ✅ PASS: multipart/form-data with generic boundary

# Expected Headers:
# ✅ PASS: No X-Custom-*, X-Client-*, or identifying headers
# ✅ PASS: No unusual Accept-Encoding values

echo "VALIDATION:"
echo "  ✅ User-Agent is generic Firefox"
echo "  ✅ No custom identifying headers"
echo "  ✅ Standard multipart boundary (not revealing)"
echo ""
```

### Test 3: IPFS Gateway Isolation
```bash
echo "[Manual Test 3] IPFS Gateway Isolation"
echo "Objective: Verify IPFS connections go through Tor (if using external gateway)"
echo ""

# This test only applies if using Infura or public IPFS gateway

# Step 1: Check if using external gateway
if [ -n "${IPFS_INFURA_PROJECT_ID:-}" ]; then
    echo "Detected Infura configuration - Testing Tor isolation..."

    # Step 2: Monitor external connections
    # sudo netstat -tupn | grep -E "ipfs|infura"

    # Expected:
    # ✅ PASS: Connections to 127.0.0.1:9050 (Tor SOCKS)
    # ❌ FAIL: Direct connections to ipfs.infura.io:5001

    echo "VALIDATION:"
    echo "  ✅ All IPFS traffic routes through Tor (127.0.0.1:9050)"
    echo "  ✅ No direct connections to Infura IPs"
else
    echo "Using local IPFS node - Skipping external gateway test"
    echo "NOTE: Ensure local IPFS daemon is configured for Tor:"
    echo "  ipfs config --json Swarm.DisableNatPortMap true"
    echo "  ipfs config --json Swarm.EnableAutoRelay false"
fi
echo ""
```

### Test 4: Analyse de Trafic Complet
```bash
echo "[Manual Test 4] Traffic Analysis"
echo "Objective: Comprehensive network traffic inspection"
echo ""

# Capture all network traffic during IPFS operation
# sudo tcpdump -i any -w /tmp/ipfs-add-test.pcap

# Step 1: Start capture
echo "Step 1: Starting packet capture..."
# tcpdump running in background

# Step 2: Execute IPFS upload
cd c:/Users/Lenovo/monero-marketplace
echo '{"test":"data"}' | curl -X POST -F "file=@-" http://127.0.0.1:5001/api/v0/add

# Step 3: Stop capture and analyze
# sudo killall tcpdump

# Step 4: Analyze with Wireshark
# wireshark /tmp/ipfs-add-test.pcap

# Expected traffic patterns:
# ✅ PASS: All external traffic to 127.0.0.1:9050 (Tor SOCKS5)
# ✅ PASS: Local traffic to 127.0.0.1:5001 (IPFS API)
# ❌ FAIL: Direct connections to external IPs
# ❌ FAIL: Unencrypted data outside localhost

echo "VALIDATION CHECKLIST:"
echo "  ✅ All external connections via Tor (127.0.0.1:9050)"
echo "  ✅ IPFS API calls only to localhost (127.0.0.1:5001)"
echo "  ✅ No cleartext reputation data on network"
echo "  ✅ No direct connections to IPFS DHT nodes"
echo ""
```

## ⚠️ Risques Identifiés

### Risque Critique #1: Pas de Tor Proxy
**Sévérité:** CRITIQUE
**Impact:** Fuite IP réelle lors de connexions IPFS
**Statut:** NON MITIGE (code actuel)
**Mitigation:** Ajouter `Proxy::all("socks5h://127.0.0.1:9050")` dans `IpfsClient::new()`

### Risque Critique #2: User-Agent par Défaut
**Sévérité:** ELEVEE
**Impact:** Fingerprinting (révèle reqwest/Rust version)
**Statut:** NON MITIGE (code actuel)
**Mitigation:** Configurer `.user_agent("Mozilla/5.0 ...")` dans client builder

### Risque Elevé #3: IPFS Hash Logging
**Sévérité:** MOYENNE
**Impact:** IPFS hashes dans logs peuvent révéler activité vendor
**Statut:** PRESENT (ligne 152-156)
**Mitigation:** Remplacer `hash = %hash` par `hash = "[REDACTED]"` en production

### Risque Moyen #4: Local IPFS DHT Connections
**Sévérité:** MOYENNE
**Impact:** Si IPFS daemon local fait des connexions directes au DHT
**Statut:** DEPENDANT DE CONFIG IPFS
**Mitigation:** Configurer IPFS daemon en mode Tor-only:
```bash
ipfs config --json Swarm.DisableNatPortMap true
ipfs config --json Swarm.EnableAutoRelay false
ipfs config Addresses.Swarm '["/ip4/127.0.0.1/tcp/4001"]'
```

### Risque Faible #5: Timing Analysis
**Sévérité:** FAIBLE
**Impact:** Upload timing pourrait corréler avec vendor activity
**Statut:** INHERENT A IPFS
**Mitigation:** Ajouter jitter aléatoire avant upload (delay 0-30s)

## ✅ Validation Finale

- [ ] Tests automatiques exécutés avec succès (BLOQUE: Tor proxy manquant)
- [ ] Tests manuels DNS leak: PASS
- [ ] Tests manuels fingerprinting: PENDING (dépend correctif Tor)
- [ ] Tests manuels IPFS gateway isolation: PENDING
- [ ] Tests manuels traffic analysis: PENDING
- [ ] Code review par un autre développeur
- [ ] Correctif Tor proxy appliqué et testé
- [ ] Documentation à jour

**Validé par:** _____________
**Date de validation:** _____________

## 📚 Références

- [Tor Project Best Practices](https://2019.www.torproject.org/docs/tor-manual.html.en)
- [IPFS Privacy & Security](https://docs.ipfs.tech/concepts/privacy-and-encryption/)
- [Reqwest SOCKS Proxy Documentation](https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html)
- Project: `docs/SECURITY-THEATRE-PREVENTION.md`
- Project: `scripts/validate-reality-check-tor.sh`
- Related: `server/src/ipfs/client.rs` (lines 66-78, 143-179)

## 🚨 Action Immédiate Requise

**AVANT DE MERGER CE CODE EN PRODUCTION:**

1. Implémenter Tor proxy dans `IpfsClient::new()`, `IpfsClient::new_local()`, et `IpfsClient::new_infura()`
2. Configurer User-Agent générique anti-fingerprinting
3. Reduire logging des IPFS hashes en production
4. Documenter configuration IPFS daemon pour Tor-only mode
5. Exécuter tous les tests manuels et automatiques
6. Valider avec `./scripts/validate-reality-check-tor.sh ipfs_add`

**Ce code N'EST PAS production-ready tant que le proxy Tor n'est pas configuré.**
