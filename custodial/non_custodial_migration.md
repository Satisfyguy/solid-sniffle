# Spécification : Migration vers Architecture NON-CUSTODIALE

## 📋 Métadonnées

- **ID:** SPEC-NON-CUSTODIAL-001
- **Date:** 2025-10-22
- **Auteur:** Claude
- **Statut:** 🔴 DRAFT
- **Priorité:** 🔥 CRITIQUE (Sécurité fondamentale)
- **Estimation:** 21 jours (3 semaines)
- **Rapport d'Audit Associé:** [CUSTODIAL-AUDIT-2025-10-22.md](../audits/CUSTODIAL-AUDIT-2025-10-22.md)

---

## 🔗 Documents Connexes

- **Audit Complet:** [docs/audits/CUSTODIAL-AUDIT-2025-10-22.md](../audits/CUSTODIAL-AUDIT-2025-10-22.md) - Analyse détaillée de tous les points custodials
- **Vision Technique:** [guidtechnique.md](../../guidtechnique.md) - Architecture non-custodiale de référence
- **Architecture Actuelle:** À documenter après audit

---

## 🎯 Objectif

Transformer l'architecture actuelle **CUSTODIALE** (serveur contrôle toutes les clés) en architecture **NON-CUSTODIALE** (utilisateurs contrôlent leurs propres clés) pour éliminer le risque d'exit scam et respecter la vision du projet définie dans `guidtechnique.md`.

---

## ⚠️ Problème Actuel

### Code Problématique

**Fichier:** `server/src/services/escrow.rs:200-220`

```rust
// 🔴 CUSTODIAL: Le serveur gère TOUS les wallets
let mut wallet_manager = self.wallet_manager.lock().await;

// Serveur génère wallet pour l'acheteur (MAUVAIS)
let buyer_info = wallet_manager
    .make_multisig(buyer_wallet_id, vec![])
    .await?;

// Serveur génère wallet pour le vendeur (MAUVAIS)
let vendor_info = wallet_manager
    .make_multisig(vendor_wallet_id, vec![])
    .await?;

// Serveur génère wallet pour l'arbitre (OK, c'est le serveur)
let arbiter_info = wallet_manager
    .make_multisig(arbiter_wallet_id, vec![])
    .await?;
```

### Risques

1. **Exit Scam Possible:** Le serveur peut voler tous les fonds
2. **Point de Défaillance Unique:** Hack du serveur = perte de tous les fonds
3. **Contradiction avec la Vision:** `guidtechnique.md` ligne 102 : *"les clés privées de l'Acheteur et du Vendeur ne doivent jamais, sous aucun prétexte, transiter par les serveurs de la plateforme"*

---

## ✅ Architecture Cible NON-CUSTODIALE

### Principes Fondamentaux

1. **Client-Side Key Generation:**
   - Acheteur génère ses clés dans son navigateur/app
   - Vendeur génère ses clés dans son navigateur/app
   - Serveur génère UNIQUEMENT sa propre clé (arbitre)

2. **Seules les infos publiques transitent par le serveur:**
   - `prepare_multisig()` output (chaîne MultisigV1...)
   - `make_multisig()` output (pour échange)
   - `export_multisig_info()` (pour sync)

3. **Jamais de clés privées sur le serveur:**
   - Aucune clé de dépense (spend key)
   - Aucune seed phrase
   - Aucun fichier wallet pour acheteur/vendeur

---

## 🏗️ Nouvelle Architecture

### Flux NON-CUSTODIAL

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│  CLIENT ACHETEUR│         │  SERVEUR ARBITRE│         │  CLIENT VENDEUR │
└────────┬────────┘         └────────┬────────┘         └────────┬────────┘
         │                           │                           │
    1. prepare_multisig()       prepare_multisig()       prepare_multisig()
         │                           │                           │
         ├────── MultisigV1_A ──────>│                           │
         │                           │<────── MultisigV1_V ──────┤
         │                           │                           │
    2.   │<── MultisigV1_V+Arb ─────┤─────── MultisigV1_A+Arb ─>│
         │                           │                           │
    3. make_multisig(V, Arb)    make_multisig(A, V)    make_multisig(A, Arb)
         │                           │                           │
         ├──── MakeInfo_A ──────────>│                           │
         │                           │<──────── MakeInfo_V ──────┤
         │                           │                           │
    4.   │<── MakeInfo_V+Arb ────────┤───── MakeInfo_A+Arb ─────>│
         │                           │                           │
    5. finalize_multisig()      finalize_multisig()      finalize_multisig()
         │                           │                           │
         └─────── TOUS ONT LA MÊME ADRESSE MULTISIG ────────────┘
                   (mais clés privées séparées!)
```

### Composants Techniques

#### 1. Client-Side WASM Module

**Fichier:** `client-wallet/src/lib.rs` (nouveau crate)

```rust
use wasm_bindgen::prelude::*;
use monero::{Network, PrivateKey, Address};

#[wasm_bindgen]
pub struct ClientWallet {
    private_key: PrivateKey,
    multisig_state: Option<MultisigState>,
}

#[wasm_bindgen]
impl ClientWallet {
    /// Générer un nouveau wallet côté client
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ClientWallet, JsValue> {
        let private_key = PrivateKey::from_random_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(ClientWallet {
            private_key,
            multisig_state: None,
        })
    }

    /// Étape 1: Générer prepare_multisig info
    #[wasm_bindgen]
    pub fn prepare_multisig(&mut self) -> Result<String, JsValue> {
        // Appelle monero-wallet-rpc via WASM
        let info = self.private_key.prepare_multisig()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(info) // Retourne "MultisigV1..."
    }

    /// Étape 2: Créer multisig avec infos des autres
    #[wasm_bindgen]
    pub fn make_multisig(
        &mut self,
        threshold: u32,
        other_infos: Vec<String>,
    ) -> Result<String, JsValue> {
        let result = self.private_key.make_multisig(threshold, other_infos)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.multisig_state = Some(result.state);
        Ok(result.info) // Info à envoyer aux autres
    }

    /// Obtenir l'adresse multisig
    #[wasm_bindgen]
    pub fn get_multisig_address(&self) -> Result<String, JsValue> {
        self.multisig_state
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Multisig not initialized"))?
            .address()
            .map(|addr| addr.to_string())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Signer une transaction partiellement signée
    #[wasm_bindgen]
    pub fn sign_multisig_tx(&self, unsigned_tx: String) -> Result<String, JsValue> {
        self.multisig_state
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Multisig not initialized"))?
            .sign_transaction(unsigned_tx)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

#### 2. Backend API Endpoints (NON-CUSTODIAL)

**Fichier:** `server/src/handlers/multisig_exchange.rs` (nouveau)

```rust
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stocker prepare_multisig info d'un participant
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitPrepareInfoRequest {
    pub escrow_id: Uuid,
    pub user_id: Uuid,
    pub prepare_info: String, // "MultisigV1..."
}

/// Endpoint: POST /api/escrow/{escrow_id}/prepare
pub async fn submit_prepare_info(
    req: web::Json<SubmitPrepareInfoRequest>,
    db: web::Data<DbPool>,
) -> Result<HttpResponse> {
    // Valider que user_id est participant de l'escrow
    let escrow = db_load_escrow(&db, req.escrow_id).await?;

    if ![escrow.buyer_id, escrow.vendor_id, escrow.arbiter_id]
        .contains(&req.user_id.to_string())
    {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Not a participant"
        })));
    }

    // Stocker l'info (PUBLIQUE, pas de clé privée)
    db_store_multisig_info(
        &db,
        req.escrow_id,
        req.user_id,
        "prepare",
        &req.prepare_info,
    )
    .await?;

    // Vérifier si tous ont soumis
    let count = db_count_multisig_infos(&db, req.escrow_id, "prepare").await?;

    if count == 3 {
        // Tous ont soumis, redistribuer les infos
        redistribute_prepare_infos(&db, req.escrow_id).await?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "submitted",
        "waiting_for": 3 - count
    })))
}

/// Redistribuer les prepare_infos à tous les participants
async fn redistribute_prepare_infos(
    db: &DbPool,
    escrow_id: Uuid,
) -> Result<()> {
    // Récupérer toutes les infos
    let infos = db_get_all_multisig_infos(db, escrow_id, "prepare").await?;

    // Pour chaque participant, envoyer les infos DES AUTRES
    for participant in [buyer_id, vendor_id, arbiter_id] {
        let other_infos: Vec<_> = infos
            .iter()
            .filter(|info| info.user_id != participant)
            .collect();

        // Notifier via WebSocket
        websocket.do_send(WsEvent::MultisigInfosReady {
            escrow_id,
            user_id: participant,
            other_infos,
        });
    }

    Ok(())
}

/// Endpoint: GET /api/escrow/{escrow_id}/prepare/{user_id}
pub async fn get_prepare_infos(
    path: web::Path<(Uuid, Uuid)>,
    db: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let (escrow_id, user_id) = path.into_inner();

    // Récupérer les infos DES AUTRES participants
    let infos = db_get_other_multisig_infos(
        &db,
        escrow_id,
        user_id,
        "prepare",
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "infos": infos,
        "ready": infos.len() == 2
    })))
}
```

#### 3. Frontend JavaScript (HTMX + WASM)

**Fichier:** `static/js/multisig-setup.js` (nouveau)

```javascript
import init, { ClientWallet } from './client_wallet.js';

class MultisigSetup {
    constructor(escrowId, userId) {
        this.escrowId = escrowId;
        this.userId = userId;
        this.wallet = null;
    }

    async initialize() {
        // Charger le module WASM
        await init();

        // Créer wallet côté client
        this.wallet = new ClientWallet();
        console.log("✅ Wallet généré côté client (clé privée jamais envoyée)");
    }

    async step1_prepareMul tisig() {
        // Générer prepare_multisig info
        const prepareInfo = this.wallet.prepare_multisig();

        // Envoyer AU SERVEUR (info publique seulement)
        const response = await fetch(`/api/escrow/${this.escrowId}/prepare`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                escrow_id: this.escrowId,
                user_id: this.userId,
                prepare_info: prepareInfo
            })
        });

        const data = await response.json();
        console.log(`📤 Prepare info envoyée. En attente de ${data.waiting_for} participants...`);

        // Attendre les infos des autres (WebSocket)
        this.waitForOthers();
    }

    waitForOthers() {
        const ws = new WebSocket(`wss://${location.host}/ws`);

        ws.onmessage = async (event) => {
            const msg = JSON.parse(event.data);

            if (msg.type === 'MultisigInfosReady' && msg.escrow_id === this.escrowId) {
                console.log("✅ Infos des autres participants reçues");
                await this.step2_makeMul tisig(msg.other_infos);
            }
        };
    }

    async step2_makeMultisig(otherInfos) {
        // Créer multisig LOCALEMENT avec les infos des autres
        const makeInfo = this.wallet.make_multisig(2, otherInfos);

        // Envoyer make_info au serveur
        await fetch(`/api/escrow/${this.escrowId}/make`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                escrow_id: this.escrowId,
                user_id: this.userId,
                make_info: makeInfo
            })
        });

        console.log("📤 Make info envoyée");

        // Attendre finalisation...
    }

    async step3_finalize(otherMakeInfos) {
        // Finaliser multisig LOCALEMENT
        this.wallet.finalize_multisig(otherMakeInfos);

        // Obtenir l'adresse finale
        const address = this.wallet.get_multisig_address();

        console.log(`✅ Multisig créé ! Adresse: ${address}`);
        console.log(`🔒 Votre clé privée est UNIQUEMENT sur votre appareil`);

        // Sauvegarder dans localStorage (chiffré)
        this.saveWalletLocally();
    }

    saveWalletLocally() {
        // Chiffrer avec mot de passe utilisateur
        const encrypted = this.wallet.export_encrypted(this.userPassword);
        localStorage.setItem(`wallet_${this.escrowId}`, encrypted);
    }
}

// Utilisation dans la page escrow
document.addEventListener('DOMContentLoaded', async () => {
    const setup = new MultisigSetup(escrowId, currentUserId);
    await setup.initialize();
    await setup.step1_prepareMultisig();
});
```

#### 4. Refactorisation EscrowOrchestrator

**Fichier:** `server/src/services/escrow.rs` (modifications)

```rust
pub struct EscrowOrchestrator {
    // ❌ RETIRER: wallet_manager (plus de gestion de wallets clients)
    // wallet_manager: Arc<Mutex<WalletManager>>,

    // ✅ GARDER: Seulement le wallet de l'arbitre
    arbiter_wallet: Arc<Mutex<ArbiterWallet>>,

    db: DbPool,
    websocket: Addr<WebSocketServer>,
    encryption_key: Vec<u8>,
}

impl EscrowOrchestrator {
    /// Init escrow (étape 1) - INCHANGÉ
    pub async fn init_escrow(
        &self,
        order_id: Uuid,
        buyer_id: Uuid,
        vendor_id: Uuid,
        amount_atomic: i64,
    ) -> Result<Escrow> {
        // Assign arbiter
        let arbiter_id = self.assign_arbiter().await?;

        // Create escrow in DB
        let escrow = db_insert_escrow(&self.db, NewEscrow {
            id: Uuid::new_v4().to_string(),
            order_id: order_id.to_string(),
            buyer_id: buyer_id.to_string(),
            vendor_id: vendor_id.to_string(),
            arbiter_id: arbiter_id.to_string(),
            amount: amount_atomic,
            status: "created".to_string(),
        })
        .await?;

        // Notify parties
        self.websocket.do_send(WsEvent::EscrowInit {
            escrow_id: escrow.id.parse()?,
        });

        Ok(escrow)
    }

    /// ❌ RETIRER: setup_multisig() - plus géré par le serveur
    // pub async fn setup_multisig(...) { ... }

    /// ✅ NOUVEAU: Arbitre soumet sa prepare_info
    pub async fn arbiter_submit_prepare_info(
        &self,
        escrow_id: Uuid,
    ) -> Result<String> {
        let mut arbiter = self.arbiter_wallet.lock().await;

        // Arbitre génère SA propre prepare_info
        let prepare_info = arbiter.prepare_multisig().await?;

        // Stocker dans DB
        db_store_multisig_info(
            &self.db,
            escrow_id,
            Uuid::parse_str(&self.arbiter_id)?,
            "prepare",
            &prepare_info,
        )
        .await?;

        Ok(prepare_info)
    }

    /// ✅ NOUVEAU: Vérifier si multisig est prêt
    pub async fn check_multisig_ready(
        &self,
        escrow_id: Uuid,
    ) -> Result<bool> {
        let count = db_count_multisig_infos(&self.db, escrow_id, "finalized").await?;
        Ok(count == 3)
    }

    /// ✅ MODIFIÉ: Release funds (arbitre signe seulement si dispute)
    pub async fn release_funds_as_arbiter(
        &self,
        escrow_id: Uuid,
        destination: TransferDestination,
        unsigned_tx: String, // Reçu du client gagnant
    ) -> Result<String> {
        let arbiter = self.arbiter_wallet.lock().await;

        // Arbitre signe la transaction
        let signed_tx = arbiter.sign_multisig_tx(unsigned_tx).await?;

        // Diffuser sur le réseau Monero
        let txid = arbiter.submit_multisig_tx(signed_tx).await?;

        // Update DB
        db_update_escrow_status(&self.db, escrow_id, "completed").await?;

        Ok(txid)
    }
}
```

---

## 🗄️ Modifications Base de Données

### Nouvelle Table: `multisig_infos`

```sql
CREATE TABLE multisig_infos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    escrow_id UUID NOT NULL REFERENCES escrows(id),
    user_id UUID NOT NULL REFERENCES users(id),
    info_type VARCHAR(20) NOT NULL, -- 'prepare', 'make', 'finalized'
    info_data TEXT NOT NULL, -- Chaîne MultisigV1..., encrypted
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(escrow_id, user_id, info_type)
);

CREATE INDEX idx_multisig_escrow ON multisig_infos(escrow_id);
CREATE INDEX idx_multisig_type ON multisig_infos(escrow_id, info_type);
```

### Migration de `escrows`

```sql
-- Retirer les champs custodiales
ALTER TABLE escrows
DROP COLUMN IF EXISTS buyer_wallet_id,
DROP COLUMN IF EXISTS vendor_wallet_id,
DROP COLUMN IF EXISTS arbiter_wallet_id;

-- Ajouter champs pour multisig public
ALTER TABLE escrows
ADD COLUMN multisig_setup_complete BOOLEAN DEFAULT FALSE,
ADD COLUMN multisig_finalized_at TIMESTAMP;
```

---

## 🧪 Plan de Tests

### 1. Tests Unitaires WASM

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_generation() {
        let wallet = ClientWallet::new().unwrap();
        assert!(wallet.private_key.is_valid());
    }

    #[test]
    fn test_prepare_multisig() {
        let mut wallet = ClientWallet::new().unwrap();
        let info = wallet.prepare_multisig().unwrap();
        assert!(info.starts_with("MultisigV1"));
    }

    #[test]
    fn test_make_multisig() {
        let mut w1 = ClientWallet::new().unwrap();
        let mut w2 = ClientWallet::new().unwrap();
        let mut w3 = ClientWallet::new().unwrap();

        let info1 = w1.prepare_multisig().unwrap();
        let info2 = w2.prepare_multisig().unwrap();
        let info3 = w3.prepare_multisig().unwrap();

        let make1 = w1.make_multisig(2, vec![info2.clone(), info3.clone()]).unwrap();
        let make2 = w2.make_multisig(2, vec![info1.clone(), info3.clone()]).unwrap();
        let make3 = w3.make_multisig(2, vec![info1, info2]).unwrap();

        // Vérifier que les 3 ont la même adresse
        let addr1 = w1.get_multisig_address().unwrap();
        let addr2 = w2.get_multisig_address().unwrap();
        let addr3 = w3.get_multisig_address().unwrap();

        assert_eq!(addr1, addr2);
        assert_eq!(addr2, addr3);
    }
}
```

### 2. Tests E2E (Playwright)

```javascript
// tests/e2e/non-custodial-escrow.spec.js
import { test, expect } from '@playwright/test';

test('Complete non-custodial escrow flow', async ({ page, context }) => {
    // Ouvrir 3 contextes (acheteur, vendeur, arbitre)
    const buyerPage = await context.newPage();
    const vendorPage = await context.newPage();
    const arbiterPage = await context.newPage();

    // 1. Acheteur crée wallet côté client
    await buyerPage.goto('/escrow/123/setup');
    await expect(buyerPage.locator('#wallet-status')).toContainText('Wallet généré localement');

    // Vérifier que localStorage contient le wallet (chiffré)
    const buyerWallet = await buyerPage.evaluate(() =>
        localStorage.getItem('wallet_123')
    );
    expect(buyerWallet).toBeTruthy();
    expect(buyerWallet).not.toContain('private'); // Pas de clé en clair

    // 2. Acheteur soumet prepare_info
    await buyerPage.click('#btn-prepare-multisig');
    await expect(buyerPage.locator('#status')).toContainText('En attente de 2 participants');

    // 3. Vendeur fait pareil
    await vendorPage.goto('/escrow/123/setup');
    await vendorPage.click('#btn-prepare-multisig');

    // 4. Arbitre (serveur) soumet automatiquement
    // ... (WebSocket notification)

    // 5. Tous reçoivent les infos des autres
    await expect(buyerPage.locator('#status')).toContainText('Infos reçues', { timeout: 10000 });
    await expect(vendorPage.locator('#status')).toContainText('Infos reçues', { timeout: 10000 });

    // 6. make_multisig côté client
    await buyerPage.click('#btn-make-multisig');
    await vendorPage.click('#btn-make-multisig');

    // 7. Vérifier adresse multisig créée
    const buyerAddr = await buyerPage.locator('#multisig-address').textContent();
    const vendorAddr = await vendorPage.locator('#multisig-address').textContent();
    expect(buyerAddr).toBe(vendorAddr);

    // 8. Vérifier aucune clé privée envoyée au serveur
    const serverLogs = await getServerLogs();
    expect(serverLogs).not.toContain('private_key');
    expect(serverLogs).not.toContain('spend_key');
});
```

### 3. Tests de Sécurité

```rust
#[test]
fn test_no_private_keys_in_database() {
    let db = setup_test_db();

    // Créer un escrow complet
    let escrow_id = create_test_escrow(&db).await;

    // Scanner TOUTES les tables pour clés privées
    let tables = ["escrows", "multisig_infos", "users"];

    for table in tables {
        let rows = db.query(&format!("SELECT * FROM {}", table)).await.unwrap();

        for row in rows {
            let json = serde_json::to_string(&row).unwrap();

            // Vérifier aucune mention de clés privées
            assert!(!json.contains("private_key"));
            assert!(!json.contains("spend_key"));
            assert!(!json.contains("view_key")); // View key partagée OK, mais pas dans DB
            assert!(!json.contains("seed"));
            assert!(!json.contains("mnemonic"));
        }
    }
}

#[test]
fn test_server_cannot_spend_funds() {
    // Simuler un serveur compromis tentant de voler les fonds
    let escrow = create_funded_escrow().await;

    let arbiter = EscrowOrchestrator::new(...);

    // Tentative de vol : arbitre essaie de créer une transaction seul
    let result = arbiter
        .arbiter_wallet
        .lock()
        .await
        .create_transaction(escrow.multisig_address, attacker_address, escrow.amount)
        .await;

    // DOIT échouer (besoin de 2/3 signatures)
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Insufficient signatures");
}
```

---

## 📅 Plan de Migration (3 Semaines)

### Semaine 1 : Fondations Client-Side (7 jours)

#### Jour 1-2 : WASM Wallet Module
- [ ] Créer crate `client-wallet`
- [ ] Implémenter `ClientWallet::new()`
- [ ] Implémenter `prepare_multisig()`
- [ ] Tests unitaires WASM
- [ ] Compiler vers WASM (`wasm-pack build`)

#### Jour 3-4 : API Backend (Échange d'Infos)
- [ ] Créer table `multisig_infos`
- [ ] Implémenter `/api/escrow/{id}/prepare` (POST/GET)
- [ ] Implémenter `/api/escrow/{id}/make` (POST/GET)
- [ ] WebSocket events pour notifications
- [ ] Tests API

#### Jour 5-7 : Frontend JavaScript
- [ ] Intégrer WASM dans HTMX templates
- [ ] Créer `multisig-setup.js`
- [ ] UI pour génération wallet côté client
- [ ] LocalStorage chiffré pour wallets
- [ ] Tests E2E Playwright

### Semaine 2 : Refactorisation Backend (7 jours)

#### Jour 8-10 : Retirer Code Custodial
- [ ] Supprimer `WalletManager` de `EscrowOrchestrator`
- [ ] Créer `ArbiterWallet` (wallet unique serveur)
- [ ] Migrer `setup_multisig()` → logic client-side
- [ ] Migrer `release_funds()` → signature arbitre seulement
- [ ] Tests de régression

#### Jour 11-12 : Migration Base de Données
- [ ] Script de migration SQL
- [ ] Convertir escrows existants (si testnet)
- [ ] Retirer colonnes `*_wallet_id`
- [ ] Ajouter `multisig_setup_complete`
- [ ] Tests migration

#### Jour 13-14 : Intégration Complète
- [ ] Connecter frontend WASM ↔ backend API
- [ ] Tester flux end-to-end (3 participants)
- [ ] Vérifier aucune clé privée en DB
- [ ] Monitoring logs (détecter fuites)

### Semaine 3 : Tests & Documentation (7 jours)

#### Jour 15-17 : Tests de Sécurité
- [ ] Audit code pour clés privées
- [ ] Tests de pénétration (serveur compromis)
- [ ] Vérifier isolation wallets
- [ ] Scanner logs pour données sensibles
- [ ] Tests de charge (100+ escrows simultanés)

#### Jour 18-19 : Documentation
- [ ] Mettre à jour `ARCHITECTURE.md`
- [ ] Créer `docs/NON-CUSTODIAL-GUIDE.md`
- [ ] Tutoriel utilisateur (génération wallet)
- [ ] Diagrammes architecture (mermaid)
- [ ] Vidéo explicative (optionnel)

#### Jour 20-21 : Déploiement Testnet
- [ ] Déployer sur testnet
- [ ] Tests avec vrais utilisateurs (beta)
- [ ] Collecte feedback
- [ ] Corrections bugs
- [ ] **Célébration 🎉 : Exit Scam IMPOSSIBLE !**

---

## ✅ Checklist de Validation

### Sécurité

- [ ] Aucune clé privée dans la base de données
- [ ] Aucune clé privée dans les logs
- [ ] Aucune clé privée dans les requêtes HTTP/WebSocket
- [ ] Wallets clients chiffrés dans localStorage
- [ ] Serveur ne peut pas créer de transaction seul
- [ ] Tests de pénétration passés

### Fonctionnel

- [ ] 3 participants peuvent créer multisig
- [ ] Acheteur + Vendeur peuvent signer sans arbitre (happy path)
- [ ] Arbitre + Acheteur peuvent signer (remboursement)
- [ ] Arbitre + Vendeur peuvent signer (paiement vendeur)
- [ ] Transactions diffusées avec succès sur testnet

### Performance

- [ ] Temps de setup multisig < 30 secondes
- [ ] WASM module < 500KB
- [ ] Aucune fuite mémoire côté client
- [ ] Support 100+ escrows simultanés

### Documentation

- [ ] Guide utilisateur complet
- [ ] Architecture documentée
- [ ] Code commenté (intentions de sécurité)
- [ ] Diagrammes à jour

---

## 🚨 Risques et Mitigations

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| WASM incompatibilité navigateurs | Haut | Moyen | Fallback vers monero-wallet-rpc local |
| Perte wallet localStorage | Haut | Faible | Export backup chiffré, recovery via seed |
| Complexité UX | Moyen | Haut | Tutoriel interactif, UI simplifiée |
| Bugs multisig Monero | Critique | Faible | Tests exhaustifs, testnet only initially |
| Performance WASM lente | Moyen | Faible | Optimisation Rust, workers threads |

---

## 📚 Références

- [Monero Multisig](https://www.getmonero.org/resources/user-guides/multisig-messaging-system.html)
- [Haveno (Monero DEX non-custodial)](https://github.com/haveno-dex/haveno)
- [WASM-Bindgen](https://rustwasm.github.io/wasm-bindgen/)
- `guidtechnique.md` (lignes 58-111) : Architecture non-custodiale

---

## 📊 Métriques de Succès

### Avant (Custodial)
- ❌ Serveur contrôle 100% des wallets
- ❌ Exit scam possible
- ❌ Point de défaillance unique
- ❌ Contradiction avec vision

### Après (Non-Custodial)
- ✅ Serveur contrôle 33% (arbitre seulement)
- ✅ Exit scam **IMPOSSIBLE** (besoin 2/3)
- ✅ Clés distribuées
- ✅ Aligné avec `guidtechnique.md`

---

**Statut:** 🔴 DRAFT - En attente de validation

**Prochaine Action:** Validation par équipe + Début Semaine 1 (WASM Module)
