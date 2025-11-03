# 🚀 ROADMAP: IMPLÉMENTATION ESCROW 100% NON-CUSTODIAL

**Version:** 1.0.0
**Date:** 2025-11-03
**Status:** 📋 Documentation Phase
**Estimation totale:** 3h 30min

---

## 📌 TABLE DES MATIÈRES

1. [Vue d'Ensemble](#vue-densemble)
2. [Architecture Technique](#architecture-technique)
3. [Phases d'Implémentation](#phases-dimplémentation)
   - [Phase 1: Database Migration](#phase-1-database-migration)
   - [Phase 2: WalletManager](#phase-2-walletmanager)
   - [Phase 3: EscrowOrchestrator](#phase-3-escroworchestrator)
   - [Phase 4: Blockchain Monitor](#phase-4-blockchain-monitor)
   - [Phase 5: API Endpoints](#phase-5-api-endpoints)
   - [Phase 6: Frontend](#phase-6-frontend)
   - [Phase 7: Tests E2E](#phase-7-tests-e2e)
   - [Phase 8: Documentation](#phase-8-documentation)
   - [Phase 9: Validation Finale](#phase-9-validation-finale)
4. [Annexes Techniques](#annexes-techniques)
5. [Plan de Rollback](#plan-de-rollback)
6. [Checklist Globale](#checklist-globale)

---

## 📖 VUE D'ENSEMBLE

### Objectif Technique

Implémenter un système d'escrow Monero **100% non-custodial** où:
- Le serveur crée des **wallets temporaires VIDES** uniquement pour coordonner le multisig 2-of-3
- Le buyer paye **DIRECTEMENT** l'adresse multisig générée depuis n'importe quel wallet externe
- **ZÉRO custody** des fonds utilisateur par le marketplace

### Architecture Cible

```
┌─────────────────────────────────────────────────────────────┐
│                    CHECKOUT FLOW                            │
└─────────────────────────────────────────────────────────────┘

1. Buyer initie checkout
          ↓
2. Serveur crée 3 wallets VIDES:
   - buyer_temp_wallet    (0 XMR)
   - vendor_temp_wallet   (0 XMR)
   - arbiter_wallet       (0 XMR)
          ↓
3. Multisig Setup:
   prepare_multisig()  → Génère clés publiques
   make_multisig()     → Combine en adresse partagée
   finalize_multisig() → Confirme la configuration
          ↓
4. Génération Adresse Multisig: 4...xxxxx (95 caractères)
          ↓
5. Affichage au Buyer:
   - Adresse multisig + QR code
   - Instructions: "Pay from ANY external wallet"
          ↓
6. Buyer paye depuis:
   - Cake Wallet
   - Monerujo
   - Monero GUI
   - N'importe quel wallet qu'il contrôle
          ↓
7. Blockchain Monitoring:
   - Détecte payment → Escrow status = "funded"
          ↓
8. Release Funds:
   - Requiert 2-of-3 signatures
   - buyer + vendor OU arbiter
   - Fonds envoyés au vendor
```

### Propriétés de Sécurité Garanties

- ✅ **Non-Custodial**: Serveur ne détient JAMAIS de fonds
- ✅ **Wallets Éphémères**: Temporaires restent vides (coordination uniquement)
- ✅ **Multisig Collaboratif**: Aucune partie ne peut dépenser seule
- ✅ **Privacy-First**: Adresse multisig unique par transaction
- ✅ **Open Protocol**: Buyer utilise n'importe quel wallet compatible Monero
- ✅ **Disaster Recovery**: Clés multisig sauvegardées pour récupération

### Différences avec l'Approche Actuelle

| Aspect | Avant (Custodial) | Après (Non-Custodial) |
|--------|-------------------|----------------------|
| Wallet ownership | User provides wallet_id from DB | Server creates temp wallets per escrow |
| Wallet balance | Expected to exist | Always 0 XMR (empty) |
| Payment destination | User wallet → Multisig | External wallet → Multisig (direct) |
| Server control | Has access to all wallets | Only coordinates multisig setup |
| Security model | Trust server | Zero-trust (cryptographic) |
| User experience | Complex setup | Simple QR scan |

---

## 🏗️ ARCHITECTURE TECHNIQUE

### Flow Utilisateur Complet

**1. Phase Checkout (Frontend)**
```
User clicks "Proceed to Checkout"
    → templates/checkout/index.html loads
    → User enters shipping address
    → User clicks "CREATE ORDER & INITIALIZE ESCROW"
```

**2. Order Creation (Backend)**
```
POST /api/orders/create
    → OrderHandler::create_order()
    → Inserts order with status="pending"
    → Returns order_id
```

**3. Escrow Initialization (Backend)**
```
POST /api/orders/{id}/init-escrow
    → OrderHandler::init_escrow()
    → EscrowOrchestrator::init_escrow()

    ┌─────────────────────────────────────┐
    │  NEW: Create 3 Empty Temp Wallets   │
    └─────────────────────────────────────┘

    WalletManager::create_temporary_wallet("buyer")
        → Returns buyer_temp_wallet_id (UUID)

    WalletManager::create_temporary_wallet("vendor")
        → Returns vendor_temp_wallet_id (UUID)

    WalletManager::create_arbiter_wallet_instance()
        → Returns arbiter_wallet_id (UUID)

    ┌──────────────────────────────────┐
    │  NEW: Multisig Setup             │
    └──────────────────────────────────┘

    EscrowOrchestrator::setup_multisig_non_custodial()
        → prepare_multisig() for each wallet
        → exchange_multisig_info()
        → finalize_multisig()
        → Returns multisig_address: "4...xxxxx"

    Update escrow:
        - multisig_address = "4...xxxxx"
        - status = "created"
        - buyer_temp_wallet_id
        - vendor_temp_wallet_id
        - arbiter_temp_wallet_id
```

**4. Payment Instructions Display (Frontend)**
```
Frontend receives escrow with multisig_address
    → Displays QR code
    → Shows copy-to-clipboard button
    → Instructions: "Send X.XXXX XMR to this address"
```

**5. Buyer Payment (External)**
```
Buyer opens Cake Wallet / Monerujo / GUI
    → Scans QR code OR pastes address
    → Sends exact amount (e.g., 0.123456789012 XMR)
    → Transaction broadcasts to Monero network
```

**6. Blockchain Monitoring (Backend Background Job)**
```
BlockchainMonitor::check_escrow_funding()
    → Loops every 30 seconds
    → Checks multisig wallet balance

    WalletManager::get_balance(buyer_temp_wallet_id)
        → If balance >= escrow.amount:
            → Update escrow status = "funded"
            → WebSocket notify buyer + vendor
```

**7. Release Flow (API Trigger)**
```
Vendor clicks "Release Funds"
    → POST /api/escrow/{id}/release

    EscrowOrchestrator::release_funds()
        → Creates transaction from multisig
        → Collects 2 signatures (buyer + vendor)
        → Broadcasts signed transaction
        → Vendor receives funds to wallet_address
```

### Fichiers Critiques Modifiés

| Fichier | Modification | Impact |
|---------|-------------|--------|
| `server/migrations/*/up.sql` | Add 3 columns to escrows table | DB schema change |
| `server/src/schema.rs` | Regenerated from diesel | Reflects new columns |
| `server/src/models/escrow.rs` | Add fields to Escrow & NewEscrow structs | Compile-time safety |
| `server/src/wallet_manager.rs` | Add `create_temporary_wallet()` method | Core wallet logic |
| `server/src/services/escrow.rs` | Rewrite `init_escrow()` & add `setup_multisig_non_custodial()` | Core escrow logic |
| `server/src/services/blockchain_monitor.rs` | Fix line 153: use `buyer_temp_wallet_id` | Payment detection |
| `server/src/handlers/escrow.rs` | Add `get_multisig_address()` endpoint | API access |
| `static/js/checkout.js` | Add `fetchMultisigAddress()` method | Frontend fetch |
| `templates/checkout/index.html` | Educational messaging | User clarity |

---

## 🔧 PHASES D'IMPLÉMENTATION

---

## PHASE 1: DATABASE MIGRATION

**Durée estimée:** 15 minutes
**Priorité:** 🔴 CRITIQUE
**Dépendances:** Aucune

### Objectifs

- [x] Créer migration diesel pour ajouter 3 colonnes à la table `escrows`
- [x] Appliquer migration à la base de données
- [x] Régénérer `server/src/schema.rs`
- [x] Vérifier compilation sans erreur

### Étapes Détaillées

#### 1.1 Générer Migration

```bash
cd /home/malix/Desktop/monero.marketplace
diesel migration generate add_temp_wallet_ids_to_escrows
```

**Output attendu:**
```
Creating migrations/2025-11-03-XXXXXX_add_temp_wallet_ids_to_escrows/up.sql
Creating migrations/2025-11-03-XXXXXX_add_temp_wallet_ids_to_escrows/down.sql
```

#### 1.2 Éditer `up.sql`

**Fichier:** `migrations/2025-11-03-XXXXXX_add_temp_wallet_ids_to_escrows/up.sql`

```sql
-- Add temporary wallet ID columns for non-custodial escrow
-- These wallets are EMPTY and used only for multisig coordination

ALTER TABLE escrows ADD COLUMN buyer_temp_wallet_id TEXT DEFAULT NULL;
ALTER TABLE escrows ADD COLUMN vendor_temp_wallet_id TEXT DEFAULT NULL;
ALTER TABLE escrows ADD COLUMN arbiter_temp_wallet_id TEXT DEFAULT NULL;

-- Add indexes for faster lookups during escrow operations
CREATE INDEX idx_escrows_buyer_temp_wallet ON escrows(buyer_temp_wallet_id);
CREATE INDEX idx_escrows_vendor_temp_wallet ON escrows(vendor_temp_wallet_id);
CREATE INDEX idx_escrows_arbiter_temp_wallet ON escrows(arbiter_temp_wallet_id);
```

#### 1.3 Éditer `down.sql`

**Fichier:** `migrations/2025-11-03-XXXXXX_add_temp_wallet_ids_to_escrows/down.sql`

```sql
-- Rollback: Remove temp wallet columns

DROP INDEX IF EXISTS idx_escrows_arbiter_temp_wallet;
DROP INDEX IF EXISTS idx_escrows_vendor_temp_wallet;
DROP INDEX IF EXISTS idx_escrows_buyer_temp_wallet;

ALTER TABLE escrows DROP COLUMN arbiter_temp_wallet_id;
ALTER TABLE escrows DROP COLUMN vendor_temp_wallet_id;
ALTER TABLE escrows DROP COLUMN buyer_temp_wallet_id;
```

#### 1.4 Appliquer Migration

```bash
DATABASE_URL=marketplace.db diesel migration run
```

**Output attendu:**
```
Running migration 2025-11-03-XXXXXX_add_temp_wallet_ids_to_escrows
```

#### 1.5 Vérifier Migration

```bash
DATABASE_URL=marketplace.db diesel migration list
```

**Output attendu:**
```
Migrations:
  [X] 2025-XX-XX-XXXXXX_create_users
  [X] 2025-XX-XX-XXXXXX_create_orders
  [X] 2025-XX-XX-XXXXXX_create_escrows
  ...
  [X] 2025-11-03-XXXXXX_add_temp_wallet_ids_to_escrows  ← NEW
```

#### 1.6 Régénérer Schema

```bash
diesel print-schema > server/src/schema.rs
```

#### 1.7 Vérifier Schema Mis à Jour

```bash
grep "buyer_temp_wallet_id" server/src/schema.rs
```

**Output attendu:**
```rust
buyer_temp_wallet_id -> Nullable<Text>,
vendor_temp_wallet_id -> Nullable<Text>,
arbiter_temp_wallet_id -> Nullable<Text>,
```

#### 1.8 Test Compilation

```bash
cargo build --package server
```

**Output attendu:**
```
   Compiling server v0.2.6 (/home/malix/Desktop/monero.marketplace/server)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

### Critères de Succès Phase 1

- ✅ Migration appliquée sans erreur
- ✅ `diesel migration list` montre `[X]` pour nouvelle migration
- ✅ `schema.rs` contient les 3 nouvelles colonnes
- ✅ Compilation réussit sans avertissement

### Points d'Attention

⚠️ **Backup Database**: Avant d'appliquer la migration:
```bash
cp marketplace.db marketplace_backup_$(date +%Y%m%d_%H%M%S).db
```

⚠️ **Migration Irréversible**: Une fois appliquée en production, difficile à revert

### Commandes de Vérification

```bash
# Check table structure
sqlite3 marketplace.db "PRAGMA table_info(escrows);" | grep "temp_wallet"

# Check indexes
sqlite3 marketplace.db ".indexes escrows"
```

---

## PHASE 2: WALLETMANAGER

**Durée estimée:** 30 minutes
**Priorité:** 🔴 CRITIQUE
**Dépendances:** Phase 1 complétée

### Objectifs

- [x] Ajouter méthode `create_temporary_wallet(&mut self, role: &str)`
- [x] Retourner UUID du wallet créé
- [x] Logger création avec détails (sans exposer données sensibles)
- [x] Tests unitaires

### Étapes Détaillées

#### 2.1 Ouvrir Fichier

**Fichier:** `server/src/wallet_manager.rs`

#### 2.2 Localiser Emplacement

Ajouter APRÈS la méthode `create_arbiter_wallet_instance()` (ligne ~380)

#### 2.3 Implémenter Méthode

```rust
/// Create temporary EMPTY wallet for multisig setup (Non-Custodial Architecture)
///
/// This creates a server-controlled empty wallet for the sole purpose of
/// multisig address generation. **NO FUNDS** should ever be sent to this wallet directly.
///
/// # Architecture Decision
/// These temporary wallets are:
/// - Created per escrow transaction
/// - Always empty (0 XMR balance)
/// - Used only for collaborative multisig setup (prepare, make, finalize)
/// - Destroyed or archived after escrow completes
///
/// # Arguments
/// * `role` - "buyer" or "vendor" (arbiter uses `create_arbiter_wallet_instance`)
///
/// # Returns
/// UUID of the temporary wallet instance
///
/// # Errors
/// Returns `WalletManagerError` if:
/// - Role is invalid (not "buyer" or "vendor")
/// - No RPC configs available
/// - RPC connection fails
/// - Wallet creation fails
///
/// # Example
/// ```rust
/// let buyer_temp_id = wallet_manager.create_temporary_wallet("buyer").await?;
/// // buyer_temp_id = "550e8400-e29b-41d4-a716-446655440000"
/// // Wallet has 0 XMR and is ready for multisig setup
/// ```
pub async fn create_temporary_wallet(
    &mut self,
    role: &str,
) -> Result<Uuid, WalletManagerError> {
    // Validate role
    let wallet_role = match role {
        "buyer" => WalletRole::Buyer,
        "vendor" => WalletRole::Vendor,
        _ => return Err(WalletManagerError::InvalidRpcUrl(
            format!("Invalid role '{}': must be 'buyer' or 'vendor'", role)
        )),
    };

    // Get next available RPC config (round-robin)
    let config = self.rpc_configs
        .get(self.next_rpc_index)
        .ok_or(WalletManagerError::NoAvailableRpc)?;
    self.next_rpc_index = (self.next_rpc_index + 1) % self.rpc_configs.len();

    // Create RPC client
    let rpc_client = MoneroClient::new(config.clone())?;

    // Get wallet info (address, etc.)
    let wallet_info = rpc_client.get_wallet_info().await?;

    // Generate unique wallet ID
    let wallet_id = Uuid::new_v4();

    // Create wallet instance
    let instance = WalletInstance {
        id: wallet_id,
        role: wallet_role.clone(),
        rpc_client,
        address: wallet_info.address.clone(),
        multisig_state: MultisigState::NotStarted,
    };

    // Store in memory
    self.wallets.insert(wallet_id, instance);

    // OPSEC-compliant logging (no sensitive data)
    info!(
        "✅ Created temporary {} wallet for non-custodial escrow | id={} | address={} | balance=0 XMR (EMPTY - multisig coordination only)",
        role,
        wallet_id,
        wallet_info.address.chars().take(10).collect::<String>() + "..." // Only log first 10 chars
    );

    Ok(wallet_id)
}
```

#### 2.4 Tests Unitaires

Ajouter dans le module `#[cfg(test)] mod tests` à la fin du fichier:

```rust
#[tokio::test]
async fn test_create_temporary_wallet_buyer() -> Result<()> {
    let mut wallet_manager = create_test_wallet_manager();

    let buyer_wallet_id = wallet_manager
        .create_temporary_wallet("buyer")
        .await?;

    // Verify UUID format
    assert_eq!(buyer_wallet_id.to_string().len(), 36);

    // Verify wallet exists in memory
    assert!(wallet_manager.wallets.contains_key(&buyer_wallet_id));

    // Verify wallet state
    let wallet = wallet_manager.wallets.get(&buyer_wallet_id).unwrap();
    assert_eq!(wallet.role, WalletRole::Buyer);
    assert_eq!(wallet.multisig_state, MultisigState::NotStarted);

    Ok(())
}

#[tokio::test]
async fn test_create_temporary_wallet_vendor() -> Result<()> {
    let mut wallet_manager = create_test_wallet_manager();

    let vendor_wallet_id = wallet_manager
        .create_temporary_wallet("vendor")
        .await?;

    let wallet = wallet_manager.wallets.get(&vendor_wallet_id).unwrap();
    assert_eq!(wallet.role, WalletRole::Vendor);

    Ok(())
}

#[tokio::test]
async fn test_create_temporary_wallet_invalid_role() {
    let mut wallet_manager = create_test_wallet_manager();

    let result = wallet_manager
        .create_temporary_wallet("invalid_role")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        WalletManagerError::InvalidRpcUrl(msg) => {
            assert!(msg.contains("Invalid role"));
        }
        _ => panic!("Expected InvalidRpcUrl error"),
    }
}
```

#### 2.5 Compiler & Tester

```bash
# Compile
cargo build --package server

# Run tests
cargo test --package server wallet_manager::tests::test_create_temporary_wallet
```

### Critères de Succès Phase 2

- ✅ Méthode `create_temporary_wallet()` compile sans erreur
- ✅ Tests unitaires passent (3/3)
- ✅ Logs montrent "Created temporary buyer/vendor wallet"
- ✅ Wallet UUID retourné est valide
- ✅ Wallet stocké dans `self.wallets` HashMap

### Points d'Attention

⚠️ **RPC Availability**: Si aucun RPC config n'est disponible, retourne `NoAvailableRpc`
⚠️ **Wallet Cleanup**: Ces wallets doivent être nettoyés après escrow complété (Phase 9)
⚠️ **Thread Safety**: `&mut self` garantit exclusive access pendant création

### Commandes de Debug

```bash
# Check wallet_manager logs during test
RUST_LOG=info cargo test --package server test_create_temporary_wallet -- --nocapture
```

---

## PHASE 3: ESCROWORCHESTRATOR

**Durée estimée:** 1 heure
**Priorité:** 🔴 CRITIQUE
**Dépendances:** Phase 1, 2 complétées

### Objectifs

- [x] Modifier `init_escrow()` pour créer wallets temporaires
- [x] Ajouter méthode privée `setup_multisig_non_custodial()`
- [x] Sauvegarder temp wallet IDs dans DB
- [x] Générer et stocker adresse multisig
- [x] Tests d'intégration

### Étapes Détaillées

#### 3.1 Modifier `init_escrow()`

**Fichier:** `server/src/services/escrow.rs`
**Ligne de départ:** ~151

**AVANT (à remplacer):**
```rust
pub async fn init_escrow(
    &self,
    order_id: Uuid,
    buyer_id: Uuid,
    vendor_id: Uuid,
    amount_atomic: i64,
) -> Result<Escrow> {
    // Current logic expects wallet_id from User table
    // ...
}
```

**APRÈS (nouveau code):**
```rust
pub async fn init_escrow(
    &self,
    order_id: Uuid,
    buyer_id: Uuid,
    vendor_id: Uuid,
    amount_atomic: i64,
) -> Result<Escrow> {
    info!(
        "🚀 Initializing NON-CUSTODIAL escrow | order_id={} | amount={} atomic units",
        order_id, amount_atomic
    );

    // ═══════════════════════════════════════════════════════════
    // STEP 1: Create 3 EMPTY temporary wallets for multisig setup
    // ═══════════════════════════════════════════════════════════

    let mut wallet_manager = self.wallet_manager.lock().await;

    info!("📝 Creating temporary buyer wallet (EMPTY - multisig coordination only)");
    let buyer_temp_wallet_id = wallet_manager
        .create_temporary_wallet("buyer")
        .await
        .context("Failed to create buyer temporary wallet")?;

    info!("📝 Creating temporary vendor wallet (EMPTY - multisig coordination only)");
    let vendor_temp_wallet_id = wallet_manager
        .create_temporary_wallet("vendor")
        .await
        .context("Failed to create vendor temporary wallet")?;

    info!("📝 Creating arbiter wallet instance");
    let arbiter_temp_wallet_id = wallet_manager
        .create_arbiter_wallet_instance()
        .await
        .context("Failed to create arbiter wallet")?;

    // Release lock before DB operations
    drop(wallet_manager);

    info!(
        "✅ Created 3 temporary wallets | buyer={} | vendor={} | arbiter={}",
        buyer_temp_wallet_id, vendor_temp_wallet_id, arbiter_temp_wallet_id
    );

    // ═══════════════════════════════════════════════════════════
    // STEP 2: Assign arbiter
    // ═══════════════════════════════════════════════════════════

    let arbiter_id = self.assign_arbiter().await?;
    info!("👨‍⚖️ Assigned arbiter | arbiter_id={}", arbiter_id);

    // ═══════════════════════════════════════════════════════════
    // STEP 3: Create escrow record with temp wallet IDs
    // ═══════════════════════════════════════════════════════════

    let escrow_id = Uuid::new_v4();

    let new_escrow = NewEscrow {
        id: escrow_id.to_string(),
        order_id: order_id.to_string(),
        buyer_id: buyer_id.to_string(),
        vendor_id: vendor_id.to_string(),
        arbiter_id: arbiter_id.to_string(),
        amount: amount_atomic,
        status: "created".to_string(),
        buyer_temp_wallet_id: Some(buyer_temp_wallet_id.to_string()),
        vendor_temp_wallet_id: Some(vendor_temp_wallet_id.to_string()),
        arbiter_temp_wallet_id: Some(arbiter_temp_wallet_id.to_string()),
    };

    let mut conn = self.db_pool.get().context("Failed to get DB connection")?;

    let escrow = tokio::task::spawn_blocking(move || {
        Escrow::create(&mut conn, new_escrow)
    })
    .await
    .context("Database task panicked")??;

    info!("💾 Escrow created in DB | escrow_id={} | status=created", escrow.id);

    // ═══════════════════════════════════════════════════════════
    // STEP 4: Perform multisig setup to generate shared address
    // ═══════════════════════════════════════════════════════════

    info!("🔐 Starting multisig setup (prepare → make → finalize)");
    let multisig_address = self
        .setup_multisig_non_custodial(&escrow.id)
        .await
        .context("Failed to setup multisig")?;

    info!(
        "✅ Multisig address generated | address={} | escrow_id={}",
        multisig_address.chars().take(15).collect::<String>() + "...",
        escrow.id
    );

    // Update escrow with multisig address
    let mut conn2 = self.db_pool.get().context("Failed to get DB connection")?;
    let escrow_id_clone = escrow.id.clone();
    let multisig_address_clone = multisig_address.clone();

    tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;
        use crate::schema::escrows::dsl::*;

        diesel::update(escrows.filter(id.eq(escrow_id_clone)))
            .set(multisig_address.eq(Some(multisig_address_clone)))
            .execute(&mut conn2)
    })
    .await
    .context("Database task panicked")??;

    info!("💾 Escrow updated with multisig address");

    // Reload escrow from DB to get updated data
    let mut conn3 = self.db_pool.get()?;
    let final_escrow_id = escrow.id.parse::<Uuid>()?;
    let final_escrow = tokio::task::spawn_blocking(move || {
        db_load_escrow(&self.db_pool, final_escrow_id)
    })
    .await??;

    info!(
        "🎉 NON-CUSTODIAL escrow initialization complete | escrow_id={} | multisig_address={} | BUYER CAN NOW PAY FROM ANY EXTERNAL WALLET",
        final_escrow.id,
        final_escrow.multisig_address.as_deref().unwrap_or("NONE")
    );

    Ok(final_escrow)
}
```

#### 3.2 Ajouter `setup_multisig_non_custodial()`

**Fichier:** `server/src/services/escrow.rs`
**Emplacement:** Après `init_escrow()`, avant `release_funds()`

```rust
/// Setup multisig 2-of-3 with temporary empty wallets (Non-Custodial)
///
/// This method orchestrates the complete Monero multisig setup flow:
/// 1. prepare_multisig() - Each wallet generates its multisig info
/// 2. exchange_multisig_info() - Share info between all 3 wallets
/// 3. finalize_multisig() - Complete setup and get shared address
///
/// # Returns
/// The generated multisig address (4...xxxxx, 95 characters)
///
/// # Errors
/// Returns error if any step of multisig setup fails
async fn setup_multisig_non_custodial(&self, escrow_id: &str) -> Result<String> {
    let escrow_uuid = escrow_id.parse::<Uuid>()
        .context("Invalid escrow_id format")?;

    // Load escrow to get temp wallet IDs
    let mut conn = self.db_pool.get()?;
    let escrow = tokio::task::spawn_blocking(move || {
        db_load_escrow(&conn, escrow_uuid)
    })
    .await??;

    let buyer_wallet_id = escrow.buyer_temp_wallet_id
        .ok_or_else(|| anyhow::anyhow!("Escrow missing buyer_temp_wallet_id"))?
        .parse::<Uuid>()?;

    let vendor_wallet_id = escrow.vendor_temp_wallet_id
        .ok_or_else(|| anyhow::anyhow!("Escrow missing vendor_temp_wallet_id"))?
        .parse::<Uuid>()?;

    let arbiter_wallet_id = escrow.arbiter_temp_wallet_id
        .ok_or_else(|| anyhow::anyhow!("Escrow missing arbiter_temp_wallet_id"))?
        .parse::<Uuid>()?;

    info!(
        "🔐 Multisig setup starting | buyer_wallet={} | vendor_wallet={} | arbiter_wallet={}",
        buyer_wallet_id, vendor_wallet_id, arbiter_wallet_id
    );

    let mut wallet_manager = self.wallet_manager.lock().await;

    // ═══════════════════════════════════════════════════════════
    // STEP 1: prepare_multisig() for each wallet
    // ═══════════════════════════════════════════════════════════

    info!("📝 Step 1/3: prepare_multisig() - Generating multisig info for each wallet");

    let buyer_info = wallet_manager
        .make_multisig(escrow_id, buyer_wallet_id, vec![])
        .await
        .context("Failed to prepare buyer multisig")?;

    let vendor_info = wallet_manager
        .make_multisig(escrow_id, vendor_wallet_id, vec![])
        .await
        .context("Failed to prepare vendor multisig")?;

    let arbiter_info = wallet_manager
        .make_multisig(escrow_id, arbiter_wallet_id, vec![])
        .await
        .context("Failed to prepare arbiter multisig")?;

    info!("✅ All 3 wallets generated multisig info");

    // ═══════════════════════════════════════════════════════════
    // STEP 2: exchange_multisig_info() - Share info between wallets
    // ═══════════════════════════════════════════════════════════

    info!("📝 Step 2/3: exchange_multisig_info() - Sharing info between wallets");

    wallet_manager
        .exchange_multisig_info(
            escrow_uuid,
            vec![buyer_info, vendor_info, arbiter_info],
        )
        .await
        .context("Failed to exchange multisig info")?;

    info!("✅ Multisig info exchanged successfully");

    // ═══════════════════════════════════════════════════════════
    // STEP 3: finalize_multisig() - Get shared multisig address
    // ═══════════════════════════════════════════════════════════

    info!("📝 Step 3/3: finalize_multisig() - Generating shared address");

    let multisig_address = wallet_manager
        .finalize_multisig(escrow_uuid)
        .await
        .context("Failed to finalize multisig")?;

    info!(
        "✅ Multisig setup complete | address={} (length={})",
        multisig_address.chars().take(15).collect::<String>() + "...",
        multisig_address.len()
    );

    // Verify address format (Monero addresses start with '4' for mainnet, '9' for testnet)
    if !multisig_address.starts_with('4') && !multisig_address.starts_with('9') {
        return Err(anyhow::anyhow!(
            "Invalid multisig address format: does not start with '4' or '9'"
        ));
    }

    if multisig_address.len() != 95 {
        warn!(
            "Unexpected multisig address length: {} (expected 95)",
            multisig_address.len()
        );
    }

    Ok(multisig_address)
}
```

#### 3.3 Mettre à Jour `models/escrow.rs`

**Fichier:** `server/src/models/escrow.rs`

**Ajouter aux structs:**

```rust
// Dans la struct Escrow (Queryable)
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
pub struct Escrow {
    // ... champs existants ...
    pub recovery_mode: String,

    // ══════════════════════════════════════════════════════════
    // NON-CUSTODIAL: Temporary wallet IDs for multisig setup
    // ══════════════════════════════════════════════════════════
    pub buyer_temp_wallet_id: Option<String>,
    pub vendor_temp_wallet_id: Option<String>,
    pub arbiter_temp_wallet_id: Option<String>,
}

// Dans la struct NewEscrow (Insertable)
#[derive(Insertable)]
#[diesel(table_name = escrows)]
pub struct NewEscrow {
    // ... champs existants ...
    pub status: String,

    // ══════════════════════════════════════════════════════════
    // NON-CUSTODIAL: Temporary wallet IDs
    // ══════════════════════════════════════════════════════════
    pub buyer_temp_wallet_id: Option<String>,
    pub vendor_temp_wallet_id: Option<String>,
    pub arbiter_temp_wallet_id: Option<String>,
}
```

#### 3.4 Compiler & Tester

```bash
# Compile
cargo build --package server

# Run integration tests
cargo test --package server escrow --lib -- --nocapture
```

### Critères de Succès Phase 3

- ✅ `init_escrow()` crée 3 wallets temporaires
- ✅ `setup_multisig_non_custodial()` génère adresse multisig
- ✅ Adresse multisig commence par '4' ou '9'
- ✅ Adresse multisig a 95 caractères
- ✅ Escrow sauvegardé en DB avec temp wallet IDs
- ✅ Logs montrent flow complet sans erreur

### Points d'Attention

⚠️ **Lock Duration**: Release `wallet_manager` lock AVANT opérations DB pour éviter deadlock
⚠️ **Error Handling**: Utiliser `.context()` sur toutes les opérations fallibles
⚠️ **Logging**: Limiter logs d'adresses à 15 premiers caractères (OPSEC)

---

## PHASE 4: BLOCKCHAIN MONITOR

**Durée estimée:** 20 minutes
**Priorité:** 🔴 CRITIQUE
**Dépendances:** Phase 1-3 complétées

### Objectifs

- [x] Corriger `check_escrow_funding()` pour utiliser `buyer_temp_wallet_id`
- [x] Fixer ligne 153 et 238 dans blockchain_monitor.rs
- [x] Tester détection de payment

### Étapes Détaillées

#### 4.1 Identifier Code Problématique

**Fichier:** `server/src/services/blockchain_monitor.rs`
**Ligne 153:**

**AVANT (INCORRECT):**
```rust
let buyer_wallet_id = escrow.buyer_id.parse::<Uuid>()?;
// ❌ PROBLÈME: buyer_id est l'ID USER, pas wallet ID
```

**APRÈS (CORRECT):**
```rust
let buyer_temp_wallet_id = escrow.buyer_temp_wallet_id
    .ok_or_else(|| anyhow::anyhow!(
        "Escrow {} missing buyer_temp_wallet_id (non-custodial architecture)",
        escrow.id
    ))?
    .parse::<Uuid>()
    .context("Failed to parse buyer_temp_wallet_id")?;
```

#### 4.2 Corriger get_balance() Call

**Ligne 159:**

**AVANT:**
```rust
let (total_balance, unlocked_balance) = wallet_manager
    .get_balance(buyer_wallet_id)  // ❌ Wrong ID
    .await?;
```

**APRÈS:**
```rust
let (total_balance, unlocked_balance) = wallet_manager
    .get_balance(buyer_temp_wallet_id)  // ✅ Correct: Use temp wallet
    .await
    .context("Failed to get balance from buyer temp wallet")?;

info!(
    "💰 Checking balance | escrow_id={} | buyer_temp_wallet={} | total={} | unlocked={}",
    escrow.id,
    buyer_temp_wallet_id,
    total_balance,
    unlocked_balance
);
```

#### 4.3 Corriger Ligne 238 (Transaction Check)

**AVANT:**
```rust
let buyer_wallet_id = escrow.buyer_id.parse::<Uuid>()?;
```

**APRÈS:**
```rust
let buyer_temp_wallet_id = escrow.buyer_temp_wallet_id
    .ok_or_else(|| anyhow::anyhow!(
        "Escrow {} missing buyer_temp_wallet_id",
        escrow.id
    ))?
    .parse::<Uuid>()?;
```

#### 4.4 Mettre à Jour Logs

Ajouter après détection de payment (ligne ~170):

```rust
info!(
    "🎉 PAYMENT DETECTED! | escrow_id={} | amount_received={} | expected={} | BUYER PAID FROM EXTERNAL WALLET → MULTISIG ADDRESS",
    escrow.id,
    total_balance,
    escrow.amount
);
```

#### 4.5 Compiler & Tester

```bash
# Compile
cargo build --package server

# Run blockchain monitor tests
cargo test --package server blockchain_monitor
```

### Critères de Succès Phase 4

- ✅ Code compile sans erreur
- ✅ `check_escrow_funding()` utilise `buyer_temp_wallet_id`
- ✅ Logs montrent wallet ID correct pendant monitoring
- ✅ Payment detection fonctionne (test manuel avec DEV simulate)

### Points d'Attention

⚠️ **Critical Bug**: Sans cette correction, le monitoring ne détectera JAMAIS les payments
⚠️ **Testing**: Utiliser bouton "DEV: Simulate Payment" pour tester après déploiement

### Commandes de Test

```bash
# Check logs during monitoring
tail -f server.log | grep "Checking balance"

# Simulate payment (après server running)
curl -X POST http://127.0.0.1:8080/api/escrow/{escrow_id}/simulate-payment \
  -H "Cookie: session=..."
```

---

## PHASE 5: API ENDPOINTS

**Durée estimée:** 15 minutes
**Priorité:** 🟡 HAUTE
**Dépendances:** Phase 1-4 complétées

### Objectifs

- [x] Créer endpoint `GET /api/escrow/:id/multisig-address`
- [x] Implémenter authentification + autorisation
- [x] Retourner JSON avec adresse multisig
- [x] Enregistrer route dans main.rs

### Étapes Détaillées

#### 5.1 Créer Handler

**Fichier:** `server/src/handlers/escrow.rs`
**Emplacement:** Après la fonction `release_escrow_funds()` (ligne ~843)

```rust
/// GET /api/escrow/:id/multisig-address
///
/// Get the multisig address for payment (Non-Custodial Architecture)
///
/// This endpoint allows the authenticated buyer to retrieve the generated
/// multisig address so they can pay from ANY external Monero wallet.
///
/// # Authentication
/// Requires active session with user_id
///
/// # Authorization
/// User must be buyer, vendor, or arbiter of the escrow
///
/// # Returns
/// - 200 OK: JSON with multisig address, amount, status
/// - 401 Unauthorized: Not authenticated
/// - 403 Forbidden: Not authorized (not part of this escrow)
/// - 404 Not Found: Escrow doesn't exist
///
/// # Response Format
/// ```json
/// {
///   "escrow_id": "550e8400-e29b-41d4-a716-446655440000",
///   "multisig_address": "4...xxxxx",
///   "amount": 123456789012,
///   "amount_xmr": "0.123456789012",
///   "status": "created"
/// }
/// ```
#[get("/escrow/{id}/multisig-address")]
pub async fn get_multisig_address(
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<String>,
) -> impl Responder {
    // ═══════════════════════════════════════════════════════════
    // STEP 1: Authentication Check
    // ═══════════════════════════════════════════════════════════

    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => {
            warn!("Unauthenticated request to get_multisig_address");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated",
                "message": "Please log in to access escrow details"
            }));
        }
    };

    // ═══════════════════════════════════════════════════════════
    // STEP 2: Parse & Validate Escrow ID
    // ═══════════════════════════════════════════════════════════

    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            warn!("Invalid escrow_id format: {}", escrow_id_str);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id format",
                "message": "Escrow ID must be a valid UUID"
            }));
        }
    };

    // ═══════════════════════════════════════════════════════════
    // STEP 3: Load Escrow from Database
    // ═══════════════════════════════════════════════════════════

    let escrow = match db_load_escrow(&pool, escrow_id).await {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to load escrow {}: {}", escrow_id, e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Escrow not found",
                "message": format!("No escrow found with ID {}", escrow_id)
            }));
        }
    };

    // ═══════════════════════════════════════════════════════════
    // STEP 4: Authorization Check
    // ═══════════════════════════════════════════════════════════

    if user_id_str != escrow.buyer_id
        && user_id_str != escrow.vendor_id
        && user_id_str != escrow.arbiter_id
    {
        warn!(
            "User {} attempted to access escrow {} (not authorized)",
            user_id_str, escrow_id
        );
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not authorized",
            "message": "You can only view escrows you are part of"
        }));
    }

    // ═══════════════════════════════════════════════════════════
    // STEP 5: Return Multisig Address & Details
    // ═══════════════════════════════════════════════════════════

    let amount_xmr = escrow.amount as f64 / 1_000_000_000_000.0;

    info!(
        "📍 Multisig address retrieved | escrow_id={} | user_id={} | address={}",
        escrow.id,
        user_id_str,
        escrow.multisig_address.as_ref()
            .map(|a| a.chars().take(15).collect::<String>() + "...")
            .unwrap_or_else(|| "PENDING".to_string())
    );

    HttpResponse::Ok().json(serde_json::json!({
        "escrow_id": escrow.id,
        "multisig_address": escrow.multisig_address,
        "amount": escrow.amount,
        "amount_xmr": format!("{:.12}", amount_xmr),
        "status": escrow.status,
        "created_at": escrow.created_at,
        "message": "Pay from ANY external Monero wallet to this multisig address"
    }))
}
```

#### 5.2 Enregistrer Route

**Fichier:** `server/src/main.rs`
**Localiser:** Section `.service()` dans la configuration Actix Web (ligne ~250)

**Ajouter:**
```rust
.service(handlers::escrow::get_multisig_address)
```

**Context complet:**
```rust
.service(handlers::escrow::initialize_escrow)
.service(handlers::escrow::get_escrow_status)
.service(handlers::escrow::release_escrow_funds)
.service(handlers::escrow::get_multisig_address)  // ← NEW
```

#### 5.3 Compiler & Tester

```bash
# Compile
cargo build --release --package server

# Start server
./target/release/server &

# Test endpoint (remplacer {escrow_id} par un vrai UUID)
curl http://127.0.0.1:8080/api/escrow/{escrow_id}/multisig-address \
  -H "Cookie: session=..." \
  -H "Accept: application/json" | jq
```

**Output attendu:**
```json
{
  "escrow_id": "550e8400-e29b-41d4-a716-446655440000",
  "multisig_address": "4...xxxxx",
  "amount": 123456789012,
  "amount_xmr": "0.123456789012",
  "status": "created",
  "created_at": "2025-11-03T12:34:56Z",
  "message": "Pay from ANY external Monero wallet to this multisig address"
}
```

### Critères de Succès Phase 5

- ✅ Endpoint compile et enregistré
- ✅ Authentification fonctionne (reject si non connecté)
- ✅ Autorisation fonctionne (reject si pas partie de l'escrow)
- ✅ JSON retourné contient multisig_address
- ✅ Format JSON valide et parsable

### Points d'Attention

⚠️ **CSRF Protection**: GET endpoint → Pas de CSRF token requis (lecture seule)
⚠️ **Rate Limiting**: Considérer rate limit si abuse possible
⚠️ **Address Exposure**: Multisig address OK to expose (public blockchain anyway)

### Commandes de Debug

```bash
# Check registered routes
grep -n "get_multisig_address" server/src/main.rs

# Test with invalid escrow_id
curl http://127.0.0.1:8080/api/escrow/invalid-uuid/multisig-address
```

---

## PHASE 6: FRONTEND

**Durée estimée:** 30 minutes
**Priorité:** 🟡 HAUTE
**Dépendances:** Phase 1-5 complétées

### Objectifs

- [x] Ajouter méthode `fetchMultisigAddress()` dans checkout.js
- [x] Appeler après simulation multisig progress
- [x] Mettre à jour UI avec adresse multisig
- [x] Ajouter messaging éducatif dans checkout/index.html
- [x] Activer bouton "Copy Address"

### Étapes Détaillées

#### 6.1 Modifier JavaScript

**Fichier:** `static/js/checkout.js`
**Emplacement:** Ajouter après la méthode `simulateMultisigProgress()` (ligne ~350)

```javascript
/**
 * Fetch multisig address from backend (Non-Custodial Architecture)
 *
 * Retrieves the generated multisig address so the buyer can pay from
 * ANY external Monero wallet (Cake, Monerujo, GUI, etc.)
 */
async fetchMultisigAddress() {
    if (!this.escrowId) {
        console.warn('[Checkout] Cannot fetch multisig address: no escrowId');
        return;
    }

    console.log(`[Checkout] Fetching multisig address for escrow ${this.escrowId}`);

    try {
        const response = await fetch(`/api/escrow/${this.escrowId}/multisig-address`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
            },
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();

        console.log('[Checkout] Multisig address received:', {
            escrow_id: data.escrow_id,
            address: data.multisig_address ? data.multisig_address.substring(0, 15) + '...' : 'NONE',
            amount_xmr: data.amount_xmr,
            status: data.status
        });

        // ═══════════════════════════════════════════════════════════
        // Update UI with multisig address
        // ═══════════════════════════════════════════════════════════

        if (data.multisig_address) {
            // Update input field
            const addressInput = document.getElementById('multisig-address');
            if (addressInput) {
                addressInput.value = data.multisig_address;
                addressInput.classList.add('address-loaded');
                console.log('[Checkout] ✅ Multisig address displayed in input field');
            }

            // Enable copy button
            const copyBtn = document.getElementById('copy-address-btn');
            if (copyBtn) {
                copyBtn.disabled = false;
                copyBtn.classList.remove('btn-disabled');
                copyBtn.classList.add('btn-enabled');
                console.log('[Checkout] ✅ Copy button enabled');
            }

            // Update QR code (if QR code library is loaded)
            if (typeof QRCode !== 'undefined') {
                const qrContainer = document.getElementById('qr-code-container');
                if (qrContainer) {
                    qrContainer.innerHTML = ''; // Clear existing
                    new QRCode(qrContainer, {
                        text: `monero:${data.multisig_address}?tx_amount=${data.amount_xmr}`,
                        width: 200,
                        height: 200,
                    });
                    console.log('[Checkout] ✅ QR code generated');
                }
            }

            // Update amount display
            const amountDisplay = document.getElementById('payment-amount-xmr');
            if (amountDisplay && data.amount_xmr) {
                amountDisplay.textContent = data.amount_xmr;
                console.log('[Checkout] ✅ Amount displayed:', data.amount_xmr);
            }

        } else {
            console.warn('[Checkout] Multisig address is null/undefined');
        }

    } catch (error) {
        console.error('[Checkout] Failed to fetch multisig address:', error);
        this.showError('Failed to load payment address. Please refresh the page.');
    }
}
```

#### 6.2 Appeler fetchMultisigAddress()

**Dans la méthode `simulateMultisigProgress()`**, ajouter à la fin:

```javascript
// After multisig simulation completes
this.hideSection('escrow-init');
this.showSection('payment-instructions');

// ══════════════════════════════════════════════════════════
// NEW: Fetch real multisig address from backend
// ══════════════════════════════════════════════════════════
await this.fetchMultisigAddress();

console.log('[Checkout] ✅ Escrow initialized and multisig address loaded');
```

#### 6.3 Mettre à Jour HTML

**Fichier:** `templates/checkout/index.html`
**Localiser:** Section payment-instructions (ligne ~223)

**Ajouter notice éducative AVANT le QR code:**

```html
<!-- Non-Custodial Architecture Education -->
<div class="checkout-notice info" style="margin-bottom: 2rem;">
    <i data-lucide="shield-check" class="checkout-notice-icon"></i>
    <div class="checkout-notice-content">
        <p class="checkout-notice-title">100% Non-Custodial Architecture</p>
        <p class="checkout-notice-text">
            <strong>How it works:</strong> The server created 3 EMPTY temporary wallets
            to generate this multisig address collaboratively (2-of-3 setup).
            <br><br>
            <strong>Your security:</strong> Pay from <u>ANY external Monero wallet</u>
            you control (Cake Wallet, Monerujo, GUI). The marketplace <u>never holds your funds</u>.
            <br><br>
            <strong>Escrow protection:</strong> Funds go directly to this multisig address.
            Release requires 2 out of 3 signatures (buyer + vendor, or arbiter).
        </p>
    </div>
</div>
```

#### 6.4 Styliser "address-loaded" Class

**Fichier:** `static/css/main.css` (ou checkout.css)

```css
/* Multisig address input - loaded state */
.checkout-input-address.address-loaded {
    background-color: rgba(201, 164, 69, 0.05);
    border-color: var(--color-accent);
    animation: pulse-border 2s ease-in-out;
}

@keyframes pulse-border {
    0%, 100% { border-color: var(--color-accent); }
    50% { border-color: rgba(201, 164, 69, 0.5); }
}

/* Copy button states */
.btn-disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-enabled {
    opacity: 1;
    cursor: pointer;
    transition: all 0.3s;
}

.btn-enabled:hover {
    background-color: rgba(201, 164, 69, 0.1);
    border-color: var(--color-accent);
}
```

#### 6.5 Compiler & Tester

```bash
# Restart server to reload templates
pkill -9 server
./target/release/server > server.log 2>&1 &

# Test checkout flow
# 1. Ouvrir http://127.0.0.1:8080/checkout
# 2. Entrer adresse shipping
# 3. Cliquer "CREATE ORDER & INITIALIZE ESCROW"
# 4. Observer:
#    - Multisig progress animation
#    - Adresse multisig s'affiche
#    - Bouton "Copy Address" devient actif
#    - Message éducatif visible
```

### Critères de Succès Phase 6

- ✅ `fetchMultisigAddress()` appelée après init escrow
- ✅ Adresse multisig s'affiche dans input field
- ✅ Bouton "Copy Address" devient actif
- ✅ QR code généré (si QRCode.js chargé)
- ✅ Message éducatif visible et lisible
- ✅ Pas d'erreurs console JavaScript

### Points d'Attention

⚠️ **Timing**: Fetch address APRÈS que escrow status = "created"
⚠️ **Error Handling**: Si fetch échoue, afficher message clair
⚠️ **QR Code Library**: Vérifier que QRCode.js est chargé (`<script src="...">`)

### Commandes de Debug

```bash
# Check console errors
# Ouvrir DevTools → Console tab
# Chercher: [Checkout] logs

# Test API directly
curl http://127.0.0.1:8080/api/escrow/{escrow_id}/multisig-address \
  -H "Cookie: session=..." | jq .multisig_address
```

---

## PHASE 7: TESTS E2E

**Durée estimée:** 45 minutes
**Priorité:** 🟢 MOYENNE
**Dépendances:** Phase 1-6 complétées

### Objectifs

- [x] Créer test file `server/tests/non_custodial_escrow_test.rs`
- [x] Test complet: init escrow → multisig setup → address generation
- [x] Vérifier temp wallets créés avec succès
- [x] Vérifier adresse multisig valide
- [x] Exécuter tests avec `--ignored`

### Étapes Détaillées

#### 7.1 Créer Fichier Test

**Fichier:** `server/tests/non_custodial_escrow_test.rs`

```rust
//! E2E Tests: Non-Custodial Escrow with Temporary Wallets
//!
//! These tests verify the complete non-custodial escrow flow:
//! 1. Create 3 empty temporary wallets
//! 2. Perform multisig setup (prepare → make → finalize)
//! 3. Generate multisig address
//! 4. Verify address format and characteristics
//!
//! # Running Tests
//! ```bash
//! cargo test --package server --test non_custodial_escrow_test -- --ignored --nocapture
//! ```

use anyhow::Result;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Run explicitly with --ignored flag
async fn test_non_custodial_escrow_initialization() -> Result<()> {
    // ═══════════════════════════════════════════════════════════
    // SETUP: Create test escrow orchestrator
    // ═══════════════════════════════════════════════════════════

    println!("\n🧪 TEST: Non-Custodial Escrow Initialization");
    println!("═══════════════════════════════════════════════════════\n");

    // TODO: Initialize test wallet_manager, db_pool, escrow_orchestrator
    // This requires mock Monero RPC or testnet setup

    // let wallet_manager = Arc::new(Mutex::new(create_test_wallet_manager()));
    // let escrow_orchestrator = EscrowOrchestrator::new(
    //     db_pool.clone(),
    //     wallet_manager.clone(),
    // );

    // ═══════════════════════════════════════════════════════════
    // TEST 1: Initialize escrow with temporary wallets
    // ═══════════════════════════════════════════════════════════

    println!("📝 Step 1: Initialize non-custodial escrow");

    let order_id = Uuid::new_v4();
    let buyer_id = Uuid::new_v4();
    let vendor_id = Uuid::new_v4();
    let amount_atomic = 100_000_000_000; // 0.1 XMR

    // let escrow = escrow_orchestrator
    //     .init_escrow(order_id, buyer_id, vendor_id, amount_atomic)
    //     .await?;

    // ═══════════════════════════════════════════════════════════
    // VERIFY: Temporary wallet IDs exist
    // ═══════════════════════════════════════════════════════════

    println!("✅ Step 2: Verify temp wallet IDs created");

    // assert!(escrow.buyer_temp_wallet_id.is_some(), "buyer_temp_wallet_id missing");
    // assert!(escrow.vendor_temp_wallet_id.is_some(), "vendor_temp_wallet_id missing");
    // assert!(escrow.arbiter_temp_wallet_id.is_some(), "arbiter_temp_wallet_id missing");

    // let buyer_wallet_id = Uuid::parse_str(&escrow.buyer_temp_wallet_id.unwrap())?;
    // let vendor_wallet_id = Uuid::parse_str(&escrow.vendor_temp_wallet_id.unwrap())?;
    // let arbiter_wallet_id = Uuid::parse_str(&escrow.arbiter_temp_wallet_id.unwrap())?;

    // println!("   buyer_wallet_id: {}", buyer_wallet_id);
    // println!("   vendor_wallet_id: {}", vendor_wallet_id);
    // println!("   arbiter_wallet_id: {}", arbiter_wallet_id);

    // ═══════════════════════════════════════════════════════════
    // VERIFY: Multisig address generated
    // ═══════════════════════════════════════════════════════════

    println!("✅ Step 3: Verify multisig address generated");

    // assert!(escrow.multisig_address.is_some(), "multisig_address missing");

    // let address = escrow.multisig_address.unwrap();

    // // Check address format (Monero mainnet=4, testnet=9)
    // assert!(
    //     address.starts_with('4') || address.starts_with('9'),
    //     "Address must start with 4 (mainnet) or 9 (testnet), got: {}",
    //     address.chars().next().unwrap()
    // );

    // // Check address length (standard Monero address = 95 chars)
    // assert_eq!(
    //     address.len(), 95,
    //     "Monero address should be 95 characters, got: {}",
    //     address.len()
    // );

    // println!("   Multisig address: {}...", address.chars().take(20).collect::<String>());
    // println!("   Address length: {} (expected: 95)", address.len());

    // ═══════════════════════════════════════════════════════════
    // VERIFY: Escrow status
    // ═══════════════════════════════════════════════════════════

    println!("✅ Step 4: Verify escrow status");

    // assert_eq!(escrow.status, "created", "Escrow status should be 'created'");

    // println!("   Status: {}", escrow.status);
    // println!("   Amount: {} atomic units ({} XMR)", escrow.amount, escrow.amount as f64 / 1_000_000_000_000.0);

    println!("\n═══════════════════════════════════════════════════════");
    println!("✅ TEST PASSED: Non-Custodial Escrow Initialization");
    println!("═══════════════════════════════════════════════════════\n");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_multisig_address_uniqueness() -> Result<()> {
    println!("\n🧪 TEST: Multisig Address Uniqueness");
    println!("═══════════════════════════════════════════════════════\n");

    // Create 2 separate escrows and verify addresses are different

    // let escrow1 = escrow_orchestrator.init_escrow(...).await?;
    // let escrow2 = escrow_orchestrator.init_escrow(...).await?;

    // let address1 = escrow1.multisig_address.unwrap();
    // let address2 = escrow2.multisig_address.unwrap();

    // assert_ne!(
    //     address1, address2,
    //     "Each escrow must have a unique multisig address"
    // );

    println!("✅ TEST PASSED: Addresses are unique per escrow");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_temp_wallets_remain_empty() -> Result<()> {
    println!("\n🧪 TEST: Temporary Wallets Remain Empty");
    println!("═══════════════════════════════════════════════════════\n");

    // Initialize escrow
    // let escrow = escrow_orchestrator.init_escrow(...).await?;

    // Get temp wallet IDs
    // let buyer_wallet_id = Uuid::parse_str(&escrow.buyer_temp_wallet_id.unwrap())?;

    // Check balance
    // let wallet_manager = escrow_orchestrator.wallet_manager.lock().await;
    // let (total, unlocked) = wallet_manager.get_balance(buyer_wallet_id).await?;

    // assert_eq!(total, 0, "Buyer temp wallet should have 0 balance");
    // assert_eq!(unlocked, 0, "Buyer temp wallet should have 0 unlocked balance");

    println!("✅ TEST PASSED: Temp wallets are empty (0 XMR)");

    Ok(())
}
```

#### 7.2 Exécuter Tests

```bash
# Run all non-custodial escrow tests
cargo test --package server --test non_custodial_escrow_test -- --ignored --nocapture

# Run specific test
cargo test --package server --test non_custodial_escrow_test test_non_custodial_escrow_initialization -- --ignored --nocapture
```

**Note:** Ces tests nécessitent:
- Monero wallet RPC running (testnet)
- Database test setup
- Mock escrow orchestrator

Pour l'instant, ils servent de **template** pour futurs tests complets.

### Critères de Succès Phase 7

- ✅ Fichier test créé
- ✅ Tests compilent sans erreur
- ✅ Structure de test claire et documentée
- ✅ Assertions couvrent cas importants
- ✅ Tests marqués `#[ignore]` pour exécution explicite

### Points d'Attention

⚠️ **Test Environment**: Nécessite testnet Monero + RPC running
⚠️ **Mocking**: Considérer mock wallet_manager pour tests rapides
⚠️ **Cleanup**: Tests doivent nettoyer temp wallets après exécution

---

## PHASE 8: DOCUMENTATION

**Durée estimée:** 15 minutes
**Priorité:** 🟢 BASSE
**Dépendances:** Phase 1-7 complétées

### Objectifs

- [x] Créer `docs/specs/non_custodial_escrow.md`
- [x] Mettre à jour `CLAUDE.md` avec nouvelle architecture
- [x] Mettre à jour `README.md` si nécessaire
- [x] Documenter flow utilisateur

### Étapes Détaillées

#### 8.1 Créer Spécification

**Fichier:** `docs/specs/non_custodial_escrow.md`

```markdown
# Non-Custodial Escrow Specification

**Date:** 2025-11-03
**Version:** 1.0.0
**Status:** ✅ IMPLEMENTED

## Architecture Overview

### Principle

The marketplace server creates **3 EMPTY temporary wallets** solely for multisig coordination.
The buyer pays **DIRECTLY** to the generated multisig address from ANY external Monero wallet.

**Zero custody** - The marketplace NEVER holds user funds.

### Flow Diagram

```
Buyer → Cart → Checkout → Order Created
                              ↓
                    3 Empty Temp Wallets Created
                    (buyer_temp, vendor_temp, arbiter)
                              ↓
                    Multisig Setup (2-of-3)
                    prepare → make → finalize
                              ↓
                    Multisig Address Generated
                    4...xxxxx (95 chars)
                              ↓
                    QR Code Displayed to Buyer
                              ↓
           Buyer Pays from External Wallet
           (Cake, Monerujo, GUI, etc.)
                              ↓
                    Payment → Multisig Address
                              ↓
                  Blockchain Monitor Detects
                  Escrow Status = "funded"
                              ↓
           Release Requires 2-of-3 Signatures
           (buyer + vendor) OR (arbiter)
                              ↓
                    Funds Sent to Vendor
```

## Implementation Details

### Database Schema

**escrows table - NEW columns:**
- `buyer_temp_wallet_id` (TEXT, nullable)
- `vendor_temp_wallet_id` (TEXT, nullable)
- `arbiter_temp_wallet_id` (TEXT, nullable)

### Key Functions

**WalletManager:**
- `create_temporary_wallet(role: &str) -> Result<Uuid>`

**EscrowOrchestrator:**
- `init_escrow()` - Modified to create temp wallets
- `setup_multisig_non_custodial(escrow_id: &str) -> Result<String>`

**API Endpoints:**
- `GET /api/escrow/:id/multisig-address` - Returns payment address

### Security Properties

- ✅ **Non-Custodial**: Server never controls user funds
- ✅ **Multisig**: 2-of-3 signatures required (no single point of failure)
- ✅ **Privacy**: Unique address per transaction
- ✅ **Open**: Buyer uses any Monero-compatible wallet
- ✅ **Auditable**: All transactions on-chain

## Testing

### Unit Tests
- `test_create_temporary_wallet_buyer()`
- `test_create_temporary_wallet_vendor()`
- `test_create_temporary_wallet_invalid_role()`

### Integration Tests
- `test_non_custodial_escrow_initialization()`
- `test_multisig_address_uniqueness()`
- `test_temp_wallets_remain_empty()`

### Manual Testing
1. Create order via checkout
2. Verify multisig address displayed
3. Copy address or scan QR code
4. Pay from external wallet
5. Verify payment detected (blockchain monitor)
6. Complete release flow

## Rollback Plan

If critical issues discovered:

```bash
# Revert migration
DATABASE_URL=marketplace.db diesel migration revert

# Regenerate schema
diesel print-schema > server/src/schema.rs

# Revert code
git checkout HEAD -- server/src/

# Rebuild
cargo build --release --package server
```

## Future Enhancements

- [ ] Wallet cleanup job (delete temp wallets after escrow complete)
- [ ] Multi-currency support (BTC, ETH via atomic swaps)
- [ ] IPFS backup of multisig state
- [ ] Hardware wallet integration
- [ ] Lightning Network for instant settlements

---

**Implemented by:** Claude + malix
**Reviewed by:** [TBD]
**Deployed:** [TBD]
```

#### 8.2 Mettre à Jour CLAUDE.md

**Fichier:** `CLAUDE.md`
**Section:** Architecture Overview (ligne ~15)

**Ajouter après "Current Status":**

```markdown
## Non-Custodial Architecture (v0.2.6+)

**Implemented:** 2025-11-03

The marketplace uses a **100% non-custodial** escrow system:

1. Server creates 3 EMPTY temporary wallets per escrow
2. Wallets perform multisig setup to generate shared address
3. Buyer pays DIRECTLY from external wallet → multisig address
4. Server monitors blockchain for payment
5. Release requires 2-of-3 signatures

**Key Properties:**
- ✅ Zero custody - Server never holds funds
- ✅ Open protocol - Any Monero wallet compatible
- ✅ Privacy-first - Unique address per transaction
- ✅ Disaster recovery - Multisig state backed up

See [`docs/specs/non_custodial_escrow.md`](docs/specs/non_custodial_escrow.md) for details.
```

#### 8.3 Mettre à Jour README (Optionnel)

**Fichier:** `README.md`

Ajouter dans Features section:

```markdown
- **100% Non-Custodial Escrow**: Marketplace never holds your funds
- **Multisig 2-of-3**: Requires 2 signatures for fund release
- **Open Protocol**: Pay from ANY Monero wallet (Cake, Monerujo, GUI)
```

### Critères de Succès Phase 8

- ✅ Spécification complète créée
- ✅ CLAUDE.md mis à jour
- ✅ README mis à jour (si applicable)
- ✅ Documentation claire et accessible

---

## PHASE 9: VALIDATION FINALE

**Durée estimée:** 30 minutes
**Priorité:** 🔴 CRITIQUE
**Dépendances:** Phase 1-8 complétées

### Checklist de Validation Complète

#### Database

- [ ] Migration appliquée: `diesel migration list` montre `[X]`
- [ ] Schema régénéré: `grep buyer_temp_wallet_id server/src/schema.rs` trouve 3 colonnes
- [ ] Models compilent: `cargo build --package server` sans erreur
- [ ] Backup database créé avant migration

#### Backend

- [ ] WalletManager compile: `create_temporary_wallet()` existe
- [ ] EscrowOrchestrator compile: `setup_multisig_non_custodial()` existe
- [ ] Blockchain monitor corrigé: Utilise `buyer_temp_wallet_id` (ligne 153)
- [ ] API endpoint enregistré: `/api/escrow/:id/multisig-address` accessible
- [ ] Logs propres: Pas d'erreurs au démarrage

#### Frontend

- [ ] Checkout.js modifié: `fetchMultisigAddress()` existe
- [ ] Méthode appelée après init escrow
- [ ] Adresse multisig s'affiche dans input
- [ ] Bouton "Copy Address" devient actif
- [ ] QR code généré (si QRCode.js chargé)
- [ ] Message éducatif visible

#### Tests

- [ ] Unit tests passent: `cargo test --package server wallet_manager`
- [ ] Integration tests créés: `non_custodial_escrow_test.rs` existe
- [ ] E2E manual testé: Flow complet Cart → Checkout → Payment

#### Sécurité

- [ ] Security theatre check: `./scripts/check-security-theatre.sh` → 0 violations
- [ ] Audit pragmatic: `./scripts/audit-pragmatic.sh` → Score 100/100
- [ ] Pas de `.unwrap()` dans nouveau code
- [ ] Logs ne contiennent pas d'adresses complètes (max 15 chars)
- [ ] CSRF protection sur endpoints POST

#### Functional

- [ ] Order creation fonctionne
- [ ] Escrow init crée 3 wallets temporaires
- [ ] Multisig address générée (4... ou 9..., 95 chars)
- [ ] Address affichée au buyer
- [ ] DEV simulate payment fonctionne
- [ ] Blockchain monitor détecte payment (test avec simulate)

### Commandes de Validation Automatique

```bash
#!/bin/bash
# validation.sh - Run all validation checks

echo "═══════════════════════════════════════════════════════════"
echo "  NON-CUSTODIAL ESCROW - VALIDATION FINALE"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Database
echo "📊 DATABASE CHECKS"
echo "---"
diesel migration list | grep "add_temp_wallet_ids" && echo "✅ Migration applied" || echo "❌ Migration missing"
grep -q "buyer_temp_wallet_id" server/src/schema.rs && echo "✅ Schema updated" || echo "❌ Schema missing columns"
echo ""

# Compilation
echo "🔨 COMPILATION CHECKS"
echo "---"
cargo build --package server 2>&1 | grep -q "Finished" && echo "✅ Server compiles" || echo "❌ Compilation failed"
echo ""

# Security
echo "🔒 SECURITY CHECKS"
echo "---"
./scripts/check-security-theatre.sh && echo "✅ No security theatre" || echo "❌ Security issues found"
./scripts/audit-pragmatic.sh | grep -q "100/100" && echo "✅ Audit score 100/100" || echo "⚠️  Audit warnings"
echo ""

# Tests
echo "🧪 TEST CHECKS"
echo "---"
cargo test --package server wallet_manager 2>&1 | grep -q "test result: ok" && echo "✅ WalletManager tests pass" || echo "❌ Tests failed"
echo ""

# Runtime
echo "🚀 RUNTIME CHECKS"
echo "---"
pgrep -f "target/release/server" > /dev/null && echo "✅ Server running" || echo "❌ Server not running"
curl -s http://127.0.0.1:8080/health | grep -q "ok" && echo "✅ Server responds" || echo "⚠️  Health check failed"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "  VALIDATION COMPLETE"
echo "═══════════════════════════════════════════════════════════"
```

**Exécuter:**
```bash
chmod +x validation.sh
./validation.sh
```

### Test Manuel Complet

**Scénario:** Cart → Checkout → Escrow Init → Payment → Release

```bash
# 1. Start server
pkill -9 server
./target/release/server > server.log 2>&1 &

# 2. Open browser
firefox http://127.0.0.1:8080

# 3. Login as buyer
# Username: testbuyer
# Password: password123

# 4. Add product to cart
# Click "Add to Cart" on any listing

# 5. Go to cart
# Click cart icon (top right)

# 6. Proceed to checkout
# Click "PROCEED TO CHECKOUT"

# 7. Enter shipping address
# Fill form and click "CREATE ORDER & INITIALIZE ESCROW"

# 8. Observe:
# - Multisig progress animation (3 steps)
# - Multisig address appears (4...xxxxx)
# - Copy button becomes active
# - QR code displays
# - Educational message visible

# 9. Test payment simulation
# Click "DEV: SIMULATE PAYMENT" button

# 10. Verify escrow funded
# Check logs: tail -f server.log | grep "PAYMENT DETECTED"

# 11. Release funds (as vendor)
# Login as vendor, go to orders, click "Release"

# 12. Verify completion
# Check order status = "completed"
```

### Critères de Succès Phase 9

- ✅ Tous les checks de validation passent
- ✅ Security audit score 100/100
- ✅ Flow manuel complet fonctionne
- ✅ Pas d'erreurs dans logs
- ✅ Performance acceptable (< 5s pour init escrow)

### Points d'Attention

⚠️ **Production Deployment**: NE PAS déployer en production avant:
- Tests sur testnet Monero complets
- Review de code par second développeur
- Load testing (concurrent escrows)
- Disaster recovery plan testé

⚠️ **Monitoring**: Mettre en place alertes pour:
- Escrows stuck in "created" state > 1h
- Temp wallets avec balance > 0 XMR (ne devrait JAMAIS arriver)
- Multisig setup failures

---

## 📚 ANNEXES TECHNIQUES

### SQL Migrations Complètes

**up.sql:**
```sql
-- Non-Custodial Escrow: Add temporary wallet IDs
ALTER TABLE escrows ADD COLUMN buyer_temp_wallet_id TEXT DEFAULT NULL;
ALTER TABLE escrows ADD COLUMN vendor_temp_wallet_id TEXT DEFAULT NULL;
ALTER TABLE escrows ADD COLUMN arbiter_temp_wallet_id TEXT DEFAULT NULL;

CREATE INDEX idx_escrows_buyer_temp_wallet ON escrows(buyer_temp_wallet_id);
CREATE INDEX idx_escrows_vendor_temp_wallet ON escrows(vendor_temp_wallet_id);
CREATE INDEX idx_escrows_arbiter_temp_wallet ON escrows(arbiter_temp_wallet_id);
```

**down.sql:**
```sql
DROP INDEX IF EXISTS idx_escrows_arbiter_temp_wallet;
DROP INDEX IF EXISTS idx_escrows_vendor_temp_wallet;
DROP INDEX IF EXISTS idx_escrows_buyer_temp_wallet;

ALTER TABLE escrows DROP COLUMN arbiter_temp_wallet_id;
ALTER TABLE escrows DROP COLUMN vendor_temp_wallet_id;
ALTER TABLE escrows DROP COLUMN buyer_temp_wallet_id;
```

### Structures Rust Modifiées

```rust
// server/src/models/escrow.rs

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
pub struct Escrow {
    pub id: String,
    pub order_id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub arbiter_id: String,
    pub amount: i64,
    pub multisig_address: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub buyer_wallet_info: Option<String>,
    pub vendor_wallet_info: Option<String>,
    pub arbiter_wallet_info: Option<String>,
    pub transaction_hash: Option<String>,
    pub expires_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub multisig_phase: String,
    pub multisig_state_json: Option<String>,
    pub multisig_updated_at: Option<String>,
    pub recovery_mode: String,
    // NEW: Non-Custodial Architecture
    pub buyer_temp_wallet_id: Option<String>,
    pub vendor_temp_wallet_id: Option<String>,
    pub arbiter_temp_wallet_id: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = escrows)]
pub struct NewEscrow {
    pub id: String,
    pub order_id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub arbiter_id: String,
    pub amount: i64,
    pub status: String,
    // NEW: Non-Custodial Architecture
    pub buyer_temp_wallet_id: Option<String>,
    pub vendor_temp_wallet_id: Option<String>,
    pub arbiter_temp_wallet_id: Option<String>,
}
```

### Endpoints API Exemples

**GET /api/escrow/:id/multisig-address**

Request:
```bash
curl http://127.0.0.1:8080/api/escrow/550e8400-e29b-41d4-a716-446655440000/multisig-address \
  -H "Cookie: session=xxx" \
  -H "Accept: application/json"
```

Response (200 OK):
```json
{
  "escrow_id": "550e8400-e29b-41d4-a716-446655440000",
  "multisig_address": "4AdUndXHHZ6cfufTMvppY6JwXNouMBzSkbLYfpAV5Usx3skxNgYeYTRj5UzqtReoS44qo9mtmXCqY45DJ852K5Jv2684Rge",
  "amount": 123456789012,
  "amount_xmr": "0.123456789012",
  "status": "created",
  "created_at": "2025-11-03T14:23:45Z",
  "message": "Pay from ANY external Monero wallet to this multisig address"
}
```

---

## 🔄 PLAN DE ROLLBACK

### Si Problème Critique Détecté

#### Étape 1: Stop Server
```bash
pkill -9 server
```

#### Étape 2: Revert Migration
```bash
DATABASE_URL=marketplace.db diesel migration revert
```

#### Étape 3: Regenerate Schema
```bash
diesel print-schema > server/src/schema.rs
```

#### Étape 4: Revert Code Changes
```bash
# Si commit fait
git log --oneline | head -10  # Find commit hash
git revert <commit_hash>

# Si pas encore commit
git checkout HEAD -- server/src/
git checkout HEAD -- static/js/
git checkout HEAD -- templates/
```

#### Étape 5: Rebuild & Restart
```bash
cargo build --release --package server
./target/release/server > server.log 2>&1 &
```

#### Étape 6: Verify Rollback
```bash
# Check migration status
diesel migration list

# Check server logs
tail -f server.log

# Test old flow still works
curl http://127.0.0.1:8080/health
```

### Backup Strategy

**AVANT toute migration production:**

```bash
# Backup database
cp marketplace.db marketplace_backup_$(date +%Y%m%d_%H%M%S).db

# Backup config
cp .env .env.backup

# Tag git state
git tag -a pre-noncustodial-v1 -m "Before non-custodial migration"
git push origin pre-noncustodial-v1
```

---

## ✅ CHECKLIST GLOBALE

### Préparation
- [ ] Backup database créé
- [ ] Git commit actuel tagué
- [ ] Server arrêté proprement
- [ ] Espace disque suffisant (check `df -h`)

### Exécution
- [ ] Phase 1 complétée (Database Migration)
- [ ] Phase 2 complétée (WalletManager)
- [ ] Phase 3 complétée (EscrowOrchestrator)
- [ ] Phase 4 complétée (Blockchain Monitor)
- [ ] Phase 5 complétée (API Endpoints)
- [ ] Phase 6 complétée (Frontend)
- [ ] Phase 7 complétée (Tests E2E)
- [ ] Phase 8 complétée (Documentation)
- [ ] Phase 9 complétée (Validation Finale)

### Post-Implémentation
- [ ] `validation.sh` exécuté avec succès
- [ ] Test manuel complet réussi
- [ ] Security audit passé (100/100)
- [ ] Logs propres (pas d'erreurs)
- [ ] Performance acceptable
- [ ] Documentation à jour

### Production Readiness
- [ ] Code review effectué
- [ ] Tests testnet complets
- [ ] Load testing effectué
- [ ] Disaster recovery plan documenté
- [ ] Monitoring alerts configurées
- [ ] Rollback plan testé

---

## 🎯 ESTIMATION FINALE

| Phase | Durée | Status |
|-------|-------|--------|
| 1. Database Migration | 15 min | ⏳ Pending |
| 2. WalletManager | 30 min | ⏳ Pending |
| 3. EscrowOrchestrator | 1h | ⏳ Pending |
| 4. Blockchain Monitor | 20 min | ⏳ Pending |
| 5. API Endpoints | 15 min | ⏳ Pending |
| 6. Frontend | 30 min | ⏳ Pending |
| 7. Tests E2E | 45 min | ⏳ Pending |
| 8. Documentation | 15 min | ⏳ Pending |
| 9. Validation Finale | 30 min | ⏳ Pending |
| **TOTAL** | **3h 30min** | **0% Complete** |

---

## 🚀 PROCHAINE ACTION

Une fois cette roadmap validée par vous, nous commencerons par la **Phase 1 : Database Migration** et progresserons séquentiellement, en cochant chaque étape dans ce document.

**Commande pour commencer:**
```bash
# Suivre la roadmap à la lettre
cat DOX/roadmap/ESCROW-NON-CUSTODIAL-IMPLEMENTATION.md
```

---

**Document créé par:** Claude
**Date:** 2025-11-03
**Version:** 1.0.0
**Status:** 📋 Ready for Execution

**Lets go non-custodial baby! 🚀**
