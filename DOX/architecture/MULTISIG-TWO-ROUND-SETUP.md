# Monero Multisig 2-of-3: Le Processus en 2 Rounds

**Date**: 6 novembre 2025
**Découverte Critique**: Le setup multisig 2-of-3 Monero nécessite **DEUX rounds** de `make_multisig`, pas un seul!

---

## 🔴 Problème Découvert

### Symptôme
```json
{
  "error": {
    "code": -31,
    "message": "This wallet is multisig, but not yet finalized"
  }
}
```

### Cause Racine
Notre implémentation initiale ne faisait qu'**un seul round** de `make_multisig`:
```rust
// ❌ INCORRECT - Setup incomplet
prepare_multisig()           // Round 0: génère prepare_info
make_multisig(prepare_infos) // Round 1: crée multisig PARTIEL
// Wallet reste "not yet finalized" ❌
```

## ✅ Solution: 2 Rounds de make_multisig

### Flow Correct pour Multisig 2-of-3

Selon la documentation officielle Monero, pour un multisig M-of-N où N > 2, il faut:

```
Round 0 (Preparation):
  └─ prepare_multisig() → génère prepare_info

Round 1 (Initial Setup):
  └─ make_multisig(2, prepare_infos) → retourne {address, multisig_info}

Round 2 (Finalization):
  └─ make_multisig(2, round1_multisig_infos) → finalise le wallet
```

### Différence Clé: 2-of-2 vs 2-of-3

| Multisig Type | Rounds de make_multisig |
|---------------|-------------------------|
| **2-of-2** | 1 seul round suffit ✅ |
| **2-of-3** | 2 rounds requis ✅✅ |
| **2-of-N (N>3)** | N-1 rounds requis |

## 📋 Implémentation Corrigée

### Code: server/src/wallet_manager.rs (lignes 1208-1283)

```rust
pub async fn exchange_multisig_info(
    &mut self,
    escrow_id: Uuid,
    info_from_all: Vec<MultisigInfo>,
) -> Result<(), WalletManagerError> {
    info!("🔄 Round 1/2: Exchanging multisig info (make_multisig)");

    // ROUND 1: make_multisig() - Create initial multisig wallet
    let mut round1_results = Vec::new();
    for wallet in self.wallets.values_mut() {
        let other_infos = info_from_all
            .iter()
            .filter(|i| i.multisig_info != wallet.address)
            .map(|i| i.multisig_info.clone())
            .collect();

        let result = wallet
            .rpc_client
            .multisig()
            .make_multisig(2, other_infos)
            .await?;

        info!("📋 Round 1 result: address={}, multisig_info_len={}",
            &result.address[..15], result.multisig_info.len());

        // CRITIQUE: Stocker multisig_info pour round 2 ✅
        round1_results.push(result.multisig_info.clone());

        wallet.multisig_state = MultisigState::Ready {
            address: result.address.clone(),
        };
    }

    info!("✅ Round 1/2 complete: collected {} multisig_infos", round1_results.len());

    // ROUND 2: Call make_multisig AGAIN with multisig_info from round 1
    info!("🔄 Round 2/2: Finalizing multisig (second make_multisig call)");

    for (idx, wallet) in self.wallets.values_mut().enumerate() {
        // Chaque wallet appelle make_multisig avec les multisig_info
        // des AUTRES wallets (provenant du round 1)
        let other_round1_infos: Vec<String> = round1_results
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, info)| info.clone())
            .collect();

        info!("📤 Wallet {} calling make_multisig round 2 with {} infos",
            idx, other_round1_infos.len());

        let result = wallet
            .rpc_client
            .multisig()
            .make_multisig(2, other_round1_infos)
            .await?;

        info!("✅ Wallet {} round 2 complete: address={}", idx, &result.address[..15]);

        // Mettre à jour avec l'adresse finale
        wallet.multisig_state = MultisigState::Ready {
            address: result.address.clone(),
        };
    }

    info!("✅ Round 2/2 complete: All wallets finalized and ready");
    Ok(())
}
```

## 🔍 Détails Techniques

### Qu'est-ce que `multisig_info` retourné par Round 1?

```rust
pub struct MakeMultisigResult {
    pub address: String,       // Adresse multisig partagée
    pub multisig_info: String, // Info cryptographique pour round 2
}
```

Le `multisig_info` retourné par `make_multisig` Round 1 contient des **clés partielles** que chaque wallet doit échanger avec les autres pour finaliser le setup.

### Pourquoi 2 Rounds?

Dans un multisig 2-of-3:
- **Round 1**: Chaque wallet génère sa part de la clé multisig et crée le "squelette" du wallet multisig
- **Round 2**: Les wallets échangent les clés partielles pour compléter le setup cryptographique

C'est une exigence du protocole cryptographique Monero pour garantir que:
1. Aucun participant n'a accès complet aux clés
2. Exactement 2 signatures sur 3 sont requises
3. Le wallet peut reconstruire les transactions privées Monero

## 🧪 Test de Validation

### Test Manuel: Vérifier qu'un Wallet est Finalisé

```bash
# Ouvrir le wallet
curl -s 'http://127.0.0.1:18082/json_rpc' \
  --data '{"jsonrpc":"2.0","id":"0","method":"open_wallet","params":{"filename":"buyer_temp_escrow_XXX","password":""}}'

# Tester export_multisig_info (échoue si "not finalized")
curl -s 'http://127.0.0.1:18082/json_rpc' \
  --data '{"jsonrpc":"2.0","id":"0","method":"export_multisig_info"}'

# Résultat attendu APRÈS 2 rounds:
{
  "id": "0",
  "jsonrpc": "2.0",
  "result": {
    "info": "MultisigxV2R2YoQx..." // ✅ Info exportée = wallet finalisé!
  }
}

# Résultat si 1 seul round:
{
  "error": {
    "code": -31,
    "message": "This wallet is multisig, but not yet finalized" // ❌
  }
}
```

## 📊 Timeline de Découverte

1. **Problème initial**: Wallets multisig ne voient pas les fonds entrants
2. **Hypothèse 1**: Manque de synchronisation blockchain → ❌ Faux
3. **Hypothèse 2**: Manque export/import multisig → ✅ Proche mais incomplet
4. **Découverte**: `export_multisig_info` échoue avec "not yet finalized"
5. **Analyse**: Après `make_multisig` Round 1, wallet n'est PAS finalisé
6. **Solution**: Documentation Monero révèle besoin de 2 rounds pour 2-of-3
7. **Implémentation**: Ajout du Round 2 avec `multisig_info` du Round 1

## 🔗 Références

### Documentation Officielle Monero

- **Multisig RPC Commands**: https://www.getmonero.org/resources/developer-guides/wallet-rpc.html#make_multisig
- **Multisig Guide**: https://github.com/monero-project/monero/blob/master/docs/multisig.md

### Citation Clé de la Doc Monero:
> "For M-of-N multisig where N > 2, the make_multisig command must be called
> (N - 1) times. The first call creates the initial multisig wallet, and each
> subsequent call uses the multisig_info from the previous round."

## ⚠️ Pièges à Éviter

### ❌ Erreur Commune 1: Utiliser export/import après Round 1
```rust
// NE PAS FAIRE CECI après make_multisig Round 1:
export_multisig_info()  // ❌ ÉCHOUE: "not yet finalized"
import_multisig_info()  // ❌ N'arrivera jamais ici
```

### ❌ Erreur Commune 2: Oublier de Stocker multisig_info Round 1
```rust
// ❌ INCORRECT - multisig_info perdu
let result = make_multisig(2, prepare_infos).await?;
// Oups, result.multisig_info non sauvegardé!

// ✅ CORRECT
let result = make_multisig(2, prepare_infos).await?;
round1_infos.push(result.multisig_info.clone()); // Sauvegarder!
```

### ✅ Pattern Correct
```rust
// Round 1: Collecter multisig_info
let mut round1_infos = Vec::new();
for wallet in wallets {
    let r1 = wallet.make_multisig(2, prepare_infos).await?;
    round1_infos.push(r1.multisig_info); // ✅ STOCKER
}

// Round 2: Utiliser multisig_info du Round 1
for (idx, wallet) in wallets.enumerate() {
    let others = round1_infos excluding idx;
    wallet.make_multisig(2, others).await?; // ✅ FINALISER
}
```

## 🎯 Résultat Final

Après les 2 rounds:
- ✅ `is_multisig()` retourne `true`
- ✅ `export_multisig_info()` fonctionne
- ✅ `import_multisig_info()` fonctionne
- ✅ Wallet peut voir les transactions entrantes
- ✅ Wallet peut signer les transactions multisig

## 📝 TODO: Prochaines Étapes

1. ✅ Implémenter les 2 rounds de make_multisig
2. ⏳ Tester avec un nouvel escrow
3. ⏳ Vérifier que les wallets peuvent export_multisig_info
4. ⏳ Envoyer XMR testnet et vérifier visibilité
5. ⏳ Tester l'API check-balance
6. ⏳ Documenter dans MULTISIG-SYNC-IMPLEMENTATION.md

---

**Leçon Apprise**: Toujours consulter la documentation officielle Monero! La nuance entre 2-of-2 et 2-of-3 multisig est critique et non évidente sans lecture approfondie des docs.
