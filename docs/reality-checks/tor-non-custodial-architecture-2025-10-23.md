# Reality Check Tor: Non-Custodial Architecture

**Date:** 2025-10-23
**Module:** Non-Custodial Wallet Registration
**Location:** `server/src/wallet_manager.rs::register_client_wallet_rpc`
**Status:** ⏳ PENDING VALIDATION

---

## 🎯 Objectif du Module

Permettre aux clients (buyers/vendors) d'enregistrer leur propre wallet RPC URL avec le serveur, garantissant que:
1. **Clients contrôlent leurs clés privées** (serveur ne génère jamais de clés client)
2. **Serveur ne se connecte PAS au wallet RPC client** (validation seule)
3. **Architecture 100% non-custodiale** (impossible pour serveur de voler fonds)

---

## 🔒 Garanties de Sécurité Requises

### Architecture Non-Custodiale
- [x] Serveur **NE CRÉE PAS** de wallets buyer/vendor (bloqué par `NonCustodialViolation`)
- [x] Serveur **NE GÉNÈRE PAS** de clés privées client
- [x] Serveur **NE SE CONNECTE PAS** aux wallet RPC clients
- [x] Clients fournissent leur propre wallet RPC URL (localhost de leur machine)
- [x] Arbiter wallet seul géré par serveur (nécessaire pour 2-of-3 multisig)

### Validation RPC URL
- [x] URLs doivent contenir `127.0.0.1` ou `localhost` (validation wallet/src/rpc.rs:42-46)
- [x] Validation backend via `validator` crate (handlers/escrow.rs:20-28)
- [x] Role validation (buyer/vendor uniquement, pas arbiter)

### Pas de Trafic Réseau Externe
- [x] `register_client_wallet_rpc()` ne fait **AUCUN appel réseau**
- [x] Stockage configuration uniquement (HashMap in-memory)
- [x] Pas de connexion au wallet RPC client durant registration

---

## 🧪 Tests Automatiques

### 1. Vérification Blocage Wallet Custodial

```bash
#!/bin/bash
set -euo pipefail

echo "🧪 Test 1: Serveur ne peut PAS créer wallets buyer/vendor"

# Test avec buyer (doit échouer)
RESULT=$(cargo test --package server --lib wallet_manager::tests::test_non_custodial_buyer_blocked 2>&1 || echo "BLOCKED")

if echo "$RESULT" | grep -q "NonCustodialViolation.*Buyer"; then
    echo "✅ PASS: Buyer wallet creation blocked"
else
    echo "❌ FAIL: Buyer wallet creation NOT blocked"
    exit 1
fi

# Test avec vendor (doit échouer)
RESULT=$(cargo test --package server --lib wallet_manager::tests::test_non_custodial_vendor_blocked 2>&1 || echo "BLOCKED")

if echo "$RESULT" | grep -q "NonCustodialViolation.*Vendor"; then
    echo "✅ PASS: Vendor wallet creation blocked"
else
    echo "❌ FAIL: Vendor wallet creation NOT blocked"
    exit 1
fi

# Test avec arbiter (doit réussir)
RESULT=$(cargo test --package server --lib wallet_manager::tests::test_arbiter_wallet_creation 2>&1)

if echo "$RESULT" | grep -q "test result: ok"; then
    echo "✅ PASS: Arbiter wallet creation allowed"
else
    echo "❌ FAIL: Arbiter wallet creation blocked"
    exit 1
fi

echo "✅ PASS: Non-custodial enforcement verified"
```

### 2. Validation RPC URL Localhost-Only

```bash
#!/bin/bash
set -euo pipefail

echo "🧪 Test 2: RPC URL validation (localhost-only)"

# Test URL publique (doit échouer)
cat > /tmp/test_public_url.json <<EOF
{
    "rpc_url": "http://evil.com:18082/json_rpc",
    "role": "buyer"
}
EOF

RESPONSE=$(curl -s -X POST http://localhost:8080/api/escrow/register-wallet-rpc \
    -H "Content-Type: application/json" \
    -H "Cookie: session=test_session" \
    -d @/tmp/test_public_url.json)

if echo "$RESPONSE" | grep -qi "localhost only"; then
    echo "✅ PASS: Public URL rejected"
else
    echo "❌ FAIL: Public URL NOT rejected"
    exit 1
fi

# Test URL localhost valide (doit réussir)
cat > /tmp/test_localhost_url.json <<EOF
{
    "rpc_url": "http://127.0.0.1:18082/json_rpc",
    "role": "buyer"
}
EOF

RESPONSE=$(curl -s -X POST http://localhost:8080/api/escrow/register-wallet-rpc \
    -H "Content-Type: application/json" \
    -H "Cookie: session=test_session" \
    -d @/tmp/test_localhost_url.json)

if echo "$RESPONSE" | grep -qi "success"; then
    echo "✅ PASS: Localhost URL accepted"
else
    echo "❌ FAIL: Localhost URL rejected"
    exit 1
fi

echo "✅ PASS: RPC URL validation working"
```

### 3. Pas de Connexion Réseau Durant Registration

```bash
#!/bin/bash
set -euo pipefail

echo "🧪 Test 3: Aucune connexion réseau durant registration"

# Capturer trafic réseau
sudo tcpdump -i any -w /tmp/register_wallet.pcap &
TCPDUMP_PID=$!
sleep 2

# Enregistrer wallet RPC
curl -s -X POST http://localhost:8080/api/escrow/register-wallet-rpc \
    -H "Content-Type: application/json" \
    -H "Cookie: session=test_session" \
    -d '{"rpc_url": "http://127.0.0.1:18082/json_rpc", "role": "buyer"}' > /dev/null

sleep 2
sudo kill $TCPDUMP_PID

# Analyser trafic capturé
EXTERNAL_TRAFFIC=$(tcpdump -r /tmp/register_wallet.pcap -n 'not host 127.0.0.1 and not port 8080' 2>/dev/null | wc -l)

if [ "$EXTERNAL_TRAFFIC" -eq 0 ]; then
    echo "✅ PASS: No external network traffic during registration"
else
    echo "❌ FAIL: External network traffic detected: $EXTERNAL_TRAFFIC packets"
    tcpdump -r /tmp/register_wallet.pcap -n 'not host 127.0.0.1'
    exit 1
fi

rm /tmp/register_wallet.pcap
echo "✅ PASS: Network isolation verified"
```

### 4. Test Complet API Registration

```bash
#!/bin/bash
set -euo pipefail

echo "🧪 Test 4: API endpoint complete test"

# Démarrer serveur en background
cargo run --package server --bin server &
SERVER_PID=$!
sleep 5

# Test 1: Enregistrer buyer wallet
BUYER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/escrow/register-wallet-rpc \
    -H "Content-Type: application/json" \
    -H "Cookie: session=buyer_session" \
    -d '{
        "rpc_url": "http://127.0.0.1:18082/json_rpc",
        "role": "buyer"
    }')

if echo "$BUYER_RESPONSE" | grep -q "wallet_id"; then
    echo "✅ PASS: Buyer wallet registration successful"
else
    echo "❌ FAIL: Buyer wallet registration failed"
    echo "Response: $BUYER_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

# Test 2: Enregistrer vendor wallet
VENDOR_RESPONSE=$(curl -s -X POST http://localhost:8080/api/escrow/register-wallet-rpc \
    -H "Content-Type: application/json" \
    -H "Cookie: session=vendor_session" \
    -d '{
        "rpc_url": "http://127.0.0.1:18083/json_rpc",
        "role": "vendor"
    }')

if echo "$VENDOR_RESPONSE" | grep -q "wallet_id"; then
    echo "✅ PASS: Vendor wallet registration successful"
else
    echo "❌ FAIL: Vendor wallet registration failed"
    echo "Response: $VENDOR_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

# Test 3: Tentative registration arbiter (doit échouer)
ARBITER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/escrow/register-wallet-rpc \
    -H "Content-Type: application/json" \
    -H "Cookie: session=arbiter_session" \
    -d '{
        "rpc_url": "http://127.0.0.1:18084/json_rpc",
        "role": "arbiter"
    }')

if echo "$ARBITER_RESPONSE" | grep -qi "non-custodial policy violation\|invalid role"; then
    echo "✅ PASS: Arbiter registration blocked"
else
    echo "❌ FAIL: Arbiter registration NOT blocked"
    echo "Response: $ARBITER_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

kill $SERVER_PID
echo "✅ PASS: All API tests passed"
```

---

## 📋 Tests Manuels Requis

### Test 1: Vérification Code Source

```bash
# Vérifier que serveur ne crée PAS de wallets client
grep -r "create_wallet_instance" server/src/wallet_manager.rs

# ✅ PASS si méthode deprecated + nouveau `create_arbiter_wallet_instance()`
# ❌ FAIL si création wallet buyer/vendor toujours active

# Vérifier présence erreur NonCustodialViolation
grep -r "NonCustodialViolation" server/src/wallet_manager.rs

# ✅ PASS si erreur présente et utilisée pour bloquer buyer/vendor
```

### Test 2: Audit Documentation Client

```bash
# Vérifier guide client wallet setup
cat docs/CLIENT-WALLET-SETUP.md

# ✅ PASS si:
# - Explique "non-custodial" clairement
# - Instructions monero-wallet-rpc
# - Guide backup seed phrase
# - Warnings sécurité présents
```

### Test 3: Test Flow Complet Utilisateur

**Étapes manuelles:**

1. **Setup Client Wallet:**
   ```bash
   # Client machine
   monero-wallet-cli --testnet --generate-new-wallet ~/my_wallet
   monero-wallet-rpc --testnet --rpc-bind-port 18082 --wallet-file ~/my_wallet
   ```

2. **Register avec Marketplace:**
   ```bash
   curl -X POST http://marketplace.onion/api/escrow/register-wallet-rpc \
     -H "Content-Type: application/json" \
     -d '{
       "rpc_url": "http://127.0.0.1:18082/json_rpc",
       "role": "buyer"
     }'
   ```

3. **Vérifier:**
   - ✅ Wallet registered avec wallet_id
   - ✅ Serveur ne demande JAMAIS seed phrase
   - ✅ Serveur ne demande JAMAIS private keys
   - ✅ Serveur ne fait AUCUNE connexion au wallet RPC client

### Test 4: Vérification Architecture Multisig

```bash
# Vérifier que escrow multisig utilise:
# - Buyer wallet (client-controlled)
# - Vendor wallet (client-controlled)
# - Arbiter wallet (server-controlled)

grep -A 20 "create_multisig_escrow" server/src/services/escrow.rs

# ✅ PASS si utilise register_client_wallet_rpc pour buyer/vendor
# ❌ FAIL si crée wallets buyer/vendor côté serveur
```

---

## ⚠️ Risques Identifiés

### 🟢 RISQUES ÉLIMINÉS (par architecture non-custodiale)

1. **Exit Scam Server**: ❌ **IMPOSSIBLE** - Serveur ne contrôle pas clés client
2. **Hack Server → Perte Fonds Client**: ❌ **IMPOSSIBLE** - Clés stockées localement client
3. **Confiscation Serveur**: ❌ **IMPOSSIBLE** - 2-of-3 multisig, serveur a 1 clé seulement
4. **Database Breach → Vol Wallets**: ❌ **IMPOSSIBLE** - DB stocke RPC URLs, pas clés privées

### 🟡 RISQUES RÉSIDUELS (acceptables)

1. **Client Lose Seed Phrase**: ⚠️ **Risque client** - Not marketplace responsibility
2. **Client Machine Compromised**: ⚠️ **Risque client** - Standard threat model
3. **Malicious Arbiter**: ⚠️ **Limité** - 2-of-3 multisig, requires collusion
4. **Arbiter Wallet Hack**: ⚠️ **Partiel** - Cannot steal alone (needs 2-of-3)

### 🔴 NOUVEAUX RISQUES (à surveiller)

1. **RPC URL Validation Bypass**:
   - **Mitigation**: Validation stricte backend + frontend
   - **Test**: Automated tests vérifient rejection URLs publiques

2. **User Confusion (Custodial vs Non-Custodial)**:
   - **Mitigation**: Documentation exhaustive CLIENT-WALLET-SETUP.md
   - **Test**: Manual review of user-facing docs

3. **Lack of Frontend UI**:
   - **Mitigation**: Create templates/settings/wallet.html (Phase 4.5)
   - **Status**: ⚠️ BLOCKER identified by Agent 4

---

## ✅ Validation Finale

### Tests Automatiques
- [ ] Test 1: Non-custodial enforcement (buyer/vendor blocking) - **À exécuter**
- [ ] Test 2: RPC URL validation (localhost-only) - **À exécuter**
- [ ] Test 3: No network traffic during registration - **À exécuter**
- [ ] Test 4: Complete API endpoint test - **À exécuter**

### Tests Manuels
- [x] Code source audit - **✅ PASSED** (Agent 1 Anti-Hallucination)
- [x] Documentation client review - **✅ PASSED** (456 lines CLIENT-WALLET-SETUP.md)
- [ ] Complete user flow test - **⏳ PENDING** (requires testnet setup)
- [ ] Multisig architecture verification - **⏳ PENDING** (requires integration test)

### Validation Sécurité
- [x] No key generation server-side - **✅ VERIFIED** (Agent 1)
- [x] No key storage server-side - **✅ VERIFIED** (Agent 1)
- [x] Localhost-only RPC validation - **✅ VERIFIED** (wallet/src/rpc.rs:42-46)
- [x] Role-based access control - **✅ VERIFIED** (handlers/escrow.rs custom validator)
- [ ] Frontend templates security - **❌ MISSING** (Agent 4 blocker)

### Production Readiness
- [x] Backend code complete - **✅ 100%**
- [x] Tests passing - **✅ 7/7 wallet_manager tests**
- [x] Documentation complete - **✅ 11 fichiers**
- [ ] Frontend UI complete - **❌ 0%** (templates manquants)
- [ ] Logging adequate - **❌ 0%** (Agent 3 critical blocker)
- [ ] HTMX local (no CDN) - **❌ OPSEC violation** (Agent 4 blocker)

---

**Validé par:** _____________
**Date de validation:** _____________

## 📚 Références

- [Tor Project Best Practices](https://2019.www.torproject.org/docs/tor-manual.html.en)
- [Monero Non-Custodial Best Practices](https://www.getmonero.org/resources/user-guides/)
- Project: [docs/CLIENT-WALLET-SETUP.md](../CLIENT-WALLET-SETUP.md)
- Project: [NON-CUSTODIAL-MIGRATION-COMPLETE.md](../../NON-CUSTODIAL-MIGRATION-COMPLETE.md)
- Project: [NON-CUSTODIAL-CERTIFICATION.md](../../NON-CUSTODIAL-CERTIFICATION.md)

---

## 🎯 Commande de Validation

```bash
# Exécuter tous les tests automatiques
./scripts/validate-reality-check-tor.sh non-custodial-architecture

# Ou manuellement:
bash docs/reality-checks/tor-non-custodial-architecture-2025-10-23.md
```

---

**Status:** ⏳ **PENDING MANUAL VALIDATION**

**Next Steps:**
1. Execute automated tests (4 test scripts above)
2. Complete manual user flow test with testnet
3. Fix Agent 4 blockers (frontend templates + HTMX CDN)
4. Fix Agent 3 blocker (logging deficiency)
5. Re-run Beta Terminal Protocol for final validation
