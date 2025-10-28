# PLAN CLAUDE - Phase 3.2 Transaction Finalization + Phase 4 Frontend

**Projet:** Monero Marketplace
**Votre Mission:** Compléter escrow flow + créer frontend HTMX
**Durée:** 25 jours (Phase 3.2: 10 jours + Phase 4: 15 jours)
**Parallèle:** Gemini travaille sur Phase 4.5 (Infrastructure) dans dossier `4.5/`

---

## 🎯 VUE D'ENSEMBLE

Vous allez compléter **deux phases critiques** :

**Phase 3.2 : Transaction Finalization (10 jours)**
- Compléter blockchain monitor logic (actuellement placeholder)
- Implémenter transactions multisig réelles (sign + broadcast)
- Tests end-to-end flow escrow complet
- Score cible : 95/100

**Phase 4 : Frontend HTMX (15 jours)**
- Templates server-side (Tera/Askama)
- HTMX pour interactivité (zero JavaScript frameworks)
- Interface pour : Auth, Listings, Orders, Escrow
- Responsive design (CSS simple)

**RÈGLE IMPORTANTE :** Ne touchez PAS au dossier `4.5/` (réservé à Gemini)

---

## 📊 STATUT ACTUEL

**Phase 3.1 : ✅ COMPLÈTE (Commit 4705304)**
- 6 endpoints API escrow implémentés
- Handlers avec validation input
- Blockchain monitor (structure seulement)
- Score : 92/100

**Ce qui manque (Phase 3.2) :**
- 🟡 Blockchain monitor logic placeholder (lignes 151-225)
- 🟡 Intégration WalletManager pour multisig signatures
- 🟡 Tests E2E flow complet

---

## 📋 PHASE 3.2 : TRANSACTION FINALIZATION

### Durée : 10 jours (4 milestones)

---

## 📋 MILESTONE 3.2.1 : Blockchain Monitor Logic (3 jours)

### Objectif
Remplacer les placeholders par implémentation réelle du monitoring blockchain

### ✅ TÂCHES

#### Tâche 1.1 : Compléter check_escrow_funding()
**Fichier:** `server/src/services/blockchain_monitor.rs`

**Lignes à remplacer:** 151-167

**Implémentation :**
```rust
async fn check_escrow_funding(&self, escrow_id: Uuid) -> Result<()> {
    let escrow = db_load_escrow(&self.db, escrow_id).await?;

    let multisig_address = escrow
        .multisig_address
        .ok_or_else(|| anyhow::anyhow!("Escrow {} has no multisig address", escrow_id))?;

    info!(
        "Checking funding for escrow {} at address {}",
        escrow_id,
        &multisig_address[..10]  // Truncate pour OPSEC
    );

    // 1. Get wallet balance
    let wallet_manager = self.wallet_manager.lock().await;

    // Determine which wallet to use (buyer's wallet for this escrow)
    let escrow_wallet_id = escrow.buyer_id.parse::<Uuid>()?;

    let balance_result = wallet_manager
        .get_balance(escrow_wallet_id)
        .await
        .context("Failed to get wallet balance")?;

    let balance_atomic = balance_result.balance;

    drop(wallet_manager);

    // 2. Compare to expected amount
    if balance_atomic >= escrow.amount {
        info!(
            "Escrow {} funded! Balance: {} >= Required: {}",
            escrow_id, balance_atomic, escrow.amount
        );

        // 3. Update escrow status
        db_update_escrow_status(&self.db, escrow_id, "funded")
            .await
            .context("Failed to update escrow status to funded")?;

        // 4. Notify parties via WebSocket
        self.websocket.do_send(WsEvent::EscrowFunded {
            escrow_id,
            amount: balance_atomic,
        });
    } else {
        info!(
            "Escrow {} not yet funded. Balance: {} < Required: {}",
            escrow_id, balance_atomic, escrow.amount
        );
    }

    Ok(())
}
```

**Tests à ajouter :**
```rust
#[tokio::test]
async fn test_check_escrow_funding() -> Result<()> {
    // Setup test escrow avec multisig address
    // Mock wallet balance
    // Call check_escrow_funding
    // Assert status updated to "funded"
    // Assert WebSocket event sent
    Ok(())
}
```

#### Tâche 1.2 : Compléter check_transaction_confirmations()
**Fichier:** `server/src/services/blockchain_monitor.rs`

**Lignes à remplacer:** 191-225

**Implémentation :**
```rust
async fn check_transaction_confirmations(&self, escrow_id: Uuid) -> Result<()> {
    let escrow = db_load_escrow(&self.db, escrow_id).await?;

    info!(
        "Checking transaction confirmations for escrow {} (status: {})",
        escrow_id, escrow.status
    );

    // Get transaction hash from database
    let transaction = db_get_transaction_by_escrow(&self.db, escrow_id)
        .await
        .context("No transaction found for escrow")?;

    let tx_hash = transaction.tx_hash;

    // Query blockchain for transaction details
    let wallet_manager = self.wallet_manager.lock().await;

    let tx_info = wallet_manager
        .get_transfer_by_txid(&tx_hash)
        .await
        .context("Failed to get transaction info from blockchain")?;

    let confirmations = tx_info.confirmations;

    drop(wallet_manager);

    // Update confirmations in database
    db_update_transaction_confirmations(&self.db, &tx_hash, confirmations)
        .await
        .context("Failed to update transaction confirmations")?;

    info!(
        "Transaction {} has {} confirmations (required: {})",
        &tx_hash[..8], confirmations, self.config.required_confirmations
    );

    // Finalize if threshold reached
    if confirmations >= self.config.required_confirmations {
        let final_status = match escrow.status.as_str() {
            "releasing" => "completed",
            "refunding" => "refunded",
            _ => {
                warn!(
                    "Unexpected escrow status for finalization: {}",
                    escrow.status
                );
                return Ok(());
            }
        };

        db_update_escrow_status(&self.db, escrow_id, final_status)
            .await
            .context("Failed to update escrow to final status")?;

        self.websocket.do_send(WsEvent::TransactionConfirmed {
            tx_hash: tx_hash.clone(),
            confirmations,
        });

        info!(
            "Escrow {} transaction confirmed and finalized to status: {}",
            escrow_id, final_status
        );
    }

    Ok(())
}
```

**Tests à ajouter :**
```rust
#[tokio::test]
async fn test_check_transaction_confirmations_threshold() -> Result<()> {
    // Setup escrow in "releasing" status
    // Mock transaction with 10 confirmations
    // Call check_transaction_confirmations
    // Assert status updated to "completed"
    // Assert WebSocket event sent
    Ok(())
}
```

#### Tâche 1.3 : Fonctions DB manquantes
**Fichier:** `server/src/db/mod.rs`

**Ajouter :**
```rust
pub async fn db_get_transaction_by_escrow(
    pool: &DbPool,
    escrow_id: Uuid,
) -> Result<Transaction> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    let escrow_id_str = escrow_id.to_string();

    tokio::task::spawn_blocking(move || {
        use crate::schema::transactions::dsl::*;
        use diesel::prelude::*;

        transactions
            .filter(order_id.eq(escrow_id_str))
            .first::<Transaction>(&mut conn)
            .context("Transaction not found for escrow")
    })
    .await
    .context("Task join error")?
}

pub async fn db_update_transaction_confirmations(
    pool: &DbPool,
    tx_hash: &str,
    confs: u32,
) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    let tx_hash_str = tx_hash.to_string();

    tokio::task::spawn_blocking(move || {
        use crate::schema::transactions::dsl::*;
        use diesel::prelude::*;

        diesel::update(transactions.filter(tx_hash.eq(tx_hash_str)))
            .set(confirmations.eq(confs as i32))
            .execute(&mut conn)
            .context("Failed to update transaction confirmations")
    })
    .await
    .context("Task join error")??;

    Ok(())
}
```

### ✅ VALIDATION MILESTONE 1

**Tests unitaires :**
```bash
cargo test --package server test_check_escrow_funding
cargo test --package server test_check_transaction_confirmations
```

**Critères d'acceptance :**
- [ ] check_escrow_funding() implémenté (appel RPC réel)
- [ ] check_transaction_confirmations() implémenté
- [ ] 2 fonctions DB ajoutées
- [ ] 2 tests unitaires passent
- [ ] Aucun .unwrap() dans le code
- [ ] Tracing logs (info/warn) ajoutés

---

## 📋 MILESTONE 3.2.2 : Multisig Transactions (4 jours)

### Objectif
Implémenter signatures multisig réelles + broadcast transactions

### ✅ TÂCHES

#### Tâche 2.1 : WalletManager - Sign Transaction
**Fichier:** `server/src/wallet_manager.rs`

**Ajouter méthode :**
```rust
/// Sign a multisig transaction
pub async fn sign_multisig_transaction(
    &self,
    wallet_id: Uuid,
    tx_metadata: &str,
) -> Result<SignedTransaction> {
    let wallet_name = self.get_wallet_name(wallet_id)?;

    let request = RpcRequest::new("sign_multisig")
        .with_param("tx_data_hex", tx_metadata);

    let response: RpcResponse<SignMultisigResult> = self
        .monero_client
        .call(&wallet_name, request)
        .await
        .context("Failed to sign multisig transaction")?;

    let result = response.result
        .ok_or_else(|| anyhow::anyhow!("No result in sign_multisig response"))?;

    info!(
        "Transaction signed by wallet {} ({})",
        &wallet_name,
        &wallet_id.to_string()[..8]
    );

    Ok(SignedTransaction {
        tx_data_hex: result.tx_data_hex,
        tx_hash_list: result.tx_hash_list,
    })
}

/// Finalize multisig transaction (combine signatures)
pub async fn finalize_multisig_transaction(
    &self,
    wallet_id: Uuid,
    signed_txs: Vec<String>, // tx_data_hex from each signer
) -> Result<FinalizedTransaction> {
    let wallet_name = self.get_wallet_name(wallet_id)?;

    let request = RpcRequest::new("submit_multisig")
        .with_param("tx_data_hex", signed_txs);

    let response: RpcResponse<SubmitMultisigResult> = self
        .monero_client
        .call(&wallet_name, request)
        .await
        .context("Failed to finalize multisig transaction")?;

    let result = response.result
        .ok_or_else(|| anyhow::anyhow!("No result in submit_multisig response"))?;

    Ok(FinalizedTransaction {
        tx_hash_list: result.tx_hash_list,
    })
}

/// Broadcast finalized transaction
pub async fn broadcast_transaction(
    &self,
    wallet_id: Uuid,
) -> Result<Vec<String>> {
    // submit_multisig already broadcasts automatically in Monero
    // This is a confirmation that transaction was relayed

    info!("Transaction broadcasted for wallet {}", wallet_id);

    Ok(vec![]) // Placeholder, submit_multisig handles broadcast
}
```

**Types à ajouter :**
```rust
#[derive(Debug, Deserialize)]
pub struct SignMultisigResult {
    pub tx_data_hex: String,
    pub tx_hash_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitMultisigResult {
    pub tx_hash_list: Vec<String>,
}

#[derive(Debug)]
pub struct SignedTransaction {
    pub tx_data_hex: String,
    pub tx_hash_list: Vec<String>,
}

#[derive(Debug)]
pub struct FinalizedTransaction {
    pub tx_hash_list: Vec<String>,
}
```

#### Tâche 2.2 : EscrowOrchestrator - Release Funds Complete
**Fichier:** `server/src/services/escrow.rs`

**Compléter méthode release_funds() (lignes 174-177) :**
```rust
pub async fn release_funds(
    &self,
    escrow_id: Uuid,
    requester_id: Uuid,
    vendor_address: String,
) -> Result<String> {
    let escrow = db_load_escrow(&self.db, escrow_id).await?;

    // 1. Verify escrow is in correct state
    if escrow.status != "funded" {
        return Err(anyhow::anyhow!(
            "Escrow not in funded state (current: {})",
            escrow.status
        ));
    }

    // 2. Verify requester is buyer
    if requester_id.to_string() != escrow.buyer_id {
        return Err(anyhow::anyhow!("Only buyer can release funds"));
    }

    // 3. Get wallet IDs for buyer + arbiter (2-of-3 multisig)
    let buyer_wallet_id = escrow.buyer_id.parse::<Uuid>()?;
    let arbiter_wallet_id = escrow.arbiter_id.parse::<Uuid>()?;

    // 4. Create unsigned transaction to vendor address
    let mut wallet_manager = self.wallet_manager.lock().await;

    let destinations = vec![TransferDestination {
        address: vendor_address.clone(),
        amount: escrow.amount as u64,
    }];

    let unsigned_tx = wallet_manager
        .create_transfer(buyer_wallet_id, destinations)
        .await
        .context("Failed to create release transaction")?;

    let tx_metadata = unsigned_tx.multisig_txset;

    // 5. Sign with buyer wallet
    let buyer_sig = wallet_manager
        .sign_multisig_transaction(buyer_wallet_id, &tx_metadata)
        .await
        .context("Failed to sign with buyer wallet")?;

    // 6. Sign with arbiter wallet
    let arbiter_sig = wallet_manager
        .sign_multisig_transaction(arbiter_wallet_id, &tx_metadata)
        .await
        .context("Failed to sign with arbiter wallet")?;

    // 7. Finalize with 2 signatures
    let finalized = wallet_manager
        .finalize_multisig_transaction(
            buyer_wallet_id,
            vec![buyer_sig.tx_data_hex, arbiter_sig.tx_data_hex],
        )
        .await
        .context("Failed to finalize multisig transaction")?;

    let tx_hash = finalized
        .tx_hash_list
        .first()
        .ok_or_else(|| anyhow::anyhow!("No transaction hash returned"))?
        .clone();

    drop(wallet_manager);

    // 8. Store transaction in DB
    db_insert_transaction(
        &self.db,
        NewTransaction {
            id: Uuid::new_v4().to_string(),
            escrow_id: escrow_id.to_string(),
            tx_hash: tx_hash.clone(),
            amount: escrow.amount,
            confirmations: 0,
            tx_type: "release".to_string(),
        },
    )
    .await
    .context("Failed to store transaction in database")?;

    // 9. Update escrow status
    db_update_escrow_status(&self.db, escrow_id, "releasing")
        .await
        .context("Failed to update escrow status")?;

    // 10. Notify parties
    self.websocket.do_send(WsEvent::EscrowReleasing {
        escrow_id,
        tx_hash: tx_hash.clone(),
    });

    info!(
        "Funds released for escrow {} (tx: {})",
        escrow_id,
        &tx_hash[..8]
    );

    Ok(tx_hash)
}
```

#### Tâche 2.3 : EscrowOrchestrator - Refund Funds Complete
**Fichier:** `server/src/services/escrow.rs`

**Compléter méthode refund_funds() (lignes 259-261) :**
```rust
pub async fn refund_funds(
    &self,
    escrow_id: Uuid,
    requester_id: Uuid,
    buyer_address: String,
) -> Result<String> {
    let escrow = db_load_escrow(&self.db, escrow_id).await?;

    // 1. Verify escrow status
    if escrow.status != "funded" && escrow.status != "disputed" {
        return Err(anyhow::anyhow!(
            "Escrow not eligible for refund (status: {})",
            escrow.status
        ));
    }

    // 2. Verify requester is vendor or arbiter
    if requester_id.to_string() != escrow.vendor_id
        && requester_id.to_string() != escrow.arbiter_id {
        return Err(anyhow::anyhow!(
            "Only vendor or arbiter can initiate refund"
        ));
    }

    // 3. Get wallet IDs (vendor + arbiter for refund)
    let vendor_wallet_id = escrow.vendor_id.parse::<Uuid>()?;
    let arbiter_wallet_id = escrow.arbiter_id.parse::<Uuid>()?;

    // 4. Create transaction to buyer address
    let mut wallet_manager = self.wallet_manager.lock().await;

    let destinations = vec![TransferDestination {
        address: buyer_address.clone(),
        amount: escrow.amount as u64,
    }];

    let unsigned_tx = wallet_manager
        .create_transfer(vendor_wallet_id, destinations)
        .await
        .context("Failed to create refund transaction")?;

    // 5. Sign with vendor + arbiter
    let vendor_sig = wallet_manager
        .sign_multisig_transaction(vendor_wallet_id, &unsigned_tx.multisig_txset)
        .await?;

    let arbiter_sig = wallet_manager
        .sign_multisig_transaction(arbiter_wallet_id, &unsigned_tx.multisig_txset)
        .await?;

    // 6. Finalize
    let finalized = wallet_manager
        .finalize_multisig_transaction(
            vendor_wallet_id,
            vec![vendor_sig.tx_data_hex, arbiter_sig.tx_data_hex],
        )
        .await?;

    let tx_hash = finalized.tx_hash_list.first().unwrap().clone();

    drop(wallet_manager);

    // 7. Store transaction
    db_insert_transaction(
        &self.db,
        NewTransaction {
            id: Uuid::new_v4().to_string(),
            escrow_id: escrow_id.to_string(),
            tx_hash: tx_hash.clone(),
            amount: escrow.amount,
            confirmations: 0,
            tx_type: "refund".to_string(),
        },
    )
    .await?;

    // 8. Update status
    db_update_escrow_status(&self.db, escrow_id, "refunding").await?;

    // 9. Notify
    self.websocket.do_send(WsEvent::EscrowRefunding {
        escrow_id,
        tx_hash: tx_hash.clone(),
    });

    Ok(tx_hash)
}
```

### ✅ VALIDATION MILESTONE 2

**Tests unitaires :**
```bash
cargo test --package server test_sign_multisig_transaction
cargo test --package server test_release_funds_complete
cargo test --package server test_refund_funds_complete
```

**Critères d'acceptance :**
- [ ] WalletManager: sign/finalize/broadcast implémentés
- [ ] release_funds() complet avec 2-of-3 signatures
- [ ] refund_funds() complet
- [ ] Types SignedTransaction, FinalizedTransaction ajoutés
- [ ] 3 tests unitaires passent
- [ ] Logs tracing (info) ajoutés

---

## 📋 MILESTONE 3.2.3 : Dispute Resolution (2 jours)

### Objectif
Compléter flow arbitrage avec résolution automatique

### ✅ TÂCHES

#### Tâche 3.1 : Complete resolve_dispute()
**Fichier:** `server/src/services/escrow.rs`

**Compléter méthode resolve_dispute() (lignes 432-434) :**
```rust
pub async fn resolve_dispute(
    &self,
    escrow_id: Uuid,
    arbiter_id: Uuid,
    resolution: &str, // "buyer" or "vendor"
) -> Result<()> {
    let escrow = db_load_escrow(&self.db, escrow_id).await?;

    // 1. Verify escrow is in disputed state
    if escrow.status != "disputed" {
        return Err(anyhow::anyhow!(
            "Escrow not in disputed state (current: {})",
            escrow.status
        ));
    }

    // 2. Verify requester is the assigned arbiter
    if arbiter_id.to_string() != escrow.arbiter_id {
        return Err(anyhow::anyhow!("Only assigned arbiter can resolve dispute"));
    }

    // 3. Validate resolution
    if resolution != "buyer" && resolution != "vendor" {
        return Err(anyhow::anyhow!("Invalid resolution: must be 'buyer' or 'vendor'"));
    }

    // 4. Update escrow status based on resolution
    let new_status = match resolution {
        "buyer" => "resolved_buyer", // Arbiter will trigger refund
        "vendor" => "resolved_vendor", // Arbiter will trigger release
        _ => unreachable!(),
    };

    db_update_escrow_status(&self.db, escrow_id, new_status)
        .await
        .context("Failed to update escrow status after resolution")?;

    // 5. Store dispute resolution in database
    db_update_dispute_resolution(&self.db, escrow_id, resolution)
        .await
        .context("Failed to store dispute resolution")?;

    // 6. Notify all parties
    self.websocket.do_send(WsEvent::DisputeResolved {
        escrow_id,
        resolution: resolution.to_string(),
        decided_by: arbiter_id,
    });

    info!(
        "Dispute resolved for escrow {} in favor of {}",
        escrow_id, resolution
    );

    // 7. Auto-trigger appropriate action based on resolution
    match resolution {
        "buyer" => {
            // Arbiter triggers refund
            let buyer_address = escrow.buyer_address
                .ok_or_else(|| anyhow::anyhow!("Buyer address not found"))?;

            self.refund_funds(escrow_id, arbiter_id, buyer_address).await?;
        }
        "vendor" => {
            // Arbiter triggers release
            let vendor_address = escrow.vendor_address
                .ok_or_else(|| anyhow::anyhow!("Vendor address not found"))?;

            self.release_funds(escrow_id, arbiter_id, vendor_address).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
```

#### Tâche 3.2 : DB Function for Dispute
**Fichier:** `server/src/db/mod.rs`

**Ajouter :**
```rust
pub async fn db_update_dispute_resolution(
    pool: &DbPool,
    escrow_id: Uuid,
    resolution: &str,
) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    let escrow_id_str = escrow_id.to_string();
    let resolution_str = resolution.to_string();

    tokio::task::spawn_blocking(move || {
        use crate::schema::disputes::dsl::*;
        use diesel::prelude::*;

        diesel::update(disputes.filter(escrow_id.eq(escrow_id_str)))
            .set((
                status.eq("resolved"),
                resolution.eq(resolution_str),
                resolved_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .context("Failed to update dispute resolution")
    })
    .await
    .context("Task join error")??;

    Ok(())
}
```

### ✅ VALIDATION MILESTONE 3

**Tests :**
```bash
cargo test --package server test_resolve_dispute_buyer
cargo test --package server test_resolve_dispute_vendor
```

**Critères d'acceptance :**
- [ ] resolve_dispute() complet avec auto-trigger
- [ ] db_update_dispute_resolution() ajouté
- [ ] 2 tests (resolution buyer + vendor) passent
- [ ] WebSocket notifications envoyées

---

## 📋 MILESTONE 3.2.4 : Tests End-to-End (1 jour)

### Objectif
Tests E2E pour flow escrow complet de A à Z

### ✅ TÂCHES

#### Tâche 4.1 : Test Flow Complet
**Fichier:** `server/tests/escrow_e2e.rs`

**Créer test :**
```rust
#[tokio::test]
async fn test_complete_escrow_flow() -> Result<()> {
    // Setup: 3 users (buyer, vendor, arbiter)
    let (buyer, vendor, arbiter) = setup_test_users().await?;

    // Step 1: Vendor creates listing
    let listing = create_listing(vendor.id, 1000000).await?;

    // Step 2: Buyer creates order
    let order = create_order(buyer.id, listing.id).await?;

    // Step 3: Escrow auto-initialized
    let escrow = get_escrow_by_order(order.id).await?;
    assert_eq!(escrow.status, "created");

    // Step 4: All parties prepare multisig
    prepare_multisig(escrow.id, buyer.id).await?;
    prepare_multisig(escrow.id, vendor.id).await?;
    prepare_multisig(escrow.id, arbiter.id).await?;

    // Step 5: Wait for auto-orchestration (make_multisig + sync)
    wait_for_status(escrow.id, "ready", 60).await?;

    // Step 6: Buyer funds escrow
    fund_escrow(buyer.id, escrow.id, 1000000).await?;

    // Step 7: Wait for blockchain monitor to detect funding
    wait_for_status(escrow.id, "funded", 120).await?;

    // Step 8: Buyer releases funds to vendor
    let tx_hash = release_funds(buyer.id, escrow.id, vendor.address).await?;
    assert!(tx_hash.len() == 64); // Monero tx hash length

    // Step 9: Wait for confirmations
    wait_for_status(escrow.id, "releasing", 10).await?;

    // Step 10: Mock confirmations (blockchain monitor polls)
    mock_confirmations(tx_hash, 10).await?;

    // Step 11: Escrow should be completed
    wait_for_status(escrow.id, "completed", 30).await?;

    Ok(())
}

#[tokio::test]
async fn test_dispute_flow() -> Result<()> {
    // Similar setup
    let (buyer, vendor, arbiter) = setup_test_users().await?;

    // ... (steps 1-7 same as above)

    // Step 8: Buyer opens dispute instead of releasing
    open_dispute(buyer.id, escrow.id, "Product not as described").await?;
    assert_eq!(get_escrow_status(escrow.id).await?, "disputed");

    // Step 9: Arbiter resolves in favor of buyer
    resolve_dispute(arbiter.id, escrow.id, "buyer").await?;

    // Step 10: Auto-refund triggered
    wait_for_status(escrow.id, "refunding", 30).await?;

    // Step 11: Wait for confirmations
    mock_confirmations(last_tx_hash, 10).await?;
    wait_for_status(escrow.id, "refunded", 30).await?;

    Ok(())
}
```

### ✅ VALIDATION MILESTONE 4

**Tests E2E :**
```bash
cargo test --package server --test escrow_e2e test_complete_escrow_flow
cargo test --package server --test escrow_e2e test_dispute_flow
```

**Critères d'acceptance :**
- [ ] test_complete_escrow_flow() passe (release normal)
- [ ] test_dispute_flow() passe (dispute + refund)
- [ ] Tous les états intermédiaires vérifiés
- [ ] Aucun .unwrap() dans tests (use .expect())

---

## ✅ VALIDATION GLOBALE PHASE 3.2

### Checklist Complète

**Blockchain Monitor :**
- [ ] check_escrow_funding() implémenté
- [ ] check_transaction_confirmations() implémenté
- [ ] 2 fonctions DB ajoutées
- [ ] Tests unitaires passent

**Multisig Transactions :**
- [ ] WalletManager: sign/finalize/broadcast
- [ ] release_funds() complet (2-of-3 signatures)
- [ ] refund_funds() complet
- [ ] Tests unitaires passent

**Dispute Resolution :**
- [ ] resolve_dispute() avec auto-trigger
- [ ] db_update_dispute_resolution()
- [ ] Tests passent

**Tests E2E :**
- [ ] test_complete_escrow_flow() passe
- [ ] test_dispute_flow() passe

**Score attendu :** 95/100

---

## 📋 PHASE 4 : FRONTEND HTMX (15 jours)

### Durée : 15 jours (5 milestones)

---

## 📋 MILESTONE 4.1 : Setup Frontend (3 jours)

### Objectif
Configuration template engine + HTMX + structure projet

### ✅ TÂCHES

#### Tâche 1.1 : Ajouter Dépendances
**Fichier:** `server/Cargo.toml`

**Ajouter :**
```toml
[dependencies]
# Existing dependencies...

# Template engine
tera = "1.19"
# Or askama = "0.12" (alternative)

# Static file serving
actix-files = "0.6"

# Session cookies (déjà présent)
actix-session = { version = "0.9", features = ["cookie-session"] }
```

#### Tâche 1.2 : Structure Dossiers
**Créer :**
```
templates/
├── base.html
├── partials/
│   ├── header.html
│   └── footer.html
├── auth/
│   ├── login.html
│   └── register.html
├── listings/
│   ├── index.html
│   ├── show.html
│   └── create.html
├── orders/
│   ├── index.html
│   └── show.html
└── escrow/
    └── show.html

static/
├── css/
│   └── main.css
├── js/
│   └── htmx.min.js (CDN ou local)
└── images/
```

#### Tâche 1.3 : Template Base
**Fichier:** `templates/base.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Monero Marketplace{% endblock %}</title>
    <link rel="stylesheet" href="/static/css/main.css">
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body>
    {% include "partials/header.html" %}

    <main class="container">
        {% block content %}{% endblock %}
    </main>

    {% include "partials/footer.html" %}
</body>
</html>
```

#### Tâche 1.4 : Configuration Tera
**Fichier:** `server/src/main.rs`

```rust
use tera::Tera;
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize Tera
    let tera = Tera::new("templates/**/*.html")
        .expect("Failed to initialize Tera templates");

    HttpServer::new(move || {
        App::new()
            // Existing middleware...
            .app_data(web::Data::new(tera.clone()))

            // Static files
            .service(fs::Files::new("/static", "./static").show_files_listing())

            // API routes (existing)
            .configure(api_routes)

            // Frontend routes (new)
            .configure(frontend_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### ✅ VALIDATION MILESTONE 1

**Tests :**
```bash
# Compiler
cargo build --package server

# Vérifier templates parse
# Démarrer serveur et visiter http://localhost:8080/
```

**Critères d'acceptance :**
- [ ] Tera configuré et compile
- [ ] Structure dossiers créée
- [ ] Template base.html créé
- [ ] Static files servis à /static/

---

## 📋 MILESTONE 4.2 : Authentication Frontend (3 jours)

### Objectif
Pages login/register avec HTMX (zero full page reload)

### ✅ TÂCHES

#### Tâche 2.1 : Template Login
**Fichier:** `templates/auth/login.html`

```html
{% extends "base.html" %}

{% block title %}Login - Monero Marketplace{% endblock %}

{% block content %}
<div class="auth-container">
    <h1>Login</h1>

    <form
        hx-post="/api/auth/login"
        hx-target="#auth-result"
        hx-swap="innerHTML"
    >
        <div class="form-group">
            <label for="username">Username</label>
            <input type="text" id="username" name="username" required>
        </div>

        <div class="form-group">
            <label for="password">Password</label>
            <input type="password" id="password" name="password" required>
        </div>

        <button type="submit">Login</button>
    </form>

    <div id="auth-result"></div>

    <p>Don't have an account? <a href="/register">Register here</a></p>
</div>
{% endblock %}
```

#### Tâche 2.2 : Template Register
**Fichier:** `templates/auth/register.html`

```html
{% extends "base.html" %}

{% block title %}Register - Monero Marketplace{% endblock %}

{% block content %}
<div class="auth-container">
    <h1>Create Account</h1>

    <form
        hx-post="/api/auth/register"
        hx-target="#auth-result"
        hx-swap="innerHTML"
    >
        <div class="form-group">
            <label for="username">Username</label>
            <input type="text" id="username" name="username" required minlength="3">
        </div>

        <div class="form-group">
            <label for="password">Password</label>
            <input type="password" id="password" name="password" required minlength="8">
        </div>

        <div class="form-group">
            <label for="role">Role</label>
            <select id="role" name="role" required>
                <option value="buyer">Buyer</option>
                <option value="vendor">Vendor</option>
            </select>
        </div>

        <button type="submit">Register</button>
    </form>

    <div id="auth-result"></div>

    <p>Already have an account? <a href="/login">Login here</a></p>
</div>
{% endblock %}
```

#### Tâche 2.3 : Frontend Handlers
**Fichier:** `server/src/handlers/frontend.rs`

```rust
use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use tera::{Tera, Context};

/// GET /login
pub async fn show_login(tera: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tera.render("auth/login.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// GET /register
pub async fn show_register(tera: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tera.render("auth/register.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// GET / (homepage)
pub async fn index(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let mut ctx = Context::new();

    // Check if user is logged in
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
    } else {
        ctx.insert("logged_in", &false);
    }

    match tera.render("listings/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}
```

#### Tâche 2.4 : Routes Configuration
**Fichier:** `server/src/main.rs`

```rust
fn frontend_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(frontend::index))
        .route("/login", web::get().to(frontend::show_login))
        .route("/register", web::get().to(frontend::show_register))
        .route("/logout", web::post().to(frontend::logout));
}
```

### ✅ VALIDATION MILESTONE 2

**Tests manuels :**
```bash
# Démarrer serveur
cargo run --package server

# Visiter pages
http://localhost:8080/login
http://localhost:8080/register

# Tester login/register via HTMX
```

**Critères d'acceptance :**
- [ ] Page /login affichée
- [ ] Page /register affichée
- [ ] HTMX login fonctionne (pas de reload)
- [ ] HTMX register fonctionne
- [ ] Session cookies persistés

---

## 📋 MILESTONE 4.3 : Listings Frontend (3 jours)

### Objectif
Interface pour browse/search/create listings

### ✅ TÂCHES

#### Tâche 3.1 : Listings Index
**Fichier:** `templates/listings/index.html`

```html
{% extends "base.html" %}

{% block content %}
<div class="listings-container">
    <h1>Browse Listings</h1>

    <!-- Search bar -->
    <div class="search-bar">
        <input
            type="search"
            name="search"
            placeholder="Search listings..."
            hx-get="/api/listings/search"
            hx-trigger="keyup changed delay:500ms"
            hx-target="#listings-results"
            hx-swap="innerHTML"
        >
    </div>

    <!-- Listings grid -->
    <div id="listings-results" class="listings-grid">
        {% for listing in listings %}
        <div class="listing-card">
            <h3>{{ listing.title }}</h3>
            <p>{{ listing.description | truncate(length=100) }}</p>
            <p class="price">{{ listing.price_xmr | format_xmr }} XMR</p>
            <a href="/listings/{{ listing.id }}" class="btn">View Details</a>
        </div>
        {% endfor %}
    </div>

    {% if logged_in and role == "vendor" %}
    <a href="/listings/new" class="btn btn-primary">Create Listing</a>
    {% endif %}
</div>
{% endblock %}
```

#### Tâche 3.2 : Create Listing Form
**Fichier:** `templates/listings/create.html`

```html
{% extends "base.html" %}

{% block content %}
<div class="listing-form-container">
    <h1>Create New Listing</h1>

    <form
        hx-post="/api/listings"
        hx-target="#listing-result"
    >
        <div class="form-group">
            <label for="title">Title</label>
            <input type="text" id="title" name="title" required maxlength="200">
        </div>

        <div class="form-group">
            <label for="description">Description</label>
            <textarea id="description" name="description" required rows="5"></textarea>
        </div>

        <div class="form-group">
            <label for="price_xmr">Price (XMR)</label>
            <input type="number" id="price_xmr" name="price_xmr" step="0.000000000001" required>
        </div>

        <div class="form-group">
            <label for="stock">Stock</label>
            <input type="number" id="stock" name="stock" min="1" required>
        </div>

        <button type="submit" class="btn btn-primary">Create Listing</button>
    </form>

    <div id="listing-result"></div>
</div>
{% endblock %}
```

#### Tâche 3.3 : Handler Listings
**Fichier:** `server/src/handlers/frontend.rs`

```rust
/// GET /listings
pub async fn show_listings(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    let mut ctx = Context::new();

    // Get all listings from DB
    let listings = match db_get_all_listings(&pool, 0, 50).await {
        Ok(listings) => listings,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load listings: {}", e));
        }
    };

    ctx.insert("listings", &listings);

    // Check auth
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    }

    match tera.render("listings/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// GET /listings/new
pub async fn show_create_listing(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    // Check auth + role vendor
    let role = match session.get::<String>("role") {
        Ok(Some(r)) => r,
        _ => return HttpResponse::Unauthorized().body("Must be logged in as vendor"),
    };

    if role != "vendor" {
        return HttpResponse::Forbidden().body("Only vendors can create listings");
    }

    let ctx = Context::new();

    match tera.render("listings/create.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}
```

### ✅ VALIDATION MILESTONE 3

**Tests manuels :**
```bash
# Démarrer serveur
cargo run --package server

# Visiter
http://localhost:8080/
http://localhost:8080/listings/new (en tant que vendor)

# Tester search HTMX (keyup delay)
```

**Critères d'acceptance :**
- [ ] Page listings index affichée
- [ ] Search HTMX avec debounce fonctionne
- [ ] Create listing (vendors only)
- [ ] Listings affichés en grille

---

## 📋 MILESTONE 4.4 : Orders & Escrow Frontend (4 jours)

### Objectif
Interface pour créer orders + suivre escrow status

### ✅ TÂCHES

#### Tâche 4.1 : Order Creation (via listing)
**Fichier:** `templates/listings/show.html`

```html
{% extends "base.html" %}

{% block content %}
<div class="listing-detail">
    <h1>{{ listing.title }}</h1>
    <p class="price">{{ listing.price_xmr | format_xmr }} XMR</p>
    <p>{{ listing.description }}</p>
    <p>Stock: {{ listing.stock }}</p>
    <p>Vendor: {{ vendor.username }}</p>

    {% if logged_in and role == "buyer" %}
    <form
        hx-post="/api/orders"
        hx-target="#order-result"
    >
        <input type="hidden" name="listing_id" value="{{ listing.id }}">

        <div class="form-group">
            <label for="quantity">Quantity</label>
            <input type="number" id="quantity" name="quantity" min="1" max="{{ listing.stock }}" value="1">
        </div>

        <button type="submit" class="btn btn-primary">Create Order</button>
    </form>

    <div id="order-result"></div>
    {% endif %}
</div>
{% endblock %}
```

#### Tâche 4.2 : Escrow Status Page
**Fichier:** `templates/escrow/show.html`

```html
{% extends "base.html" %}

{% block content %}
<div class="escrow-detail">
    <h1>Escrow #{{ escrow.id | truncate(length=8) }}</h1>

    <!-- Status indicator -->
    <div class="status-badge status-{{ escrow.status }}">
        Status: {{ escrow.status | upper }}
    </div>

    <!-- Timeline -->
    <div class="timeline">
        <div class="step {% if escrow.status == 'created' %}active{% endif %}">
            1. Created
        </div>
        <div class="step {% if escrow.status == 'ready' %}active{% endif %}">
            2. Multisig Ready
        </div>
        <div class="step {% if escrow.status == 'funded' %}active{% endif %}">
            3. Funded
        </div>
        <div class="step {% if escrow.status == 'releasing' or escrow.status == 'completed' %}active{% endif %}">
            4. Releasing
        </div>
        <div class="step {% if escrow.status == 'completed' %}active{% endif %}">
            5. Completed
        </div>
    </div>

    <!-- Multisig address (if ready) -->
    {% if escrow.multisig_address %}
    <div class="multisig-info">
        <h3>Multisig Address</h3>
        <code>{{ escrow.multisig_address }}</code>
        <p>Amount: {{ escrow.amount | format_xmr }} XMR</p>
    </div>
    {% endif %}

    <!-- Actions based on status & role -->
    {% if escrow.status == "created" and is_party %}
    <div class="actions">
        <h3>Action Required</h3>
        <form
            hx-post="/api/escrow/{{ escrow.id }}/prepare"
            hx-target="#escrow-result"
        >
            <div class="form-group">
                <label for="multisig_info">Multisig Info (from your wallet)</label>
                <textarea id="multisig_info" name="multisig_info" required rows="5"></textarea>
            </div>
            <button type="submit" class="btn btn-primary">Submit Multisig Info</button>
        </form>
    </div>
    {% endif %}

    {% if escrow.status == "funded" and user_id == escrow.buyer_id %}
    <div class="actions">
        <button
            hx-post="/api/escrow/{{ escrow.id }}/release"
            hx-confirm="Release funds to vendor?"
            hx-target="#escrow-result"
            class="btn btn-success"
        >
            Release Funds
        </button>

        <button
            hx-post="/api/escrow/{{ escrow.id }}/dispute"
            hx-target="#dispute-form"
            class="btn btn-warning"
        >
            Open Dispute
        </button>
    </div>

    <div id="dispute-form"></div>
    {% endif %}

    {% if escrow.status == "disputed" and user_id == escrow.arbiter_id %}
    <div class="actions">
        <h3>Resolve Dispute</h3>
        <form
            hx-post="/api/escrow/{{ escrow.id }}/resolve"
            hx-target="#escrow-result"
        >
            <div class="form-group">
                <label>Decision:</label>
                <label>
                    <input type="radio" name="resolution" value="buyer" required> Refund Buyer
                </label>
                <label>
                    <input type="radio" name="resolution" value="vendor" required> Release to Vendor
                </label>
            </div>
            <button type="submit" class="btn btn-primary">Resolve Dispute</button>
        </form>
    </div>
    {% endif %}

    <div id="escrow-result"></div>

    <!-- WebSocket live updates -->
    <script>
        // Connect to WebSocket for real-time status updates
        // (optional, mais recommandé pour UX)
    </script>
</div>
{% endblock %}
```

#### Tâche 4.3 : Handler Escrow Frontend
**Fichier:** `server/src/handlers/frontend.rs`

```rust
/// GET /escrow/{id}
pub async fn show_escrow(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<String>,
) -> impl Responder {
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid escrow ID"),
    };

    // Get user_id from session
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().body("Not authenticated"),
    };

    // Load escrow
    let escrow = match db_load_escrow(&pool, escrow_id).await {
        Ok(e) => e,
        Err(e) => return HttpResponse::NotFound().body(format!("Escrow not found: {}", e)),
    };

    // Verify user is party to escrow
    let is_party = user_id_str == escrow.buyer_id
        || user_id_str == escrow.vendor_id
        || user_id_str == escrow.arbiter_id;

    if !is_party {
        return HttpResponse::Forbidden().body("You are not authorized to view this escrow");
    }

    let mut ctx = Context::new();
    ctx.insert("escrow", &escrow);
    ctx.insert("user_id", &user_id_str);
    ctx.insert("is_party", &is_party);

    match tera.render("escrow/show.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}
```

### ✅ VALIDATION MILESTONE 4

**Tests manuels :**
```bash
# Créer order → escrow auto-created
# Visiter /escrow/{id}
# Tester actions HTMX (prepare, release, dispute)
```

**Critères d'acceptance :**
- [ ] Page escrow/show affichée
- [ ] Timeline visuelle (5 étapes)
- [ ] Actions conditionnelles selon role
- [ ] HTMX forms fonctionnels

---

## 📋 MILESTONE 4.5 : CSS Styling (2 jours)

### Objectif
Design responsive simple et propre

### ✅ TÂCHES

#### Tâche 5.1 : Main CSS
**Fichier:** `static/css/main.css`

```css
/* Reset & Base */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

/* Header */
header {
    background: #2c3e50;
    color: white;
    padding: 1rem 0;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

header nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

/* Buttons */
.btn {
    display: inline-block;
    padding: 10px 20px;
    background: #3498db;
    color: white;
    text-decoration: none;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    transition: background 0.3s;
}

.btn:hover {
    background: #2980b9;
}

.btn-primary {
    background: #27ae60;
}

.btn-primary:hover {
    background: #229954;
}

.btn-warning {
    background: #f39c12;
}

.btn-success {
    background: #27ae60;
}

/* Listings Grid */
.listings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 20px;
    margin-top: 20px;
}

.listing-card {
    background: white;
    padding: 20px;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    transition: transform 0.2s;
}

.listing-card:hover {
    transform: translateY(-4px);
}

.price {
    font-size: 1.5rem;
    color: #27ae60;
    font-weight: bold;
}

/* Forms */
.form-group {
    margin-bottom: 1rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
}

.form-group input,
.form-group textarea,
.form-group select {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
}

/* Escrow Timeline */
.timeline {
    display: flex;
    justify-content: space-between;
    margin: 30px 0;
    position: relative;
}

.timeline::before {
    content: '';
    position: absolute;
    top: 20px;
    left: 0;
    right: 0;
    height: 2px;
    background: #ddd;
    z-index: 0;
}

.step {
    background: white;
    padding: 10px 20px;
    border-radius: 20px;
    border: 2px solid #ddd;
    position: relative;
    z-index: 1;
}

.step.active {
    border-color: #27ae60;
    background: #27ae60;
    color: white;
}

/* Status Badges */
.status-badge {
    display: inline-block;
    padding: 8px 16px;
    border-radius: 20px;
    font-weight: bold;
    margin: 10px 0;
}

.status-created { background: #3498db; color: white; }
.status-ready { background: #9b59b6; color: white; }
.status-funded { background: #f39c12; color: white; }
.status-releasing { background: #e67e22; color: white; }
.status-completed { background: #27ae60; color: white; }
.status-disputed { background: #e74c3c; color: white; }

/* Responsive */
@media (max-width: 768px) {
    .listings-grid {
        grid-template-columns: 1fr;
    }

    .timeline {
        flex-direction: column;
    }

    .timeline::before {
        display: none;
    }
}
```

### ✅ VALIDATION MILESTONE 5

**Tests visuels :**
```bash
# Visiter toutes les pages
# Vérifier responsive (mobile + desktop)
# Tester dark mode (optionnel)
```

**Critères d'acceptance :**
- [ ] CSS responsive (mobile + desktop)
- [ ] Listings grid propre
- [ ] Timeline escrow visuelle
- [ ] Buttons styled
- [ ] Forms styled

---

## ✅ VALIDATION GLOBALE PHASE 4

### Checklist Complète

**Setup :**
- [ ] Tera configuré
- [ ] Structure templates/ créée
- [ ] Static files servis

**Auth :**
- [ ] Page /login
- [ ] Page /register
- [ ] HTMX login/register fonctionnel

**Listings :**
- [ ] Page /listings (index)
- [ ] Page /listings/new (create)
- [ ] Search HTMX avec debounce

**Orders & Escrow :**
- [ ] Create order via listing
- [ ] Page /escrow/{id}
- [ ] Timeline visuelle
- [ ] Actions conditionnelles (prepare, release, dispute)

**Styling :**
- [ ] CSS responsive
- [ ] Design propre et moderne

---

## 🎯 WORKFLOW COORDINATION

### Pendant votre travail (Phase 3.2 + 4)

**Fichiers que VOUS modifiez :**
```
server/src/
├── services/
│   ├── blockchain_monitor.rs (compléter logic)
│   └── escrow.rs (compléter transactions)
├── wallet_manager.rs (sign/finalize/broadcast)
├── handlers/
│   └── frontend.rs (NOUVEAU - routes templates)
├── db/mod.rs (nouvelles fonctions)
└── main.rs (config Tera)

server/tests/
└── escrow_e2e.rs (NOUVEAU - tests E2E)

templates/ (NOUVEAU dossier)
├── base.html
├── auth/
├── listings/
├── orders/
└── escrow/

static/ (NOUVEAU dossier)
├── css/main.css
└── js/

server/Cargo.toml (ajouter deps: tera, actix-files)
```

**Fichiers que vous NE touchez PAS :**
```
4.5/ (TOUT le dossier - réservé à Gemini)
├── docker/
├── monitoring/
├── scripts/
├── ci-cd/
├── docs/
└── ...
```

### Après Phase 3.2 + 4 complètes

**Vous me dites : "Phase 3.2 + 4 terminées"**

**Je (Claude) ferai :**
1. Review code (blockchain monitor, multisig, frontend)
2. Tester compilation
3. Vérifier tests E2E passent
4. Valider frontend (templates Tera)
5. Commit : "feat: Phase 3.2 + 4 Complete - Escrow finalization + Frontend HTMX"

### Après que Gemini finit Phase 4.5

**Gemini vous dit : "Milestone 4.5.X terminé"**

**Je (Claude) vérifierai :**
1. Fichiers créés dans `4.5/`
2. Syntaxe bash/YAML/Docker
3. Cohérence configurations
4. Donner feedback à Gemini

**Quand Gemini finit tout :**

**Je (Claude) ferai :**
1. Review complète dossier `4.5/`
2. Déplacer fichiers vers racine :
   ```bash
   mv 4.5/docker/Dockerfile ./
   mv 4.5/docker/docker-compose.yml ./
   mv 4.5/monitoring/ ./monitoring/
   mv 4.5/scripts/* ./scripts/
   # etc.
   ```
3. Intégrer metrics.rs dans server/src/
4. Tester docker build + docker-compose up
5. Commit : "feat: Phase 4.5 Infrastructure Complete"

---

## 🚀 COMMENCEZ MAINTENANT

**Votre première action :**
```
Commencer Milestone 3.2.1 : Blockchain Monitor Logic
→ Fichier: server/src/services/blockchain_monitor.rs
→ Compléter check_escrow_funding() (lignes 151-167)
```

**Bonne chance ! Je suis prêt à valider votre travail après chaque milestone.**
