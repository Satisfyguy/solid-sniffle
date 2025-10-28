# Analyse Architecture Non-Custodiale - 23 Octobre 2025

## Résumé Exécutif

**Question:** Le marketplace Monero est-il **custodial** ou **non-custodial**?

**Réponse:** 🟡 **HYBRIDE / AMBIGU** - L'architecture peut être configurée des deux façons

---

## Analyse du Code Actuel

### ✅ Points Positifs (Non-Custodial)

1. **Pas de génération de clés côté serveur**
   - Le code ne génère JAMAIS de `PrivateKey` ou `seed phrase`
   - Aucun appel à `create_wallet()` avec nouvelles clés
   - Fichier: `wallet/src/` - Aucune génération de clés trouvée

2. **Utilisation de wallet RPC externes**
   ```rust
   // server/src/wallet_manager.rs:94
   let rpc_client = MoneroClient::new(config.clone())?;
   let wallet_info = rpc_client.get_wallet_info().await?;
   ```
   - Se connecte à des wallets RPC **préexistants**
   - Ne crée pas les wallets, juste des connexions

3. **Multisig 2-of-3 implémenté correctement**
   - `prepare_multisig()` - ✅ Présent
   - `make_multisig()` - ✅ Présent
   - `export_multisig_info()` - ✅ Présent
   - `import_multisig_info()` - ✅ Présent

### ⚠️ Zones Grises (Problématiques)

1. **Configuration RPC ambiguë**
   ```rust
   // server/src/main.rs
   let monero_configs = vec![
       MoneroConfig {
           rpc_url: "http://127.0.0.1:18082".to_string(),
           // ... config pour TOUS les wallets (buyer, vendor, arbiter)
       }
   ];
   ```

   **Problème:** Si `127.0.0.1:18082` pointe vers:
   - **Cas A (CUSTODIAL ❌):** Un seul `monero-wallet-rpc` sur le serveur avec 3 wallets
   - **Cas B (NON-CUSTODIAL ✅):** Proxy/relais vers les wallets RPC des clients

2. **WalletManager crée instances pour TOUS les rôles**
   ```rust
   // server/src/wallet_manager.rs:84-108
   pub async fn create_wallet_instance(
       &mut self,
       role: WalletRole, // Buyer, Vendor, OU Arbiter
   ) -> Result<Uuid, WalletManagerError>
   ```

   **Problème:** Le serveur peut créer instances pour:
   - ✅ Arbiter (OK - c'est le marketplace)
   - ⚠️ Buyer (Devrait être sur machine du buyer!)
   - ⚠️ Vendor (Devrait être sur machine du vendor!)

3. **Aucun mécanisme de client-side wallet**
   - Pas de module WASM pour générer clés côté client
   - Pas d'API pour que clients fournissent leurs propres RPC URLs
   - Frontend assume que serveur gère tout

## Architecture Actuelle (Inférée)

### Scénario Probable: CUSTODIAL ❌

```
┌─────────────────────────────────────┐
│      SERVEUR MARKETPLACE            │
│  (Contrôle TOUT)                    │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  monero-wallet-rpc          │   │
│  │  Port 18082                 │   │
│  │                             │   │
│  │  - buyer_wallet_123.keys    │   │  ❌ CUSTODIAL
│  │  - vendor_wallet_456.keys   │   │  (serveur a les clés)
│  │  - arbiter_wallet_789.keys  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  WalletManager               │   │
│  │  - Crée instances pour       │   │
│  │    buyer, vendor, arbiter    │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

**Risques:**
- 🔴 Exit scam possible
- 🔴 Hack serveur = perte tous fonds
- 🔴 Contradiction avec vision non-custodiale

### Scénario Souhaité: NON-CUSTODIAL ✅

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ CLIENT BUYER │         │  SERVEUR     │         │CLIENT VENDOR │
│              │         │  MARKETPLACE │         │              │
│ wallet-rpc   │         │              │         │ wallet-rpc   │
│ :18082       │         │ wallet-rpc   │         │ :18083       │
│ (sa machine) │         │ :18084       │         │ (sa machine) │
│              │         │ (arbiter)    │         │              │
└──────────────┘         └──────────────┘         └──────────────┘
       │                        │                        │
       └────────────────────────┴────────────────────────┘
                    Multisig 2-of-3 XMR
               (Chacun contrôle sa clé)
```

**Avantages:**
- ✅ Pas d'exit scam possible
- ✅ Hack serveur ≠ perte fonds clients
- ✅ Conforme vision crypto

## Tests pour Déterminer le Statut

### Test 1: Vérifier Configuration RPC

```bash
# Où est monero-wallet-rpc?
ps aux | grep monero-wallet-rpc

# Si output montre plusieurs instances:
# - 127.0.0.1:18082 (arbiter) → OK
# - 127.0.0.1:18083 (buyer) → ❌ CUSTODIAL
# - 127.0.0.1:18084 (vendor) → ❌ CUSTODIAL
```

### Test 2: Inspecter Fichiers Wallet

```bash
cd ~/.bitmonero/testnet/wallets/
ls -la

# Si on voit:
# - buyer_wallet_*.keys → ❌ CUSTODIAL
# - vendor_wallet_*.keys → ❌ CUSTODIAL
# - arbiter_wallet.keys → ✅ OK (marketplace)
```

### Test 3: Analyser Logs Serveur

```bash
cargo run -p server 2>&1 | grep "wallet"

# Chercher des messages comme:
# "Created wallet for buyer" → ❌ CUSTODIAL
# "Connected to buyer's wallet RPC at ..." → ✅ NON-CUSTODIAL
```

## Plan de Migration NON-CUSTODIAL

### Phase 1: Audit Configuration Actuelle - ✅ COMPLÉTÉ (23 Oct 2025)

**Actions:**
1. ✅ Documenter configuration RPC actuelle
2. ✅ Identifier tous les wallets gérés par serveur
3. ✅ Déterminer si custodial ou non

#### Résultats de l'Audit

**1. ✅ Processus Monero sur serveur**
```bash
ps aux | grep monero-wallet-rpc
```
**Résultat:** Aucun processus actif

**2. ✅ Fichiers wallet sur serveur**
```bash
ls ~/.bitmonero/
```
**Résultat:** Aucun répertoire .bitmonero trouvé

**3. ✅ Configuration RPC analysée**

**Fichier:** `common/src/lib.rs:21`
```rust
pub const MONERO_RPC_URL: &str = "http://127.0.0.1:18082/json_rpc";
```

**Fichier:** `common/src/types.rs:282-290`
```rust
impl Default for MoneroConfig {
    fn default() -> Self {
        Self {
            rpc_url: MONERO_RPC_URL.to_string(),  // localhost:18082
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 30,
        }
    }
}
```

**Trouvaille:** Configuration codée en dur = localhost uniquement

**4. ✅ Base de données analysée**

**Schema:** `database/schema.sql`

**Table `users`:**
- `wallet_address VARCHAR(95)` - Adresse publique uniquement ✅
- `wallet_id VARCHAR(36)` - UUID de référence ✅
- **AUCUN** champ pour clés privées ✅
- **AUCUN** champ pour seed phrases ✅

**Table `escrows`:**
```sql
buyer_wallet_info TEXT,    -- ENCRYPTED ⚠️
vendor_wallet_info TEXT,   -- ENCRYPTED ⚠️
arbiter_wallet_info TEXT,  -- ENCRYPTED ⚠️
multisig_address VARCHAR(95)
```

**⚠️ ALERTE CRITIQUE:** Les champs `*_wallet_info` sont suspects!
**Action requise:** Analyser ce qui est stocké exactement dans ces champs

**5. ✅ Code wallet_manager.rs analysé**

**Fichier:** `server/src/wallet_manager.rs:84-108`

**Méthode problématique:**
```rust
pub async fn create_wallet_instance(
    &mut self,
    role: WalletRole,  // ⚠️ Peut être Buyer, Vendor OU Arbiter
) -> Result<Uuid, WalletManagerError> {
    let config = self.rpc_configs.get(self.next_rpc_index)...;
    // Utilise toujours MoneroConfig::default() = localhost
    let rpc_client = MoneroClient::new(config.clone())?;
    let wallet_info = rpc_client.get_wallet_info().await?;
    // Se connecte à un wallet RPC EXISTANT (ne crée pas de clés)
}
```

**Trouvailles:**
- ❌ **PROBLÈME #1:** Serveur crée instances pour TOUS les rôles (Buyer, Vendor, Arbiter)
- ❌ **PROBLÈME #2:** Utilise toujours localhost (pas d'option pour RPC URL client)
- ✅ **POSITIF:** Ne génère PAS de clés privées
- ✅ **POSITIF:** Se connecte à wallets RPC pré-existants

**Méthode multisig:** `make_multisig()` ligne 110
```rust
pub async fn make_multisig(
    &mut self,
    wallet_id: Uuid,
    _participants: Vec<String>,
) -> Result<MultisigInfo, WalletManagerError> {
    let wallet = self.wallets.get_mut(&wallet_id)...;
    let info = wallet.rpc_client.multisig().prepare_multisig().await?;
    // ⚠️ Appelle prepare_multisig() au nom du client!
}
```

**Trouvaille:** Le serveur appelle `prepare_multisig()` directement!

#### Verdict Phase 1

**Statut:** 🟡 **ARCHITECTURE HYBRIDE - POTENTIELLEMENT CUSTODIAL**

**Éléments non-custodiaux (positifs):**
- ✅ Aucun processus wallet-rpc actif sur serveur
- ✅ Aucun fichier wallet stocké sur serveur
- ✅ Code ne génère jamais de clés privées
- ✅ Base de données ne stocke pas de clés/seeds

**Éléments custodiaux (problématiques):**
- ❌ Serveur crée instances pour wallets buyer/vendor (devrait être client-side)
- ❌ Pas d'API pour clients fournissent leur propre RPC URL
- ❌ Configuration localhost codée en dur
- ❌ Serveur appelle `prepare_multisig()` au nom des clients
- ⚠️ Champs `*_wallet_info` chiffrés suspects (à investiguer)

**Conclusion:**
L'architecture actuelle **RESSEMBLE** à non-custodiale car pas de génération/stockage de clés, MAIS le workflow **FORCE** un modèle custodial où le serveur doit contrôler les wallets RPC de tous les participants.

**Action immédiate:** Phase 2 requise pour rendre vraiment non-custodial

### Phase 2: Supprimer Aspects Custodial (2-3 jours)

**SI custodial détecté:**

1. **Supprimer génération wallets buyer/vendor**
   ```rust
   // ❌ SUPPRIMER
   pub async fn create_wallet_for_buyer(...) { ... }
   pub async fn create_wallet_for_vendor(...) { ... }

   // ✅ GARDER
   pub async fn create_arbiter_wallet(...) { ... }
   ```

2. **Ajouter API pour clients fournissent RPC URL**
   ```rust
   // server/src/handlers/escrow.rs
   #[derive(Deserialize)]
   pub struct CreateEscrowRequest {
       buyer_rpc_url: String,  // ← NOUVEAU (client-side)
       vendor_rpc_url: String, // ← NOUVEAU (client-side)
       amount_xmr: f64,
   }
   ```

3. **Documenter setup client-side wallet**
   ```markdown
   # docs/CLIENT-WALLET-SETUP.md

   ## Pour Buyers/Vendors

   1. Installer monero-wallet-rpc sur votre machine
   2. Créer wallet: `monero-wallet-cli --testnet --generate-new-wallet`
   3. Lancer RPC: `monero-wallet-rpc --testnet --rpc-bind-port 18082`
   4. Fournir URL au marketplace: `https://your-domain.onion:18082`
   ```

### Phase 3: Client-Side WASM (1-2 semaines)

**Créer module WASM pour navigateur:**

1. **Nouveau crate `client-wallet/`**
   ```toml
   [package]
   name = "monero-client-wallet"
   edition = "2021"

   [dependencies]
   wasm-bindgen = "0.2"
   monero = "0.20"
   ```

2. **API JavaScript**
   ```javascript
   import init, { ClientWallet } from './pkg/client_wallet.js';

   await init();

   const wallet = new ClientWallet();
   const prepareInfo = wallet.prepare_multisig();

   // Envoyer prepareInfo au serveur (pas de clé privée!)
   fetch('/api/escrow/init', {
       method: 'POST',
       body: JSON.stringify({ multisig_info: prepareInfo })
   });
   ```

3. **Frontend intégration**
   - Générer clés dans navigateur
   - Stocker dans LocalStorage chiffré
   - Jamais envoyer au serveur

### Phase 4: Documentation & Tests (3-5 jours)

1. **Certification Non-Custodial**
   - Audit externe
   - Preuve cryptographique
   - Badges/seals

2. **Tests E2E**
   - Escrow complet avec clients séparés
   - Vérifier aucune clé sur serveur
   - Tester dispute resolution

3. **Documentation utilisateur**
   - Guide setup wallet personnel
   - FAQ non-custodial
   - Avantages sécurité

## Checklist Certification NON-CUSTODIAL

- [ ] **Aucune génération de clés côté serveur**
  - Pas de `PrivateKey::from_random_bytes()` dans server/
  - Pas de `create_wallet()` pour buyer/vendor

- [ ] **Aucun stockage de clés privées**
  - Pas de fichiers `.keys` pour clients sur serveur
  - Pas de seed phrases en DB

- [ ] **Clients contrôlent leurs wallets RPC**
  - Buyer lance son propre wallet-rpc
  - Vendor lance son propre wallet-rpc
  - Serveur lance UNIQUEMENT arbiter wallet-rpc

- [ ] **API permet fourniture RPC URL client**
  - Endpoint accepte `buyer_rpc_url`
  - Endpoint accepte `vendor_rpc_url`

- [ ] **Module WASM client-side** (optionnel mais recommandé)
  - Génération clés dans navigateur
  - Stockage local chiffré
  - Aucune transmission clé privée

- [ ] **Documentation claire**
  - README explique architecture non-custodiale
  - Guide setup wallet client
  - Preuves cryptographiques

- [ ] **Tests prouvent non-custodial**
  - Test E2E avec wallets séparés
  - Vérification filesystem serveur
  - Audit externe validé

## Estimation Effort

| Phase | Durée | Complexité |
|-------|-------|------------|
| Phase 1: Audit config | 1 jour | FAIBLE |
| Phase 2: Supprimer custodial | 2-3 jours | MOYENNE |
| Phase 3: WASM client | 1-2 semaines | HAUTE |
| Phase 4: Docs & tests | 3-5 jours | MOYENNE |
| **TOTAL** | **3-4 semaines** | - |

## Recommandation

**PRIORITÉ CRITIQUE:** Effectuer Phase 1 (Audit) IMMÉDIATEMENT pour déterminer statut actuel.

**Si custodial détecté:** Bloquer production jusqu'à migration Phase 2 complète.

**Si non-custodial:** Ajouter Phase 4 (certification) pour le prouver publiquement.

---

**Prochaine action:** Lancer audit configuration (`Phase 1`)

**Responsable:** Équipe technique
**Deadline:** Avant tout déploiement production
