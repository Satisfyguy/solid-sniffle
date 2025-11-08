# Plan de Migration Non-Custodial
## Monero Marketplace - Transition vers Architecture Haveno

**Date:** 2025-11-08
**Status:** PLAN - Non implémenté
**Auteur:** Audit basé sur analyse Haveno DEX
**Score actuel:** 56% non-custodial → **Objectif:** 100% non-custodial

---

## Table des Matières
1. [Résumé Exécutif](#résumé-exécutif)
2. [Analyse Comparative Haveno](#analyse-comparative-haveno)
3. [État Actuel (Master)](#état-actuel-master)
4. [Architecture Cible](#architecture-cible)
5. [Plan de Migration en 4 Phases](#plan-de-migration-en-4-phases)
6. [Risques et Mitigations](#risques-et-mitigations)
7. [Checklist de Validation](#checklist-de-validation)

---

## Résumé Exécutif

### Pourquoi Migrer?
**Problème actuel:** Le serveur crée et gère les wallets multisig, ce qui viole le principe non-custodial.

**Architecture actuelle (custodiale):**
```
Client → Server crée wallets → Server exécute prepare_multisig → Server gère clés
```

**Architecture cible (non-custodiale Haveno):**
```
Client local wallet-rpc → prepare_multisig local → Server coordonne échange infos UNIQUEMENT
```

### Principes de Migration
✅ **Backward compatible** - Pas de breaking changes
✅ **Progressive** - Migration par phases testables
✅ **Dual mode** - Ancien et nouveau système coexistent
✅ **Testable** - Chaque phase a des tests de validation
✅ **Rollback** - Possibilité de revenir en arrière à chaque phase

---

## Analyse Comparative Haveno

### Architecture Haveno (100% Non-Custodial)

**Fichiers analysés:**
- `haveno/core/src/main/java/haveno/core/xmr/wallet/TradeWalletService.java`
- `haveno/core/src/main/java/haveno/core/trade/protocol/tasks/ProcessInitMultisigRequest.java`
- `haveno/core/src/main/java/haveno/core/trade/Trade.java`

**Pattern Haveno:**
```java
// 1. Client lance son propre monero-wallet-rpc (LOCAL)
XmrWalletService walletService = new XmrWalletService();
walletService.createWallet(); // LOCAL, pas sur serveur

// 2. Client exécute prepare_multisig LOCALEMENT
String multisigInfo = walletService.prepareMultisig();

// 3. Client envoie uniquement l'INFO au serveur (pas le wallet)
sendMultisigInfoToServer(multisigInfo);

// 4. Serveur COORDONNE l'échange (ne touche jamais aux wallets)
tradeProtocol.exchangeMultisigInfo(buyerInfo, sellerInfo, arbiterInfo);

// 5. Client reçoit infos des autres et finalise LOCALEMENT
walletService.makeMultisig(threshold, otherParticipantsInfo);
```

**Validation Haveno (sécurité):**
```java
// Threshold validation (2-of-3 strict)
if (multisigInfo.getThreshold() != 2)
    throw new RuntimeException("Multisig wallet has unexpected threshold: " + multisigInfo.getThreshold());

// Participant count validation
if (multisigInfo.getNumParticipants() != 3)
    throw new RuntimeException("Multisig wallet has unexpected number of participants: " + multisigInfo.getNumParticipants());
```

### Points Clés Haveno à Adopter

1. **Wallet Local Obligatoire**
   - Chaque participant (buyer, seller, arbiter) lance `monero-wallet-rpc` localement
   - Serveur ne crée JAMAIS de wallets
   - Clés privées ne quittent JAMAIS le wallet local

2. **Serveur = Coordinateur Pur**
   - Serveur échange uniquement les `multisig_info` (strings publiques)
   - Serveur valide les formats et seuils
   - Serveur ne fait AUCUNE opération cryptographique

3. **Validation Stricte**
   - Threshold = 2 (TOUJOURS)
   - Participants = 3 (TOUJOURS)
   - Validation côté serveur ET client

4. **États de Synchronisation**
   - États clairs: `PREPARED`, `MADE`, `SYNCED`, `READY`
   - 2 rounds d'export/import obligatoires
   - Vérification état avant chaque transition

---

## État Actuel (Master)

### Architecture Custodiale Actuelle

**Fichiers concernés:**
- `server/src/wallet_manager.rs` (ligne 653: `create_temporary_wallet`)
- `server/src/services/escrow.rs` (lignes 163-286: `init_escrow`)
- `server/src/handlers/orders.rs` (ligne 1051: appel init_escrow)

**Flux actuel (PROBLÉMATIQUE):**
```rust
// 1. Server crée wallets temporaires (CUSTODIAL ❌)
let buyer_temp_wallet_id = wallet_manager
    .create_temporary_wallet(escrow_id, "buyer").await?;

// 2. Server exécute prepare_multisig (CUSTODIAL ❌)
let info = wallet.rpc_client.multisig().prepare_multisig().await?;

// 3. Server gère wallets dans WalletPool (CUSTODIAL ❌)
wallet_pool.add_wallet(wallet_id, wallet).await;
```

### Code Existant Réutilisable ✅

**API non-custodiale déjà présente mais inutilisée:**
```rust
// server/src/main.rs:430-433
.route(
    "/escrow/register-wallet-rpc",
    web::post().to(escrow::register_wallet_rpc),
)
```

**Wallet crate déjà capable:**
```rust
// wallet/src/multisig.rs - MultisigManager
// wallet/src/rpc.rs - MoneroRpcClient avec validate_localhost_strict
// wallet/src/escrow.rs - EscrowManager logique métier
```

---

## Architecture Cible

### Nouveau Flux Non-Custodial

```
┌─────────────────────────────────────────────────────────────┐
│ BUYER (Client)                                              │
│ ┌─────────────────────────────┐                            │
│ │ monero-wallet-rpc (LOCAL)   │                            │
│ │ Port: 18083                  │                            │
│ │ prepare_multisig() → info_B │                            │
│ └─────────────────────────────┘                            │
│            ↓ POST /escrow/register-wallet-rpc              │
└─────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────┐
│ SERVER (Coordinateur UNIQUEMENT)                           │
│ ┌─────────────────────────────────────────────────────────┐│
│ │ EscrowCoordinator (PAS de wallets)                     ││
│ │ - Stocke RPC URLs (http://127.0.0.1:18083, etc.)       ││
│ │ - Échange info_B ↔ info_S ↔ info_A                    ││
│ │ - Valide threshold=2, participants=3                   ││
│ │ - Coordonne 2 rounds export/import                     ││
│ └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                    ↓ infos échangées
┌─────────────────────────────────────────────────────────────┐
│ SELLER (Client)           │  ARBITER (Client)              │
│ monero-wallet-rpc LOCAL   │  monero-wallet-rpc LOCAL       │
│ Port: 18084               │  Port: 18085                   │
│ make_multisig([info_B])   │  make_multisig([info_B])       │
└─────────────────────────────────────────────────────────────┘
```

### Nouvelles Structures de Données

```rust
// common/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWalletConfig {
    pub user_id: String,
    pub role: EscrowRole, // Buyer, Seller, Arbiter
    pub rpc_url: String,  // Must be localhost
    pub wallet_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowRole {
    Buyer,
    Seller,
    Arbiter,
}

// server/src/models/escrow_coordination.rs
pub struct EscrowCoordination {
    pub escrow_id: String,
    pub buyer_rpc_url: String,
    pub seller_rpc_url: String,
    pub arbiter_rpc_url: String,
    pub state: CoordinationState,
    pub multisig_infos: HashMap<EscrowRole, String>,
}

pub enum CoordinationState {
    AwaitingRegistrations,    // Attente des 3 wallets
    Prepared,                 // 3 prepare_multisig reçus
    MadeMultisig,             // make_multisig effectué (clients)
    ExportRound1Complete,     // Premier round export/import
    ExportRound2Complete,     // Deuxième round export/import
    Ready,                    // Multisig prêt
    Funded,                   // Funds reçus
}
```

---

## Plan de Migration en 4 Phases

### 📋 Phase 1: Dual Mode (4-6 jours)
**Objectif:** Introduire API non-custodiale sans casser l'ancien système

#### 1.1 Créer EscrowCoordinator (Nouveau)
```bash
# Créer nouveau module
touch server/src/coordination/escrow_coordinator.rs
touch server/src/coordination/mod.rs
```

**Implementation:**
```rust
// server/src/coordination/escrow_coordinator.rs
pub struct EscrowCoordinator {
    coordinations: Arc<RwLock<HashMap<String, EscrowCoordination>>>,
}

impl EscrowCoordinator {
    /// Enregistre un wallet client (NON-CUSTODIAL)
    pub async fn register_client_wallet(
        &self,
        escrow_id: &str,
        config: ClientWalletConfig,
    ) -> Result<()> {
        // 1. Valider localhost strict
        validate_localhost_strict(&config.rpc_url)?;

        // 2. Vérifier connectivité
        let client = MoneroRpcClient::new(MoneroConfig {
            rpc_url: config.rpc_url.clone(),
            ..Default::default()
        })?;
        client.check_connection().await?;

        // 3. Stocker URL (PAS le wallet)
        let mut coords = self.coordinations.write().await;
        let coord = coords.entry(escrow_id.to_string())
            .or_insert_with(|| EscrowCoordination::new(escrow_id));

        match config.role {
            EscrowRole::Buyer => coord.buyer_rpc_url = config.rpc_url,
            EscrowRole::Seller => coord.seller_rpc_url = config.rpc_url,
            EscrowRole::Arbiter => coord.arbiter_rpc_url = config.rpc_url,
        }

        Ok(())
    }

    /// Coordonne échange multisig_info (COORDINATION UNIQUEMENT)
    pub async fn coordinate_multisig_exchange(
        &self,
        escrow_id: &str,
    ) -> Result<MultisigExchangeResult> {
        let coords = self.coordinations.read().await;
        let coord = coords.get(escrow_id)
            .ok_or(Error::EscrowNotFound(escrow_id.to_string()))?;

        // 1. Vérifier que 3 wallets enregistrés
        if coord.buyer_rpc_url.is_empty() ||
           coord.seller_rpc_url.is_empty() ||
           coord.arbiter_rpc_url.is_empty() {
            return Err(Error::InvalidState("Missing wallet registrations".into()));
        }

        // 2. Demander prepare_multisig à chaque wallet
        let buyer_info = self.request_prepare_multisig(&coord.buyer_rpc_url).await?;
        let seller_info = self.request_prepare_multisig(&coord.seller_rpc_url).await?;
        let arbiter_info = self.request_prepare_multisig(&coord.arbiter_rpc_url).await?;

        // 3. Valider formats
        validate_multisig_info(&buyer_info)?;
        validate_multisig_info(&seller_info)?;
        validate_multisig_info(&arbiter_info)?;

        // 4. Échanger infos (chacun reçoit les 2 autres)
        Ok(MultisigExchangeResult {
            buyer_receives: vec![seller_info.clone(), arbiter_info.clone()],
            seller_receives: vec![buyer_info.clone(), arbiter_info.clone()],
            arbiter_receives: vec![buyer_info, seller_info],
        })
    }

    /// Helper: demande prepare_multisig à un wallet client
    async fn request_prepare_multisig(&self, rpc_url: &str) -> Result<String> {
        let client = MoneroRpcClient::new(MoneroConfig {
            rpc_url: rpc_url.to_string(),
            ..Default::default()
        })?;

        let info = client.prepare_multisig().await?;
        Ok(info.multisig_info)
    }
}
```

#### 1.2 Nouvelles API Routes
```rust
// server/src/handlers/escrow.rs

/// POST /api/escrow/register-wallet (NON-CUSTODIAL)
pub async fn register_client_wallet(
    coordinator: web::Data<Arc<EscrowCoordinator>>,
    req: web::Json<RegisterWalletRequest>,
) -> Result<HttpResponse> {
    coordinator.register_client_wallet(
        &req.escrow_id,
        req.config.clone(),
    ).await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "registered",
        "escrow_id": req.escrow_id,
        "role": req.config.role,
    })))
}

/// POST /api/escrow/coordinate-exchange (NON-CUSTODIAL)
pub async fn coordinate_multisig_exchange(
    coordinator: web::Data<Arc<EscrowCoordinator>>,
    escrow_id: web::Path<String>,
) -> Result<HttpResponse> {
    let result = coordinator.coordinate_multisig_exchange(&escrow_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
```

#### 1.3 Feature Flag
```rust
// common/src/lib.rs
pub const ENABLE_NONCUSTODIAL_MODE: bool = std::env::var("NONCUSTODIAL_MODE")
    .unwrap_or_else(|_| "false".to_string())
    .parse()
    .unwrap_or(false);
```

#### 1.4 Tests Phase 1
```bash
# Créer tests E2E dual mode
touch server/tests/escrow_dual_mode.rs
```

**Test:**
```rust
#[tokio::test]
#[ignore]
async fn test_dual_mode_noncustodial_flow() {
    // 1. Lancer 3 wallet-rpc locaux (buyer, seller, arbiter)
    // 2. Enregistrer avec /api/escrow/register-wallet
    // 3. Coordonner avec /api/escrow/coordinate-exchange
    // 4. Vérifier multisig créé LOCALEMENT (pas sur serveur)
}

#[tokio::test]
#[ignore]
async fn test_dual_mode_custodial_still_works() {
    // 1. Utiliser ancien flow (create_temporary_wallet)
    // 2. Vérifier fonctionne toujours
}
```

**Validation Phase 1:**
- [ ] EscrowCoordinator compile
- [ ] API register_client_wallet accessible
- [ ] Validation localhost strict fonctionne
- [ ] Test dual mode passe
- [ ] Ancien système fonctionne toujours

---

### 📋 Phase 2: Migration des Flux (5-7 jours)
**Objectif:** Migrer progressivement vers mode non-custodial

#### 2.1 Client CLI Non-Custodial
```bash
# Créer outil CLI pour clients
touch cli/src/noncustodial_wallet.rs
```

**Implementation:**
```rust
// cli/src/noncustodial_wallet.rs
pub struct NonCustodialClient {
    local_wallet_rpc: MoneroRpcClient,
    server_url: String,
    role: EscrowRole,
}

impl NonCustodialClient {
    /// Initialise escrow flow (NON-CUSTODIAL)
    pub async fn init_escrow(&self, escrow_id: &str) -> Result<()> {
        println!("🔐 Initializing non-custodial escrow...");

        // 1. Créer wallet local
        println!("📁 Creating local wallet...");
        self.local_wallet_rpc.create_wallet(
            &format!("escrow_{}", escrow_id),
            "",
        ).await?;

        // 2. Enregistrer avec serveur
        println!("📡 Registering with coordinator...");
        let config = ClientWalletConfig {
            user_id: "user123".to_string(),
            role: self.role.clone(),
            rpc_url: "http://127.0.0.1:18083".to_string(),
            wallet_filename: format!("escrow_{}", escrow_id),
        };

        self.register_with_server(escrow_id, config).await?;

        // 3. Attendre coordination
        println!("⏳ Waiting for other participants...");
        let exchange_result = self.wait_for_exchange(escrow_id).await?;

        // 4. Finaliser multisig LOCALEMENT
        println!("🔧 Finalizing multisig locally...");
        let multisig = MultisigManager::new(self.local_wallet_rpc.clone());
        multisig.make_multisig(2, exchange_result.received_infos).await?;

        println!("✅ Non-custodial escrow ready!");
        Ok(())
    }
}
```

#### 2.2 Documentation Utilisateur
```bash
# Guide utilisateur non-custodial
touch DOX/guides/NON-CUSTODIAL-USER-GUIDE.md
```

**Contenu:**
```markdown
# Guide Utilisateur Non-Custodial

## Prérequis
1. Installer monero-wallet-rpc localement
2. Lancer daemon: `monero-wallet-rpc --testnet --rpc-bind-port 18083 --disable-rpc-login`

## Utilisation
```bash
# Buyer
cargo run --bin cli -- noncustodial init-escrow \
    --escrow-id escrow_123 \
    --role buyer \
    --wallet-port 18083

# Seller
cargo run --bin cli -- noncustodial init-escrow \
    --escrow-id escrow_123 \
    --role seller \
    --wallet-port 18084

# Arbiter
cargo run --bin cli -- noncustodial init-escrow \
    --escrow-id escrow_123 \
    --role arbiter \
    --wallet-port 18085
```

**Validation Phase 2:**
- [ ] CLI noncustodial fonctionne
- [ ] Guide utilisateur clair
- [ ] Tests E2E avec 3 wallets locaux passent
- [ ] Coordination serveur fonctionne
- [ ] Aucune clé privée sur serveur

---

### 📋 Phase 3: Dépréciation Mode Custodial (3-4 jours)
**Objectif:** Marquer ancien code comme deprecated, encourager migration

#### 3.1 Warnings de Dépréciation
```rust
// server/src/wallet_manager.rs
#[deprecated(
    since = "0.3.0",
    note = "Use EscrowCoordinator with client wallets instead. This custodial mode will be removed in v0.4.0"
)]
pub async fn create_temporary_wallet(
    &self,
    escrow_id: &str,
    role: &str,
) -> Result<String> {
    tracing::warn!(
        "⚠️  DEPRECATED: create_temporary_wallet is custodial. Migrate to EscrowCoordinator."
    );
    // ... ancien code
}
```

#### 3.2 Migration Guide
```bash
touch DOX/guides/MIGRATION-TO-NONCUSTODIAL.md
```

**Validation Phase 3:**
- [ ] Warnings affichés
- [ ] Guide de migration publié
- [ ] Utilisateurs informés
- [ ] Nouveau mode par défaut

---

### 📋 Phase 4: Suppression Mode Custodial (2-3 jours)
**Objectif:** Supprimer complètement le code custodial

#### 4.1 Suppression Code
```bash
# Supprimer anciennes fonctions
git rm server/src/wallet_manager.rs  # ou refactor complet
git rm server/src/wallet_pool.rs
```

#### 4.2 Tests Finaux
```bash
# Vérifier AUCUN code custodial
./scripts/audit-noncustodial-final.sh
```

**Validation Phase 4:**
- [ ] Aucun create_temporary_wallet
- [ ] Aucun wallet sur serveur
- [ ] Tests E2E 100% noncustodial
- [ ] Audit sécurité passe

---

## Risques et Mitigations

### Risque 1: Breaking Changes
**Probabilité:** Moyenne
**Impact:** Élevé
**Mitigation:**
- Phase 1 dual mode garde compatibilité
- Feature flags permettent rollback
- Tests automatiques détectent regressions

### Risque 2: Complexité Utilisateur
**Probabilité:** Élevée
**Impact:** Moyen
**Mitigation:**
- CLI simplifié pour utilisateurs
- Guide utilisateur détaillé
- Scripts d'automatisation fournis

### Risque 3: Problèmes de Synchronisation
**Probabilité:** Moyenne
**Impact:** Élevé
**Mitigation:**
- États de coordination clairs
- Timeouts et retries implémentés
- Logs détaillés pour debug

### Risque 4: RPC Wallets Non Disponibles
**Probabilité:** Faible
**Impact:** Critique
**Mitigation:**
- Health checks réguliers
- Validation connexion avant coordination
- Fallback gracieux avec messages clairs

---

## Checklist de Validation

### Validation Sécurité
- [ ] `validate_localhost_strict()` utilisé partout
- [ ] Aucun wallet créé sur serveur
- [ ] Aucune clé privée stockée/loggée
- [ ] RPC URLs validées (127.0.0.1 uniquement)
- [ ] Threshold=2, Participants=3 validés
- [ ] 2 rounds export/import obligatoires

### Validation Fonctionnelle
- [ ] Escrow flow complet fonctionne
- [ ] Multisig 2-of-3 créé correctement
- [ ] Signatures coordonnées sans erreur
- [ ] États transitions valides
- [ ] Tests E2E passent

### Validation Performance
- [ ] Coordination <5 secondes
- [ ] Pas de timeout RPC
- [ ] Gestion concurrence wallets
- [ ] Mémoire stable (pas de leaks)

### Validation UX
- [ ] CLI intuitif
- [ ] Messages erreur clairs
- [ ] Documentation complète
- [ ] Logs utiles pour debug

---

## Timeline Estimée

| Phase | Durée | Milestone |
|-------|-------|-----------|
| Phase 1: Dual Mode | 4-6 jours | API non-custodiale fonctionnelle |
| Phase 2: Migration Flux | 5-7 jours | CLI et guide utilisateur |
| Phase 3: Dépréciation | 3-4 jours | Warnings et migration guide |
| Phase 4: Suppression | 2-3 jours | 100% non-custodial |
| **TOTAL** | **14-20 jours** | **Architecture Haveno-style** |

---

## Commandes Utiles

### Tester Mode Non-Custodial
```bash
# Lancer 3 wallets locaux
./scripts/start-noncustodial-wallets.sh

# Tester coordination
cargo test --package server --test escrow_noncustodial_e2e -- --ignored --nocapture

# Audit final
./scripts/audit-noncustodial-complete.sh
```

### Rollback Si Problème
```bash
# Désactiver mode noncustodial
export NONCUSTODIAL_MODE=false

# Revenir à branche stable
git checkout master

# Relancer serveur
cargo run --release --bin server
```

---

## Références

### Haveno DEX
- Repository: https://github.com/haveno-dex/haveno
- Architecture: Client-side wallets, server coordination only
- Validation: Strict threshold and participant checks

### Documentation Interne
- `CLAUDE.md` - Règles développement
- `DOX/reports/NON-CUSTODIAL-AUDIT-COMPLETE-2025-10-23.md` - Audit précédent (outdated)
- `wallet/src/multisig.rs` - Implémentation multisig actuelle

---

**Next Step:** Commencer Phase 1 - Créer `EscrowCoordinator` avec dual mode support.
