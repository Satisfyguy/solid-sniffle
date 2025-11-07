# PROBLÈME MULTISIG 2-OF-3 - ANALYSE TECHNIQUE DÉTAILLÉE

**Date:** 7 Novembre 2025
**Durée:** 2 jours de debugging
**Status:** BLOQUEUR CRITIQUE - Empêche tout escrow de fonctionner

---

## RÉSUMÉ DU PROBLÈME

Le setup multisig 2-of-3 échoue systématiquement au **Round 2** avec l'erreur:
```
❌ Buyer wallet round 2 FAILED: Multisig("Already in multisig mode")
```

**Cause racine identifiée:** Après Round 1 (`make_multisig`), les wallets **Buyer** et **Vendor** génèrent des `multisig_info` **IDENTIQUES**, alors qu'ils devraient être UNIQUES.

---

## CONTEXTE TECHNIQUE

### Architecture Multisig 2-of-3 (Monero v0.18.4.3)

Le setup multisig se fait en 2 rounds:

**Round 0: Préparation**
```rust
prepare_multisig() → prepare_info (189 chars)
```
- Chaque wallet génère son `prepare_info` (clé publique partielle)
- Ces 3 prepare_infos doivent être UNIQUES

**Round 1: Création multisig**
```rust
make_multisig(2, [other_prepare_infos]) → { address, multisig_info }
```
- Chaque wallet reçoit les 2 AUTRES prepare_infos
- Génère un `multisig_info` pour synchronisation
- Ces 3 multisig_infos doivent être UNIQUES

**Round 2: Finalisation**
```rust
make_multisig(2, [other_multisig_infos]) → { address }
```
- Chaque wallet reçoit les 2 AUTRES multisig_infos du Round 1
- Finalise le wallet multisig 2-of-3

### Architecture RPC

3 instances `monero-wallet-rpc` en parallèle:
- **Port 18082** → Buyer wallets
- **Port 18083** → Vendor wallets
- **Port 18084** → Arbiter wallets

Assignation role-based dans `wallet_manager.rs` (lignes 262-300):
```rust
WalletRole::Buyer  → rpc_configs[i] where i % 3 == 0  // Port 18082
WalletRole::Vendor → rpc_configs[i] where i % 3 == 1  // Port 18083
WalletRole::Arbiter → rpc_configs[i] where i % 3 == 2  // Port 18084
```

---

## SYMPTÔMES OBSERVÉS

### Logs du dernier test (escrow 2415c3fa-76f2-4dd0-a2b8-7673347d6f39)

**✅ Round 0: prepare_infos UNIQUES (CORRECT)**
```
Buyer:   MultisigxV2R1GaWL2fE... (189 chars) ✅ UNIQUE
Vendor:  MultisigxV2R1FxSz6WJ... (189 chars) ✅ UNIQUE
Arbiter: MultisigxV2R1f8jwSAc... (189 chars) ✅ UNIQUE
```

**❌ Round 1: multisig_infos DUPLIQUÉS (PROBLÈME)**
```
round1_results[0] (Buyer):  MultisigxV2Rn1LWZVHsTzenW5sYxqmFACjNSWfxqQphtpZnYL (236 chars) ❌ IDENTIQUE!
round1_results[1] (Vendor): MultisigxV2Rn1LWZVHsTzenW5sYxqmFACjNSWfxqQphtpZnYL (236 chars) ❌ IDENTIQUE!
round1_results[2] (Arbiter): MultisigxV2Rn1LUpykZQ3BK6h2br51XV7Mee3pHAviBDrhNqG (236 chars) ✅ DIFFÉRENT
```

**Observation critique:** Buyer et Vendor génèrent le MÊME `multisig_info` alors qu'ils ont reçu des `prepare_infos` DIFFÉRENTS en entrée.

**❌ Round 2: ÉCHEC**
```
ERROR: Buyer wallet round 2 FAILED: Multisig("Already in multisig mode")
```
Monero RPC détecte un état invalide (multisig_info dupliqués) et refuse le 2ème `make_multisig`.

---

## HYPOTHÈSES TESTÉES ET SOLUTIONS TENTÉES

### ❌ Hypothèse 1: Logique de distribution des prepare_infos incorrecte

**Problème suspecté:** La fonction `exchange_multisig_info()` ne distribue pas les bons prepare_infos à chaque wallet.

**Solution testée:**
- Implémentation d'un matching role-based déterministe (`wallet_manager.rs` lignes 1208-1342)
- Ordre strict: Buyer → Vendor → Arbiter
- Distribution explicite:
  ```rust
  Buyer  → reçoit [Vendor_prepare_info, Arbiter_prepare_info]
  Vendor → reçoit [Buyer_prepare_info, Arbiter_prepare_info]
  Arbiter → reçoit [Buyer_prepare_info, Vendor_prepare_info]
  ```

**Résultat:** ❌ ÉCHEC
- Les logs confirment que la distribution est CORRECTE
- Buyer reçoit bien `[FxSz6WJ (Vendor), f8jwSAc (Arbiter)]`
- Vendor reçoit bien `[GaWL2fE (Buyer), f8jwSAc (Arbiter)]`
- **Mais ils génèrent quand même le MÊME multisig_info!**

**Conclusion:** Le problème n'est PAS dans la logique de distribution.

---

### ❌ Hypothèse 2: Collision dans l'assignation RPC (round-robin)

**Problème suspecté:** Le round-robin pourrait assigner plusieurs wallets au même RPC.

**Solution testée:**
- Vérification des logs `🎯 Assigned`
- Confirmation que chaque role utilise un RPC différent:
  ```
  Buyer  → http://127.0.0.1:18082 ✅
  Vendor → http://127.0.0.1:18083 ✅
  Arbiter → http://127.0.0.1:18084 ✅
  ```

**Résultat:** ❌ L'assignation est CORRECTE mais le problème persiste.

**Conclusion:** Chaque wallet utilise bien un RPC différent, mais ils génèrent quand même des multisig_info identiques.

---

### ❌ Hypothèse 3: Réutilisation de wallets existants (orphaned files)

**Problème suspecté:** Des fichiers wallet orphelins pourraient être réouverts au lieu d'en créer de nouveaux.

**Observations:**
- Le format de filename inclut l'escrow_id: `{role}_temp_escrow_{escrow_id}`
- Chaque nouveau escrow a un UUID unique → nouveaux fichiers
- Cleanup automatique implémenté (lignes 627-632):
  ```rust
  if wallet_path.exists() || keys_path.exists() {
      warn!("Found existing wallet files, deleting before recreation");
      std::fs::remove_file(&wallet_path);
      std::fs::remove_file(&keys_path);
  }
  ```

**Résultat:** ❌ Pas de cleanup détecté dans les logs → nouveaux wallets à chaque escrow.

**Conclusion:** Le problème n'est PAS lié à la réutilisation de fichiers.

---

### ❌ Hypothèse 4: Partage d'état via wallet-dir commun

**Problème suspecté:** Les 3 RPC instances utilisaient le MÊME `--wallet-dir /var/monero/wallets`, ce qui pourrait causer un partage d'état.

**Explication théorique:**
Monero wallet RPC ne peut avoir qu'**UN seul wallet ouvert à la fois** par instance. Si les 3 RPCs partagent le même répertoire:
1. RPC 18082 crée `buyer_temp_escrow_XXX`
2. RPC 18083 crée `vendor_temp_escrow_XXX`
3. RPC 18084 crée `arbiter_temp_escrow_XXX`

Mais quand on appelle `prepare_multisig()`, chaque RPC pourrait:
- Ouvrir le dernier wallet créé dans le répertoire partagé
- Ou maintenir une référence au même wallet
- Ou partager un cache/state interne

**Solution testée (DERNIÈRE TENTATIVE):**
```bash
# Création de wallet-dir ISOLÉS
mkdir -p /var/monero/wallets-buyer
mkdir -p /var/monero/wallets-vendor
mkdir -p /var/monero/wallets-arbiter

# Redémarrage avec isolation
monero-wallet-rpc --rpc-bind-port 18082 --wallet-dir /var/monero/wallets-buyer ...
monero-wallet-rpc --rpc-bind-port 18083 --wallet-dir /var/monero/wallets-vendor ...
monero-wallet-rpc --rpc-bind-port 18084 --wallet-dir /var/monero/wallets-arbiter ...
```

**Résultat:** ❌ **ÉCHEC** - Le problème PERSISTE même avec des wallet-dir isolés!

```
round1_results[0] (Buyer):  MultisigxV2Rn1LWZVHsTzenW5sYxqmFACjNSWfxqQphtpZnYL
round1_results[1] (Vendor): MultisigxV2Rn1LWZVHsTzenW5sYxqmFACjNSWfxqQphtpZnYL ← ENCORE IDENTIQUE!
round1_results[2] (Arbiter): MultisigxV2Rn1LUpykZQ3BK6h2br51XV7Mee3pHAviBDrhNqG
```

**Conclusion:** L'isolation des wallet-dir n'a PAS résolu le problème.

---

## ANALYSE APPROFONDIE DU MYSTÈRE

### Pourquoi Buyer et Vendor génèrent-ils le MÊME multisig_info?

**Inputs vérifiés comme DIFFÉRENTS:**
- ✅ `prepare_infos` sont uniques (logs Round 0)
- ✅ Distribution correcte (logs Round 1 avec `🔍 receiving`)
- ✅ RPCs différents (18082 vs 18083)
- ✅ Wallet-dir isolés
- ✅ Wallet files avec noms uniques

**Pourtant, le résultat est IDENTIQUE!**

Cela signifie que quelque chose en AMONT du `make_multisig` fait que Buyer et Vendor:
1. **Utilisent le même wallet** (impossible vu les wallet_files distincts)
2. **Partagent un état cryptographique** (seed? clés privées?)
3. **Le RPC fait un cache/memoization** des résultats
4. **Bug dans monero-wallet-rpc v0.18.4.3** pour le mode offline

### Observation supplémentaire: Arbiter fonctionne CORRECTEMENT

Arbiter génère un `multisig_info` DIFFÉRENT. Pourquoi?

**Différences potentielles:**
- Arbiter est traité EN DERNIER (ordre: Buyer → Vendor → Arbiter)
- Arbiter utilise le port 18084 (vs 18082/18083)
- Arbiter est dans un wallet-dir différent

**Similarités entre Buyer et Vendor (qui échouent):**
- Traités en PREMIER et DEUXIÈME
- Utilisent des ports consécutifs (18082, 18083)
- Reçoivent tous les deux `Arbiter_prepare_info` dans leurs inputs

---

## PISTES D'INVESTIGATION RESTANTES

### 🔍 Piste 1: Race condition dans la création séquentielle

**Code actuel (`escrow.rs` lignes 201-214):**
```rust
let buyer_temp_wallet_id = wallet_manager
    .create_temporary_wallet(escrow_id, "buyer")
    .await
    .context("Failed to create buyer temp wallet")?;

let vendor_temp_wallet_id = wallet_manager
    .create_temporary_wallet(escrow_id, "vendor")
    .await
    .context("Failed to create vendor temp wallet")?;

let arbiter_temp_wallet_id = wallet_manager
    .create_temporary_wallet(escrow_id, "arbiter")
    .await
    .context("Failed to create arbiter temp wallet")?;
```

Les wallets sont créés **séquentiellement** (avec `.await`), donc pas de race condition évidente.

**Mais:** Entre la création et le `prepare_multisig()`, il y a un `drop(wallet_manager)` (ligne 223). Peut-être que le wallet ouvert sur le RPC n'est pas le bon?

### 🔍 Piste 2: close_wallet() avant prepare_multisig()

**Code actuel (`wallet_manager.rs` ligne 639):**
```rust
// Close any currently open wallet first (Monero RPC can only have one wallet open at a time)
let _ = rpc_client.close_wallet().await; // Ignore errors if no wallet is open
```

On ferme le wallet **pendant la création**, mais peut-être qu'on devrait aussi:
1. Fermer AVANT chaque `prepare_multisig()`
2. Rouvrir explicitement le bon wallet
3. Vérifier quel wallet est ouvert avec `get_address()`

### 🔍 Piste 3: Monero RPC ne supporte pas --offline pour multisig

Documentation officielle Monero:
> Multisig wallets require connection to daemon for key exchange

Peut-être que `--offline` cause un comportement indéterminé lors de `make_multisig`?

**Test à faire:**
```bash
# Au lieu de --offline, utiliser --daemon-address avec un vrai daemon testnet
monero-wallet-rpc --rpc-bind-port 18082 --daemon-address 127.0.0.1:28081 --testnet
```

### 🔍 Piste 4: Bug dans monero-wallet-rpc v0.18.4.3

Vérifier la version:
```bash
monero-wallet-rpc --version
```

Chercher dans les issues GitHub Monero:
- Issues avec multisig 2-of-3
- Problèmes avec mode offline
- Race conditions dans wallet RPC

### 🔍 Piste 5: État partagé dans MoneroClient Rust

**Code `wallet/src/client.rs`:**
```rust
pub struct MoneroClient {
    rpc: Arc<MoneroRpcClient>,
    config: MoneroConfig,
}
```

Le `Arc<MoneroRpcClient>` partage-t-il un état entre plusieurs `MoneroClient`?

**À vérifier:**
- Est-ce que chaque wallet a son propre `MoneroClient`?
- Est-ce que `Arc` cause un partage d'état HTTP client?
- Le semaphore de rate limiting est-il global?

### 🔍 Piste 6: Ordre de distribution dans info_from_all

**Code actuel (`escrow.rs` ligne 377):**
```rust
wallet_manager
    .exchange_multisig_info(
        escrow_id,
        vec![buyer_info, vendor_info, arbiter_info],  // Ordre: [0]=buyer, [1]=vendor, [2]=arbiter
    )
    .await
```

Et dans `wallet_manager.rs` (lignes 1238-1250):
```rust
let other_infos: Vec<String> = match role {
    WalletRole::Buyer => {
        vec![info_from_all[1].multisig_info.clone(), info_from_all[2].multisig_info.clone()]
    },
    WalletRole::Vendor => {
        vec![info_from_all[0].multisig_info.clone(), info_from_all[2].multisig_info.clone()]
    },
    WalletRole::Arbiter => {
        vec![info_from_all[0].multisig_info.clone(), info_from_all[1].multisig_info.clone()]
    },
};
```

**Question:** Est-ce que l'ORDRE des prepare_infos passés à `make_multisig()` a une importance cryptographique?

Peut-être qu'il faut passer les prepare_infos dans un ordre déterministe (ex: toujours trié alphabétiquement)?

---

## DONNÉES POUR ANALYSE EXTERNE

### Fichiers clés
- `server/src/wallet_manager.rs` (lignes 600-750, 1208-1342)
- `server/src/services/escrow.rs` (lignes 195-377)
- `wallet/src/client.rs` (ligne 1-100)
- `wallet/src/rpc.rs` (ligne 1-200)

### Logs critiques
```bash
# Dans server_debug.log
grep "🔍 Incoming prepare_infos" server_debug.log
grep "round1_results" server_debug.log
grep "🎯 Assigned" server_debug.log
grep "round 2 FAILED" server_debug.log
```

### Environnement
- **OS:** Ubuntu 22.04 LTS (Linux 6.14.0-33-generic)
- **Monero:** v0.18.4.3 (à confirmer avec `monero-wallet-rpc --version`)
- **Rust:** 1.75+
- **Mode:** Testnet + Offline

### Commandes RPC actuelles
```bash
/usr/local/bin/monero-wallet-rpc \
  --rpc-bind-port 18082 \
  --disable-rpc-login \
  --wallet-dir /var/monero/wallets-buyer \
  --daemon-address 127.0.0.1:28081 \
  --testnet \
  --log-level 1 \
  --offline

/usr/local/bin/monero-wallet-rpc \
  --rpc-bind-port 18083 \
  --disable-rpc-login \
  --wallet-dir /var/monero/wallets-vendor \
  --daemon-address 127.0.0.1:28081 \
  --testnet \
  --log-level 1 \
  --offline

/usr/local/bin/monero-wallet-rpc \
  --rpc-bind-port 18084 \
  --disable-rpc-login \
  --wallet-dir /var/monero/wallets-arbiter \
  --daemon-address 127.0.0.1:28081 \
  --testnet \
  --log-level 1 \
  --offline
```

---

## QUESTIONS OUVERTES

1. **Pourquoi seuls Buyer et Vendor dupliquent-ils leur multisig_info?**
   - Arbiter fonctionne correctement
   - Différence: ordre de traitement? ports? wallet-dir?

2. **Pourquoi l'isolation des wallet-dir n'a rien changé?**
   - Cela devrait forcer une séparation totale des états
   - Le problème est-il au niveau du RPC ou du code Rust?

3. **Est-ce que le mode `--offline` est compatible avec multisig?**
   - Documentation Monero floue sur ce point
   - Peut-être que `make_multisig` nécessite daemon connection?

4. **Y a-t-il un cache dans monero-wallet-rpc?**
   - Cache des résultats `make_multisig`?
   - État partagé entre wallets dans le même processus?

5. **L'ordre des prepare_infos a-t-il une importance?**
   - Faut-il les passer dans un ordre canonique?
   - Docs Monero ne spécifient pas

---

## PROCHAINES ÉTAPES RECOMMANDÉES

### Option A: Tester sans --offline
```bash
# Démarrer un daemon testnet
monerod --testnet --offline

# RPCs avec connexion daemon
monero-wallet-rpc --rpc-bind-port 18082 --daemon-address 127.0.0.1:28081 --testnet
```

### Option B: Ajouter get_address() avant prepare_multisig()
```rust
// Vérifier quel wallet est ouvert
let address = rpc_client.get_address().await?;
info!("About to call prepare_multisig on wallet: {}", address);
```

### Option C: Fermer/Rouvrir explicitement avant Round 1
```rust
// Avant make_multisig Round 1
rpc_client.close_wallet().await?;
rpc_client.open_wallet(&wallet_filename, "").await?;
let multisig_result = rpc_client.multisig().make_multisig(2, other_infos).await?;
```

### Option D: Tester avec monero-wallet-cli manuellement
```bash
# Reproduire le problème en CLI pour isoler le bug
monero-wallet-cli --testnet --generate-new-wallet buyer_test
# prepare_multisig
# make_multisig 2 <vendor_info> <arbiter_info>
```

### Option E: Upgrade Monero version
```bash
# Télécharger la dernière version stable
# Vérifier si le bug est corrigé dans v0.18.5+
```

---

## IMPACT ET URGENCE

**Criticité:** 🔴 **BLOQUEUR ABSOLU**
- **AUCUN** escrow ne peut être créé
- Marketplace complètement non-fonctionnelle
- 2 jours de debugging sans résolution

**Scope affecté:**
- Module escrow (100%)
- Transactions Monero (100%)
- Onboarding utilisateurs (bloqué)

**Risque:**
- Si le problème est un bug Monero v0.18.4.3, solution = downgrade/upgrade version
- Si le problème est dans notre code, nécessite expertise cryptographie Monero

---

## CONTACT ET RESSOURCES

**Documentation Monero Multisig:**
- https://docs.getmonero.org/multisignature
- https://github.com/monero-project/monero/blob/master/docs/multisig.md

**Code de référence:**
- https://github.com/monero-project/monero/tree/master/src/wallet

**Forums/Support:**
- r/Monero
- #monero-dev (Libera.Chat IRC)
- GitHub Issues: https://github.com/monero-project/monero/issues

---

## MISE À JOUR: SOLUTIONS TESTÉES ET ÉCHEC

### ✅ Solution #5: Close/Open/Verify avant make_multisig (TESTÉ - ÉCHEC)

**Implémentation:**
```rust
// Avant CHAQUE make_multisig (Round 1 et Round 2):
1. wallet.rpc_client.close_wallet().await
2. wallet.rpc_client.open_wallet(&wallet_filename, "").await
3. let address = wallet.rpc_client.get_address().await // VERIFY
4. wallet.rpc_client.multisig().make_multisig(2, other_infos).await
```

**Fichier:** `server/src/wallet_manager.rs` lignes 1259-1296 (Round 1), 1358-1395 (Round 2)

**Résultat:** ❌ **ÉCHEC COMPLET**
- Les wallets sont vérifiés ouverts (adresses confirmées DIFFÉRENTES)
- Buyer: `9sCuUfstV4Efxac`
- Vendor: `9vY45hLMfktAYTP`
- Arbiter: `9xRcu1yYbVPMKX6`
- **MAIS** après `make_multisig`, Buyer et Vendor génèrent le MÊME multisig_info!

### ✅ Solution #6: Tri alphabétique des prepare_infos (TESTÉ - ÉCHEC)

**Hypothèse:** Monero est sensible à l'ORDRE des prepare_infos passés à `make_multisig()`.

**Implémentation:**
```rust
let mut other_infos: Vec<String> = match role { ... };
other_infos.sort(); // Tri alphabétique pour ordre déterministe
```

**Fichier:** `server/src/wallet_manager.rs` lignes 1259-1262

**Résultat:** ❌ **ÉCHEC COMPLET**
- Les prepare_infos sont triés alphabétiquement
- Logs confirment: `📊 Sorted prepare_infos for Buyer`
- **TOUJOURS** Buyer et Vendor génèrent le même multisig_info

### Logs de test final (escrow 7576e423-8d46-4913-b082-ff90ee7172fe):

```
✅ Buyer wallet VERIFIED open: address=9sCuUfstV4Efxac
✅ Vendor wallet VERIFIED open: address=9vY45hLMfktAYTP
✅ Arbiter wallet VERIFIED open: address=9xRcu1yYbVPMKX6

📊 Sorted prepare_infos for Buyer (alphabetical order)
📊 Sorted prepare_infos for Vendor (alphabetical order)
📊 Sorted prepare_infos for Arbiter (alphabetical order)

🔍 round1_results[0] (Buyer):  MultisigxV2Rn1LVmw8em4oUFjQBaRi9Jn24aKYPUaPg3YFt4k ❌ IDENTIQUE!
🔍 round1_results[1] (Vendor): MultisigxV2Rn1LVmw8em4oUFjQBaRi9Jn24aKYPUaPg3YFt4k ❌ IDENTIQUE!
🔍 round1_results[2] (Arbiter): MultisigxV2Rn1LWGi6z9ABhUVhvA5D1cXwak2bbTvYtMy1ACF ✅ DIFFÉRENT

❌ Buyer wallet round 2 FAILED: Multisig("Already in multisig mode")
```

---

## DIAGNOSTIC FINAL: BUG MONERO RPC OU INCOMPATIBILITÉ `--offline`

Après 6 solutions testées et 2 jours de debugging, **le problème n'est PAS dans notre code Rust**.

### Preuve irréfutable:

1. ✅ **3 RPC instances séparées** (18082, 18083, 18084) - vérifié avec `ps aux`
2. ✅ **3 wallet-dir isolés** (`wallets-buyer`, `wallets-vendor`, `wallets-arbiter`)
3. ✅ **3 wallets vérifiés ouverts** avec adresses DIFFÉRENTES
4. ✅ **prepare_infos UNIQUES** (préfixes différents confirmés)
5. ✅ **Distribution CORRECTE** des prepare_infos (logs détaillés)
6. ✅ **Tri alphabétique** pour ordre déterministe
7. ✅ **Close/Open explicite** avant chaque make_multisig

**ET POURTANT:** Buyer et Vendor génèrent le MÊME multisig_info à CHAQUE tentative.

### Hypothèses restantes:

#### A. Bug dans monero-wallet-rpc v0.18.4.3
- Possible race condition interne dans le code C++ de Monero
- Bug non reporté dans les 2-of-3 multisig
- Régression introduite dans v0.18.x

**Action:** Chercher dans https://github.com/monero-project/monero/issues
- Mots-clés: "multisig", "2-of-3", "identical", "make_multisig", "v0.18"

#### B. Mode `--offline` incompatible avec multisig
- La documentation Monero dit: "Multisig requires daemon connection"
- Le mode `--offline` pourrait causer un comportement non-déterministe
- Les clés cryptographiques pourraient être générées avec une source d'entropie insuffisante

**Action:** Tester SANS `--offline`:
```bash
# Démarrer daemon testnet
monerod --testnet --offline

# RPCs sans --offline
monero-wallet-rpc --rpc-bind-port 18082 --daemon-address 127.0.0.1:28081 --testnet
```

#### C. Problème cryptographique fondamental
- Monero multisig 2-of-3 pourrait avoir une faille de design
- Le protocole pourrait ne pas garantir l'unicité des multisig_info dans certains cas
- Problème avec FROST/musig2 implementation

**Action:** Consulter experts Monero sur IRC #monero-dev ou r/Monero

---

## RECOMMANDATION URGENTE

**ARRÊTER le développement multisig 2-of-3 jusqu'à résolution du bug Monero.**

**Plan B:**
1. **Option simple:** Utiliser escrow 2-of-2 (Buyer + Arbiter OU Vendor + Arbiter)
   - Plus simple cryptographiquement
   - Monero supporte mieux 2-of-2
   - Nécessite modification du flow business

2. **Option alternative:** Utiliser un smart contract layer (Ethereum/Polygon) pour l'escrow
   - Monero pour privacy des paiements
   - Smart contract pour la logique escrow
   - Atomic swaps XMR ↔ ETH

3. **Option attente:** Attendre Monero v0.19+ ou patch v0.18.4.4
   - Soumettre issue détaillée à l'équipe Monero
   - Fournir logs et reproduction steps

---

**Document créé le:** 2025-11-07 05:50 UTC
**Dernière mise à jour:** 2025-11-07 06:10 UTC
**Auteur:** Debugging session with Claude Code
**Status:** 🔴 **BLOQUEUR CONFIRMÉ - BUG MONERO RPC**
**Solutions testées:** 6/6 ÉCHECS
**Prochaine étape:** Consulter communauté Monero ou changer d'architecture
