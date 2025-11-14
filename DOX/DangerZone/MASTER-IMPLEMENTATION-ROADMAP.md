# MASTER IMPLEMENTATION ROADMAP
## Monero Marketplace - Production Launch Plan

**Document Version:** 1.0 UNIFIED
**Date:** 2025-11-12
**Status:** 🚀 READY TO START
**Architecture:** 100% Non-Custodial + Generic Arbitration System
**Timeline:** 9 days (Day 0 + 8 implementation days)

---

## 📋 TABLE OF CONTENTS

1. [Executive Summary](#executive-summary)
2. [Day 0: Blocker Resolution](#day-0-blocker-resolution-7h)
3. [Phase 1: Race Condition Fix](#phase-1-race-condition-fix-2-days)
4. [Phase 2: Arbitration System](#phase-2-arbitration-system-4-days)
5. [Phase 3: Monitoring Infrastructure](#phase-3-monitoring-infrastructure-1-day)
6. [Phase 4: Beta Validation](#phase-4-beta-validation-1-day)
7. [Complete Implementation Timeline](#complete-implementation-timeline)
8. [Code Appendices](#code-appendices)

---

## 🔴 EXECUTIVE SUMMARY

### Mission
Transform Monero Marketplace from "testnet alpha" to "production-ready beta" with complete dispute resolution, monitoring, and fault tolerance.

### Architecture Principles

**100% Non-Custodial:**
- Server does NOT control any wallet keys (including arbiter)
- All funds remain under user control via 2-of-3 multisig
- Manual arbitration for disputes (no auto-refund)

**Generic Arbiter Design:**
- **v1 (Beta):** Single admin arbiter (fast iteration)
- **v2 (Production):** Random arbiter pool (decentralized)
- Code written generically to support both without refactoring

**Security First:**
- Zero-tolerance for race conditions
- All failure modes have documented recovery paths
- Comprehensive monitoring without exposing infrastructure

### Critical Blockers Resolved in This Plan

**BLOCKER #1:** Race condition on wallet RPC mappings → **FIXED in Phase 1** (2 days)
**BLOCKER #2:** No dispute resolution mechanism → **FIXED in Phase 2** (4 days)
**BLOCKER #3:** No production monitoring → **FIXED in Phase 3** (1 day)

**Pre-Phase Blockers:** 3 technical issues → **FIXED in Day 0** (7 hours)

### Timeline Summary

```
Day 0:    Blocker Resolution          [7 hours]
Phase 1:  Race Condition Fix          [2 days]
Phase 2:  Arbitration System          [4 days]
Phase 3:  Monitoring Infrastructure   [1 day]
Phase 4:  Beta Validation             [1 day]
─────────────────────────────────────────────
TOTAL:    9 calendar days (120h effective work)
```

### Beta Launch Limits

- **Max per escrow:** 5 XMR (~€1000)
- **Total beta volume:** 50 XMR max during first month
- **Testnet validation:** Minimum 100 transactions before mainnet
- **Monitoring:** 24/7 for first 2 weeks
- **Dispute arbitration:** Manual admin intervention (v1)

### What This Plan Delivers

✅ **Database migrations** for race condition fix
✅ **Complete arbitration portal** with evidence collection
✅ **IPFS integration** for dispute photos
✅ **Fee distribution** via simplified fee sweep (2% arbiter fee)
✅ **Production monitoring** via Prometheus + Grafana + Tor
✅ **Admin dashboard** for dispute management
✅ **Comprehensive tests** for all new functionality
✅ **Documentation** for operators and arbiters

---

## 🔧 DAY 0: BLOCKER RESOLUTION (7h)

**Purpose:** Fix 3 technical blockers preventing Phase 2 implementation.

### BLOCKER #1: Arbiter Fee Distribution (2-3h)

**Problem:** Need to collect 2% fee from losing party for arbiter compensation.

**Solution:** Two-Transaction Fee Sweep (simplified approach)

**Why Not Multi-Output?** Original plan required complex multi-output multisig (6-8h). Simplified to TWO single-output transactions using existing code.

**Process:**
1. Release `(amount - fee)` to winner
2. Wait 120 seconds for confirmation
3. Sweep remaining balance to arbiter

**Implementation:**

Add to `wallet/src/client.rs`:
```rust
impl MoneroClient {
    /// Get balance of currently opened wallet
    pub async fn get_balance(&self) -> Result<(u64, u64), MoneroError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_balance"
        });

        let response = self.rpc_client
            .post(&format!("{}/json_rpc", self.rpc_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| MoneroError::NetworkError(e.to_string()))?;

        let rpc_response: RpcResponse<BalanceResponse> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(e.to_string()))?;

        if let Some(error) = rpc_response.error {
            return Err(MoneroError::RpcError(error.message));
        }

        let result = rpc_response.result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result".to_string()))?;

        Ok((result.balance, result.unlocked_balance))
    }
}

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    balance: u64,
    unlocked_balance: u64,
}
```

Add to `server/src/services/escrow.rs`:
```rust
impl EscrowOrchestrator {
    /// Release funds with arbiter fee automatically deducted
    pub async fn release_with_arbiter_fee(
        &self,
        escrow_id: Uuid,
        winner_address: String,
        arbiter_address: String,
        fee_percentage: f64,
    ) -> Result<(String, String)> {
        let escrow = self.load_escrow(escrow_id).await?;
        let total_amount = escrow.amount as u64;

        let fee_amount = (total_amount as f64 * fee_percentage) as u64;
        let winner_amount = total_amount - fee_amount;

        info!(
            escrow_id = %escrow_id,
            total_atomic = total_amount,
            winner_atomic = winner_amount,
            fee_atomic = fee_amount,
            "Releasing funds with arbiter fee"
        );

        // Transaction 1: Winner gets (amount - fee)
        let winner_tx_hash = self.create_and_sign_multisig_tx(
            escrow_id,
            winner_address,
            winner_amount,
        ).await?;

        // Wait for confirmation
        tokio::time::sleep(Duration::from_secs(120)).await;

        // Transaction 2: Sweep remaining to arbiter
        let fee_tx_hash = self.sweep_multisig_to_arbiter(
            escrow_id,
            arbiter_address,
        ).await?;

        Ok((winner_tx_hash, fee_tx_hash))
    }

    async fn sweep_multisig_to_arbiter(
        &self,
        escrow_id: Uuid,
        arbiter_address: String,
    ) -> Result<String> {
        let wallet_manager = self.wallet_manager.lock().await;
        let balance = wallet_manager
            .get_multisig_balance(escrow_id)
            .await?;

        if balance == 0 {
            return Err(anyhow::anyhow!("No balance to sweep"));
        }

        self.create_and_sign_multisig_tx(
            escrow_id,
            arbiter_address,
            balance,
        ).await
    }
}
```

Add to `server/src/wallet_manager.rs`:
```rust
impl WalletManager {
    /// Get balance of multisig wallet for an escrow
    pub async fn get_multisig_balance(&self, escrow_id: Uuid) -> Result<u64> {
        let wallet_id = self.get_wallet_id_for_escrow(escrow_id)?;
        let client = self.get_client_for_wallet(wallet_id)?;
        let (_, unlocked) = client.get_balance().await?;
        Ok(unlocked)
    }
}
```

**Testing:**
```bash
# Compile
cargo build --release --package wallet
cargo build --release --package server

# Unit tests
cargo test --package wallet test_get_balance
cargo test --package server test_fee_sweep
```

### BLOCKER #2: IPFS Upload Handler (2-3h)

**Problem:** Need to upload dispute evidence (photos) to IPFS.

**Solution:** Extend existing IPFS client with upload functionality.

**Implementation:**

Add to `server/src/ipfs/client.rs`:
```rust
impl IpfsClient {
    /// Upload dispute evidence to IPFS
    ///
    /// # Arguments
    /// * `file_data` - Raw bytes of the file
    /// * `filename` - Original filename for metadata
    ///
    /// # Returns
    /// IPFS hash (CID v1)
    pub async fn upload_dispute_evidence(
        &self,
        file_data: Vec<u8>,
        filename: String,
    ) -> Result<String> {
        // Validate file size (max 10MB)
        if file_data.len() > 10 * 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large (max 10MB)"));
        }

        // Validate file type (images only)
        let content_type = self.detect_content_type(&file_data)?;
        if !content_type.starts_with("image/") {
            return Err(anyhow::anyhow!("Only images allowed"));
        }

        // Upload to IPFS
        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_data)
                    .file_name(filename)
                    .mime_str(&content_type)?,
            );

        let response = self.client
            .post(&format!("{}/api/v0/add", self.base_url))
            .multipart(form)
            .send()
            .await
            .context("IPFS upload failed")?;

        let result: IpfsAddResponse = response.json().await?;

        info!(
            ipfs_hash = %result.hash,
            size_bytes = result.size,
            "Dispute evidence uploaded to IPFS"
        );

        Ok(result.hash)
    }

    fn detect_content_type(&self, data: &[u8]) -> Result<String> {
        let kind = infer::get(data)
            .ok_or_else(|| anyhow::anyhow!("Unknown file type"))?;

        Ok(kind.mime_type().to_string())
    }
}

#[derive(Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Size")]
    size: String,
}
```

**Dependencies:**
Add to `server/Cargo.toml`:
```toml
infer = "0.15"  # File type detection
```

**Testing:**
```bash
# Ensure IPFS daemon is running
ipfs daemon &

# Run tests
cargo test --package server test_ipfs_upload
```

### BLOCKER #3: Arbiter Configuration (30 min)

**Problem:** No arbiter user_id configured for v1 beta.

**Solution:** Add configuration for single admin arbiter.

**Implementation:**

Add to `server/src/config/mod.rs`:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ArbiterConfig {
    /// User ID of the arbiter (admin for v1)
    pub arbiter_user_id: String,

    /// Arbiter's Monero address for fee collection
    pub arbiter_address: String,

    /// Fee percentage (0.02 = 2%)
    pub fee_percentage: f64,

    /// v1 = single admin, v2 = random pool
    pub mode: String,
}

impl Config {
    pub fn arbiter(&self) -> &ArbiterConfig {
        &self.arbiter
    }
}
```

Add to `.env`:
```bash
# Arbiter Configuration (v1 Beta)
ARBITER_USER_ID=admin_user_12345
ARBITER_ADDRESS=your_monero_address_here
ARBITER_FEE_PERCENTAGE=0.02
ARBITER_MODE=single_admin
```

**Testing:**
```bash
# Validate config loads
cargo run --package server --bin validate_config
```

### Day 0 Checklist

```
✅ BLOCKER #1: Fee sweep implementation (2-3h)
✅ BLOCKER #2: IPFS upload handler (2-3h)
✅ BLOCKER #3: Arbiter config (30 min)
✅ All tests passing
✅ Configuration validated
✅ IPFS daemon running
─────────────────────────────────
Total: 7 hours → Ready for Phase 1
```

---

## 📦 PHASE 1: RACE CONDITION FIX (2 days)

**Duration:** 16 hours effective work
**Priority:** P0 - Must complete before any other work
**Goal:** Eliminate race condition on wallet RPC mappings

### Problem Statement

**Current Code Issue** (`server/src/services/escrow.rs:115-120`):
```rust
.register_client_wallet_rpc(
    "temp-escrow-needs-refactor",  // ❌ HARDCODED STRING
    role,
    rpc_url.clone(),
    rpc_user,
    rpc_password,
    "manual",  // ❌ Recovery mode hardcoded
)
```

**Race Condition Scenario:**
```
T+0ms:  User A creates Escrow_1 (buyer)
T+5ms:  User B creates Escrow_2 (buyer)
T+10ms: Both register wallet RPC with role "buyer"
T+15ms: Wallet manager assigns SAME wallet_id to different escrows
T+20ms: Escrow_1 and Escrow_2 share wallet → FUNDS LOSS RISK
```

**Why Critical:**
- No unique constraint on (escrow_id, role) mapping
- HashMap in memory can be overwritten by concurrent requests
- Server restart = mapping lost (no persistence)
- Funds sent to wrong escrow = irrecoverable

### Solution Architecture

#### 1.1 Database Schema Changes

**New Table: `escrow_wallet_mappings`**
```sql
CREATE TABLE escrow_wallet_mappings (
    -- Primary key: one row per escrow
    escrow_id TEXT PRIMARY KEY NOT NULL,

    -- Buyer wallet registration (client-provided RPC)
    buyer_user_id TEXT NOT NULL,
    buyer_rpc_url TEXT NOT NULL,
    buyer_rpc_user TEXT,
    buyer_rpc_password TEXT,  -- Encrypted
    buyer_wallet_id TEXT,     -- Set after registration
    buyer_registered_at TIMESTAMP,

    -- Vendor wallet registration (client-provided RPC)
    vendor_user_id TEXT NOT NULL,
    vendor_rpc_url TEXT NOT NULL,
    vendor_rpc_user TEXT,
    vendor_rpc_password TEXT,  -- Encrypted
    vendor_wallet_id TEXT,     -- Set after registration
    vendor_registered_at TIMESTAMP,

    -- Arbiter wallet (server-controlled for coordination only)
    arbiter_rpc_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:18082/json_rpc',
    arbiter_wallet_id TEXT,
    arbiter_registered_at TIMESTAMP,

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Foreign keys
    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE,
    FOREIGN KEY (buyer_user_id) REFERENCES users(id),
    FOREIGN KEY (vendor_user_id) REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX idx_escrow_wallet_buyer ON escrow_wallet_mappings(buyer_user_id);
CREATE INDEX idx_escrow_wallet_vendor ON escrow_wallet_mappings(vendor_user_id);
CREATE UNIQUE INDEX idx_escrow_wallet_unique ON escrow_wallet_mappings(escrow_id);
```

**Migration File:** `migrations/{timestamp}_create_escrow_wallet_mappings/up.sql`

**Why This Schema?**
- ✅ One row per escrow (no race condition possible)
- ✅ Unique constraint enforced at DB level
- ✅ Passwords encrypted (existing encryption_key)
- ✅ Tracks registration timestamp (debugging)
- ✅ Cascade delete (cleanup on escrow deletion)

#### 1.2 Rust Model Changes

**New File: `server/src/models/escrow_wallet_mapping.rs`**
```rust
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::escrow_wallet_mappings;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = escrow_wallet_mappings)]
#[diesel(primary_key(escrow_id))]
pub struct EscrowWalletMapping {
    pub escrow_id: String,

    // Buyer
    pub buyer_user_id: String,
    pub buyer_rpc_url: String,
    pub buyer_rpc_user: Option<String>,
    pub buyer_rpc_password: Option<String>,  // Encrypted
    pub buyer_wallet_id: Option<String>,
    pub buyer_registered_at: Option<NaiveDateTime>,

    // Vendor
    pub vendor_user_id: String,
    pub vendor_rpc_url: String,
    pub vendor_rpc_user: Option<String>,
    pub vendor_rpc_password: Option<String>,  // Encrypted
    pub vendor_wallet_id: Option<String>,
    pub vendor_registered_at: Option<NaiveDateTime>,

    // Arbiter
    pub arbiter_rpc_url: String,
    pub arbiter_wallet_id: Option<String>,
    pub arbiter_registered_at: Option<NaiveDateTime>,

    // Metadata
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = escrow_wallet_mappings)]
pub struct NewEscrowWalletMapping {
    pub escrow_id: String,
    pub buyer_user_id: String,
    pub buyer_rpc_url: String,
    pub vendor_user_id: String,
    pub vendor_rpc_url: String,
    pub arbiter_rpc_url: String,
}

impl EscrowWalletMapping {
    /// Create new wallet mapping for an escrow
    pub fn create(
        conn: &mut SqliteConnection,
        new_mapping: NewEscrowWalletMapping,
    ) -> Result<Self> {
        diesel::insert_into(escrow_wallet_mappings::table)
            .values(&new_mapping)
            .execute(conn)
            .context("Failed to insert escrow wallet mapping")?;

        escrow_wallet_mappings::table
            .filter(escrow_wallet_mappings::escrow_id.eq(new_mapping.escrow_id))
            .first(conn)
            .context("Failed to retrieve created mapping")
    }

    /// Find mapping by escrow ID
    pub fn find_by_escrow_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
    ) -> Result<Self> {
        escrow_wallet_mappings::table
            .filter(escrow_wallet_mappings::escrow_id.eq(escrow_id.clone()))
            .first(conn)
            .context(format!("Mapping for escrow {} not found", escrow_id))
    }

    /// Update buyer wallet ID after registration
    pub fn set_buyer_wallet_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
        wallet_id: String,
    ) -> Result<()> {
        diesel::update(
            escrow_wallet_mappings::table
                .filter(escrow_wallet_mappings::escrow_id.eq(escrow_id))
        )
        .set((
            escrow_wallet_mappings::buyer_wallet_id.eq(Some(wallet_id)),
            escrow_wallet_mappings::buyer_registered_at.eq(Some(diesel::dsl::now)),
            escrow_wallet_mappings::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;
        Ok(())
    }

    /// Update vendor wallet ID after registration
    pub fn set_vendor_wallet_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
        wallet_id: String,
    ) -> Result<()> {
        diesel::update(
            escrow_wallet_mappings::table
                .filter(escrow_wallet_mappings::escrow_id.eq(escrow_id))
        )
        .set((
            escrow_wallet_mappings::vendor_wallet_id.eq(Some(wallet_id)),
            escrow_wallet_mappings::vendor_registered_at.eq(Some(diesel::dsl::now)),
            escrow_wallet_mappings::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;
        Ok(())
    }

    /// Update arbiter wallet ID after registration
    pub fn set_arbiter_wallet_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
        wallet_id: String,
    ) -> Result<()> {
        diesel::update(
            escrow_wallet_mappings::table
                .filter(escrow_wallet_mappings::escrow_id.eq(escrow_id))
        )
        .set((
            escrow_wallet_mappings::arbiter_wallet_id.eq(Some(wallet_id)),
            escrow_wallet_mappings::arbiter_registered_at.eq(Some(diesel::dsl::now)),
            escrow_wallet_mappings::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;
        Ok(())
    }
}
```

#### 1.3 WalletManager Refactor

**Update `server/src/wallet_manager.rs`:**

Replace in-memory HashMap with database-backed storage.

**Remove:**
```rust
// ❌ DELETE THIS
escrow_roles: Arc<RwLock<HashMap<String, EscrowRole>>>,
```

**Add:**
```rust
// ✅ ADD THIS
db_pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,
```

**Refactor `register_client_wallet_rpc`:**
```rust
pub async fn register_client_wallet_rpc(
    &mut self,
    escrow_id: Uuid,  // ✅ Changed from String
    role: EscrowRole,
    rpc_url: String,
    rpc_user: Option<String>,
    rpc_password: Option<String>,
) -> Result<String> {
    // 1. Validate RPC URL
    self.validate_rpc_url(&rpc_url)?;

    // 2. Load mapping from DB (atomic, no race condition)
    let mut conn = self.db_pool.get()?;
    let mut mapping = EscrowWalletMapping::find_by_escrow_id(
        &mut conn,
        escrow_id.to_string(),
    )?;

    // 3. Check if this role already registered
    match role {
        EscrowRole::Buyer if mapping.buyer_wallet_id.is_some() => {
            return Err(anyhow::anyhow!("Buyer wallet already registered"));
        }
        EscrowRole::Vendor if mapping.vendor_wallet_id.is_some() => {
            return Err(anyhow::anyhow!("Vendor wallet already registered"));
        }
        EscrowRole::Arbiter if mapping.arbiter_wallet_id.is_some() => {
            return Err(anyhow::anyhow!("Arbiter wallet already registered"));
        }
        _ => {}
    }

    // 4. Create MoneroClient for this RPC
    let client = MoneroClient::new_with_auth(
        rpc_url.clone(),
        rpc_user,
        rpc_password,
    )?;

    // 5. Generate unique wallet ID
    let wallet_id = format!("wallet_{}_{}", escrow_id, role.as_str());

    // 6. Store client
    self.clients.write().await.insert(wallet_id.clone(), client);

    // 7. Update database (atomic)
    match role {
        EscrowRole::Buyer => {
            EscrowWalletMapping::set_buyer_wallet_id(
                &mut conn,
                escrow_id.to_string(),
                wallet_id.clone(),
            )?;
        }
        EscrowRole::Vendor => {
            EscrowWalletMapping::set_vendor_wallet_id(
                &mut conn,
                escrow_id.to_string(),
                wallet_id.clone(),
            )?;
        }
        EscrowRole::Arbiter => {
            EscrowWalletMapping::set_arbiter_wallet_id(
                &mut conn,
                escrow_id.to_string(),
                wallet_id.clone(),
            )?;
        }
    }

    info!(
        escrow_id = %escrow_id,
        role = %role.as_str(),
        wallet_id = %wallet_id,
        "Registered client wallet RPC"
    );

    Ok(wallet_id)
}
```

**Add helper method:**
```rust
pub fn get_wallet_id_for_role(
    &self,
    escrow_id: Uuid,
    role: EscrowRole,
) -> Result<String> {
    let mut conn = self.db_pool.get()?;
    let mapping = EscrowWalletMapping::find_by_escrow_id(
        &mut conn,
        escrow_id.to_string(),
    )?;

    let wallet_id = match role {
        EscrowRole::Buyer => mapping.buyer_wallet_id,
        EscrowRole::Vendor => mapping.vendor_wallet_id,
        EscrowRole::Arbiter => mapping.arbiter_wallet_id,
    };

    wallet_id.ok_or_else(|| anyhow::anyhow!(
        "Wallet ID not found for role {} in escrow {}",
        role.as_str(),
        escrow_id
    ))
}
```

#### 1.4 EscrowOrchestrator Updates

**Update `server/src/services/escrow.rs`:**

**Fix hardcoded string:**
```rust
// ❌ OLD (with race condition)
coordinator.register_client_wallet(
    "temp-escrow-needs-refactor",
    EscrowRole::Buyer,
    "http://127.0.0.1:18083",
).await?;

// ✅ NEW (atomic, no race condition)
coordinator.register_client_wallet(
    escrow_id,  // UUID
    EscrowRole::Buyer,
    "http://127.0.0.1:18083",
).await?;
```

**Create mapping on escrow creation:**
```rust
pub async fn create_escrow(
    &self,
    order_id: Uuid,
    buyer_id: String,
    vendor_id: String,
    amount_atomic: u64,
) -> Result<Escrow> {
    let escrow_id = Uuid::new_v4();

    // 1. Create escrow record
    let mut conn = self.db_pool.get()?;
    let new_escrow = NewEscrow {
        id: escrow_id.to_string(),
        order_id: order_id.to_string(),
        buyer_id: buyer_id.clone(),
        vendor_id: vendor_id.clone(),
        arbiter_id: self.config.arbiter().arbiter_user_id.clone(),
        amount: amount_atomic as i64,
        status: "pending".to_string(),
    };

    let escrow = Escrow::create(&mut conn, new_escrow)?;

    // 2. Create wallet mapping (prevents race condition)
    let new_mapping = NewEscrowWalletMapping {
        escrow_id: escrow_id.to_string(),
        buyer_user_id: buyer_id,
        buyer_rpc_url: "".to_string(),  // Set during registration
        vendor_user_id: vendor_id,
        vendor_rpc_url: "".to_string(),  // Set during registration
        arbiter_rpc_url: "http://127.0.0.1:18082/json_rpc".to_string(),
    };

    EscrowWalletMapping::create(&mut conn, new_mapping)?;

    info!(
        escrow_id = %escrow_id,
        "Created escrow with wallet mapping"
    );

    Ok(escrow)
}
```

### Phase 1 Implementation Checklist

**Day 1 (8 hours):**
```
✅ Create migration files (up.sql, down.sql)
✅ Apply migration: DATABASE_URL=marketplace.db diesel migration run
✅ Regenerate schema: diesel print-schema > server/src/schema.rs
✅ Create models/escrow_wallet_mapping.rs
✅ Add mod escrow_wallet_mapping to models/mod.rs
✅ Write unit tests for model methods
✅ Test: cargo test --package server test_escrow_wallet_mapping
```

**Day 2 (8 hours):**
```
✅ Refactor WalletManager (remove HashMap, add DB pool)
✅ Update register_client_wallet_rpc method
✅ Add get_wallet_id_for_role helper
✅ Update EscrowOrchestrator to create mappings
✅ Fix all "temp-escrow-needs-refactor" references
✅ Write integration tests
✅ Test: cargo test --package server -- --test-threads=1
✅ Manual testing with concurrent requests
```

### Phase 1 Validation

**Concurrency Test:**
```bash
# Start server
./target/release/server &

# Launch 10 concurrent escrow creations
for i in {1..10}; do
  curl -X POST http://localhost:8080/api/escrows \
    -H "Content-Type: application/json" \
    -d '{"order_id":"'$(uuidgen)'","buyer_id":"buyer1","vendor_id":"vendor1","amount":100000000000}' &
done

# Wait for all requests
wait

# Check database integrity
sqlite3 marketplace.db "SELECT COUNT(DISTINCT escrow_id) FROM escrow_wallet_mappings;"
# Should be exactly 10

sqlite3 marketplace.db "SELECT escrow_id, buyer_wallet_id, vendor_wallet_id FROM escrow_wallet_mappings;"
# All wallet_ids should be unique
```

---

## ⚖️ PHASE 2: ARBITRATION SYSTEM (4 days)

**Duration:** 32 hours effective work
**Priority:** P0 - Core functionality for beta launch
**Goal:** Complete dispute resolution system with evidence collection and arbiter portal

### Architecture Overview

**Generic Arbiter Design:**
- Code written to support ANY arbiter assignment logic
- v1 Beta: Single admin arbiter (fast iteration)
- v2 Production: Random arbiter pool (no code changes)

**Dispute Flow:**
```
1. Buyer/Vendor opens dispute
2. Evidence collection (messages + IPFS photos)
3. Arbiter reviews case in admin portal
4. Arbiter makes decision (release or refund)
5. System executes with fee deduction (2% to arbiter)
```

### 2.1 Database Schema

**New Tables:**

```sql
-- Main dispute case table
CREATE TABLE dispute_cases (
    id TEXT PRIMARY KEY NOT NULL,
    escrow_id TEXT NOT NULL UNIQUE,
    buyer_id TEXT NOT NULL,
    vendor_id TEXT NOT NULL,
    arbiter_id TEXT NOT NULL,

    opened_by TEXT NOT NULL,  -- 'buyer' or 'vendor'
    opened_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    buyer_claim TEXT NOT NULL,
    vendor_claim TEXT,

    decision TEXT,  -- NULL, 'release', or 'refund'
    decided_at TIMESTAMP,

    winner_tx_hash TEXT,
    arbiter_fee_tx_hash TEXT,
    fee_atomic INTEGER,

    status TEXT NOT NULL DEFAULT 'open',  -- 'open', 'under_review', 'resolved'

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE,
    FOREIGN KEY (buyer_id) REFERENCES users(id),
    FOREIGN KEY (vendor_id) REFERENCES users(id),
    FOREIGN KEY (arbiter_id) REFERENCES users(id)
);

CREATE INDEX idx_dispute_cases_escrow ON dispute_cases(escrow_id);
CREATE INDEX idx_dispute_cases_arbiter ON dispute_cases(arbiter_id);
CREATE INDEX idx_dispute_cases_status ON dispute_cases(status);

-- Dispute messages (threaded conversation)
CREATE TABLE dispute_messages (
    id TEXT PRIMARY KEY NOT NULL,
    case_id TEXT NOT NULL,

    sender_id TEXT NOT NULL,
    sender_role TEXT NOT NULL,  -- 'buyer', 'vendor', 'arbiter'

    message TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (case_id) REFERENCES dispute_cases(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES users(id)
);

CREATE INDEX idx_dispute_messages_case ON dispute_messages(case_id);

-- Dispute evidence (photos via IPFS)
CREATE TABLE dispute_photos (
    id TEXT PRIMARY KEY NOT NULL,
    case_id TEXT NOT NULL,

    uploader_id TEXT NOT NULL,
    uploader_role TEXT NOT NULL,  -- 'buyer', 'vendor'

    ipfs_hash TEXT NOT NULL,
    description TEXT,

    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (case_id) REFERENCES dispute_cases(id) ON DELETE CASCADE,
    FOREIGN KEY (uploader_id) REFERENCES users(id)
);

CREATE INDEX idx_dispute_photos_case ON dispute_photos(case_id);
```

**Migration:**
```bash
diesel migration generate create_dispute_tables
# Edit up.sql with above schema
# Edit down.sql with DROP TABLE statements
DATABASE_URL=marketplace.db diesel migration run
diesel print-schema > server/src/schema.rs
```

### 2.2 Rust Models

**New File: `server/src/models/dispute_case.rs`**
```rust
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

use crate::schema::{dispute_cases, dispute_messages, dispute_photos};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = dispute_cases)]
pub struct DisputeCase {
    pub id: String,
    pub escrow_id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub arbiter_id: String,
    pub opened_by: String,
    pub opened_at: NaiveDateTime,
    pub buyer_claim: String,
    pub vendor_claim: Option<String>,
    pub decision: Option<String>,
    pub decided_at: Option<NaiveDateTime>,
    pub winner_tx_hash: Option<String>,
    pub arbiter_fee_tx_hash: Option<String>,
    pub fee_atomic: Option<i64>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = dispute_cases)]
pub struct NewDisputeCase {
    pub id: String,
    pub escrow_id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub arbiter_id: String,
    pub opened_by: String,
    pub buyer_claim: String,
    pub status: String,
}

impl DisputeCase {
    /// Create a new dispute case
    pub fn create(
        conn: &mut SqliteConnection,
        new_case: NewDisputeCase,
    ) -> Result<Self> {
        diesel::insert_into(dispute_cases::table)
            .values(&new_case)
            .execute(conn)
            .context("Failed to insert dispute case")?;

        dispute_cases::table
            .filter(dispute_cases::id.eq(new_case.id))
            .first(conn)
            .context("Failed to retrieve created dispute case")
    }

    /// Find dispute by ID
    pub fn find_by_id(
        conn: &mut SqliteConnection,
        case_id: String,
    ) -> Result<Self> {
        dispute_cases::table
            .filter(dispute_cases::id.eq(case_id.clone()))
            .first(conn)
            .context(format!("Dispute case {} not found", case_id))
    }

    /// Find dispute by escrow ID
    pub fn find_by_escrow_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
    ) -> Result<Self> {
        dispute_cases::table
            .filter(dispute_cases::escrow_id.eq(escrow_id.clone()))
            .first(conn)
            .context(format!("No dispute found for escrow {}", escrow_id))
    }

    /// Get all open disputes assigned to an arbiter
    pub fn find_by_arbiter(
        conn: &mut SqliteConnection,
        arbiter_id: String,
    ) -> Result<Vec<Self>> {
        dispute_cases::table
            .filter(dispute_cases::arbiter_id.eq(arbiter_id))
            .filter(dispute_cases::status.ne("resolved"))
            .order(dispute_cases::opened_at.desc())
            .load(conn)
            .context("Failed to load arbiter disputes")
    }

    /// Update status
    pub fn update_status(
        conn: &mut SqliteConnection,
        case_id: String,
        new_status: &str,
    ) -> Result<()> {
        diesel::update(dispute_cases::table.filter(dispute_cases::id.eq(case_id)))
            .set((
                dispute_cases::status.eq(new_status),
                dispute_cases::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)?;
        Ok(())
    }

    /// Record arbiter decision
    pub fn record_decision(
        conn: &mut SqliteConnection,
        case_id: String,
        decision: &str,  // "release" or "refund"
        winner_tx_hash: String,
        arbiter_fee_tx_hash: String,
        fee_atomic: u64,
    ) -> Result<()> {
        diesel::update(dispute_cases::table.filter(dispute_cases::id.eq(case_id)))
            .set((
                dispute_cases::decision.eq(Some(decision)),
                dispute_cases::decided_at.eq(Some(diesel::dsl::now)),
                dispute_cases::winner_tx_hash.eq(Some(winner_tx_hash)),
                dispute_cases::arbiter_fee_tx_hash.eq(Some(arbiter_fee_tx_hash)),
                dispute_cases::fee_atomic.eq(Some(fee_atomic as i64)),
                dispute_cases::status.eq("resolved"),
                dispute_cases::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)?;
        Ok(())
    }

    /// Add vendor response to dispute
    pub fn add_vendor_claim(
        conn: &mut SqliteConnection,
        case_id: String,
        vendor_claim: String,
    ) -> Result<()> {
        diesel::update(dispute_cases::table.filter(dispute_cases::id.eq(case_id)))
            .set((
                dispute_cases::vendor_claim.eq(Some(vendor_claim)),
                dispute_cases::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(table_name = dispute_messages)]
pub struct DisputeMessage {
    pub id: String,
    pub case_id: String,
    pub sender_id: String,
    pub sender_role: String,
    pub message: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = dispute_messages)]
pub struct NewDisputeMessage {
    pub id: String,
    pub case_id: String,
    pub sender_id: String,
    pub sender_role: String,
    pub message: String,
}

impl DisputeMessage {
    /// Add a message to a dispute
    pub fn create(
        conn: &mut SqliteConnection,
        new_message: NewDisputeMessage,
    ) -> Result<Self> {
        diesel::insert_into(dispute_messages::table)
            .values(&new_message)
            .execute(conn)?;

        dispute_messages::table
            .filter(dispute_messages::id.eq(new_message.id))
            .first(conn)
            .context("Failed to retrieve created message")
    }

    /// Get all messages for a dispute
    pub fn find_by_case(
        conn: &mut SqliteConnection,
        case_id: String,
    ) -> Result<Vec<Self>> {
        dispute_messages::table
            .filter(dispute_messages::case_id.eq(case_id))
            .order(dispute_messages::timestamp.asc())
            .load(conn)
            .context("Failed to load dispute messages")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(table_name = dispute_photos)]
pub struct DisputePhoto {
    pub id: String,
    pub case_id: String,
    pub uploader_id: String,
    pub uploader_role: String,
    pub ipfs_hash: String,
    pub description: Option<String>,
    pub timestamp: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = dispute_photos)]
pub struct NewDisputePhoto {
    pub id: String,
    pub case_id: String,
    pub uploader_id: String,
    pub uploader_role: String,
    pub ipfs_hash: String,
    pub description: Option<String>,
}

impl DisputePhoto {
    /// Upload a photo to a dispute
    pub fn create(
        conn: &mut SqliteConnection,
        new_photo: NewDisputePhoto,
    ) -> Result<Self> {
        diesel::insert_into(dispute_photos::table)
            .values(&new_photo)
            .execute(conn)?;

        dispute_photos::table
            .filter(dispute_photos::id.eq(new_photo.id))
            .first(conn)
            .context("Failed to retrieve uploaded photo")
    }

    /// Get all photos for a dispute
    pub fn find_by_case(
        conn: &mut SqliteConnection,
        case_id: String,
    ) -> Result<Vec<Self>> {
        dispute_photos::table
            .filter(dispute_photos::case_id.eq(case_id))
            .order(dispute_photos::timestamp.asc())
            .load(conn)
            .context("Failed to load dispute photos")
    }
}
```

### 2.3 Dispute Handlers

**New File: `server/src/handlers/dispute.rs`**
```rust
use actix_web::{web, HttpResponse, Result};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::dispute_case::{DisputeCase, NewDisputeCase, NewDisputeMessage, NewDisputePhoto};
use crate::models::escrow::Escrow;
use crate::services::escrow::EscrowOrchestrator;
use crate::ipfs::client::IpfsClient;
use crate::AppState;

#[derive(Deserialize)]
pub struct OpenDisputeRequest {
    pub escrow_id: String,
    pub claim: String,
}

#[derive(Serialize)]
pub struct OpenDisputeResponse {
    pub case_id: String,
    pub arbiter_id: String,
}

/// POST /api/disputes/open - Open a new dispute
pub async fn open_dispute(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,  // From auth middleware
    req: web::Json<OpenDisputeRequest>,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Load escrow
    let escrow = Escrow::find_by_id(&mut conn, req.escrow_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 2. Validate user is buyer or vendor
    let opened_by = if escrow.buyer_id == user_id {
        "buyer"
    } else if escrow.vendor_id == user_id {
        "vendor"
    } else {
        return Err(actix_web::error::ErrorForbidden("Not a party to this escrow"));
    };

    // 3. Check escrow is in valid state for dispute
    if escrow.status != "funded" && escrow.status != "vendor_delivering" {
        return Err(actix_web::error::ErrorBadRequest("Escrow not in valid state for dispute"));
    }

    // 4. Check if dispute already exists
    if DisputeCase::find_by_escrow_id(&mut conn, req.escrow_id.clone()).is_ok() {
        return Err(actix_web::error::ErrorConflict("Dispute already exists for this escrow"));
    }

    // 5. Create dispute case
    let case_id = Uuid::new_v4().to_string();
    let new_case = NewDisputeCase {
        id: case_id.clone(),
        escrow_id: req.escrow_id.clone(),
        buyer_id: escrow.buyer_id.clone(),
        vendor_id: escrow.vendor_id.clone(),
        arbiter_id: state.config.arbiter().arbiter_user_id.clone(),
        opened_by: opened_by.to_string(),
        buyer_claim: if opened_by == "buyer" {
            req.claim.clone()
        } else {
            "".to_string()
        },
        status: "open".to_string(),
    };

    let case = DisputeCase::create(&mut conn, new_case)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 6. Update escrow status
    Escrow::update_status(&mut conn, req.escrow_id.clone(), "dispute_open")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    info!(
        case_id = %case_id,
        escrow_id = %req.escrow_id,
        opened_by = opened_by,
        "Dispute opened"
    );

    Ok(HttpResponse::Ok().json(OpenDisputeResponse {
        case_id,
        arbiter_id: case.arbiter_id,
    }))
}

#[derive(Deserialize)]
pub struct AddVendorClaimRequest {
    pub case_id: String,
    pub claim: String,
}

/// POST /api/disputes/vendor-response - Vendor responds to dispute
pub async fn add_vendor_claim(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,
    req: web::Json<AddVendorClaimRequest>,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Load dispute
    let case = DisputeCase::find_by_id(&mut conn, req.case_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 2. Validate user is vendor
    if case.vendor_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not the vendor"));
    }

    // 3. Add vendor claim
    DisputeCase::add_vendor_claim(&mut conn, req.case_id.clone(), req.claim.clone())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    info!(
        case_id = %req.case_id,
        "Vendor claim added"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "claim_added"
    })))
}

#[derive(Deserialize)]
pub struct AddMessageRequest {
    pub case_id: String,
    pub message: String,
}

/// POST /api/disputes/messages - Add a message to a dispute
pub async fn add_message(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,
    req: web::Json<AddMessageRequest>,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Load dispute
    let case = DisputeCase::find_by_id(&mut conn, req.case_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 2. Determine sender role
    let sender_role = if case.buyer_id == user_id {
        "buyer"
    } else if case.vendor_id == user_id {
        "vendor"
    } else if case.arbiter_id == user_id {
        "arbiter"
    } else {
        return Err(actix_web::error::ErrorForbidden("Not a party to this dispute"));
    };

    // 3. Create message
    let message_id = Uuid::new_v4().to_string();
    let new_message = NewDisputeMessage {
        id: message_id,
        case_id: req.case_id.clone(),
        sender_id: user_id,
        sender_role: sender_role.to_string(),
        message: req.message.clone(),
    };

    DisputeMessage::create(&mut conn, new_message)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "message_added"
    })))
}

/// POST /api/disputes/photos - Upload a photo to a dispute
pub async fn upload_photo(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();

    let mut case_id: Option<String> = None;
    let mut description: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Parse multipart form
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| actix_web::error::ErrorBadRequest(e))?;

        match field.name() {
            "case_id" => {
                let mut bytes = Vec::new();
                while let Some(chunk) = field.next().await {
                    bytes.extend_from_slice(&chunk.unwrap());
                }
                case_id = Some(String::from_utf8(bytes).unwrap());
            }
            "description" => {
                let mut bytes = Vec::new();
                while let Some(chunk) = field.next().await {
                    bytes.extend_from_slice(&chunk.unwrap());
                }
                description = Some(String::from_utf8(bytes).unwrap());
            }
            "file" => {
                filename = field.content_disposition().get_filename().map(|s| s.to_string());
                let mut bytes = Vec::new();
                while let Some(chunk) = field.next().await {
                    bytes.extend_from_slice(&chunk.unwrap());
                }
                file_data = Some(bytes);
            }
            _ => {}
        }
    }

    let case_id = case_id.ok_or_else(|| actix_web::error::ErrorBadRequest("Missing case_id"))?;
    let file_data = file_data.ok_or_else(|| actix_web::error::ErrorBadRequest("Missing file"))?;
    let filename = filename.unwrap_or_else(|| "evidence.jpg".to_string());

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Load dispute
    let case = DisputeCase::find_by_id(&mut conn, case_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 2. Validate user is buyer or vendor
    let uploader_role = if case.buyer_id == user_id {
        "buyer"
    } else if case.vendor_id == user_id {
        "vendor"
    } else {
        return Err(actix_web::error::ErrorForbidden("Not a party to this dispute"));
    };

    // 3. Upload to IPFS
    let ipfs_client = IpfsClient::new(state.config.ipfs_url.clone());
    let ipfs_hash = ipfs_client.upload_dispute_evidence(file_data, filename)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 4. Store in database
    let photo_id = Uuid::new_v4().to_string();
    let new_photo = NewDisputePhoto {
        id: photo_id,
        case_id,
        uploader_id: user_id,
        uploader_role: uploader_role.to_string(),
        ipfs_hash: ipfs_hash.clone(),
        description,
    };

    DisputePhoto::create(&mut conn, new_photo)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ipfs_hash": ipfs_hash
    })))
}

#[derive(Deserialize)]
pub struct ResolveDisputeRequest {
    pub case_id: String,
    pub decision: String,  // "release" or "refund"
}

/// POST /api/disputes/resolve - Arbiter resolves a dispute
pub async fn resolve_dispute(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,
    req: web::Json<ResolveDisputeRequest>,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Validate arbiter
    if user_id != state.config.arbiter().arbiter_user_id {
        return Err(actix_web::error::ErrorForbidden("Not the arbiter"));
    }

    // 2. Validate decision
    if req.decision != "release" && req.decision != "refund" {
        return Err(actix_web::error::ErrorBadRequest("Invalid decision"));
    }

    // 3. Load dispute
    let case = DisputeCase::find_by_id(&mut conn, req.case_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 4. Load escrow
    let escrow = Escrow::find_by_id(&mut conn, case.escrow_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 5. Determine winner address
    let winner_address = if req.decision == "release" {
        // TODO: Get vendor's Monero address from user profile
        "vendor_address_placeholder".to_string()
    } else {
        // TODO: Get buyer's Monero address from user profile
        "buyer_address_placeholder".to_string()
    };

    // 6. Execute with fee deduction
    let orchestrator = state.escrow_orchestrator.lock().await;
    let (winner_tx_hash, arbiter_fee_tx_hash) = orchestrator
        .release_with_arbiter_fee(
            Uuid::parse_str(&escrow.id).unwrap(),
            winner_address,
            state.config.arbiter().arbiter_address.clone(),
            state.config.arbiter().fee_percentage,
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 7. Record decision
    let fee_atomic = (escrow.amount as f64 * state.config.arbiter().fee_percentage) as u64;
    DisputeCase::record_decision(
        &mut conn,
        req.case_id.clone(),
        &req.decision,
        winner_tx_hash.clone(),
        arbiter_fee_tx_hash.clone(),
        fee_atomic,
    )
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 8. Update escrow status
    let final_status = if req.decision == "release" {
        "completed"
    } else {
        "refunded"
    };
    Escrow::update_status(&mut conn, case.escrow_id.clone(), final_status)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    info!(
        case_id = %req.case_id,
        decision = req.decision,
        winner_tx = %winner_tx_hash,
        fee_tx = %arbiter_fee_tx_hash,
        "Dispute resolved"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "resolved",
        "winner_tx_hash": winner_tx_hash,
        "arbiter_fee_tx_hash": arbiter_fee_tx_hash
    })))
}

/// GET /api/disputes/arbiter - Get all disputes assigned to arbiter
pub async fn list_arbiter_disputes(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Validate arbiter
    if user_id != state.config.arbiter().arbiter_user_id {
        return Err(actix_web::error::ErrorForbidden("Not the arbiter"));
    }

    // 2. Load disputes
    let disputes = DisputeCase::find_by_arbiter(&mut conn, user_id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(disputes))
}

/// GET /api/disputes/{case_id} - Get full dispute details
pub async fn get_dispute_details(
    state: web::Data<AppState>,
    user_id: web::ReqData<String>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let case_id = path.into_inner();
    let user_id = user_id.into_inner();

    let mut conn = state.db_pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 1. Load dispute
    let case = DisputeCase::find_by_id(&mut conn, case_id.clone())
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // 2. Validate access
    if case.buyer_id != user_id && case.vendor_id != user_id && case.arbiter_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    // 3. Load messages and photos
    let messages = DisputeMessage::find_by_case(&mut conn, case_id.clone())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let photos = DisputePhoto::find_by_case(&mut conn, case_id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "case": case,
        "messages": messages,
        "photos": photos
    })))
}

/// Register dispute routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/disputes")
            .route("/open", web::post().to(open_dispute))
            .route("/vendor-response", web::post().to(add_vendor_claim))
            .route("/messages", web::post().to(add_message))
            .route("/photos", web::post().to(upload_photo))
            .route("/resolve", web::post().to(resolve_dispute))
            .route("/arbiter", web::get().to(list_arbiter_disputes))
            .route("/{case_id}", web::get().to(get_dispute_details))
    );
}
```

### 2.4 Arbiter Admin Portal (Frontend)

**New File: `templates/arbiter/dispute_list.html`**
```html
{% extends "base.html" %}

{% block title %}Dispute Management{% endblock %}

{% block content %}
<div class="container">
    <h1>Dispute Management Portal</h1>
    <p>Arbiter: {{ arbiter_name }}</p>

    <div class="filters">
        <button class="filter-btn active" data-status="all">All</button>
        <button class="filter-btn" data-status="open">Open</button>
        <button class="filter-btn" data-status="under_review">Under Review</button>
        <button class="filter-btn" data-status="resolved">Resolved</button>
    </div>

    <table class="dispute-table">
        <thead>
            <tr>
                <th>Case ID</th>
                <th>Escrow</th>
                <th>Opened By</th>
                <th>Amount (XMR)</th>
                <th>Status</th>
                <th>Opened At</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody id="dispute-list">
            {% for dispute in disputes %}
            <tr data-status="{{ dispute.status }}">
                <td>{{ dispute.id | truncate(8) }}</td>
                <td>{{ dispute.escrow_id | truncate(8) }}</td>
                <td>{{ dispute.opened_by }}</td>
                <td>{{ dispute.amount_xmr }}</td>
                <td><span class="status-badge status-{{ dispute.status }}">{{ dispute.status }}</span></td>
                <td>{{ dispute.opened_at | timeago }}</td>
                <td>
                    <a href="/arbiter/disputes/{{ dispute.id }}" class="btn btn-primary">Review</a>
                </td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
</div>

<script>
// Filter functionality
document.querySelectorAll('.filter-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');

        const status = btn.dataset.status;
        document.querySelectorAll('#dispute-list tr').forEach(row => {
            if (status === 'all' || row.dataset.status === status) {
                row.style.display = '';
            } else {
                row.style.display = 'none';
            }
        });
    });
});
</script>
{% endblock %}
```

**New File: `templates/arbiter/dispute_detail.html`**
```html
{% extends "base.html" %}

{% block title %}Dispute {{ case.id }}{% endblock %}

{% block content %}
<div class="container">
    <div class="dispute-header">
        <h1>Dispute Case {{ case.id | truncate(8) }}</h1>
        <span class="status-badge status-{{ case.status }}">{{ case.status }}</span>
    </div>

    <div class="dispute-info">
        <div class="info-row">
            <strong>Escrow ID:</strong> {{ case.escrow_id }}
        </div>
        <div class="info-row">
            <strong>Amount:</strong> {{ case.amount_xmr }} XMR ({{ case.amount_atomic }} atomic)
        </div>
        <div class="info-row">
            <strong>Opened By:</strong> {{ case.opened_by }}
        </div>
        <div class="info-row">
            <strong>Opened At:</strong> {{ case.opened_at }}
        </div>
    </div>

    <div class="claims">
        <h2>Claims</h2>
        <div class="claim-box buyer-claim">
            <h3>Buyer's Claim</h3>
            <p>{{ case.buyer_claim }}</p>
        </div>
        {% if case.vendor_claim %}
        <div class="claim-box vendor-claim">
            <h3>Vendor's Response</h3>
            <p>{{ case.vendor_claim }}</p>
        </div>
        {% endif %}
    </div>

    <div class="evidence">
        <h2>Evidence</h2>
        <div class="photos-grid">
            {% for photo in photos %}
            <div class="photo-card">
                <img src="https://ipfs.io/ipfs/{{ photo.ipfs_hash }}" alt="Evidence" />
                <div class="photo-meta">
                    <strong>Uploaded by:</strong> {{ photo.uploader_role }}<br>
                    <strong>At:</strong> {{ photo.timestamp }}<br>
                    {% if photo.description %}
                    <strong>Description:</strong> {{ photo.description }}
                    {% endif %}
                </div>
            </div>
            {% endfor %}
        </div>
    </div>

    <div class="messages">
        <h2>Conversation</h2>
        <div class="message-list">
            {% for message in messages %}
            <div class="message message-{{ message.sender_role }}">
                <div class="message-header">
                    <strong>{{ message.sender_role | capitalize }}</strong>
                    <span class="timestamp">{{ message.timestamp | timeago }}</span>
                </div>
                <div class="message-body">
                    {{ message.message }}
                </div>
            </div>
            {% endfor %}
        </div>

        <form id="add-message-form">
            <textarea name="message" placeholder="Add a comment..." required></textarea>
            <button type="submit">Send</button>
        </form>
    </div>

    {% if case.status != 'resolved' %}
    <div class="decision-panel">
        <h2>Make Decision</h2>
        <form id="resolve-form">
            <input type="hidden" name="case_id" value="{{ case.id }}" />
            <div class="decision-buttons">
                <button type="submit" name="decision" value="release" class="btn btn-success">
                    Release to Vendor (Vendor Wins)
                </button>
                <button type="submit" name="decision" value="refund" class="btn btn-warning">
                    Refund to Buyer (Buyer Wins)
                </button>
            </div>
            <p class="fee-info">Arbiter fee: {{ fee_percentage }}% ({{ fee_xmr }} XMR) will be automatically collected from losing party.</p>
        </form>
    </div>
    {% else %}
    <div class="resolution-summary">
        <h2>Resolution</h2>
        <p><strong>Decision:</strong> {{ case.decision }}</p>
        <p><strong>Winner TX:</strong> <a href="https://testnet.xmrchain.net/tx/{{ case.winner_tx_hash }}" target="_blank">{{ case.winner_tx_hash }}</a></p>
        <p><strong>Arbiter Fee TX:</strong> <a href="https://testnet.xmrchain.net/tx/{{ case.arbiter_fee_tx_hash }}" target="_blank">{{ case.arbiter_fee_tx_hash }}</a></p>
        <p><strong>Fee Collected:</strong> {{ case.fee_xmr }} XMR</p>
        <p><strong>Decided At:</strong> {{ case.decided_at }}</p>
    </div>
    {% endif %}
</div>

<script>
// Add message
document.getElementById('add-message-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const message = e.target.message.value;

    const response = await fetch('/api/disputes/messages', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            case_id: '{{ case.id }}',
            message: message
        })
    });

    if (response.ok) {
        location.reload();
    }
});

// Resolve dispute
document.getElementById('resolve-form').addEventListener('submit', async (e) => {
    e.preventDefault();

    const decision = e.submitter.value;
    const confirmation = confirm(`Are you sure you want to ${decision} this dispute? This action is irreversible.`);

    if (!confirmation) return;

    const response = await fetch('/api/disputes/resolve', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            case_id: '{{ case.id }}',
            decision: decision
        })
    });

    if (response.ok) {
        alert('Dispute resolved successfully');
        location.reload();
    } else {
        alert('Failed to resolve dispute');
    }
});
</script>
{% endblock %}
```

### Phase 2 Implementation Checklist

**Day 1 (8 hours):**
```
✅ Create dispute database migrations
✅ Apply migrations and regenerate schema
✅ Create dispute models (dispute_case.rs)
✅ Write model unit tests
✅ Test: cargo test --package server test_dispute_models
```

**Day 2 (8 hours):**
```
✅ Implement dispute handlers (handlers/dispute.rs)
✅ Add IPFS upload functionality
✅ Wire up routes in main.rs
✅ Write handler integration tests
✅ Test: cargo test --package server test_dispute_handlers
```

**Day 3 (8 hours):**
```
✅ Create arbiter admin portal templates
✅ Add dispute list view
✅ Add dispute detail view
✅ Implement frontend JavaScript for messaging/resolution
✅ Test in browser with mock disputes
```

**Day 4 (8 hours):**
```
✅ End-to-end testing with real escrow + dispute flow
✅ Test fee sweep implementation
✅ Test IPFS upload/retrieval
✅ Security audit of dispute handlers
✅ Documentation for arbiters
```

---

## 📊 PHASE 3: MONITORING INFRASTRUCTURE (1 day)

**Duration:** 8 hours effective work
**Priority:** P1 - Required for safe operation
**Goal:** Production monitoring without exposing infrastructure

### Architecture

**Components:**
- **Prometheus**: Metrics collection (localhost only)
- **Grafana**: Visualization dashboard (Tor hidden service)
- **Alertmanager**: Alert routing (email/telegram)
- **Node Exporter**: System metrics

**Security:**
- All services bound to localhost
- Grafana accessible ONLY via Tor .onion
- No public exposure
- Encrypted metrics storage

### 3.1 Prometheus Setup

**File: `docker-compose-monitoring.yml`**
```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: monero_prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.listen-address=127.0.0.1:9090'
    ports:
      - '127.0.0.1:9090:9090'
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    restart: unless-stopped
    network_mode: host

  grafana:
    image: grafana/grafana:latest
    container_name: monero_grafana
    environment:
      - GF_SERVER_HTTP_ADDR=127.0.0.1
      - GF_SERVER_HTTP_PORT=3000
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
      - GF_SECURITY_SECRET_KEY=${GRAFANA_SECRET_KEY}
      - GF_USERS_ALLOW_SIGN_UP=false
    ports:
      - '127.0.0.1:3000:3000'
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources
    restart: unless-stopped
    network_mode: host
    depends_on:
      - prometheus

  alertmanager:
    image: prom/alertmanager:latest
    container_name: monero_alertmanager
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
      - '--web.listen-address=127.0.0.1:9093'
    ports:
      - '127.0.0.1:9093:9093'
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml
    restart: unless-stopped
    network_mode: host

  node_exporter:
    image: prom/node-exporter:latest
    container_name: monero_node_exporter
    command:
      - '--web.listen-address=127.0.0.1:9100'
    ports:
      - '127.0.0.1:9100:9100'
    restart: unless-stopped
    network_mode: host

volumes:
  prometheus_data:
  grafana_data:
```

**File: `monitoring/prometheus.yml`**
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # Monero Marketplace Server
  - job_name: 'marketplace'
    static_configs:
      - targets: ['127.0.0.1:8081']
    metrics_path: '/metrics'

  # System Metrics
  - job_name: 'node_exporter'
    static_configs:
      - targets: ['127.0.0.1:9100']

  # Monero RPC (if exposed)
  - job_name: 'monero_rpc'
    static_configs:
      - targets: ['127.0.0.1:18082']
    metrics_path: '/metrics'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['127.0.0.1:9093']

rule_files:
  - 'alerts.yml'
```

**File: `monitoring/alerts.yml`**
```yaml
groups:
  - name: escrow_alerts
    interval: 30s
    rules:
      # Escrow stuck in pending > 24h
      - alert: EscrowStuckPending
        expr: escrow_status{status="pending"} > 86400
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Escrow {{ $labels.escrow_id }} stuck in pending"
          description: "Escrow has been pending for {{ $value }}s (24h+)"

      # Escrow stuck in funded > 48h
      - alert: EscrowStuckFunded
        expr: escrow_status{status="funded"} > 172800
        for: 2h
        labels:
          severity: critical
        annotations:
          summary: "Escrow {{ $labels.escrow_id }} stuck in funded"
          description: "Escrow has been funded for {{ $value }}s (48h+), requires manual review"

      # Dispute open > 72h
      - alert: DisputeStale
        expr: dispute_status{status="open"} > 259200
        for: 6h
        labels:
          severity: warning
        annotations:
          summary: "Dispute {{ $labels.case_id }} open > 72h"
          description: "Arbiter has not reviewed dispute in 72h"

      # High dispute rate
      - alert: HighDisputeRate
        expr: rate(disputes_opened_total[1h]) > 0.1
        for: 2h
        labels:
          severity: warning
        annotations:
          summary: "High dispute rate detected"
          description: "Dispute rate: {{ $value }} per second (threshold: 0.1/s)"

      # Monero RPC unreachable
      - alert: MoneroRPCDown
        expr: up{job="monero_rpc"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Monero RPC unreachable"
          description: "Cannot connect to Monero wallet RPC"

      # Server high memory usage
      - alert: HighMemoryUsage
        expr: (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) < 0.1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Available memory < 10%"
```

**File: `monitoring/alertmanager.yml`**
```yaml
global:
  smtp_smarthost: 'smtp.example.com:587'
  smtp_from: 'alerts@moneromarketplace.onion'
  smtp_auth_username: 'alerts'
  smtp_auth_password: '${SMTP_PASSWORD}'

route:
  receiver: 'default'
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h

  routes:
    - match:
        severity: critical
      receiver: 'critical'
      continue: true

    - match:
        severity: warning
      receiver: 'warning'

receivers:
  - name: 'default'
    email_configs:
      - to: 'admin@example.com'

  - name: 'critical'
    email_configs:
      - to: 'admin@example.com'
        headers:
          Subject: '🚨 CRITICAL: {{ .GroupLabels.alertname }}'
    # Optional: Telegram
    # telegram_configs:
    #   - bot_token: 'YOUR_BOT_TOKEN'
    #     chat_id: YOUR_CHAT_ID

  - name: 'warning'
    email_configs:
      - to: 'admin@example.com'
        headers:
          Subject: '⚠️ WARNING: {{ .GroupLabels.alertname }}'
```

### 3.2 Server Metrics Exporter

**Add to `server/Cargo.toml`:**
```toml
prometheus = "0.13"
lazy_static = "1.4"
```

**New File: `server/src/metrics.rs`**
```rust
use prometheus::{
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Histogram, HistogramVec,
    Registry, TextEncoder, Encoder,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Escrow metrics
    pub static ref ESCROWS_CREATED: IntCounter = IntCounter::new(
        "escrows_created_total",
        "Total number of escrows created"
    ).unwrap();

    pub static ref ESCROWS_BY_STATUS: IntGaugeVec = IntGaugeVec::new(
        prometheus::opts!("escrows_by_status", "Number of escrows by status"),
        &["status"]
    ).unwrap();

    pub static ref ESCROW_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "escrow_duration_seconds",
            "Time from creation to completion"
        ),
        &["outcome"]  // "completed", "refunded", "disputed"
    ).unwrap();

    // Dispute metrics
    pub static ref DISPUTES_OPENED: IntCounter = IntCounter::new(
        "disputes_opened_total",
        "Total number of disputes opened"
    ).unwrap();

    pub static ref DISPUTES_BY_STATUS: IntGaugeVec = IntGaugeVec::new(
        prometheus::opts!("disputes_by_status", "Number of disputes by status"),
        &["status"]
    ).unwrap();

    pub static ref DISPUTE_RESOLUTION_TIME: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "dispute_resolution_seconds",
            "Time from dispute opened to resolved"
        )
    ).unwrap();

    // Monero RPC metrics
    pub static ref RPC_REQUESTS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("monero_rpc_requests_total", "Total RPC requests"),
        &["method", "status"]  // "success", "error"
    ).unwrap();

    pub static ref RPC_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "monero_rpc_duration_seconds",
            "RPC request duration"
        ),
        &["method"]
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(ESCROWS_CREATED.clone())).unwrap();
    REGISTRY.register(Box::new(ESCROWS_BY_STATUS.clone())).unwrap();
    REGISTRY.register(Box::new(ESCROW_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(DISPUTES_OPENED.clone())).unwrap();
    REGISTRY.register(Box::new(DISPUTES_BY_STATUS.clone())).unwrap();
    REGISTRY.register(Box::new(DISPUTE_RESOLUTION_TIME.clone())).unwrap();
    REGISTRY.register(Box::new(RPC_REQUESTS.clone())).unwrap();
    REGISTRY.register(Box::new(RPC_DURATION.clone())).unwrap();
}

pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

**Add to `server/src/main.rs`:**
```rust
mod metrics;

async fn metrics_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics::export_metrics())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Register metrics
    metrics::register_metrics();

    HttpServer::new(move || {
        App::new()
            .route("/metrics", web::get().to(metrics_handler))
            // ... other routes
    })
    .bind("127.0.0.1:8081")?  // Metrics on separate port
    .run()
    .await
}
```

### 3.3 Grafana Dashboard

**File: `monitoring/grafana/dashboards/marketplace.json`**
```json
{
  "dashboard": {
    "title": "Monero Marketplace - Production",
    "panels": [
      {
        "title": "Escrows by Status",
        "type": "graph",
        "targets": [
          {
            "expr": "escrows_by_status"
          }
        ]
      },
      {
        "title": "Active Disputes",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(disputes_by_status{status!='resolved'})"
          }
        ]
      },
      {
        "title": "Monero RPC Status",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job='monero_rpc'}"
          }
        ]
      },
      {
        "title": "Escrow Duration (24h)",
        "type": "heatmap",
        "targets": [
          {
            "expr": "rate(escrow_duration_seconds_bucket[24h])"
          }
        ]
      },
      {
        "title": "RPC Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(monero_rpc_requests_total[5m])"
          }
        ]
      }
    ]
  }
}
```

### 3.4 Tor Hidden Service for Grafana

**Add to `/etc/tor/torrc`:**
```
HiddenServiceDir /var/lib/tor/grafana/
HiddenServicePort 80 127.0.0.1:3000
```

**Restart Tor:**
```bash
sudo systemctl restart tor
cat /var/lib/tor/grafana/hostname
# Output: your_grafana_onion_address.onion
```

### Phase 3 Implementation Checklist

**Hours 1-4:**
```
✅ Create docker-compose-monitoring.yml
✅ Create prometheus.yml with scrape configs
✅ Create alerts.yml with escrow/dispute alerts
✅ Create alertmanager.yml with email routing
✅ Test: docker-compose -f docker-compose-monitoring.yml up -d
✅ Verify Prometheus UI at http://127.0.0.1:9090
```

**Hours 5-8:**
```
✅ Add prometheus crate to server/Cargo.toml
✅ Create server/src/metrics.rs with metric definitions
✅ Add metrics endpoint to main.rs
✅ Instrument escrow handlers with metrics
✅ Instrument dispute handlers with metrics
✅ Test: curl http://127.0.0.1:8081/metrics
✅ Configure Grafana dashboard
✅ Setup Tor hidden service for Grafana
✅ Test: Access Grafana via .onion address
```

---

## ✅ PHASE 4: BETA VALIDATION (1 day)

**Duration:** 8 hours
**Priority:** P0 - Must pass before beta launch
**Goal:** Comprehensive testing of entire system

### 4.1 Test Scenarios

**Scenario 1: Normal Escrow Flow**
```
1. Create escrow
2. Buyer registers wallet RPC
3. Vendor registers wallet RPC
4. Setup multisig
5. Buyer funds escrow
6. Vendor delivers (simulated)
7. Buyer confirms delivery
8. Funds released to vendor
9. Verify metrics updated
```

**Scenario 2: Dispute Flow with Buyer Win**
```
1. Create and fund escrow
2. Buyer opens dispute
3. Vendor responds
4. Buyer uploads evidence photos to IPFS
5. Messages exchanged
6. Arbiter reviews in portal
7. Arbiter decides: refund to buyer
8. Verify fee sweep (amount - 2%) to buyer, 2% to arbiter
9. Verify both transactions on blockchain
10. Verify dispute resolved in database
```

**Scenario 3: Dispute Flow with Vendor Win**
```
(Same as Scenario 2 but arbiter decides: release to vendor)
```

**Scenario 4: Concurrent Escrow Creation (Race Condition Test)**
```
1. Launch 20 concurrent escrow creations
2. Verify all get unique wallet mappings
3. Verify no database constraint violations
4. Verify all wallets registered correctly
```

**Scenario 5: Monitoring & Alerts**
```
1. Create escrow and leave in pending state
2. Wait 25 hours
3. Verify "EscrowStuckPending" alert fires
4. Verify email received
5. Test alert resolution
```

### 4.2 Test Automation Script

**File: `scripts/validate-beta.sh`**
```bash
#!/bin/bash
set -e

echo "🧪 BETA VALIDATION - Full System Test"
echo "======================================"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0

function test_pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    ((PASS_COUNT++))
}

function test_fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    ((FAIL_COUNT++))
}

# 1. Pre-flight checks
echo ""
echo "📋 Pre-flight Checks"
echo "--------------------"

# Check Tor
if systemctl is-active --quiet tor; then
    test_pass "Tor daemon running"
else
    test_fail "Tor daemon NOT running"
fi

# Check Monero RPC
if curl -s http://127.0.0.1:18082/json_rpc > /dev/null 2>&1; then
    test_pass "Monero RPC reachable"
else
    test_fail "Monero RPC NOT reachable"
fi

# Check IPFS
if ipfs id > /dev/null 2>&1; then
    test_pass "IPFS daemon running"
else
    test_fail "IPFS daemon NOT running"
fi

# Check database
if [ -f "marketplace.db" ]; then
    test_pass "Database exists"
else
    test_fail "Database NOT found"
fi

# Check migrations applied
PENDING=$(DATABASE_URL=marketplace.db diesel migration list | grep -c "\[ \]" || true)
if [ "$PENDING" -eq 0 ]; then
    test_pass "All migrations applied"
else
    test_fail "$PENDING migrations pending"
fi

# 2. Start services
echo ""
echo "🚀 Starting Services"
echo "--------------------"

# Start monitoring
docker-compose -f docker-compose-monitoring.yml up -d
sleep 5

if curl -s http://127.0.0.1:9090/-/healthy > /dev/null 2>&1; then
    test_pass "Prometheus started"
else
    test_fail "Prometheus failed to start"
fi

# Start server
cargo build --release --package server
./target/release/server > server.log 2>&1 &
SERVER_PID=$!
sleep 5

if kill -0 $SERVER_PID 2>/dev/null; then
    test_pass "Server started (PID: $SERVER_PID)"
else
    test_fail "Server failed to start"
fi

# 3. Run test scenarios
echo ""
echo "🧪 Test Scenarios"
echo "------------------"

# Scenario 1: Normal escrow
ESCROW_ID=$(curl -s -X POST http://localhost:8080/api/escrows \
    -H "Content-Type: application/json" \
    -d '{"order_id":"'$(uuidgen)'","buyer_id":"test_buyer","vendor_id":"test_vendor","amount":100000000000}' \
    | jq -r '.escrow_id')

if [ -n "$ESCROW_ID" ] && [ "$ESCROW_ID" != "null" ]; then
    test_pass "Scenario 1: Escrow created ($ESCROW_ID)"
else
    test_fail "Scenario 1: Escrow creation failed"
fi

# Scenario 2: Wallet registration (no race condition)
for i in {1..10}; do
    curl -s -X POST http://localhost:8080/api/escrows/$ESCROW_ID/register \
        -H "Content-Type: application/json" \
        -d '{"role":"buyer","rpc_url":"http://127.0.0.1:18083"}' &
done
wait

MAPPINGS=$(sqlite3 marketplace.db "SELECT COUNT(*) FROM escrow_wallet_mappings WHERE escrow_id='$ESCROW_ID';")
if [ "$MAPPINGS" -eq 1 ]; then
    test_pass "Scenario 2: No race condition (1 mapping created)"
else
    test_fail "Scenario 2: Race condition detected ($MAPPINGS mappings)"
fi

# Scenario 3: Dispute creation
CASE_ID=$(curl -s -X POST http://localhost:8080/api/disputes/open \
    -H "Content-Type: application/json" \
    -d '{"escrow_id":"'$ESCROW_ID'","claim":"Product not received"}' \
    | jq -r '.case_id')

if [ -n "$CASE_ID" ] && [ "$CASE_ID" != "null" ]; then
    test_pass "Scenario 3: Dispute opened ($CASE_ID)"
else
    test_fail "Scenario 3: Dispute creation failed"
fi

# Scenario 4: IPFS upload
echo "Test evidence" > /tmp/test_evidence.txt
IPFS_HASH=$(curl -s -X POST http://localhost:8080/api/disputes/photos \
    -F "case_id=$CASE_ID" \
    -F "file=@/tmp/test_evidence.txt" \
    | jq -r '.ipfs_hash')

if [ -n "$IPFS_HASH" ] && [ "$IPFS_HASH" != "null" ]; then
    test_pass "Scenario 4: IPFS upload ($IPFS_HASH)"
else
    test_fail "Scenario 4: IPFS upload failed"
fi

# Scenario 5: Metrics endpoint
METRICS=$(curl -s http://127.0.0.1:8081/metrics | grep -c "escrows_created_total" || true)
if [ "$METRICS" -gt 0 ]; then
    test_pass "Scenario 5: Metrics endpoint working"
else
    test_fail "Scenario 5: Metrics endpoint failed"
fi

# 4. Cleanup
echo ""
echo "🧹 Cleanup"
echo "----------"

kill $SERVER_PID 2>/dev/null || true
docker-compose -f docker-compose-monitoring.yml down

# 5. Summary
echo ""
echo "📊 SUMMARY"
echo "=========="
echo -e "${GREEN}PASSED${NC}: $PASS_COUNT"
echo -e "${RED}FAILED${NC}: $FAIL_COUNT"

if [ $FAIL_COUNT -eq 0 ]; then
    echo ""
    echo "🎉 ALL TESTS PASSED - READY FOR BETA LAUNCH"
    exit 0
else
    echo ""
    echo "⚠️ SOME TESTS FAILED - DO NOT LAUNCH"
    exit 1
fi
```

### 4.3 Beta Launch Checklist

**Pre-Launch:**
```
✅ All migrations applied
✅ Configuration validated (.env complete)
✅ Arbiter user configured
✅ IPFS daemon running
✅ Tor daemon running
✅ Monero RPC running (testnet)
✅ Monitoring stack deployed
✅ Grafana accessible via .onion
✅ All tests passing (./scripts/validate-beta.sh)
```

**Launch Day:**
```
✅ Start all services
✅ Verify monitoring dashboard
✅ Create test escrow (smoke test)
✅ Monitor for 1 hour
✅ Announce beta availability
```

**Post-Launch:**
```
✅ 24/7 monitoring for first 48h
✅ Daily check of Grafana dashboard
✅ Review logs for errors
✅ Monitor dispute rate
✅ Check escrow completion rate
```

---

## 📅 COMPLETE IMPLEMENTATION TIMELINE

### Week 1: Foundation

**Day 0 (Wednesday):**
- ⏰ 9:00-12:00: Blocker #1 (Fee sweep implementation)
- ⏰ 13:00-16:00: Blocker #2 (IPFS upload handler)
- ⏰ 16:00-16:30: Blocker #3 (Arbiter config)
- ⏰ 16:30-17:00: Test all blockers
- 🎯 **Deliverable:** All pre-phase blockers resolved

**Day 1 (Thursday):**
- ⏰ 9:00-13:00: Phase 1 database migration + models
- ⏰ 14:00-18:00: Phase 1 WalletManager refactor
- 🎯 **Deliverable:** Race condition fixed in database layer

**Day 2 (Friday):**
- ⏰ 9:00-13:00: Phase 1 EscrowOrchestrator updates
- ⏰ 14:00-18:00: Phase 1 testing + validation
- 🎯 **Deliverable:** Race condition eliminated, all tests passing

### Week 2: Arbitration

**Day 3 (Monday):**
- ⏰ 9:00-13:00: Phase 2 dispute database schema + models
- ⏰ 14:00-18:00: Phase 2 dispute handlers (create/respond)
- 🎯 **Deliverable:** Dispute creation + vendor response working

**Day 4 (Tuesday):**
- ⏰ 9:00-13:00: Phase 2 message/photo handlers
- ⏰ 14:00-18:00: Phase 2 resolution handler + fee sweep integration
- 🎯 **Deliverable:** Complete dispute resolution backend

**Day 5 (Wednesday):**
- ⏰ 9:00-13:00: Phase 2 arbiter admin portal (list + detail views)
- ⏰ 14:00-18:00: Phase 2 frontend JavaScript + styling
- 🎯 **Deliverable:** Arbiter portal functional

**Day 6 (Thursday):**
- ⏰ 9:00-13:00: Phase 2 end-to-end testing
- ⏰ 14:00-18:00: Phase 2 security audit + documentation
- 🎯 **Deliverable:** Dispute system production-ready

### Week 2: Launch

**Day 7 (Friday):**
- ⏰ 9:00-13:00: Phase 3 monitoring setup (Prometheus + Grafana)
- ⏰ 14:00-18:00: Phase 3 metrics instrumentation + Tor hidden service
- 🎯 **Deliverable:** Monitoring operational

**Day 8 (Saturday):**
- ⏰ 9:00-13:00: Phase 4 comprehensive testing
- ⏰ 14:00-18:00: Phase 4 final validation + beta launch prep
- 🎯 **Deliverable:** System validated, ready for beta

**Day 9 (Sunday):**
- ⏰ 10:00-12:00: Beta launch
- ⏰ 12:00-18:00: Monitoring + support
- 🎯 **Deliverable:** BETA LIVE

---

## 📚 CODE APPENDICES

### Appendix A: Database Schema Reference

**Full Schema After All Migrations:**

```sql
-- Core escrow table (existing)
CREATE TABLE escrows (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL,
    buyer_id TEXT NOT NULL,
    vendor_id TEXT NOT NULL,
    arbiter_id TEXT NOT NULL,
    amount INTEGER NOT NULL,
    multisig_address TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- ... other existing fields
);

-- Phase 1: Wallet mappings (prevents race condition)
CREATE TABLE escrow_wallet_mappings (
    escrow_id TEXT PRIMARY KEY NOT NULL,
    buyer_user_id TEXT NOT NULL,
    buyer_rpc_url TEXT NOT NULL,
    buyer_rpc_user TEXT,
    buyer_rpc_password TEXT,
    buyer_wallet_id TEXT,
    buyer_registered_at TIMESTAMP,
    vendor_user_id TEXT NOT NULL,
    vendor_rpc_url TEXT NOT NULL,
    vendor_rpc_user TEXT,
    vendor_rpc_password TEXT,
    vendor_wallet_id TEXT,
    vendor_registered_at TIMESTAMP,
    arbiter_rpc_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:18082/json_rpc',
    arbiter_wallet_id TEXT,
    arbiter_registered_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE
);

-- Phase 2: Dispute system
CREATE TABLE dispute_cases (
    id TEXT PRIMARY KEY NOT NULL,
    escrow_id TEXT NOT NULL UNIQUE,
    buyer_id TEXT NOT NULL,
    vendor_id TEXT NOT NULL,
    arbiter_id TEXT NOT NULL,
    opened_by TEXT NOT NULL,
    opened_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    buyer_claim TEXT NOT NULL,
    vendor_claim TEXT,
    decision TEXT,
    decided_at TIMESTAMP,
    winner_tx_hash TEXT,
    arbiter_fee_tx_hash TEXT,
    fee_atomic INTEGER,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE
);

CREATE TABLE dispute_messages (
    id TEXT PRIMARY KEY NOT NULL,
    case_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    sender_role TEXT NOT NULL,
    message TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (case_id) REFERENCES dispute_cases(id) ON DELETE CASCADE
);

CREATE TABLE dispute_photos (
    id TEXT PRIMARY KEY NOT NULL,
    case_id TEXT NOT NULL,
    uploader_id TEXT NOT NULL,
    uploader_role TEXT NOT NULL,
    ipfs_hash TEXT NOT NULL,
    description TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (case_id) REFERENCES dispute_cases(id) ON DELETE CASCADE
);
```

### Appendix B: Configuration Reference

**Complete `.env` Configuration:**

```bash
# Database
DATABASE_URL=marketplace.db

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
METRICS_PORT=8081

# Monero
MONERO_RPC_URL=http://127.0.0.1:18082/json_rpc
MONERO_RPC_USER=
MONERO_RPC_PASSWORD=
MONERO_NETWORK=testnet

# IPFS
IPFS_API_URL=http://127.0.0.1:5001

# Tor
TOR_SOCKS_PROXY=socks5h://127.0.0.1:9050

# Arbiter Configuration (v1 Beta)
ARBITER_USER_ID=admin_user_12345
ARBITER_ADDRESS=your_testnet_monero_address
ARBITER_FEE_PERCENTAGE=0.02
ARBITER_MODE=single_admin

# Monitoring
GRAFANA_ADMIN_PASSWORD=your_secure_password
GRAFANA_SECRET_KEY=your_secret_key
SMTP_PASSWORD=your_smtp_password

# Encryption
ENCRYPTION_KEY=your_32_byte_encryption_key_here

# Beta Limits
MAX_ESCROW_AMOUNT_ATOMIC=5000000000000
MAX_BETA_VOLUME_ATOMIC=50000000000000
```

### Appendix C: Deployment Commands

**Complete Deployment Workflow:**

```bash
# 1. Pre-deployment setup
cd /home/malix/Desktop/monero.marketplace

# 2. Apply all migrations
DATABASE_URL=marketplace.db diesel migration run
diesel print-schema > server/src/schema.rs

# 3. Build release binaries
cargo build --release --workspace

# 4. Start Tor daemon
sudo systemctl start tor
sudo systemctl enable tor

# 5. Start IPFS daemon
ipfs daemon &

# 6. Start Monero RPC (testnet)
monero-wallet-rpc \
    --testnet \
    --rpc-bind-port 18082 \
    --rpc-bind-ip 127.0.0.1 \
    --disable-rpc-login \
    --daemon-address testnet.xmr.ditatompel.com:28081 \
    --trusted-daemon \
    &

# 7. Start monitoring stack
docker-compose -f docker-compose-monitoring.yml up -d

# 8. Verify monitoring
curl http://127.0.0.1:9090/-/healthy  # Prometheus
curl http://127.0.0.1:3000/api/health  # Grafana

# 9. Get Grafana .onion address
cat /var/lib/tor/grafana/hostname

# 10. Run validation tests
./scripts/validate-beta.sh

# 11. Start marketplace server
./target/release/server > server.log 2>&1 &

# 12. Verify server
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8081/metrics

# 13. Monitor logs
tail -f server.log

# 14. Test escrow creation
curl -X POST http://localhost:8080/api/escrows \
    -H "Content-Type: application/json" \
    -d '{"order_id":"'$(uuidgen)'","buyer_id":"test1","vendor_id":"test2","amount":100000000000}'

# 15. Access Grafana dashboard (via Tor Browser)
# Navigate to: http://YOUR_ONION_ADDRESS.onion
```

---

## 🎯 SUCCESS CRITERIA

### Phase 1 Success Metrics
- ✅ Zero race conditions in 100 concurrent escrow creations
- ✅ All wallet mappings persisted correctly
- ✅ No database constraint violations
- ✅ All existing tests passing

### Phase 2 Success Metrics
- ✅ Dispute creation functional
- ✅ Evidence upload to IPFS working
- ✅ Arbiter portal accessible
- ✅ Fee sweep executes correctly (2 transactions)
- ✅ Both winner and arbiter receive funds

### Phase 3 Success Metrics
- ✅ Prometheus scraping metrics
- ✅ Grafana dashboard rendering
- ✅ Alerts firing correctly
- ✅ .onion access working
- ✅ No public exposure

### Phase 4 Success Metrics
- ✅ All test scenarios passing
- ✅ No errors in logs
- ✅ Monitoring operational
- ✅ Documentation complete

### Beta Launch Success Metrics
- ✅ First escrow completed successfully
- ✅ Zero critical errors in 24h
- ✅ Monitoring dashboard shows healthy metrics
- ✅ Dispute system functional
- ✅ No user funds lost

---

## 📞 SUPPORT & ESCALATION

### During Implementation

**Blocker Encountered:**
1. Document the issue
2. Check relevant Reality Check document
3. Consult CLAUDE.md for patterns
4. Run security audit: `./scripts/audit-pragmatic.sh`
5. If stuck > 2h, escalate

**Testing Failures:**
1. Check logs: `tail -f server.log`
2. Verify dependencies running (Tor, Monero RPC, IPFS)
3. Check database state: `sqlite3 marketplace.db`
4. Run validation: `./scripts/validate-beta.sh`

### Post-Launch

**Critical Issue (funds at risk):**
1. Stop accepting new escrows
2. Alert all active users
3. Review logs and monitoring
4. Manual intervention via admin dashboard

**Performance Issue:**
1. Check Grafana dashboard
2. Review resource usage (CPU, memory, disk)
3. Check for stuck escrows
4. Scale resources if needed

---

## 📝 DOCUMENT CHANGELOG

**v1.0 UNIFIED (2025-11-12)**
- Merged PRODUCTION-READINESS-ROADMAP.md and AMAZAWN-V1-BETA-DISPUTE-SYSTEM.md
- Retained Phase 1 (Race Condition Fix) from PRODUCTION-READINESS
- Retained Phase 2 (Arbitration System) from AMAZAWN
- Retained Phase 3 (Monitoring) - identical in both documents
- Integrated Day 0 blocker resolution with simplified fee sweep approach
- Unified timeline: 9 days total
- Single source of truth for production launch

**Previous Versions:**
- PRODUCTION-READINESS-ROADMAP.md v1.0 (2025-11-11)
- AMAZAWN-V1-BETA-DISPUTE-SYSTEM.md v2.2 (2025-11-12)

---

## ✅ FINAL CHECKLIST

Before starting Day 0:
```
□ This document reviewed and understood
□ All dependencies installed (Rust, Diesel, Docker, Tor, IPFS)
□ Repository up to date
□ Development environment configured
□ Team aligned on timeline
```

Ready to begin implementation:
```
□ Coffee prepared ☕
□ Music playlist ready 🎵
□ Focus mode activated 🎯
□ Let's build! 🚀
```

---

**END OF MASTER IMPLEMENTATION ROADMAP**

This document is the single source of truth for production launch.
All implementation should follow this roadmap exactly.
Questions? Consult this document first.

Good luck! 🎉
