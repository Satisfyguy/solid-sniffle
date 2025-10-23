# Reality Check Tor: ipfs_health

**Date:** 2025-10-22
**Function:** `IpfsClient::is_available()`
**Location:** `server/src/ipfs/client.rs::is_available` (lines 293-304)
**Status:** ⏳ PENDING VALIDATION

## 🎯 Objectif de la Fonction

Health check to verify IPFS daemon is running and accessible. This function sends a POST request to `/api/v0/version` endpoint and returns `true` if the IPFS node responds successfully, `false` otherwise. Used before attempting upload/download operations to fail fast if IPFS is unavailable.

## 🔒 Garanties de Sécurité Requises

- [ ] Health check via Tor si IPFS node externe (Infura, public gateway)
- [ ] Health check via localhost si IPFS node local (127.0.0.1:5001)
- [ ] Pas de fuite IP lors de health checks vers nodes externes
- [ ] Pas de logs sensibles (IPFS node URLs, version info)
- [ ] User-Agent générique (anti-fingerprinting)
- [ ] Timeout approprié (≥10s pour Tor latency)
- [ ] Pas de révélation de node existence à adversaires externes

## ⚠️ VULNERABILITE CRITIQUE DETECTEE

**STATUT ACTUEL:** La fonction `IpfsClient::is_available()` N'UTILISE PAS Tor proxy.

**Problème identifié (ligne 67-71):**
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
    .pool_max_idle_per_host(10)
    .build()
    .context("Failed to build HTTP client")?;
```

**Impact OPSEC:**
1. Health checks vers nodes externes révèlent IP réelle
2. Révèle existence d'un IPFS node à des adversaires réseau
3. Version endpoint peut révéler IPFS daemon version (fingerprinting)
4. Health checks fréquents créent pattern de trafic identifiable
5. User-Agent par défaut révèle client Rust/reqwest

**Correctif requis:**
```rust
use reqwest::Proxy;

// Tor proxy pour nodes externes uniquement
let proxy = Proxy::all("socks5h://127.0.0.1:9050")
    .context("Failed to configure Tor proxy")?;

let client = reqwest::Client::builder()
    .proxy(proxy)  // Skip proxy if api_base_url is localhost
    .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
    .timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(10)
    .build()
    .context("Failed to build HTTP client")?;
```

**Note importante:** Health checks vers localhost (127.0.0.1:5001) ne devraient PAS utiliser Tor (overhead inutile). Seuls les health checks vers nodes externes nécessitent Tor.

## 🧪 Tests Automatiques

### 1. Vérification Configuration Tor
```bash
#!/bin/bash
set -euo pipefail

echo "=== Reality Check: IPFS Health Check via Tor ==="
echo "Function: server/src/ipfs/client.rs::is_available()"
echo "Date: 2025-10-22"
echo ""

# Test 1: Tor daemon running (nécessaire pour nodes externes seulement)
echo "[Test 1] Checking Tor daemon status..."
if curl --socks5-hostname 127.0.0.1:9050 -s -m 10 https://check.torproject.org 2>/dev/null | grep -q "Congratulations"; then
    echo "✅ PASS: Tor daemon running (required for external IPFS nodes)"
    TOR_AVAILABLE=true
else
    echo "⚠️  WARNING: Tor daemon not running"
    echo "Required only for external IPFS nodes (Infura, public gateways)"
    TOR_AVAILABLE=false
fi

# Test 2: Distinguish localhost vs external node
echo ""
echo "[Test 2] Detecting IPFS node type..."
IPFS_API_URL="${IPFS_API_URL:-http://127.0.0.1:5001/api/v0}"

if [[ "$IPFS_API_URL" == *"127.0.0.1"* ]] || [[ "$IPFS_API_URL" == *"localhost"* ]]; then
    echo "✅ INFO: Using local IPFS node (Tor proxy not required)"
    NODE_TYPE="local"
    REQUIRES_TOR=false
elif [[ "$IPFS_API_URL" == *"infura"* ]] || [[ "$IPFS_API_URL" == *"ipfs.io"* ]]; then
    echo "⚠️  INFO: Using external IPFS node (Tor proxy REQUIRED)"
    NODE_TYPE="external"
    REQUIRES_TOR=true
else
    echo "⚠️  WARNING: Unknown IPFS node type: $IPFS_API_URL"
    NODE_TYPE="unknown"
    REQUIRES_TOR=true  # Assume external for safety
fi

# Test 3: Code uses Tor proxy for external nodes
echo ""
echo "[Test 3] Checking Tor proxy configuration in code..."
if grep -q "Proxy::all.*127.0.0.1:9050" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
    echo "✅ PASS: Code configures Tor SOCKS5 proxy"

    # Check if conditional based on URL
    if grep -q "if.*localhost\|if.*127.0.0.1" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
        echo "✅ PASS: Proxy configuration is conditional (localhost vs external)"
    else
        echo "⚠️  WARNING: Proxy applied to all requests (including localhost)"
        echo "Recommendation: Skip Tor proxy for localhost health checks (performance)"
    fi
else
    if [ "$REQUIRES_TOR" = true ]; then
        echo "❌ CRITICAL FAIL: Code does NOT use Tor proxy for external nodes"
        echo "OPSEC RISK: Health checks to external IPFS nodes leak real IP"
        echo "Location: server/src/ipfs/client.rs::new() (lines 67-71)"
        echo ""
        echo "⛔ BLOCKING ISSUE for external IPFS usage"
        exit 1
    else
        echo "⚠️  INFO: Tor proxy not configured (OK for localhost-only usage)"
    fi
fi

# Test 4: Generic User-Agent configured
echo ""
echo "[Test 4] Checking User-Agent configuration..."
if grep -q "user_agent.*Mozilla" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
    echo "✅ PASS: Generic User-Agent configured (anti-fingerprinting)"
else
    echo "⚠️  WARNING: User-Agent not configured (uses default reqwest UA)"
    echo "Fingerprinting risk: Version endpoint + reqwest UA = unique fingerprint"
fi

# Test 5: Health check doesn't expose sensitive info
echo ""
echo "[Test 5] Testing health check endpoint..."
if [ "$NODE_TYPE" = "local" ]; then
    RESPONSE=$(curl -s -X POST http://127.0.0.1:5001/api/v0/version 2>/dev/null || echo "failed")

    if [[ "$RESPONSE" == *"Version"* ]]; then
        echo "✅ PASS: Local IPFS node responding to health check"

        # Check if version is logged
        VERSION=$(echo "$RESPONSE" | jq -r '.Version' 2>/dev/null || echo "unknown")
        echo "ℹ️  IPFS Version: $VERSION"
        echo "⚠️  WARNING: Version info should not be logged in production"
    else
        echo "❌ FAIL: Local IPFS node not responding"
        echo "Expected: IPFS daemon running on 127.0.0.1:5001"
    fi
fi

# Test 6: Timing pattern analysis
echo ""
echo "[Test 6] Checking health check frequency..."
echo "⚠️  MANUAL CHECK REQUIRED"
echo "Health checks should be rate-limited to avoid traffic analysis:"
echo "  - Max frequency: 1 check per 60 seconds"
echo "  - Add random jitter: +/- 10 seconds"
echo "  - Cache results for 30 seconds minimum"

# Test 7: No version info logged in production
echo ""
echo "[Test 7] Checking for version logging..."
if grep -E "version|Version" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs | grep -i log; then
    echo "⚠️  WARNING: IPFS version may be logged"
    echo "Privacy risk: Version info aids fingerprinting"
    echo "Recommendation: Only log 'IPFS node available: true/false'"
else
    echo "✅ PASS: No version logging detected"
fi

echo ""
echo "========================================="
if [ "$REQUIRES_TOR" = true ] && ! grep -q "Proxy::all" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
    echo "⚠️  CRITICAL: Tor proxy NOT configured for external nodes"
    echo "Status: NOT PRODUCTION READY for external IPFS usage"
    exit 1
else
    echo "✅ Health check configuration validated"
    echo "Status: Ready for localhost usage only"
fi
echo "========================================="
```

### 2. Test d'Exécution de la Fonction
```bash
#!/bin/bash
# Test IPFS health check function

cd c:/Users/Lenovo/monero-marketplace

# Test 1: Local IPFS node health check
echo "Test 1: Local IPFS node health check..."
if curl -s -X POST http://127.0.0.1:5001/api/v0/version; then
    echo "✅ PASS: Local IPFS node is available"
else
    echo "❌ FAIL: Local IPFS node not responding"
    echo "Start daemon: ipfs daemon"
fi

# Test 2: External node health check via Tor
echo ""
echo "Test 2: External IPFS node health check (via Tor)..."
if curl --socks5-hostname 127.0.0.1:9050 -s -X POST https://ipfs.infura.io:5001/api/v0/version; then
    echo "✅ PASS: External IPFS node accessible via Tor"
else
    echo "⚠️  WARNING: External node not accessible (may require auth)"
fi

# Test 3: Run Rust client test
echo ""
echo "Test 3: Running IpfsClient::is_available() test..."
cargo test --package server --lib ipfs::client::tests::test_ipfs_client_creation -- --nocapture

# Test 4: Measure health check latency
echo ""
echo "Test 4: Health check latency measurement..."
echo "Local node:"
time curl -s -X POST http://127.0.0.1:5001/api/v0/version > /dev/null

echo ""
echo "Via Tor (if external):"
time curl --socks5-hostname 127.0.0.1:9050 -s -X POST https://ipfs.infura.io:5001/api/v0/version > /dev/null

# Expected latency:
# Local: < 50ms
# Via Tor: 500ms - 3000ms
```

## 📋 Tests Manuels Requis

### Test 1: DNS Leak Check
```bash
echo "[Manual Test 1] DNS Leak Detection for Health Checks"
echo "Objective: Ensure health checks don't leak DNS queries"
echo ""

# Only relevant for external IPFS nodes

# Setup: Monitor DNS traffic
# sudo tcpdump -i any -n port 53 -w /tmp/dns-leak-health.pcap &

# Step 1: Perform health check to external node
echo "Performing health check to external IPFS node..."
curl --socks5-hostname 127.0.0.1:9050 -s -X POST https://ipfs.infura.io:5001/api/v0/version

# Step 2: Analyze DNS queries
# sudo killall tcpdump
# tcpdump -r /tmp/dns-leak-health.pcap -n port 53

# Expected:
# ✅ PASS: No DNS queries to public resolvers (socks5h:// handles DNS via Tor)
# ❌ FAIL: DNS queries for "ipfs.infura.io" visible in cleartext

echo ""
echo "VALIDATION:"
echo "  ✅ DNS resolution via Tor (socks5h://)"
echo "  ✅ No cleartext DNS queries"
echo ""
```

### Test 2: Timing Pattern Analysis
```bash
echo "[Manual Test 2] Health Check Timing Pattern Analysis"
echo "Objective: Verify health checks don't create identifiable patterns"
echo ""

# Adversary threat: Network observer detects regular health check pattern

# Step 1: Perform 10 health checks with current implementation
echo "Step 1: Performing 10 health checks (observe timing)..."
for i in {1..10}; do
    time curl -s -X POST http://127.0.0.1:5001/api/v0/version > /dev/null
    sleep 5
done

# Step 2: Analyze timing pattern
# Are intervals consistent? (BAD)
# Are intervals random? (GOOD)

# Recommended pattern:
# - Base interval: 60 seconds
# - Random jitter: +/- 10 seconds
# - Result: Health checks every 50-70 seconds (unpredictable)

echo ""
echo "VALIDATION:"
echo "  ✅ Health checks have random jitter (not fixed intervals)"
echo "  ✅ Minimum interval ≥ 30 seconds (not too frequent)"
echo "  ✅ Results cached to reduce frequency"
echo ""
```

### Test 3: Fingerprinting via Version Endpoint
```bash
echo "[Manual Test 3] Version Endpoint Fingerprinting"
echo "Objective: Assess information leakage from /version endpoint"
echo ""

# Step 1: Query version endpoint
VERSION_RESPONSE=$(curl -s -X POST http://127.0.0.1:5001/api/v0/version)

echo "Version response:"
echo "$VERSION_RESPONSE" | jq .

# Example response:
# {
#   "Version": "0.14.0",
#   "Commit": "abc123...",
#   "Repo": "11",
#   "System": "amd64/linux",
#   "Golang": "go1.19.1"
# }

# Information leaked:
# ⚠️  IPFS version (aids exploit targeting)
# ⚠️  OS and architecture (fingerprinting)
# ⚠️  Go version (additional fingerprinting)

echo ""
echo "MITIGATION STRATEGIES:"
echo "  1. Don't log version info in production"
echo "  2. Only use is_available() for boolean check"
echo "  3. Consider alternative health check (e.g., /api/v0/id with limited info)"
echo "  4. Rate-limit health checks to reduce info gathering"
echo ""

echo "VALIDATION:"
echo "  ✅ Version info NOT logged or exposed to users"
echo "  ✅ Only boolean 'available' status returned"
echo "  ✅ Health check rate-limited"
echo ""
```

### Test 4: Health Check Failure Modes
```bash
echo "[Manual Test 4] Health Check Failure Handling"
echo "Objective: Test graceful handling of IPFS unavailability"
echo ""

# Test 1: IPFS daemon stopped
echo "Test 1: IPFS daemon stopped..."
# Stop daemon: killall ipfs
RESULT=$(curl -s -X POST http://127.0.0.1:5001/api/v0/version 2>&1 || echo "connection refused")

if [[ "$RESULT" == *"refused"* ]]; then
    echo "✅ PASS: Connection refused handled gracefully"
else
    echo "❌ FAIL: Unexpected response when daemon down"
fi

# Test 2: Network timeout
echo "Test 2: Network timeout simulation..."
# Use very short timeout
timeout 1s curl -s -X POST http://127.0.0.1:5001/api/v0/version || echo "✅ Timeout handled"

# Test 3: Tor unavailable (for external nodes)
echo "Test 3: Tor unavailable..."
# Stop Tor: sudo systemctl stop tor
curl --socks5-hostname 127.0.0.1:9050 -s -X POST https://ipfs.infura.io:5001/api/v0/version 2>&1 || echo "✅ Tor failure handled"

# Expected behavior:
# ✅ is_available() returns false (no panic, no error log spam)
# ✅ Application continues without IPFS functionality
# ✅ User notified gracefully: "IPFS unavailable"

echo ""
echo "VALIDATION:"
echo "  ✅ No panics on health check failure"
echo "  ✅ Boolean false returned on any error"
echo "  ✅ Errors logged at DEBUG level (not ERROR)"
echo "  ✅ Application remains functional without IPFS"
echo ""
```

### Test 5: Concurrent Health Checks
```bash
echo "[Manual Test 5] Concurrent Health Check Stress Test"
echo "Objective: Verify thread safety and rate limiting"
echo ""

# Simulate multiple concurrent health checks
echo "Performing 100 concurrent health checks..."

for i in {1..100}; do
    curl -s -X POST http://127.0.0.1:5001/api/v0/version > /dev/null &
done

wait

# Check IPFS daemon logs for errors
# Check for connection pool exhaustion
# Verify no panics or crashes

# Expected behavior:
# ✅ Connection pooling prevents IPFS overload
# ✅ Rate limiting prevents DoS against local daemon
# ✅ All requests complete successfully (or timeout gracefully)

echo ""
echo "VALIDATION:"
echo "  ✅ No connection pool exhaustion"
echo "  ✅ IPFS daemon remains responsive"
echo "  ✅ Rate limiting protects daemon"
echo "  ✅ No error log spam"
echo ""
```

## ⚠️ Risques Identifiés

### Risque Critique #1: Pas de Tor Proxy pour Nodes Externes
**Sévérité:** CRITIQUE (si Infura ou gateway public utilisé)
**Impact:** Health checks révèlent IP réelle au provider IPFS externe
**Statut:** NON MITIGE (code actuel)
**Mitigation:**
```rust
// Conditional Tor proxy
let proxy = if api_base_url.contains("127.0.0.1") || api_base_url.contains("localhost") {
    None
} else {
    Some(Proxy::all("socks5h://127.0.0.1:9050")?)
};

let mut builder = reqwest::Client::builder();
if let Some(p) = proxy {
    builder = builder.proxy(p);
}
```

### Risque Elevé #2: Version Fingerprinting
**Sévérité:** MOYENNE
**Impact:** `/version` endpoint révèle IPFS version, OS, Go version (fingerprinting détaillé)
**Statut:** INHERENT AU ENDPOINT
**Mitigation:**
- Ne pas logger version info
- Considérer endpoint alternatif pour health check (e.g., HEAD request)
- Cacher résultat health check (30-60s) pour réduire fréquence

### Risque Elevé #3: Timing Pattern de Health Checks
**Sévérité:** MOYENNE
**Impact:** Health checks réguliers créent pattern de trafic identifiable
**Statut:** PAS DE JITTER IMPLEMENTE
**Mitigation:**
- Ajouter random jitter (±10s) aux intervalles
- Cacher résultats pour réduire fréquence
- Randomiser premier check au démarrage

### Risque Moyen #4: User-Agent Fingerprinting
**Sévérité:** FAIBLE (pour health check seulement)
**Impact:** User-Agent + version endpoint = fingerprint plus précis
**Statut:** NON MITIGE (code actuel)
**Mitigation:** Configurer `.user_agent("Mozilla/5.0 ...")` même pour health checks

### Risque Faible #5: Health Check Failure Patterns
**Sévérité:** FAIBLE
**Impact:** Failures répétés peuvent indiquer attaque ou problème infra
**Statut:** LOGGING DEPENDANT
**Mitigation:**
- Log failures à DEBUG level seulement
- Aggregate failures sur période (pas de log par check)
- Alert seulement si >10 failures consécutives

### Risque Faible #6: Connection Pool Fingerprinting
**Sévérité:** FAIBLE
**Impact:** Connection pooling comportement pourrait être fingerprinted
**Statut:** INHERENT A REQWEST
**Mitigation:** Acceptable (pool_max_idle_per_host=10 est raisonnable)

## ✅ Validation Finale

- [ ] Tests automatiques exécutés avec succès
- [ ] Tests manuels DNS leak: PASS (pour external nodes)
- [ ] Tests manuels timing pattern: PASS (avec jitter)
- [ ] Tests manuels fingerprinting: PASS (version pas loggée)
- [ ] Tests manuels failure handling: PASS (graceful degradation)
- [ ] Tests manuels concurrent stress: PASS (rate limiting OK)
- [ ] Code review par un autre développeur
- [ ] Conditional Tor proxy implémenté (localhost vs external)
- [ ] Health check caching implémenté (30-60s)
- [ ] Random jitter ajouté aux intervals
- [ ] Documentation à jour

**Validé par:** _____________
**Date de validation:** _____________

## 📚 Références

- [Tor Project Best Practices](https://2019.www.torproject.org/docs/tor-manual.html.en)
- [IPFS API Documentation](https://docs.ipfs.tech/reference/kubo/rpc/)
- [Reqwest SOCKS Proxy](https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html)
- [Network Timing Attacks](https://en.wikipedia.org/wiki/Timing_attack)
- [Service Fingerprinting](https://en.wikipedia.org/wiki/TCP/IP_stack_fingerprinting)
- Project: `docs/SECURITY-THEATRE-PREVENTION.md`
- Project: `scripts/validate-reality-check-tor.sh`
- Related: `server/src/ipfs/client.rs` (lines 66-78, 293-304)

## 🚨 Action Immédiate Requise

**AVANT DE MERGER CE CODE EN PRODUCTION:**

1. **Pour usage localhost uniquement:**
   - ✅ Code actuel OK (pas de Tor nécessaire)
   - Documenter dans README: "IPFS doit être local (127.0.0.1:5001)"

2. **Pour usage external nodes (Infura, gateways):**
   - ❌ BLOQUANT: Implémenter conditional Tor proxy
   - Configurer User-Agent générique
   - Implémenter health check caching (30-60s)
   - Ajouter random jitter aux intervals
   - Ne PAS logger version info en production

3. **Tests requis:**
   - DNS leak test (pour external)
   - Timing pattern analysis
   - Failure mode handling
   - Concurrent stress test

4. **Documentation:**
   - Clarifier localhost vs external node requirements
   - Documenter IPFS daemon configuration pour Tor (si applicable)
   - Ajouter troubleshooting guide pour health check failures

**Décision architecture requise:** Ce projet utilisera-t-il uniquement local IPFS node, ou supportera-t-il external gateways? Cette décision détermine l'urgence du correctif Tor.

**Recommandation:** Pour simplifier OPSEC, **restreindre à localhost IPFS uniquement** en production. Cela élimine besoin de Tor pour health checks et réduit surface d'attaque.
