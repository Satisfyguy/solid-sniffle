# Guide de Migration vers le Mode Non-Custodial
## Monero Marketplace - Phase 3 Dépréciation

**Date:** 2025-11-08
**Status:** 🚨 Migration URGENTE Recommandée
**Deadline:** v0.4.0 (estimé 2-3 semaines)

---

## Table des Matières

1. [Pourquoi Migrer?](#pourquoi-migrer)
2. [Qu'est-ce qui Change?](#quest-ce-qui-change)
3. [Fonctions Dépréciées](#fonctions-dépréciées)
4. [Migration Pas-à-Pas](#migration-pas-à-pas)
5. [Comparaison Avant/Après](#comparaison-avantaprès)
6. [Timeline](#timeline)
7. [Support](#support)

---

## Pourquoi Migrer?

### Le Problème du Mode Custodial Actuel

**Mode custodial (ANCIEN - DÉPRÉCIÉ):**
```rust
// ❌ DÉPRÉCIÉ: Le serveur crée les wallets
let buyer_wallet = wallet_manager.create_temporary_wallet(escrow_id, "buyer").await?;
let vendor_wallet = wallet_manager.create_temporary_wallet(escrow_id, "vendor").await?;
let arbiter_wallet = wallet_manager.create_temporary_wallet(escrow_id, "arbiter").await?;

// Problèmes:
// - Serveur a accès aux fichiers wallets
// - Serveur gère les clés (même si wallets vides)
// - Violation principes non-custodial
```

**Mode non-custodial (NOUVEAU - RECOMMANDÉ):**
```rust
// ✅ NOUVEAU: Les clients fournissent leurs RPC URLs
coordinator.register_client_wallet(escrow_id, EscrowRole::Buyer, "http://127.0.0.1:18083").await?;
coordinator.register_client_wallet(escrow_id, EscrowRole::Seller, "http://127.0.0.1:18084").await?;
coordinator.register_client_wallet(escrow_id, EscrowRole::Arbiter, "http://127.0.0.1:18085").await?;

// Avantages:
// - Serveur ne crée JAMAIS de wallets
// - Serveur ne touche JAMAIS aux clés
// - 100% non-custodial (Haveno-style)
```

### Bénéfices de la Migration

✅ **Sécurité renforcée** - Clés privées restent chez les clients
✅ **Conformité non-custodial** - Architecture Haveno-style pure
✅ **Transparence** - Code serveur ne peut pas accéder aux fonds
✅ **Résilience** - Pas de point de défaillance central
✅ **Audit facilité** - Flow clairement non-custodial

---

## Qu'est-ce qui Change?

### Architecture

**AVANT (Custodial - Déprécié):**
```
Client → Serveur crée wallets → Serveur gère multisig → Client envoie fonds
         ❌ Serveur a accès aux fichiers wallets
```

**APRÈS (Non-Custodial - Recommandé):**
```
Client lance wallet local → Serveur coordonne échange infos → Client finalise multisig
                           ✅ Serveur ne touche jamais aux wallets
```

### API Changes

| Ancien (Déprécié) | Nouveau (Recommandé) |
|-------------------|----------------------|
| `POST /api/orders/init-escrow` | `POST /api/v2/escrow/register-wallet` |
| Serveur appelle `wallet_manager.create_temporary_wallet()` | Client envoie son RPC URL |
| `EscrowOrchestrator::init_escrow()` | `EscrowCoordinator::coordinate_multisig_exchange()` |

---

## Fonctions Dépréciées

### 1. `WalletManager::create_temporary_wallet()`

**Status:** 🚨 Déprécié depuis v0.3.0
**Sera supprimé:** v0.4.0 (2-3 semaines)

**Signature:**
```rust
#[deprecated(
    since = "0.3.0",
    note = "Use EscrowCoordinator with client wallets instead. Will be removed in v0.4.0"
)]
pub async fn create_temporary_wallet(&mut self, escrow_id: Uuid, role: &str) -> Result<Uuid>
```

**Pourquoi déprécié:**
- Crée wallets sur serveur (fichiers .keys accessibles)
- Serveur a accès potentiel aux clés (même si wallets vides)
- Violation principes non-custodial

**Remplacer par:**
```rust
// Clients lancent leur propre wallet RPC
// monero-wallet-rpc --rpc-bind-port 18083 --disable-rpc-login

// Serveur coordonne uniquement
coordinator.register_client_wallet(
    escrow_id,
    EscrowRole::Buyer,
    "http://127.0.0.1:18083" // URL fournie par client
).await?;
```

---

### 2. `EscrowOrchestrator::init_escrow()`

**Status:** 🚨 Déprécié depuis v0.3.0
**Sera supprimé:** v0.4.0 (2-3 semaines)

**Signature:**
```rust
#[deprecated(
    since = "0.3.0",
    note = "Server-side wallet creation is custodial. Use EscrowCoordinator instead. Will be removed in v0.4.0"
)]
pub async fn init_escrow(
    &self,
    order_id: Uuid,
    buyer_id: Uuid,
    vendor_id: Uuid,
    amount_atomic: i64,
) -> Result<Escrow>
```

**Pourquoi déprécié:**
- Utilise `create_temporary_wallet()` en interne (custodial)
- Malgré commentaires "[NON-CUSTODIAL]", c'est custodial
- Crée 3 wallets sur serveur (lignes 202-214)

**Remplacer par:**
```rust
// Workflow complet non-custodial
// 1. Clients lancent wallets RPC
// 2. Serveur coordonne
coordinator.coordinate_multisig_exchange(escrow_id).await?;
```

---

## Migration Pas-à-Pas

### Option A: Migration Utilisateur Final (Recommandée)

**Pour utilisateurs qui lancent des escrows:**

#### Étape 1: Installer Monero CLI

```bash
wget https://downloads.getmonero.org/cli/monero-linux-x64-v0.18.3.1.tar.bz2
tar -xjf monero-linux-x64-v0.18.3.1.tar.bz2
cd monero-x86_64-linux-gnu-v0.18.3.1
```

#### Étape 2: Lancer Wallet RPC Local

```bash
# Testnet (pour tests)
./monero-wallet-rpc \
  --testnet \
  --rpc-bind-port 18083 \
  --disable-rpc-login \
  --wallet-dir ~/.monero/testnet/wallets \
  --offline

# Mainnet (pour production)
./monero-wallet-rpc \
  --rpc-bind-port 18083 \
  --disable-rpc-login \
  --wallet-dir ~/.monero/wallets \
  --daemon-address node.moneroworld.com:18089
```

#### Étape 3: Utiliser le CLI Non-Custodial

```bash
cd /path/to/solid-sniffle

# Initialiser escrow non-custodial
cargo run --release --bin monero-marketplace -- noncustodial init-escrow \
  --escrow-id "escrow_abc123" \
  --role buyer \
  --wallet-name "my_buyer_wallet" \
  --local-rpc-url "http://127.0.0.1:18083" \
  --server-url "http://localhost:8080"
```

#### Étape 4: Suivre les Instructions

Le CLI guide automatiquement à travers:
1. Création wallet local
2. Enregistrement avec serveur
3. Attente autres participants
4. Coordination multisig
5. Finalisation locale

**Résultat:**
```
✅ Non-custodial escrow initialized successfully!
Multisig address: 5AYxY... (votre adresse multisig 2-of-3)
```

---

### Option B: Migration Développeur (API)

**Pour développeurs intégrant l'API:**

#### Ancien Code (Déprécié)

```rust
// ❌ ANCIEN - Génère warning de dépréciation
let escrow_orchestrator = EscrowOrchestrator::new(/* ... */);
let escrow = escrow_orchestrator.init_escrow(
    order_id,
    buyer_id,
    vendor_id,
    amount_atomic,
).await?;

// Warning:
// ⚠️ DEPRECATED: EscrowOrchestrator::init_escrow() uses server-side wallet creation (CUSTODIAL).
// Migrate to EscrowCoordinator for true non-custodial escrow.
```

#### Nouveau Code (Recommandé)

```rust
// ✅ NOUVEAU - Non-custodial
use server::coordination::{EscrowCoordinator, EscrowRole};

let coordinator = Arc::new(EscrowCoordinator::new());

// 1. Buyer enregistre son wallet RPC
coordinator.register_client_wallet(
    &escrow_id,
    EscrowRole::Buyer,
    "http://127.0.0.1:18083".to_string(), // Client's local RPC
).await?;

// 2. Seller enregistre son wallet RPC
coordinator.register_client_wallet(
    &escrow_id,
    EscrowRole::Seller,
    "http://127.0.0.1:18084".to_string(),
).await?;

// 3. Arbiter enregistre son wallet RPC
coordinator.register_client_wallet(
    &escrow_id,
    EscrowRole::Arbiter,
    "http://127.0.0.1:18085".to_string(),
).await?;

// 4. Serveur coordonne échange
let exchange_result = coordinator.coordinate_multisig_exchange(&escrow_id).await?;

// 5. Chaque client finalise localement avec les infos reçues
// (fait automatiquement par CLI ou manuellement via RPC)
```

---

## Comparaison Avant/Après

### Flow Custodial (Ancien - Déprécié)

```
┌─────────────────────────────────────────────────────────────┐
│ CLIENT                                                      │
│                                                              │
│  POST /api/orders/init-escrow                               │
│  {                                                           │
│    "order_id": "...",                                       │
│    "buyer_id": "...",                                       │
│    "vendor_id": "...",                                      │
│    "amount": 1.0                                            │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ SERVEUR (PROBLÈME: CUSTODIAL)                               │
│                                                              │
│  1. WalletManager::create_temporary_wallet("buyer")         │
│     → Crée /var/monero/wallets/buyer_temp_escrow_123        │
│     → Fichier .keys accessible au serveur ❌                │
│                                                              │
│  2. WalletManager::create_temporary_wallet("vendor")        │
│     → Crée /var/monero/wallets/vendor_temp_escrow_123       │
│     → Fichier .keys accessible au serveur ❌                │
│                                                              │
│  3. WalletManager::create_temporary_wallet("arbiter")       │
│     → Crée /var/monero/wallets/arbiter_temp_escrow_123      │
│     → Fichier .keys accessible au serveur ❌                │
│                                                              │
│  4. Setup multisig sur serveur                              │
│  5. Retourne adresse multisig                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
               Client envoie fonds à l'adresse
```

**Problèmes:**
- ❌ Serveur crée wallets (accès fichiers .keys)
- ❌ Serveur exécute prepare_multisig (opérations crypto)
- ❌ Risque théorique d'accès clés privées
- ❌ Non conforme architecture non-custodiale

---

### Flow Non-Custodial (Nouveau - Recommandé)

```
┌─────────────────────────────────────────────────────────────┐
│ CLIENT 1 (Buyer)                                            │
│                                                              │
│  1. Lance monero-wallet-rpc local (port 18083)              │
│     → Crée wallet SUR SON ORDINATEUR                        │
│     → Clés privées restent chez lui ✅                      │
│                                                              │
│  2. POST /api/v2/escrow/register-wallet                     │
│     {                                                        │
│       "escrow_id": "...",                                   │
│       "role": "buyer",                                      │
│       "rpc_url": "http://127.0.0.1:18083"                   │
│     }                                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ SERVEUR (COORDINATEUR UNIQUEMENT)                          │
│                                                              │
│  1. Stocke RPC URL: "http://127.0.0.1:18083"               │
│     → PAS de création wallet ✅                             │
│     → PAS d'accès fichiers .keys ✅                         │
│                                                              │
│  2. Attend autres participants...                           │
│  3. Quand 3 wallets enregistrés:                            │
│     - Demande prepare_multisig à chaque wallet client       │
│     - Échange multisig_info (données publiques uniquement)  │
│     - Retourne infos à chaque client                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ CLIENT 1 (Buyer) - Finalisation Locale                     │
│                                                              │
│  wallet.make_multisig(2, [seller_info, arbiter_info])      │
│  → Multisig finalisé SUR SON ORDINATEUR ✅                  │
│  → Serveur ne voit jamais les clés ✅                       │
└─────────────────────────────────────────────────────────────┘
```

**Avantages:**
- ✅ Client crée wallet localement
- ✅ Serveur ne touche jamais aux fichiers wallets
- ✅ Serveur coordonne uniquement (exchange infos publiques)
- ✅ Architecture 100% non-custodiale (Haveno-style)

---

## Timeline

### Phase 3 (ACTUELLE): Dépréciation - 3-4 jours

**Status:** 🚨 EN COURS
**Deadline:** 2025-11-12

- [x] Ajouter `#[deprecated]` à `create_temporary_wallet()`
- [x] Ajouter `#[deprecated]` à `init_escrow()`
- [x] Warnings dans logs à chaque appel
- [x] Guide de migration (ce document)
- [ ] Notification utilisateurs

**Actions:**
- ⚠️ Warnings affichés dans logs
- ⚠️ Documentation migration publiée
- ⚠️ Mode non-custodial devient recommandé

---

### Phase 4: Suppression - 2-3 jours (semaine du 2025-11-18)

**Status:** ⏳ PLANIFIÉE
**Deadline:** v0.4.0 (2025-11-25 estimé)

- [ ] Supprimer `WalletManager::create_temporary_wallet()`
- [ ] Supprimer `EscrowOrchestrator::init_escrow()`
- [ ] Supprimer routes `/api/orders/init-escrow` (custodial)
- [ ] Garder uniquement routes `/api/v2/escrow/*` (non-custodial)
- [ ] Tests finaux 100% non-custodial

**Actions:**
- ❌ Ancien code supprimé complètement
- ✅ Mode non-custodial uniquement
- ✅ Architecture 100% Haveno-style

---

## Support

### Ressources

📚 **Documentation:**
- Guide utilisateur: `DOX/guides/NON-CUSTODIAL-USER-GUIDE.md`
- Plan migration: `DOX/guides/MIGRATION-NON-CUSTODIAL-PLAN.md`
- Architecture: `server/src/coordination/README.md`

💻 **Code:**
- CLI non-custodial: `cli/src/noncustodial_wallet.rs`
- EscrowCoordinator: `server/src/coordination/escrow_coordinator.rs`
- Tests E2E: `server/tests/escrow_noncustodial_e2e.rs`

### Questions Fréquentes

**Q: Puis-je continuer à utiliser le mode custodial?**
**R:** Oui jusqu'à v0.4.0 (2-3 semaines), mais vous verrez des warnings. Migration recommandée dès que possible.

**Q: Le mode non-custodial est-il plus compliqué?**
**R:** Légèrement, car vous devez lancer votre propre wallet RPC. Mais le CLI automatise tout le processus.

**Q: Mes fonds sont-ils en danger avec le mode custodial?**
**R:** Non, car les wallets sont vides (coordination uniquement). Mais le principe custodial viole l'architecture cible.

**Q: Combien de temps prend la migration?**
**R:** 5-10 minutes pour un utilisateur avec le CLI. 1-2 heures pour intégration API complète.

### Contact

**Problèmes techniques:**
- GitHub Issues: https://github.com/Satisfyguy/solid-sniffle/issues

**Migration urgente:**
- Consultez le guide: `DOX/guides/NON-CUSTODIAL-USER-GUIDE.md`
- Exemples CLI: Section "Démarrage Rapide"

---

**Dernière mise à jour:** 2025-11-08
**Version du guide:** Phase 3 - v1.0
**Deadline suppression:** v0.4.0 (estimé 2025-11-25)
