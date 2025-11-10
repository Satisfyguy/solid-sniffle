# Diagnostic: Multisig Cassé - Analyse Complète

**Date**: 9 novembre 2025
**Auteur**: Analyse automatique du code
**Statut**: 🚨 CRITIQUE - Fonctionnalité multisig non fonctionnelle

---

## 📋 Résumé Exécutif

Le multisig 2-of-3 qui **FONCTIONNAIT** au commit `8e3f282` ("working multisig") a été **CASSÉ** par le commit `fe9e887` (feat: BlockchainMonitor) qui a **supprimé accidentellement** des sections critiques du code.

---

## 🔍 Code Supprimé (Critique)

### 1. Activation de `enable-multisig-experimental`

**Fichier**: `server/src/wallet_manager.rs`
**Lignes supprimées**: 673-705
**Impact**: CRITIQUE - Les wallets multisig ne peuvent plus être créés

**Code supprimé**:
```rust
// CRITICAL: Enable multisig experimental BEFORE any multisig operations
// This must be done immediately after wallet creation/opening
match rpc_client.rpc().set_attribute("enable-multisig-experimental", "1").await {
    Ok(_) => {
        info!("✅ Multisig experimental enabled for {}", wallet_filename);

        // CRITICAL: Close and reopen wallet for attribute to take effect
        // Monero wallet RPC requires this for the setting to be persisted
        match rpc_client.close_wallet().await {
            Ok(_) => {
                info!("🔒 Wallet closed to persist multisig experimental setting");
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                match rpc_client.open_wallet(&wallet_filename, "").await {
                    Ok(_) => {
                        info!("✅ Wallet reopened - multisig experimental setting active");
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to reopen wallet: {:?}", e);
                    }
                }
            }
            Err(e) => {
                warn!("⚠️  Failed to close wallet: {:?}", e);
            }
        }
    }
    Err(e) => {
        warn!("⚠️  Failed to enable multisig experimental: {:?} (will retry on reopen)", e);
    }
}
```

**Pourquoi c'est critique**:
- Monero wallet RPC désactive multisig par défaut
- Sans cet attribut, toutes les opérations multisig échouent avec "multisig is disabled"

---

### 2. Méthode `sync_multisig_wallets()`

**Fichier**: `server/src/wallet_manager.rs`
**Lignes supprimées**: 933-1092 (159 lignes!)
**Impact**: CRITIQUE - Impossible de voir les fonds reçus dans les wallets multisig

**Code supprimé**: La méthode complète de Lazy Sync Pattern (voir commit 8e3f282:933-1092)

**Pourquoi c'est critique**:
- Les wallets multisig Monero ne voient PAS automatiquement les transactions entrantes
- Cette méthode fait `export_multisig_info` + `import_multisig_info` pour synchroniser
- Sans elle, les escrows restent à balance = 0 même après réception de XMR

---

### 3. Flow Multisig 3-Rounds Complet

**Fichier**: `server/src/wallet_manager.rs`
**Lignes modifiées**: 1211-1565 (remplacées par 985-1009 = placeholder incorrect)
**Impact**: CRITIQUE - Setup multisig incomplet/incorrect

**Code actuel (INCORRECT)**:
```rust
// This is a simplified implementation. A real one would be more complex.
for wallet in self.wallets.values_mut() {
    let other_infos = info_from_all
        .iter()
        .filter(|i| i.multisig_info != wallet.address) // This is incorrect, just a placeholder
        .map(|i| i.multisig_info.clone())
        .collect();
    let result = wallet
        .rpc_client
        .multisig()
        .make_multisig(2, other_infos)
        .await?;
    wallet.multisig_state = MultisigState::Ready {
        address: result.address.clone(),
    };
}
```

**Code correct (commit 8e3f282)**:
- Round 1: `make_multisig(2, prepare_infos)` avec validation rigoureuse
- Round 2: `exchange_multisig_keys(round1_multisig_infos)`
- Round 3: `exchange_multisig_keys(round2_multisig_infos)` (FINALIZATION)
- Validation: SHA256 hashes, address matching, wallet open/close cycles
- Délais: 10 secondes entre chaque round pour reset RPC cache

**Pourquoi c'est critique**:
- Le placeholder actuel ne fait qu'UN seul round (make_multisig)
- Monero 2-of-3 requiert 3 rounds pour finaliser
- Sans finalisation, `export_multisig_info` échoue avec "not yet finalized"

---

## 📊 Comparaison Documentation vs Code Réel

### Documentation (DOX/architecture/MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md)

**Décrit**: 2 rounds
1. Round 1: `make_multisig`
2. Round 2: `exchange_multisig_keys`

### Code Fonctionnel (commit 8e3f282)

**Implémente**: 3 rounds
1. Round 1: `make_multisig(threshold=2, prepare_infos)`
2. Round 2: `exchange_multisig_keys(round1_multisig_infos)`
3. Round 3: `exchange_multisig_keys(round2_multisig_infos)`

### Code Actuel (commit fe9e887)

**Implémente**: 1 round incomplet (CASSÉ)
- Seulement `make_multisig` avec un filter incorrect
- Aucun appel à `exchange_multisig_keys`
- Commentaire dit "This is incorrect, just a placeholder"

---

## 🔧 Divergences Critiques Identifiées

| Composant | Documentation | Code 8e3f282 (Working) | Code fe9e887 (Actuel) | Status |
|-----------|---------------|------------------------|------------------------|--------|
| **Rounds multisig** | 2 rounds | 3 rounds | 1 round | ❌ CASSÉ |
| **enable-multisig-experimental** | Pas mentionné | Activé automatiquement | Supprimé | ❌ CASSÉ |
| **sync_multisig_wallets()** | Documenté | Implémenté | Supprimé | ❌ CASSÉ |
| **Validation SHA256** | Non documentée | Implémentée | Supprimée | ❌ CASSÉ |
| **Délais inter-rounds** | Non documentés | 10 secondes | 0 seconde | ❌ CASSÉ |
| **Wallet open/close cycles** | Non documentés | Implémentés | Supprimés | ❌ CASSÉ |

---

## 🚨 Impact sur le Fonctionnement

### Avant (commit 8e3f282)
✅ Escrow créé → Multisig finalisé → XMR reçu → Balance visible → Release OK

### Maintenant (commit fe9e887)
❌ Escrow créé → Multisig NON finalisé → XMR reçu → Balance = 0 → Release IMPOSSIBLE

---

## 🛠️ Actions Requises (Par Ordre de Priorité)

### 1. URGENT: Restaurer le code supprimé

**Fichier**: `server/src/wallet_manager.rs`

**Restaurations nécessaires**:

```bash
# Restaurer depuis commit 8e3f282
git show 8e3f282:server/src/wallet_manager.rs > /tmp/wallet_manager_working.rs

# Extraire les 3 sections critiques:
# 1. Lignes 673-705: enable-multisig-experimental
# 2. Lignes 933-1092: sync_multisig_wallets()
# 3. Lignes 1211-1565: exchange_multisig_info() 3-rounds flow
```

### 2. CRITIQUE: Tester la restauration

```bash
# 1. Compiler
cargo build --release --package server

# 2. Démarrer Monero testnet + wallet RPCs
./scripts/start-testnet.sh

# 3. Créer un escrow de test
curl -X POST http://localhost:8080/api/orders ...

# 4. Vérifier multisig finalisé
curl http://localhost:18082/json_rpc \
  --data '{"jsonrpc":"2.0","id":"0","method":"export_multisig_info"}'

# SUCCESS: {"result":{"info":"MultisigxV2R..."}}
# FAIL: {"error":{"message":"not yet finalized"}}

# 5. Envoyer XMR testnet
monero-wallet-cli --testnet
> transfer 9sCrDesy... 0.003

# 6. Sync et vérifier balance
curl -X POST http://localhost:8080/api/escrow/{id}/check-balance

# SUCCESS: {"balance_xmr":"0.003000000000"}
# FAIL: {"balance_xmr":"0.000000000000"}
```

### 3. IMPORTANT: Mettre à jour la documentation

**Fichier à corriger**: `DOX/architecture/MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md`

**Changements**:
- Section "Le Protocole Monero Multisig 2-of-3": Changer "2 rounds" → "3 rounds"
- Ajouter Round 3: Second `exchange_multisig_keys` pour finalisation
- Ajouter section "enable-multisig-experimental" dans prérequis
- Documenter les délais 10s entre rounds
- Documenter les cycles wallet open/close

---

## 📝 Cause Racine

**Git Blame**:
```
commit fe9e887 - feat(monitoring): Add BlockchainMonitor background service
Author: [Git Author]
Date: [Date]
```

**Théorie**:
- Lors de l'ajout de BlockchainMonitor, un merge/rebase incorrect a supprimé ces sections
- Ou: copier-coller depuis une version plus ancienne du fichier
- Ou: conflit de merge résolu incorrectement

**Preuve**:
Le diff montre 350+ lignes supprimées remplacées par 25 lignes de placeholder avec commentaire "This is incorrect".

---

## ✅ Validation Post-Fix

Une fois le code restauré, vérifier:

1. ✅ Compilation sans erreurs
2. ✅ `enable-multisig-experimental` activé dans logs
3. ✅ 3 rounds d'échange de clés dans logs
4. ✅ `export_multisig_info` fonctionne (pas d'erreur "not yet finalized")
5. ✅ Balance visible après sync (`sync_multisig_wallets`)
6. ✅ Release funds fonctionne

---

## 📚 Références

- Commit fonctionnel: `8e3f282` - "working multisig"
- Commit cassé: `fe9e887` - "feat(monitoring): Add BlockchainMonitor"
- Diff critique: `git diff 8e3f282 fe9e887 -- server/src/wallet_manager.rs`
- Documentation: `DOX/architecture/MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md`
- Documentation sync: `DOX/architecture/MULTISIG-SYNC-IMPLEMENTATION.md`

---

## 🎯 Conclusion

**Le multisig a été cassé accidentellement lors de l'ajout de BlockchainMonitor.**

**Solution**: Restaurer les 3 sections critiques depuis le commit `8e3f282` (dernière version fonctionnelle confirmée).

**Temps estimé de fix**: 30-60 minutes (restauration + tests)

**Risque si non fixé**: Le marketplace ne peut PAS fonctionner - les escrows ne peuvent ni recevoir ni relâcher les fonds.
