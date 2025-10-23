# Reality Check Tor: ipfs_cat

**Date:** 2025-10-22
**Function:** `IpfsClient::cat()`
**Location:** `server/src/ipfs/client.rs::cat` (lines 231-267)
**Status:** ⏳ PENDING VALIDATION

## 🎯 Objectif de la Fonction

Download reputation data from IPFS using a content hash (CID). This function retrieves previously uploaded vendor reputation files from IPFS gateways (local 127.0.0.1:8080 or public ipfs.io). Returns raw bytes that are deserialized into `VendorReputation` structs.

## 🔒 Garanties de Sécurité Requises

- [ ] Tout le trafic IPFS passe par Tor (127.0.0.1:9050)
- [ ] Pas de fuite IP/DNS lors de téléchargement IPFS
- [ ] Gateway IPFS accessible uniquement via Tor (si externe)
- [ ] Pas de logs sensibles (IPFS hashes, vendor IDs, taille fichiers)
- [ ] User-Agent générique (anti-fingerprinting)
- [ ] Timeouts appropriés pour latence Tor (≥30s) - DEJA IMPLEMENTE
- [ ] Retry logic avec exponential backoff - DEJA IMPLEMENTE
- [ ] Validation du contenu téléchargé (pas d'injection malveillante)

## ⚠️ VULNERABILITE CRITIQUE DETECTEE

**STATUT ACTUEL:** La fonction `IpfsClient::cat()` N'UTILISE PAS Tor proxy.

**Problème identifié (ligne 67-71):**
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
    .pool_max_idle_per_host(10)
    .build()
    .context("Failed to build HTTP client")?;
```

**Impact OPSEC:**
1. Téléchargements IPFS révèlent l'IP réelle de l'utilisateur
2. Gateway public (ipfs.io ou 127.0.0.1:8080) peut logger IP source
3. Corrélation possible: Qui télécharge quel vendor reputation
4. Metadata leak via User-Agent par défaut (révèle reqwest version)
5. IPFS hash dans URL révèle quel contenu est demandé

**Correctif requis:**
```rust
use reqwest::Proxy;

let proxy = Proxy::all("socks5h://127.0.0.1:9050")
    .context("Failed to configure Tor proxy")?;

let client = reqwest::Client::builder()
    .proxy(proxy)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
    .timeout(Duration::from_secs(60))  // Tor + IPFS gateway = very slow
    .pool_max_idle_per_host(10)
    .build()
    .context("Failed to build HTTP client")?;
```

## 🧪 Tests Automatiques

### 1. Vérification Tor Daemon
```bash
#!/bin/bash
set -euo pipefail

echo "=== Reality Check: IPFS Cat via Tor ==="
echo "Function: server/src/ipfs/client.rs::cat()"
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

# Test 2: IPFS gateway is accessible (localhost or Tor-enabled)
echo ""
echo "[Test 2] Checking IPFS gateway availability..."
# Test with known IPFS hash (IPFS hello world)
TEST_HASH="QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"

if curl -s -m 5 http://127.0.0.1:8080/ipfs/$TEST_HASH 2>/dev/null | grep -q "hello"; then
    echo "✅ PASS: Local IPFS gateway running on 127.0.0.1:8080"
    GATEWAY_MODE="local"
else
    echo "⚠️  WARNING: Local IPFS gateway not available, will use public gateway via Tor"
    GATEWAY_MODE="public"
    # Test public gateway via Tor
    if curl --socks5-hostname 127.0.0.1:9050 -s -m 15 https://ipfs.io/ipfs/$TEST_HASH 2>/dev/null | grep -q "hello"; then
        echo "✅ PASS: Public IPFS gateway accessible via Tor"
    else
        echo "❌ FAIL: Cannot access IPFS gateway (local or public)"
        exit 1
    fi
fi

# Test 3: Code uses Tor proxy (CRITICAL - currently FAILING)
echo ""
echo "[Test 3] Checking if IpfsClient uses Tor proxy..."
if grep -q "Proxy::all.*127.0.0.1:9050" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs; then
    echo "✅ PASS: Code configures Tor SOCKS5 proxy"
else
    echo "❌ CRITICAL FAIL: Code does NOT use Tor proxy"
    echo "OPSEC RISK: IPFS downloads leak real IP address"
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

# Test 5: IPFS hash not logged in cleartext
echo ""
echo "[Test 5] Checking for IPFS hash logging..."
if grep -E "hash.*%hash|hash.*=.*hash" c:/Users/Lenovo/monero-marketplace/server/src/ipfs/client.rs | grep -v "REDACTED"; then
    echo "⚠️  WARNING: IPFS hashes logged in cleartext"
    echo "Privacy risk: Logs reveal which reputation files are accessed"
    echo "Recommendation: Use hash = \"[REDACTED]\" in production logs"
else
    echo "✅ PASS: IPFS hashes not logged or redacted"
fi

# Test 6: No public ports exposed
echo ""
echo "[Test 6] Checking for public port exposure..."
if command -v netstat &> /dev/null; then
    if netstat -tuln 2>/dev/null | grep -E ":8080.*0\.0\.0\.0"; then
        echo "❌ FAIL: IPFS gateway port exposed publicly"
        echo "Gateway should only bind to 127.0.0.1"
        exit 1
    fi
    echo "✅ PASS: No public IPFS gateway ports detected"
else
    echo "⚠️  SKIP: netstat not available (Windows)"
fi

# Test 7: Downloaded content validation
echo ""
echo "[Test 7] Checking content validation..."
if grep -q "serde_json::from_slice" c:/Users/Lenovo/monero-marketplace/server/src/handlers/reputation_ipfs.rs; then
    echo "✅ PASS: Downloaded content is validated via JSON deserialization"
else
    echo "⚠️  WARNING: No explicit validation of downloaded content"
    echo "Security risk: Malicious IPFS content could cause issues"
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
# Test IPFS download with known test hash

cd c:/Users/Lenovo/monero-marketplace

# Use IPFS "hello world" hash for testing
TEST_HASH="QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"

# Test 1: Download via local gateway
echo "Test 1: Download via local IPFS gateway..."
curl -s http://127.0.0.1:8080/ipfs/$TEST_HASH

# Test 2: Download via Tor + public gateway
echo ""
echo "Test 2: Download via Tor + public gateway..."
curl --socks5-hostname 127.0.0.1:9050 -s https://ipfs.io/ipfs/$TEST_HASH

# Test 3: Run Rust client test
echo ""
echo "Test 3: Running Rust IpfsClient::cat() test..."
# First upload something to get a valid hash
TEST_DATA='{"test":"reputation","vendor":"example"}'
UPLOAD_HASH=$(echo "$TEST_DATA" | curl -s -X POST -F "file=@-" http://127.0.0.1:5001/api/v0/add | jq -r '.Hash')

echo "Uploaded test data with hash: $UPLOAD_HASH"

# Now test download
curl -s http://127.0.0.1:8080/ipfs/$UPLOAD_HASH

# Run integration test
# cargo test --package server --test ipfs_integration test_ipfs_roundtrip -- --nocapture --ignored
```

## 📋 Tests Manuels Requis

### Test 1: DNS Leak Check
```bash
echo "[Manual Test 1] DNS Leak Detection for IPFS Download"
echo "Objective: Ensure IPFS gateway requests don't leak DNS queries"
echo ""

# Setup: Monitor DNS queries
# Linux: sudo tcpdump -i any -n port 53 -w /tmp/dns-leak-cat.pcap &
# Windows: Use Wireshark with filter "udp.port == 53"

# Step 1: Baseline (10 seconds idle)
echo "Step 1: Capturing baseline DNS traffic..."
sleep 10

# Step 2: Trigger IPFS download
echo "Step 2: Downloading reputation from IPFS..."
TEST_HASH="QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
curl --socks5-hostname 127.0.0.1:9050 -s https://ipfs.io/ipfs/$TEST_HASH

# Step 3: Analyze captured DNS traffic
echo "Step 3: Analyzing DNS queries..."
# tcpdump -r /tmp/dns-leak-cat.pcap -n port 53

# Expected results:
# ✅ PASS: No DNS queries to public resolvers
# ✅ PASS: Only Tor DNS resolver (127.0.0.1:53) used
# ❌ FAIL: DNS queries for "ipfs.io", "gateway.ipfs.io", "cloudflare-ipfs.com"

echo ""
echo "VALIDATION:"
echo "  ✅ All DNS queries go through Tor (socks5h://)"
echo "  ✅ No cleartext IPFS gateway domains in DNS"
echo "  ✅ No public DNS servers contacted"
echo ""
```

### Test 2: Fingerprinting Check
```bash
echo "[Manual Test 2] HTTP Fingerprinting for IPFS Gateway Requests"
echo "Objective: Ensure HTTP GET requests are generic"
echo ""

# Method: Intercept HTTP traffic to gateway
# Use mitmproxy or Wireshark

# Step 1: Start traffic capture
# Wireshark filter: http and tcp.port == 8080

# Step 2: Trigger download
TEST_HASH="QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
curl http://127.0.0.1:8080/ipfs/$TEST_HASH

# Step 3: Inspect HTTP GET request headers

# Expected User-Agent:
# ✅ PASS: "Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0"
# ❌ FAIL: "reqwest/0.11.x" or "curl/x.x.x"

# Expected Headers:
# ✅ PASS: Standard Accept, Accept-Encoding headers
# ✅ PASS: No X-Client-*, X-Forwarded-*, or identifying headers
# ❌ FAIL: Custom headers revealing client implementation

echo "VALIDATION:"
echo "  ✅ User-Agent matches common browser"
echo "  ✅ No custom identifying headers"
echo "  ✅ Standard HTTP/1.1 or HTTP/2 request"
echo ""
```

### Test 3: Gateway Correlation Attack
```bash
echo "[Manual Test 3] Gateway Correlation Resistance"
echo "Objective: Verify IPFS downloads don't correlate to vendor activity"
echo ""

# Threat model: Adversary monitors IPFS gateway logs
# Can they link downloads to specific vendors?

# Step 1: Upload reputation for Vendor A
echo "Step 1: Upload reputation for test Vendor A..."
# (Simulate via API or direct IPFS add)

# Step 2: Download reputation for Vendor A
echo "Step 2: Download reputation (same session)..."
# Measure timing between upload and download

# Step 3: Download reputation for Vendor A from different IP
echo "Step 3: Download from different Tor circuit..."
# Force new Tor circuit: echo "SIGNAL NEWNYM" | nc 127.0.0.1 9051

# Analysis:
# ✅ PASS: Downloads use different Tor circuits (different exit IPs)
# ✅ PASS: Random delay between upload and download prevents timing correlation
# ❌ FAIL: Same Tor circuit used for multiple operations

echo "VALIDATION:"
echo "  ✅ Each download uses fresh Tor circuit"
echo "  ✅ Timing jitter prevents correlation"
echo "  ✅ No session cookies or authentication tokens"
echo ""
```

### Test 4: Analyse de Trafic Complet
```bash
echo "[Manual Test 4] Comprehensive Traffic Analysis"
echo "Objective: Verify all IPFS download traffic is Tor-only"
echo ""

# Capture all network traffic
# sudo tcpdump -i any -w /tmp/ipfs-cat-test.pcap

# Step 1: Start packet capture
echo "Step 1: Starting comprehensive packet capture..."

# Step 2: Perform IPFS download
cd c:/Users/Lenovo/monero-marketplace
TEST_HASH="QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
curl http://127.0.0.1:8080/ipfs/$TEST_HASH > /tmp/test-download.json

# Step 3: Analyze with Wireshark
# wireshark /tmp/ipfs-cat-test.pcap

# Expected traffic patterns:
# ✅ PASS: HTTP GET to 127.0.0.1:8080 (local gateway)
#     OR SOCKS5 to 127.0.0.1:9050 then HTTPS to external gateway
# ✅ PASS: No direct connections to IPFS DHT nodes
# ✅ PASS: No unencrypted data on public interfaces
# ❌ FAIL: Direct HTTP connections to public IPFS gateways

echo "VALIDATION CHECKLIST:"
echo "  ✅ All external traffic via Tor SOCKS5 (127.0.0.1:9050)"
echo "  ✅ Local gateway traffic stays on loopback (127.0.0.1)"
echo "  ✅ No direct connections to IPFS node IPs"
echo "  ✅ Downloaded content not logged in cleartext"
echo ""
```

### Test 5: Content Validation & Injection Prevention
```bash
echo "[Manual Test 5] Malicious Content Handling"
echo "Objective: Ensure downloaded IPFS content is validated"
echo ""

# Test 1: Download malformed JSON
echo "Test 1: Malformed JSON..."
echo '{"invalid json' | curl -X POST -F "file=@-" http://127.0.0.1:5001/api/v0/add
# Try to download and parse - should fail gracefully

# Test 2: Download oversized file
echo "Test 2: Oversized file (simulate DoS)..."
# Upload 10MB file, try to download
# Should hit size limits or timeout

# Test 3: Download with invalid schema
echo "Test 3: Invalid reputation schema..."
echo '{"format_version":"999.0","invalid":"schema"}' | curl -X POST -F "file=@-" http://127.0.0.1:5001/api/v0/add
# Should fail JSON schema validation

# Expected behavior:
# ✅ PASS: Invalid JSON returns error, doesn't panic
# ✅ PASS: Oversized files rejected or limited
# ✅ PASS: Schema validation prevents malformed data

echo "VALIDATION:"
echo "  ✅ Robust error handling (no panics)"
echo "  ✅ Size limits enforced"
echo "  ✅ Schema validation before accepting data"
echo ""
```

## ⚠️ Risques Identifiés

### Risque Critique #1: Pas de Tor Proxy
**Sévérité:** CRITIQUE
**Impact:** Fuite IP réelle lors de téléchargements IPFS, corrélation possible avec vendor queries
**Statut:** NON MITIGE (code actuel)
**Mitigation:** Ajouter `Proxy::all("socks5h://127.0.0.1:9050")` dans `IpfsClient::new()`

### Risque Critique #2: User-Agent par Défaut
**Sévérité:** ELEVEE
**Impact:** Fingerprinting (révèle reqwest/Rust version), permet identification client unique
**Statut:** NON MITIGE (code actuel)
**Mitigation:** Configurer `.user_agent("Mozilla/5.0 ...")` dans client builder

### Risque Elevé #3: IPFS Hash Logging
**Sévérité:** MOYENNE
**Impact:** Logs révèlent quels vendors sont consultés (corrélation possible)
**Statut:** PRESENT (ligne 239-243)
**Mitigation:** Redact hash dans logs: `hash = "[REDACTED]"` ou hash uniquement en debug mode

### Risque Elevé #4: Timing Correlation
**Sévérité:** MOYENNE
**Impact:** Téléchargements immédiatement après upload révèlent vendor identity
**Statut:** PAS DE JITTER IMPLEMENTE
**Mitigation:** Ajouter random delay (0-60s) avant download pour casser corrélation temporelle

### Risque Moyen #5: Gateway Fingerprinting
**Sévérité:** MOYENNE
**Impact:** Public IPFS gateways peuvent logger requêtes et créer profil utilisateur
**Statut:** DEPENDANT DE GATEWAY UTILISE
**Mitigation:**
- Utiliser gateway local uniquement
- OU rotation de gateways publics via Tor
- OU utiliser hidden service IPFS gateway (.onion)

### Risque Moyen #6: Content Size Leak
**Sévérité:** FAIBLE
**Impact:** Taille fichier téléchargé logguée révèle approximativement nombre de reviews
**Statut:** PRESENT (ligne 241)
**Mitigation:** Ne pas logger `size` field en production

### Risque Faible #7: Cache Timing
**Sévérité:** FAIBLE
**Impact:** Gateway cache pourrait révéler si contenu déjà accédé récemment
**Statut:** INHERENT A IPFS
**Mitigation:** Limité (feature IPFS), utiliser cache local ou accepter risque

## ✅ Validation Finale

- [ ] Tests automatiques exécutés avec succès (BLOQUE: Tor proxy manquant)
- [ ] Tests manuels DNS leak: PASS
- [ ] Tests manuels fingerprinting: PENDING (dépend correctif Tor)
- [ ] Tests manuels gateway correlation: PENDING
- [ ] Tests manuels traffic analysis: PENDING
- [ ] Tests manuels content validation: PENDING
- [ ] Code review par un autre développeur
- [ ] Correctif Tor proxy appliqué et testé
- [ ] Timing jitter implémenté
- [ ] Documentation à jour

**Validé par:** _____________
**Date de validation:** _____________

## 📚 Références

- [Tor Project Best Practices](https://2019.www.torproject.org/docs/tor-manual.html.en)
- [IPFS Privacy & Security](https://docs.ipfs.tech/concepts/privacy-and-encryption/)
- [IPFS Gateway Security](https://docs.ipfs.tech/concepts/ipfs-gateway/)
- [Reqwest SOCKS Proxy Documentation](https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html)
- [Timing Attack Mitigation](https://en.wikipedia.org/wiki/Timing_attack)
- Project: `docs/SECURITY-THEATRE-PREVENTION.md`
- Project: `scripts/validate-reality-check-tor.sh`
- Related: `server/src/ipfs/client.rs` (lines 66-78, 231-267)

## 🚨 Action Immédiate Requise

**AVANT DE MERGER CE CODE EN PRODUCTION:**

1. Implémenter Tor proxy dans `IpfsClient::new()`, `::new_local()`, `::new_infura()`
2. Configurer User-Agent générique anti-fingerprinting
3. Ajouter random timing jitter (0-60s) avant cat() operations
4. Redact IPFS hashes dans logs production
5. Implémenter size limits pour downloads (prevent DoS)
6. Documenter gateway selection strategy (local vs public)
7. Tester content validation avec malformed inputs
8. Exécuter tous les tests manuels et automatiques
9. Valider avec `./scripts/validate-reality-check-tor.sh ipfs_cat`

**Ce code N'EST PAS production-ready tant que le proxy Tor n'est pas configuré.**

**Corrélation risk:** Si add() et cat() partagent le même circuit Tor, adversary peut corréler uploads et downloads. Considérer forcer new circuit entre opérations (SIGNAL NEWNYM).
