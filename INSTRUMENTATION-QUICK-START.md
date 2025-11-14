# Multisig Instrumentation - Quick Start

**Status:** ✅ Implémenté et Intégré
**Date:** 2025-11-13

---

## 🎯 But

Tracer et débugger les **race conditions**, **RPC cache pollution**, et **corruptions d'état** dans les opérations multisig concurrentes.

---

## 🚀 Utilisation en 3 Étapes

### 1. Activer l'Instrumentation

```bash
export ENABLE_INSTRUMENTATION=1
cargo run --bin server
```

### 2. Reproduire le Bug

```bash
# Test avec 1 escrow (baseline)
curl -X POST http://localhost:8080/api/escrow/init \
  -H "Content-Type: application/json" \
  -d '{"buyer_id": "buyer1", "vendor_id": "vendor1", "amount": 1000000}'

# Test avec 3 escrows concurrents (chercher race conditions)
for i in {1..3}; do
  curl -X POST http://localhost:8080/api/escrow/init \
    -H "Content-Type: application/json" \
    -d "{\"buyer_id\": \"buyer$i\", \"vendor_id\": \"vendor$i\", \"amount\": 1000000}" &
done
wait
```

### 3. Analyser les Résultats

```bash
# Liste des fichiers générés
ls -lh escrow_*.json

# Analyse basique
python3 tools/analyze_escrow_json.py escrow_abc123.json

# Comparer succès vs échec
python3 tools/analyze_escrow_json.py --compare escrow_success.json escrow_failed.json

# Timeline détaillée
python3 tools/analyze_escrow_json.py --timeline escrow_failed.json

# Statistiques RPC
python3 tools/analyze_escrow_json.py --rpc-only escrow_abc123.json

# Analyse des snapshots
python3 tools/analyze_escrow_json.py --snapshots-only escrow_abc123.json
```

---

## 📊 Ce Que Vous Obtenez

### Exemple de Sortie - Escrow en Échec

```
================================================================================
ESCROW ANALYSIS: escrow_failed.json
Trace ID: escrow_abc123-1699999999999
Total events: 18
================================================================================

EVENT TIMELINE
[+    0ms] SNAPSHOT_PRE_ROUND1              role=buyer    multisig=true ❌
[+   50ms] RPC_CALL_START                   role=buyer    method=make_multisig
[+  100ms] RPC_CALL_ERROR                   role=buyer    ❌
[+  120ms] ERROR_FINAL                      role=buyer

ERRORS & ANOMALIES
Error: Wallet already in multisig mode
Context: {
  "round": 1,
  "operation": "make_multisig",
  "wallet_id": "abc-123",
  "escrow_id": "xyz-789"
}

✅ ROOT CAUSE: RPC cache pollution - wallet déjà en mode multisig
```

---

## 🔍 Points de Traçage

L'instrumentation capture l'état à **7 points critiques**:

1. **SNAPSHOT_PRE_ROUND1** - Avant `prepare_multisig`
2. **SNAPSHOT_POST_MAKE_MULTISIG** - Après `make_multisig` (×3 wallets)
3. **SNAPSHOT_PRE_ROUND2** - Avant premier `exchange_multisig_keys`
4. **SNAPSHOT_POST_EXPORT_MULTISIG** - Après export
5. **SNAPSHOT_PRE_ROUND3** - Avant second `exchange_multisig_keys`
6. **SNAPSHOT_POST_IMPORT_MULTISIG** - Après import
7. **SNAPSHOT_FINAL** - État final

**Pour chaque opération RPC:**
- Timestamp de début
- Durée (ms)
- Port RPC utilisé
- Succès/échec
- Erreurs complètes

---

## 📈 Patterns d'Erreurs Courants

### Pattern A: RPC Cache Pollution

**Symptôme:**
```
[+0ms] SNAPSHOT_PRE_ROUND1 role=buyer multisig=true ❌
```

**Diagnostic:** Wallet déjà en mode multisig avant `make_multisig()`

**Fix:**
- Augmenter délai entre opérations (10s → 15s)
- Ajouter purge explicite du cache RPC
- Vérifier état wallet AVANT chaque opération

### Pattern B: Race Condition

**Symptôme:**
```
COMPARING: escrow_1.json vs escrow_3.json
Divergence at event #15:
  File 1: [RPC_CALL_END] role=buyer
  File 3: [ERROR_FINAL] role=buyer
```

**Diagnostic:** 3e escrow échoue toujours au même point

**Fix:**
- Utiliser `WALLET_CREATION_LOCK` global mutex
- Implémenter wallet pool avec instances RPC exclusives
- Ajouter locking au niveau fichier

### Pattern C: State Divergence

**Symptôme:**
```
buyer.address_hash:   abc123...
vendor.address_hash:  abc123...
arbiter.address_hash: def456... ❌
```

**Diagnostic:** Arbiter a une adresse différente

**Fix:**
- Trier `prepare_infos` alphabétiquement avant `make_multisig()`
- Valider longueur et contenu des `prepare_infos`
- Logger SHA256 des inputs pour vérification

---

## 🛠️ Commandes Utiles

### Test de Compilation

```bash
# Vérifier que tout compile
cargo check --package server

# Test automatisé de l'instrumentation
bash tools/test-instrumentation.sh
```

### Nettoyage

```bash
# Supprimer fichiers instrumentation anciens
find . -name "escrow_*.json" -mtime +7 -delete

# Archiver avant suppression
tar -czf instrumentation_$(date +%Y%m%d).tar.gz escrow_*.json
rm escrow_*.json
```

---

## 📚 Documentation Complète

- **Guide Utilisateur:** [DOX/guides/INSTRUMENTATION-GUIDE.md](DOX/guides/INSTRUMENTATION-GUIDE.md)
- **Exemples d'Intégration:** [DOX/guides/INSTRUMENTATION-INTEGRATION-EXAMPLE.md](DOX/guides/INSTRUMENTATION-INTEGRATION-EXAMPLE.md)
- **Description du Skill:** [DOX/skills/MULTISIG-INSTRUMENTATION-SKILL.md](DOX/skills/MULTISIG-INSTRUMENTATION-SKILL.md)

---

## ⚠️ Performance

| Mode | CPU | Mémoire | Disque | Recommandation |
|------|-----|---------|--------|----------------|
| Désactivé (défaut) | 0% | 0 KB | 0 MB | Production |
| Activé | <1% | 10-50 KB/escrow | 1-5 MB/escrow | Dev/Debug uniquement |

**Important:** L'instrumentation a un overhead négligeable quand désactivée (par défaut), mais utilise de l'espace disque quand activée. À n'utiliser que pour le debugging.

---

## ✅ Status d'Implémentation

- [x] Modules Rust (events, snapshots, collector)
- [x] Python analysis tool
- [x] Intégration dans `wallet_manager.rs`
- [x] Documentation complète
- [x] Tests de compilation
- [x] Scripts de validation

**Prêt à l'emploi!** 🎯

---

**Pour Démarrer:** `export ENABLE_INSTRUMENTATION=1 && cargo run --bin server`

