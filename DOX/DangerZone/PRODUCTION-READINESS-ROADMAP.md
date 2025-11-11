# PRODUCTION READINESS ROADMAP
**Monero Marketplace - Critical Blockers Resolution Plan**

**Document Version:** 1.0
**Date:** 2025-11-11
**Status:** ACTIVE IMPLEMENTATION PLAN
**Architecture:** 100% Non-Custodial (Manual Escalation)

---

## 🔴 EXECUTIVE SUMMARY

### Critical Blockers Identified

**Analysis Date:** 2025-11-11
**Pre-Launch Status:** NOT READY FOR PRODUCTION
**Estimated Time to Production-Ready:** 5-6 days (120-144h effective work)

**Three Critical Issues Blocking Launch:**

1. **BLOCKER #1: Race Condition on Wallet RPC → Escrow → Role Mapping**
   - **Severity:** CRITICAL (potential fund loss)
   - **Current Status:** `temp-escrow-needs-refactor` TODO in codebase
   - **Impact:** Concurrent transactions can corrupt wallet-escrow associations
   - **Fix Timeline:** 2 days

2. **BLOCKER #2: No Manual Recovery Plan for Stuck Transactions**
   - **Severity:** CRITICAL (funds can be locked indefinitely)
   - **Current Status:** Only timeout alerts, no resolution path
   - **Impact:** Transaction stuck for 48h+ = no recovery mechanism
   - **Fix Timeline:** 2 days

3. **URGENT #3: No Production Monitoring Infrastructure**
   - **Severity:** HIGH (cannot operate safely without visibility)
   - **Current Status:** No metrics, no dashboard, no alerting
   - **Impact:** Cannot detect issues in real-time
   - **Fix Timeline:** 1-2 days

### Architecture Decision: 100% Non-Custodial

**Key Constraint:** Server does NOT control arbiter wallet keys.

**Implications:**
- ✅ True non-custodial (aligns with Haveno philosophy)
- ❌ No auto-refund possible after timeout
- ⚠️ Requires manual arbitration for all stuck transactions
- ⚠️ Slower dispute resolution, but more decentralized

**Why this matters:**
In a 2-of-3 multisig system, releasing funds requires 2 signatures. If server doesn't control arbiter keys:
- **Release funds:** Buyer + Arbiter sign → Cannot force if arbiter down
- **Refund funds:** Vendor + Arbiter sign → Cannot force if arbiter down
- **Dispute resolution:** Arbiter + Winner sign → Requires active arbiter

**Alternative (rejected for v1):** Server controls arbiter = quasi-custodial = against project philosophy.

### Beta Launch Limits

- **Max per escrow:** 5 XMR (~€1000 at current rates)
- **Total beta volume:** 50 XMR max during first month
- **Testnet validation:** Minimum 100 transactions before mainnet
- **Monitoring:** 24/7 for first 2 weeks of beta

---

## 📋 PHASE 1: FIX RACE CONDITION (BLOQUANT)
**Duration:** 2 days (16h effective)
**Priority:** P0 - Must be completed before any other work

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
T+0ms: User A creates Escrow_1 (buyer)
T+5ms: User B creates Escrow_2 (buyer)
T+10ms: Both register wallet RPC with role "buyer"
T+15ms: Wallet manager assigns SAME wallet_id to different escrows
T+20ms: Escrow_1 and Escrow_2 share wallet → FUNDS LOSS RISK
```

**Why This Is Critical:**
- No unique constraint on (escrow_id, role) mapping
- HashMap in memory can be overwritten by concurrent requests
- If server restarts, mapping is lost (no persistence)
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
    pub buyer_rpc_user: Option<String>,
    pub buyer_rpc_password: Option<String>,
    pub vendor_user_id: String,
    pub vendor_rpc_url: String,
    pub vendor_rpc_user: Option<String>,
    pub vendor_rpc_password: Option<String>,
    pub arbiter_rpc_url: String,
}

impl EscrowWalletMapping {
    /// Create initial mapping when escrow is created
    pub fn create(conn: &mut SqliteConnection, new_mapping: NewEscrowWalletMapping) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(escrow_wallet_mappings::table)
            .values(&new_mapping)
            .execute(conn)?;

        escrow_wallet_mappings::table
            .filter(escrow_wallet_mappings::escrow_id.eq(new_mapping.escrow_id))
            .first(conn)
    }

    /// Update buyer wallet ID after registration
    pub fn update_buyer_wallet_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
        wallet_id: String,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(escrow_wallet_mappings::table.filter(escrow_wallet_mappings::escrow_id.eq(escrow_id)))
            .set((
                escrow_wallet_mappings::buyer_wallet_id.eq(wallet_id),
                escrow_wallet_mappings::buyer_registered_at.eq(diesel::dsl::now),
                escrow_wallet_mappings::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
    }

    /// Update vendor wallet ID after registration
    pub fn update_vendor_wallet_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
        wallet_id: String,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(escrow_wallet_mappings::table.filter(escrow_wallet_mappings::escrow_id.eq(escrow_id)))
            .set((
                escrow_wallet_mappings::vendor_wallet_id.eq(wallet_id),
                escrow_wallet_mappings::vendor_registered_at.eq(diesel::dsl::now),
                escrow_wallet_mappings::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
    }

    /// Update arbiter wallet ID after registration
    pub fn update_arbiter_wallet_id(
        conn: &mut SqliteConnection,
        escrow_id: String,
        wallet_id: String,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(escrow_wallet_mappings::table.filter(escrow_wallet_mappings::escrow_id.eq(escrow_id)))
            .set((
                escrow_wallet_mappings::arbiter_wallet_id.eq(wallet_id),
                escrow_wallet_mappings::arbiter_registered_at.eq(diesel::dsl::now),
                escrow_wallet_mappings::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
    }

    /// Get wallet mapping by escrow ID
    pub fn find_by_escrow(conn: &mut SqliteConnection, escrow_id: String) -> Result<Self, diesel::result::Error> {
        escrow_wallet_mappings::table
            .filter(escrow_wallet_mappings::escrow_id.eq(escrow_id))
            .first(conn)
    }

    /// Check if all wallets are registered for an escrow
    pub fn all_wallets_registered(&self) -> bool {
        self.buyer_wallet_id.is_some()
            && self.vendor_wallet_id.is_some()
            && self.arbiter_wallet_id.is_some()
    }
}
```

#### 1.3 WalletManager Refactor

**Changes to `server/src/wallet_manager.rs`:**

```rust
use dashmap::DashMap;  // Thread-safe HashMap

pub struct WalletManager {
    // ... existing fields

    // NEW: Lock per escrow to prevent race conditions
    escrow_locks: Arc<DashMap<Uuid, Arc<Mutex<()>>>>,

    // NEW: Database pool for persistence
    db_pool: Option<DbPool>,
}

impl WalletManager {
    /// Register client wallet RPC for a specific escrow and role
    ///
    /// **CRITICAL:** This function is now escrow-scoped to prevent race conditions.
    ///
    /// # Arguments
    /// * `escrow_id` - Unique escrow identifier (NO MORE HARDCODED STRING!)
    /// * `role` - WalletRole (Buyer, Vendor, Arbiter)
    /// * `rpc_url` - Client's wallet RPC URL
    /// * `rpc_user` - Optional RPC username
    /// * `rpc_password` - Optional RPC password
    /// * `recovery_mode` - "manual" or "auto" (for future use)
    ///
    /// # Returns
    /// wallet_id (Uuid) that can be used for subsequent operations
    ///
    /// # Errors
    /// - RPC connection failed
    /// - Escrow already has wallet registered for this role
    /// - Database persistence failed
    pub async fn register_client_wallet_rpc(
        &mut self,
        escrow_id: Uuid,  // ✅ REAL ESCROW ID
        role: WalletRole,
        rpc_url: String,
        rpc_user: Option<String>,
        rpc_password: Option<String>,
        recovery_mode: &str,
    ) -> Result<Uuid, WalletManagerError> {
        // 1. Acquire escrow-specific lock (prevents race conditions)
        let lock = self.escrow_locks
            .entry(escrow_id)
            .or_insert_with(|| Arc::new(Mutex::new(())));
        let _guard = lock.lock().await;

        info!(
            escrow_id = %escrow_id,
            role = ?role,
            rpc_url = %rpc_url,
            "Registering client wallet RPC (escrow-scoped)"
        );

        // 2. Validate RPC URL (localhost or .onion only)
        self.validate_rpc_url(&rpc_url)?;

        // 3. Check if wallet already registered for this (escrow, role) pair
        if let Some(db_pool) = &self.db_pool {
            let mut conn = db_pool.get()
                .map_err(|e| WalletManagerError::RpcError(CommonError::DatabaseError(e.to_string())))?;

            let escrow_id_str = escrow_id.to_string();
            let mapping_result = tokio::task::spawn_blocking(move || {
                EscrowWalletMapping::find_by_escrow(&mut conn, escrow_id_str)
            })
            .await
            .map_err(|e| WalletManagerError::RpcError(CommonError::DatabaseError(e.to_string())))?;

            if let Ok(mapping) = mapping_result {
                // Check if role already has wallet_id
                let already_registered = match role {
                    WalletRole::Buyer => mapping.buyer_wallet_id.is_some(),
                    WalletRole::Vendor => mapping.vendor_wallet_id.is_some(),
                    WalletRole::Arbiter => mapping.arbiter_wallet_id.is_some(),
                };

                if already_registered {
                    return Err(WalletManagerError::InvalidState {
                        expected: format!("{:?} wallet not yet registered", role),
                        actual: format!("{:?} wallet already registered for escrow {}", role, escrow_id),
                    });
                }
            }
        }

        // 4. Create MoneroClient connected to RPC
        let config = MoneroConfig {
            rpc_url: rpc_url.clone(),
            rpc_user: rpc_user.clone(),
            rpc_password: rpc_password.clone(),
        };

        let rpc_client = MoneroClient::new(config)
            .map_err(WalletManagerError::RpcError)?;

        // 5. Test connection by getting wallet address
        let address = rpc_client
            .get_address()
            .await
            .map_err(WalletManagerError::RpcError)?;

        info!(
            escrow_id = %escrow_id,
            role = ?role,
            address = %address[..10],
            "RPC connection successful"
        );

        // 6. Create WalletInstance
        let wallet_id = Uuid::new_v4();
        let instance = WalletInstance {
            id: wallet_id,
            role: role.clone(),
            rpc_client,
            address: address.clone(),
            multisig_state: MultisigState::NotStarted,
            rpc_port: None,  // Client-controlled RPC (no port tracking needed)
        };

        // 7. Store in memory
        self.wallets.insert(wallet_id, instance);

        // 8. Persist to database
        if let Some(db_pool) = &self.db_pool {
            let mut conn = db_pool.get()
                .map_err(|e| WalletManagerError::RpcError(CommonError::DatabaseError(e.to_string())))?;

            let escrow_id_str = escrow_id.to_string();
            let wallet_id_str = wallet_id.to_string();
            let role_clone = role.clone();

            tokio::task::spawn_blocking(move || {
                match role_clone {
                    WalletRole::Buyer => {
                        EscrowWalletMapping::update_buyer_wallet_id(&mut conn, escrow_id_str, wallet_id_str)
                    },
                    WalletRole::Vendor => {
                        EscrowWalletMapping::update_vendor_wallet_id(&mut conn, escrow_id_str, wallet_id_str)
                    },
                    WalletRole::Arbiter => {
                        EscrowWalletMapping::update_arbiter_wallet_id(&mut conn, escrow_id_str, wallet_id_str)
                    },
                }
            })
            .await
            .map_err(|e| WalletManagerError::RpcError(CommonError::DatabaseError(e.to_string())))?
            .map_err(|e| WalletManagerError::RpcError(CommonError::DatabaseError(e.to_string())))?;

            info!(
                escrow_id = %escrow_id,
                role = ?role,
                wallet_id = %wallet_id,
                "Wallet mapping persisted to database"
            );
        }

        Ok(wallet_id)
    }

    /// Validate RPC URL (must be localhost or .onion)
    fn validate_rpc_url(&self, url: &str) -> Result<(), WalletManagerError> {
        use url::Url;

        let parsed = Url::parse(url)
            .map_err(|e| WalletManagerError::InvalidRpcUrl(format!("Invalid URL: {}", e)))?;

        let host = parsed.host_str()
            .ok_or_else(|| WalletManagerError::InvalidRpcUrl("No host in URL".to_string()))?;

        let is_localhost = host.starts_with("127.")
            || host == "localhost"
            || host.starts_with("::1");
        let is_onion = host.ends_with(".onion");

        if !is_localhost && !is_onion {
            return Err(WalletManagerError::InvalidRpcUrl(
                format!("RPC URL must be localhost or .onion, got: {}", host)
            ));
        }

        Ok(())
    }
}
```

#### 1.4 EscrowOrchestrator Changes

**File: `server/src/services/escrow.rs`**

**Before (BROKEN):**
```rust
pub async fn register_client_wallet(
    &self,
    user_id: Uuid,
    role: crate::wallet_manager::WalletRole,
    rpc_url: String,
    rpc_user: Option<String>,
    rpc_password: Option<String>,
) -> Result<(Uuid, String)> {
    // ...
    let wallet_id = wallet_manager
        .register_client_wallet_rpc(
            "temp-escrow-needs-refactor",  // ❌ HARDCODED
            role,
            rpc_url.clone(),
            rpc_user,
            rpc_password,
            "manual",
        )
        .await?;
    // ...
}
```

**After (FIXED):**
```rust
/// Register client wallet RPC for a specific escrow
///
/// **CRITICAL CHANGE:** Now requires escrow_id parameter to prevent race conditions.
///
/// # New Workflow
/// 1. Client creates order → Escrow row created in DB
/// 2. Client registers wallet RPC with escrow_id
/// 3. Server validates (escrow_id, role) is unique
/// 4. Mapping persisted to escrow_wallet_mappings table
///
/// # Arguments
/// * `escrow_id` - Unique escrow identifier (MUST exist in database)
/// * `user_id` - User registering the wallet
/// * `role` - WalletRole (Buyer or Vendor only, Arbiter handled by server)
/// * `rpc_url` - Client's wallet RPC URL
/// * `rpc_user` - Optional RPC username
/// * `rpc_password` - Optional RPC password
///
/// # Returns
/// Tuple of (wallet_id, wallet_address)
///
/// # Errors
/// - Escrow not found
/// - User not authorized for escrow
/// - Role already registered
/// - RPC connection failed
pub async fn register_client_wallet(
    &self,
    escrow_id: Uuid,  // ✅ NEW PARAMETER
    user_id: Uuid,
    role: crate::wallet_manager::WalletRole,
    rpc_url: String,
    rpc_user: Option<String>,
    rpc_password: Option<String>,
) -> Result<(Uuid, String)> {
    info!(
        escrow_id = %escrow_id,
        user_id = %user_id,
        role = ?role,
        "Registering client wallet for escrow"
    );

    // 1. Load escrow from database (verify it exists)
    let mut conn = self.db_pool.get()
        .context("Failed to get DB connection")?;

    let escrow_id_str = escrow_id.to_string();
    let escrow = tokio::task::spawn_blocking(move || {
        use crate::models::escrow::Escrow;
        Escrow::find_by_id(&mut conn, escrow_id_str)
    })
    .await
    .context("Task join error")?
    .context("Escrow not found")?;

    // 2. Verify user is authorized for this escrow and role
    let user_id_str = user_id.to_string();
    let role_matches = match role {
        crate::wallet_manager::WalletRole::Buyer => {
            escrow.buyer_id == user_id_str
        },
        crate::wallet_manager::WalletRole::Vendor => {
            escrow.vendor_id == user_id_str
        },
        crate::wallet_manager::WalletRole::Arbiter => {
            return Err(anyhow::anyhow!(
                "Non-custodial policy: Clients cannot register as arbiter. Arbiter is server-coordinated."
            ));
        },
    };

    if !role_matches {
        return Err(anyhow::anyhow!(
            "User {} is not authorized as {:?} for escrow {}",
            user_id,
            role,
            escrow_id
        ));
    }

    // 3. Register wallet RPC via WalletManager (now escrow-scoped!)
    let mut wallet_manager = self.wallet_manager.lock().await;
    let wallet_id = wallet_manager
        .register_client_wallet_rpc(
            escrow_id,  // ✅ REAL ESCROW ID
            role,
            rpc_url.clone(),
            rpc_user,
            rpc_password,
            "manual",  // Recovery mode (for future use)
        )
        .await
        .context("Failed to register client wallet RPC")?;

    // 4. Get wallet address for response
    let wallet = wallet_manager.wallets.get(&wallet_id)
        .ok_or_else(|| anyhow::anyhow!("Wallet not found after registration"))?;

    let wallet_address = wallet.address.clone();

    info!(
        escrow_id = %escrow_id,
        user_id = %user_id,
        wallet_id = %wallet_id,
        role = ?role,
        "Client wallet registered successfully"
    );

    Ok((wallet_id, wallet_address))
}
```

#### 1.5 API Handler Changes

**File: `server/src/handlers/escrow.rs`**

**Endpoint: POST /api/escrow/:id/register-wallet-rpc**

**Changes:**
```rust
/// Register client's wallet RPC endpoint (NON-CUSTODIAL)
///
/// **BREAKING CHANGE:** Now requires escrow_id in URL path.
///
/// # New Endpoint
/// POST /api/escrow/:escrow_id/register-wallet-rpc
///
/// # Request Body (unchanged)
/// {
///   "rpc_url": "http://127.0.0.1:18082/json_rpc",
///   "rpc_user": "optional",
///   "rpc_password": "optional",
///   "role": "buyer"
/// }
pub async fn register_wallet_rpc(
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,  // ✅ NOW INCLUDES ESCROW_ID
    payload: web::Json<RegisterWalletRpcRequest>,
) -> impl Responder {
    // ... authentication logic (unchanged)

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id in path"
            }));
        }
    };

    // Parse role
    let role = match payload.role.to_lowercase().as_str() {
        "buyer" => crate::wallet_manager::WalletRole::Buyer,
        "vendor" => crate::wallet_manager::WalletRole::Vendor,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid role: must be 'buyer' or 'vendor'"
            }));
        }
    };

    // Register client wallet RPC via orchestrator
    match escrow_orchestrator
        .register_client_wallet(
            escrow_id,  // ✅ PASS REAL ESCROW ID
            user_id,
            role.clone(),
            payload.rpc_url.clone(),
            payload.rpc_user.clone(),
            payload.rpc_password.clone(),
        )
        .await
    {
        Ok((wallet_id, wallet_address)) => {
            HttpResponse::Ok().json(RegisterWalletRpcResponse {
                success: true,
                message: "✅ Wallet RPC registered successfully.".to_string(),
                wallet_id: wallet_id.to_string(),
                wallet_address,
                role: payload.role.clone(),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to register wallet RPC: {}", e)
            }))
        }
    }
}
```

### 1.6 Testing Strategy

**Test File: `server/tests/race_condition_tests.rs`**

```rust
#[tokio::test]
async fn test_concurrent_wallet_registration_same_escrow() {
    // Setup: Create escrow in DB
    let escrow_id = Uuid::new_v4();
    // ... create escrow in database

    // Attempt: Two buyers try to register wallet simultaneously
    let escrow_id_1 = escrow_id;
    let escrow_id_2 = escrow_id;

    let handle1 = tokio::spawn(async move {
        // Buyer 1 registers
        register_wallet_for_escrow(escrow_id_1, WalletRole::Buyer, "http://127.0.0.1:18082").await
    });

    let handle2 = tokio::spawn(async move {
        // Buyer 2 registers (should fail)
        register_wallet_for_escrow(escrow_id_2, WalletRole::Buyer, "http://127.0.0.1:18083").await
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // Assert: One succeeds, one fails
    assert!(
        (result1.is_ok() && result2.is_err()) || (result1.is_err() && result2.is_ok()),
        "Exactly one registration should succeed"
    );
}

#[tokio::test]
async fn test_10_concurrent_escrows() {
    // Setup: Create 10 escrows
    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let escrow_id = Uuid::new_v4();
            // Create escrow
            // Register buyer wallet
            // Register vendor wallet
            // Verify mapping is correct
            escrow_id
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: All 10 escrows have unique wallet mappings
    let escrow_ids: HashSet<_> = results.into_iter()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(escrow_ids.len(), 10, "All 10 escrows should have unique mappings");
}
```

### 1.7 Implementation Checklist

**Day 1 (8 hours):**
- [ ] Create migration file `{timestamp}_create_escrow_wallet_mappings`
- [ ] Write up.sql and down.sql
- [ ] Run `diesel migration run`
- [ ] Verify migration applied: `diesel migration list`
- [ ] Update `schema.rs`: `diesel print-schema > server/src/schema.rs`
- [ ] Create `models/escrow_wallet_mapping.rs`
- [ ] Add to `models/mod.rs`
- [ ] Refactor `WalletManager::register_client_wallet_rpc()` signature
- [ ] Add `escrow_locks: Arc<DashMap<Uuid, Arc<Mutex<()>>>>` field
- [ ] Implement escrow-scoped locking logic
- [ ] Add RPC URL validation

**Day 2 (8 hours):**
- [ ] Refactor `EscrowOrchestrator::register_client_wallet()` to accept `escrow_id`
- [ ] Update all call sites in `handlers/escrow.rs`
- [ ] Update API endpoint path: `/api/escrow/:id/register-wallet-rpc`
- [ ] Write unit tests for `EscrowWalletMapping` model
- [ ] Write integration test for concurrent registration
- [ ] Write stress test: 10 concurrent escrows
- [ ] Run `cargo test --package server -- --nocapture`
- [ ] Run `./scripts/audit-pragmatic.sh` (verify score 100/100)
- [ ] Manual test: Create escrow, register buyer, register vendor
- [ ] Manual test: Attempt double registration (should fail)

**Validation Criteria:**
- ✅ All tests pass
- ✅ No compiler warnings
- ✅ Audit score 100/100
- ✅ Manual double-registration blocked
- ✅ 10 concurrent escrows all succeed with unique mappings

---

## 📋 PHASE 2: MANUAL RECOVERY ARCHITECTURE (100% NON-CUSTODIAL)
**Duration:** 2 days (16h effective)
**Priority:** P0 - Required for production launch

### Problem Statement

**Current State:**
- TimeoutMonitor detects stuck transactions after 6h
- Sends WebSocket alert to admin
- **NO RESOLUTION PATH** - admin has no tools to fix it

**Stuck Transaction Scenarios:**

1. **"releasing" + Vendor RPC down:**
   - Buyer wants funds released to vendor
   - Transaction requires Buyer + Arbiter signatures
   - Vendor RPC offline = cannot receive funds
   - Current: Alert at T+6h, no action

2. **"refunding" + Buyer RPC down:**
   - Funds need to be refunded to buyer
   - Transaction requires Vendor + Arbiter signatures
   - Buyer RPC offline = cannot receive refund
   - Current: Alert at T+6h, no action

3. **"disputed" + Arbiter unresponsive:**
   - Dispute needs resolution
   - Arbiter must sign transaction
   - Arbiter offline/refuses = deadlock
   - Current: Escalate at T+7days, no action

**Key Constraint: Server does NOT control arbiter keys.**
- ✅ Aligns with non-custodial philosophy
- ❌ Means server CANNOT force transactions
- ⚠️ Requires manual intervention from actual arbiter

### Solution: Manual Escalation Workflow

#### 2.1 Enhanced TimeoutMonitor

**File: `server/src/services/timeout_monitor.rs`**

**New Detection Logic:**
```rust
/// Enhanced timeout handling with escalation workflow
async fn handle_transaction_timeout(&self, escrow_id: Uuid, escrow: Escrow) -> Result<()> {
    let hours_stuck = self.hours_since_activity(&escrow);
    let tx_hash = escrow.transaction_hash.as_ref();

    match (escrow.status.as_str(), hours_stuck) {
        // Stage 1: Initial detection (6h)
        ("releasing" | "refunding", 6..12) => {
            self.send_stuck_alert_stage1(escrow_id, &escrow, hours_stuck).await?;
        },

        // Stage 2: Escalation (12h)
        ("releasing" | "refunding", 12..24) => {
            self.send_stuck_alert_stage2(escrow_id, &escrow, hours_stuck).await?;
        },

        // Stage 3: Critical (24h)
        ("releasing" | "refunding", 24..48) => {
            self.send_stuck_alert_stage3(escrow_id, &escrow, hours_stuck).await?;
        },

        // Stage 4: Manual intervention required (48h+)
        ("releasing" | "refunding", 48..) => {
            self.escalate_to_manual_recovery(escrow_id, &escrow, hours_stuck).await?;
        },

        // Dispute timeout (7 days)
        ("disputed", hours) if hours >= 168 => {  // 7 days = 168 hours
            self.escalate_dispute_to_admin(escrow_id, &escrow, hours).await?;
        },

        _ => {
            // Existing alert logic for other states
            self.send_generic_stuck_alert(escrow_id, &escrow).await?;
        }
    }

    Ok(())
}

/// Stage 1: Initial stuck transaction alert (T+6h)
async fn send_stuck_alert_stage1(
    &self,
    escrow_id: Uuid,
    escrow: &Escrow,
    hours_stuck: u64,
) -> Result<()> {
    warn!(
        escrow_id = %escrow_id,
        status = %escrow.status,
        hours_stuck = hours_stuck,
        "Transaction stuck - Stage 1 alert"
    );

    // Parse party IDs
    let buyer_id = escrow.buyer_id.parse::<Uuid>()?;
    let vendor_id = escrow.vendor_id.parse::<Uuid>()?;
    let arbiter_id = escrow.arbiter_id.parse::<Uuid>()?;

    // Determine who needs to act
    let (action_required, responsible_party) = match escrow.status.as_str() {
        "releasing" => (
            "Vendor: Ensure your wallet RPC is online to receive funds",
            vendor_id
        ),
        "refunding" => (
            "Buyer: Ensure your wallet RPC is online to receive refund",
            buyer_id
        ),
        _ => ("Unknown action", arbiter_id),
    };

    // Notify all parties
    for user_id in [buyer_id, vendor_id, arbiter_id] {
        self.websocket.do_send(NotifyUser {
            user_id,
            event: WsEvent::TransactionStuck {
                escrow_id,
                status: escrow.status.clone(),
                hours_stuck,
                stage: 1,
                action_required: if user_id == responsible_party {
                    action_required.to_string()
                } else {
                    "Waiting for other party's RPC to come online".to_string()
                },
            },
        });
    }

    Ok(())
}

/// Stage 4: Manual recovery required (T+48h)
async fn escalate_to_manual_recovery(
    &self,
    escrow_id: Uuid,
    escrow: &Escrow,
    hours_stuck: u64,
) -> Result<()> {
    error!(
        escrow_id = %escrow_id,
        status = %escrow.status,
        hours_stuck = hours_stuck,
        "Transaction stuck for 48h+ - MANUAL RECOVERY REQUIRED"
    );

    // Update escrow status to indicate manual recovery needed
    let mut conn = self.db.get().context("Failed to get DB connection")?;
    let escrow_id_str = escrow_id.to_string();
    let new_status = format!("{}_recovery_needed", escrow.status);

    tokio::task::spawn_blocking(move || {
        use crate::models::escrow::Escrow;
        Escrow::update_status(&mut conn, escrow_id_str, &new_status)
    })
    .await
    .context("Task join error")??;

    // Send critical alert to admin dashboard
    self.websocket.do_send(WsEvent::ManualRecoveryRequired {
        escrow_id,
        original_status: escrow.status.clone(),
        hours_stuck,
        buyer_id: escrow.buyer_id.parse()?,
        vendor_id: escrow.vendor_id.parse()?,
        arbiter_id: escrow.arbiter_id.parse()?,
        amount_atomic: escrow.amount as u64,
        multisig_address: escrow.multisig_address.clone(),
        suggested_actions: vec![
            "Contact arbiter to sign transaction manually".to_string(),
            "Verify RPC connectivity of all parties".to_string(),
            "Consider emergency arbiter replacement (governance required)".to_string(),
        ],
    });

    // Log to audit trail
    info!(
        escrow_id = %escrow_id,
        "Manual recovery escalation logged - admin intervention required"
    );

    Ok(())
}
```

#### 2.2 Admin Dashboard API

**New File: `server/src/handlers/admin_recovery.rs`**

```rust
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::services::escrow::EscrowOrchestrator;

/// List all escrows requiring manual recovery
///
/// # Endpoint
/// GET /api/admin/recovery/list
///
/// # Authentication
/// Requires admin session
#[actix_web::get("/admin/recovery/list")]
pub async fn list_recovery_cases(
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    // Verify admin authentication
    let is_admin = match session.get::<bool>("is_admin") {
        Ok(Some(true)) => true,
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Admin authentication required"
            }));
        }
    };

    // Load all escrows with status ending in "_recovery_needed"
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    let recovery_cases = match tokio::task::spawn_blocking(move || {
        use crate::schema::escrows::dsl::*;
        use diesel::prelude::*;

        escrows
            .filter(status.like("%_recovery_needed"))
            .or_filter(status.eq("disputed"))
            .order(last_activity_at.asc())  // Oldest first
            .load::<crate::models::escrow::Escrow>(&mut conn)
    })
    .await
    {
        Ok(Ok(cases)) => cases,
        Ok(Err(e)) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Query error: {}", e)
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Task error: {}", e)
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "total_cases": recovery_cases.len(),
        "cases": recovery_cases,
    }))
}

/// Request body for manual recovery action
#[derive(Debug, Deserialize, Validate)]
pub struct ManualRecoveryRequest {
    #[validate(custom = "validate_recovery_action")]
    pub action: String,  // "retry_transaction", "contact_arbiter", "mark_investigated"

    #[validate(length(min = 10, max = 1000))]
    pub admin_notes: String,
}

fn validate_recovery_action(action: &str) -> Result<(), validator::ValidationError> {
    match action {
        "retry_transaction" | "contact_arbiter" | "mark_investigated" | "escalate_governance" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_recovery_action")),
    }
}

/// Execute manual recovery action
///
/// # Endpoint
/// POST /api/admin/recovery/:escrow_id/action
///
/// # Request Body
/// {
///   "action": "retry_transaction",
///   "admin_notes": "Contacted arbiter via Signal, they will sign within 2h"
/// }
#[actix_web::post("/admin/recovery/{escrow_id}/action")]
pub async fn execute_recovery_action(
    pool: web::Data<DbPool>,
    orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,
    payload: web::Json<ManualRecoveryRequest>,
) -> impl Responder {
    // Verify admin authentication
    match session.get::<bool>("is_admin") {
        Ok(Some(true)) => {},
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Admin authentication required"
            }));
        }
    };

    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Parse escrow_id
    let escrow_id = match path.into_inner().parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Execute action based on type
    match payload.action.as_str() {
        "retry_transaction" => {
            // Attempt to retry transaction (requires RPC to be back online)
            match orchestrator.retry_stuck_transaction(escrow_id).await {
                Ok(tx_hash) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "action": "retry_transaction",
                        "tx_hash": tx_hash,
                        "message": "Transaction successfully broadcast"
                    }))
                },
                Err(e) => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Retry failed: {}", e),
                        "suggestion": "Verify all party RPCs are online, or escalate to governance"
                    }))
                }
            }
        },

        "contact_arbiter" => {
            // Log admin contact attempt
            // TODO: Integrate with external notification system (Signal, email, etc.)
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "action": "contact_arbiter",
                "message": "Contact logged. Manual follow-up required.",
                "admin_notes": payload.admin_notes
            }))
        },

        "mark_investigated" => {
            // Update escrow metadata to indicate admin has investigated
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "action": "mark_investigated",
                "message": "Case marked as investigated",
                "admin_notes": payload.admin_notes
            }))
        },

        "escalate_governance" => {
            // Escalate to community governance (future: DAO vote, bonded arbiter replacement)
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "action": "escalate_governance",
                "message": "Case escalated to governance. Awaiting community decision.",
                "admin_notes": payload.admin_notes
            }))
        },

        _ => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Unknown action"
            }))
        }
    }
}
```

#### 2.3 Retry Logic in EscrowOrchestrator

**File: `server/src/services/escrow.rs`**

```rust
impl EscrowOrchestrator {
    /// Retry a stuck transaction (admin-initiated)
    ///
    /// This function attempts to re-broadcast a transaction that is stuck
    /// in "releasing" or "refunding" status.
    ///
    /// # Prerequisite
    /// All party RPCs must be online and reachable.
    ///
    /// # Process
    /// 1. Load escrow and verify status is "*_recovery_needed"
    /// 2. Reload wallet states from multisig_state_snapshots
    /// 3. Attempt to recreate and sign transaction
    /// 4. Broadcast to network
    /// 5. Update escrow status to original (remove "_recovery_needed")
    ///
    /// # Returns
    /// Transaction hash on success
    ///
    /// # Errors
    /// - Escrow not in recovery state
    /// - RPC still unreachable
    /// - Transaction already confirmed (check blockchain first)
    pub async fn retry_stuck_transaction(&self, escrow_id: Uuid) -> Result<String> {
        info!(
            escrow_id = %escrow_id,
            "Admin-initiated retry of stuck transaction"
        );

        // 1. Load escrow
        let mut conn = self.db_pool.get().context("Failed to get DB connection")?;
        let escrow_id_str = escrow_id.to_string();

        let escrow = tokio::task::spawn_blocking(move || {
            use crate::models::escrow::Escrow;
            Escrow::find_by_id(&mut conn, escrow_id_str)
        })
        .await
        .context("Task join error")?
        .context("Escrow not found")?;

        // 2. Verify escrow is in recovery state
        if !escrow.status.ends_with("_recovery_needed") {
            return Err(anyhow::anyhow!(
                "Escrow {} is not in recovery state (status: {})",
                escrow_id,
                escrow.status
            ));
        }

        // 3. Determine original action (release or refund)
        let original_status = escrow.status.trim_end_matches("_recovery_needed");

        match original_status {
            "releasing" => {
                // Retry release to vendor
                let vendor_address = self.get_vendor_address(escrow_id).await?;
                self.release_funds_internal(escrow_id, vendor_address).await
            },
            "refunding" => {
                // Retry refund to buyer
                let buyer_address = self.get_buyer_address(escrow_id).await?;
                self.refund_funds_internal(escrow_id, buyer_address).await
            },
            _ => {
                Err(anyhow::anyhow!(
                    "Unknown recovery status: {}",
                    escrow.status
                ))
            }
        }
    }

    /// Internal helper: Get vendor address for escrow
    async fn get_vendor_address(&self, escrow_id: Uuid) -> Result<String> {
        // Load vendor wallet address from escrow_wallet_mappings
        let mut conn = self.db_pool.get().context("Failed to get DB connection")?;
        let escrow_id_str = escrow_id.to_string();

        let mapping = tokio::task::spawn_blocking(move || {
            use crate::models::escrow_wallet_mapping::EscrowWalletMapping;
            EscrowWalletMapping::find_by_escrow(&mut conn, escrow_id_str)
        })
        .await
        .context("Task join error")?
        .context("Wallet mapping not found")?;

        // Connect to vendor's wallet RPC to get address
        let vendor_wallet_id = mapping.vendor_wallet_id
            .ok_or_else(|| anyhow::anyhow!("Vendor wallet not registered"))?;

        let wallet_manager = self.wallet_manager.lock().await;
        let wallet = wallet_manager.wallets.get(&vendor_wallet_id.parse()?)
            .ok_or_else(|| anyhow::anyhow!("Vendor wallet not found in manager"))?;

        Ok(wallet.address.clone())
    }
}
```

### 2.4 WebSocket Events

**File: `server/src/websocket.rs`**

**New Events:**
```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    // ... existing events

    /// Transaction stuck alert with escalation stage
    TransactionStuck {
        escrow_id: Uuid,
        status: String,
        hours_stuck: u64,
        stage: u8,  // 1-4
        action_required: String,
    },

    /// Manual recovery required (Stage 4)
    ManualRecoveryRequired {
        escrow_id: Uuid,
        original_status: String,
        hours_stuck: u64,
        buyer_id: Uuid,
        vendor_id: Uuid,
        arbiter_id: Uuid,
        amount_atomic: u64,
        multisig_address: Option<String>,
        suggested_actions: Vec<String>,
    },

    /// Admin has initiated manual recovery action
    RecoveryActionTaken {
        escrow_id: Uuid,
        action: String,  // "retry_transaction", "contact_arbiter", etc.
        admin_notes: String,
        timestamp: i64,
    },
}
```

### 2.5 Frontend Dashboard (Simple HTML)

**New File: `server/static/admin/recovery-dashboard.html`**

```html
<!DOCTYPE html>
<html>
<head>
    <title>Recovery Dashboard - Monero Marketplace Admin</title>
    <style>
        body { font-family: monospace; background: #1a1a1a; color: #00ff00; }
        .case { border: 1px solid #00ff00; margin: 10px; padding: 10px; }
        .critical { border-color: #ff0000; color: #ff0000; }
        button { background: #00ff00; color: #000; border: none; padding: 5px 10px; cursor: pointer; }
    </style>
</head>
<body>
    <h1>🚨 Manual Recovery Dashboard</h1>
    <div id="cases"></div>

    <script>
        async function loadCases() {
            const res = await fetch('/api/admin/recovery/list');
            const data = await res.json();

            const casesDiv = document.getElementById('cases');
            casesDiv.innerHTML = data.cases.map(c => `
                <div class="case ${c.hours_stuck > 48 ? 'critical' : ''}">
                    <h3>Escrow: ${c.id}</h3>
                    <p>Status: ${c.status}</p>
                    <p>Stuck: ${c.hours_stuck}h</p>
                    <p>Amount: ${c.amount / 1e12} XMR</p>
                    <button onclick="retryTransaction('${c.id}')">Retry Transaction</button>
                    <button onclick="contactArbiter('${c.id}')">Contact Arbiter</button>
                </div>
            `).join('');
        }

        async function retryTransaction(escrowId) {
            const notes = prompt('Admin notes:');
            const res = await fetch(`/api/admin/recovery/${escrowId}/action`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    action: 'retry_transaction',
                    admin_notes: notes
                })
            });
            const data = await res.json();
            alert(data.message || data.error);
            loadCases();
        }

        setInterval(loadCases, 30000);  // Refresh every 30s
        loadCases();
    </script>
</body>
</html>
```

### 2.6 Implementation Checklist

**Day 1 (8 hours):**
- [ ] Enhance `TimeoutMonitor::handle_transaction_timeout()` with 4-stage escalation
- [ ] Implement `send_stuck_alert_stage1()` through `send_stuck_alert_stage4()`
- [ ] Implement `escalate_to_manual_recovery()`
- [ ] Add new WebSocket events: `TransactionStuck`, `ManualRecoveryRequired`
- [ ] Create `handlers/admin_recovery.rs`
- [ ] Implement `list_recovery_cases()` endpoint
- [ ] Implement `execute_recovery_action()` endpoint
- [ ] Add admin authentication middleware

**Day 2 (8 hours):**
- [ ] Implement `EscrowOrchestrator::retry_stuck_transaction()`
- [ ] Implement helper functions: `get_vendor_address()`, `get_buyer_address()`
- [ ] Create admin dashboard HTML (`static/admin/recovery-dashboard.html`)
- [ ] Add admin routes to `main.rs`
- [ ] Write integration test: Simulate stuck transaction → escalation → retry
- [ ] Manual test: Create stuck escrow, verify dashboard shows it
- [ ] Manual test: Retry transaction via dashboard
- [ ] Document recovery workflow in `DOX/guides/MANUAL-RECOVERY-GUIDE.md`

**Validation Criteria:**
- ✅ Stuck transaction detected at T+6h, T+12h, T+24h, T+48h
- ✅ WebSocket alerts sent to all parties
- ✅ Admin dashboard shows recovery cases
- ✅ Manual retry succeeds when RPCs back online
- ✅ Status updated after successful retry

---

## 📋 PHASE 3: PRODUCTION MONITORING
**Duration:** 1-2 days (8-16h effective)
**Priority:** P1 - Required before beta launch

### Problem Statement

**Current State:** Zero visibility into production operations.

**What happens when:**
- 10 escrows are created simultaneously?
- RPC instance goes down?
- Transaction stuck in mempool for 12h?
- Database query takes 5 seconds?

**Answer:** You don't know. You can't see it. You're flying blind.

### Solution: Prometheus + Grafana + Tor

#### 3.1 Metrics Collection

**New File: `server/src/monitoring/metrics.rs`**

```rust
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};
use once_cell::sync::Lazy;

/// Global Prometheus registry
pub static PROMETHEUS_REGISTRY: Lazy<Registry> = Lazy::new(|| Registry::new());

// =============================================================================
// RPC Health Metrics
// =============================================================================

pub static RPC_HEALTH_CHECKS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "rpc_health_checks_total",
        "Total number of RPC health checks performed"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static RPC_FAILURES_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "rpc_failures_total",
        "Total number of RPC connection failures"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static RPC_LATENCY_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "rpc_latency_seconds",
        "RPC request latency in seconds"
    ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]);

    let histogram = Histogram::with_opts(opts).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

// =============================================================================
// Escrow State Metrics
// =============================================================================

pub static ESCROWS_TOTAL: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new(
        "escrows_total",
        "Total number of escrows in system"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static ESCROWS_BY_STATUS: Lazy<IntGaugeVec> = Lazy::new(|| {
    let gauge = IntGaugeVec::new(
        Opts::new("escrows_by_status", "Number of escrows by status"),
        &["status"]
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static STUCK_TRANSACTIONS: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new(
        "stuck_transactions_total",
        "Number of transactions stuck >6h"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static RECOVERY_CASES_OPEN: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new(
        "recovery_cases_open",
        "Number of escrows requiring manual recovery"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

// =============================================================================
// Multisig Performance Metrics
// =============================================================================

pub static MULTISIG_SETUP_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "multisig_setup_duration_seconds",
        "Time to complete full multisig setup"
    ).buckets(vec![10.0, 30.0, 60.0, 120.0, 300.0, 600.0]);

    let histogram = Histogram::with_opts(opts).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

pub static MULTISIG_SYNC_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "multisig_sync_duration_seconds",
        "Time to sync multisig wallets"
    ).buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0]);

    let histogram = Histogram::with_opts(opts).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

// =============================================================================
// Database Performance Metrics
// =============================================================================

pub static DB_QUERY_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "db_query_duration_seconds",
        "Database query duration"
    ).buckets(vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0]);

    let histogram = Histogram::with_opts(opts).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

pub static DB_CONNECTIONS_ACTIVE: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new(
        "db_connections_active",
        "Number of active database connections"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

// =============================================================================
// Financial Metrics (NO AMOUNTS - only counts)
// =============================================================================

pub static FUNDS_RELEASED_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "funds_released_total",
        "Total number of successful fund releases"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static FUNDS_REFUNDED_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "funds_refunded_total",
        "Total number of successful refunds"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static DISPUTES_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "disputes_total",
        "Total number of disputes initiated"
    ).unwrap();
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});
```

#### 3.2 Metrics Endpoint

**File: `server/src/handlers/monitoring.rs` (add to existing)**

```rust
use actix_web::{web, HttpResponse, Responder};
use prometheus::{Encoder, TextEncoder};

/// Prometheus metrics endpoint
///
/// # Endpoint
/// GET /metrics
///
/// # Authentication
/// None (bind to localhost only in production)
///
/// # Example
/// ```bash
/// curl http://127.0.0.1:8080/metrics
/// ```
#[actix_web::get("/metrics")]
pub async fn prometheus_metrics() -> impl Responder {
    use crate::monitoring::metrics::PROMETHEUS_REGISTRY;

    let encoder = TextEncoder::new();
    let metric_families = PROMETHEUS_REGISTRY.gather();

    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}
```

#### 3.3 Automated Setup Script

**New File: `scripts/setup-monitoring.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# Monero Marketplace - Production Monitoring Setup
# =============================================================================
# This script installs and configures:
# - Prometheus (metrics collection)
# - Grafana (dashboard visualization)
# - Tor hidden service (secure access)
#
# Requirements:
# - Ubuntu/Debian system
# - sudo access
# - Tor daemon installed
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# =============================================================================
# 1. Install Dependencies
# =============================================================================

install_dependencies() {
    log_info "Installing Prometheus and Grafana..."

    # Add Grafana repository
    sudo apt-get install -y software-properties-common wget
    wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
    echo "deb https://packages.grafana.com/oss/deb stable main" | sudo tee /etc/apt/sources.list.d/grafana.list

    # Add Prometheus repository
    sudo apt-get update
    sudo apt-get install -y prometheus grafana

    log_info "Dependencies installed"
}

# =============================================================================
# 2. Configure Prometheus
# =============================================================================

configure_prometheus() {
    log_info "Configuring Prometheus..."

    cat > /tmp/prometheus.yml <<EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'monero-marketplace'
    static_configs:
      - targets: ['127.0.0.1:8080']
    metrics_path: '/metrics'
EOF

    sudo mv /tmp/prometheus.yml /etc/prometheus/prometheus.yml
    sudo chown prometheus:prometheus /etc/prometheus/prometheus.yml

    sudo systemctl enable prometheus
    sudo systemctl restart prometheus

    log_info "Prometheus configured and started on http://127.0.0.1:9090"
}

# =============================================================================
# 3. Configure Grafana
# =============================================================================

configure_grafana() {
    log_info "Configuring Grafana..."

    # Configure Grafana to bind to localhost only (Tor will expose it)
    sudo sed -i 's/;http_addr =.*/http_addr = 127.0.0.1/' /etc/grafana/grafana.ini
    sudo sed -i 's/;http_port =.*/http_port = 3000/' /etc/grafana/grafana.ini

    # Disable Grafana analytics (OPSEC)
    sudo sed -i 's/;reporting_enabled =.*/reporting_enabled = false/' /etc/grafana/grafana.ini
    sudo sed -i 's/;check_for_updates =.*/check_for_updates = false/' /etc/grafana/grafana.ini

    sudo systemctl enable grafana-server
    sudo systemctl restart grafana-server

    log_info "Grafana configured and started on http://127.0.0.1:3000"
    log_info "Default credentials: admin / admin (CHANGE IMMEDIATELY)"
}

# =============================================================================
# 4. Configure Tor Hidden Service
# =============================================================================

configure_tor() {
    log_info "Configuring Tor hidden service for Grafana..."

    # Check if Tor is installed
    if ! command -v tor &> /dev/null; then
        log_error "Tor is not installed. Install it first: sudo apt install tor"
        exit 1
    fi

    # Add hidden service config for Grafana
    if ! grep -q "HiddenServiceDir /var/lib/tor/grafana/" /etc/tor/torrc; then
        sudo bash -c 'cat >> /etc/tor/torrc <<EOF

# Monero Marketplace Grafana Dashboard
HiddenServiceDir /var/lib/tor/grafana/
HiddenServicePort 80 127.0.0.1:3000
EOF'
    fi

    sudo systemctl restart tor
    sleep 5

    # Get .onion address
    if [ -f /var/lib/tor/grafana/hostname ]; then
        ONION_ADDRESS=$(sudo cat /var/lib/tor/grafana/hostname)
        log_info "✅ Grafana available at: http://${ONION_ADDRESS}"
        echo "$ONION_ADDRESS" > "$PROJECT_ROOT/grafana.onion"
    else
        log_error "Failed to generate .onion address"
        exit 1
    fi
}

# =============================================================================
# 5. Import Grafana Dashboard
# =============================================================================

import_dashboard() {
    log_info "Importing Monero Marketplace dashboard to Grafana..."

    # Wait for Grafana to be ready
    sleep 10

    # Add Prometheus data source
    curl -X POST http://admin:admin@127.0.0.1:3000/api/datasources \
        -H "Content-Type: application/json" \
        -d '{
            "name": "Prometheus",
            "type": "prometheus",
            "url": "http://127.0.0.1:9090",
            "access": "proxy",
            "isDefault": true
        }' || log_warn "Data source may already exist"

    # Import dashboard JSON
    DASHBOARD_JSON="$PROJECT_ROOT/monitoring/grafana-dashboard.json"
    if [ -f "$DASHBOARD_JSON" ]; then
        curl -X POST http://admin:admin@127.0.0.1:3000/api/dashboards/db \
            -H "Content-Type: application/json" \
            -d @"$DASHBOARD_JSON"
        log_info "Dashboard imported successfully"
    else
        log_warn "Dashboard JSON not found at $DASHBOARD_JSON"
        log_info "You'll need to create dashboards manually"
    fi
}

# =============================================================================
# 6. Validation
# =============================================================================

validate_setup() {
    log_info "Validating monitoring setup..."

    # Check Prometheus
    if curl -s http://127.0.0.1:9090/-/healthy | grep -q "Prometheus is Healthy"; then
        log_info "✅ Prometheus: HEALTHY"
    else
        log_error "❌ Prometheus: UNHEALTHY"
    fi

    # Check Grafana
    if curl -s http://127.0.0.1:3000/api/health | grep -q "ok"; then
        log_info "✅ Grafana: HEALTHY"
    else
        log_error "❌ Grafana: UNHEALTHY"
    fi

    # Check Tor
    if sudo systemctl is-active --quiet tor; then
        log_info "✅ Tor: RUNNING"
    else
        log_error "❌ Tor: NOT RUNNING"
    fi

    # Check marketplace metrics endpoint
    if curl -s http://127.0.0.1:8080/metrics | grep -q "rpc_health_checks_total"; then
        log_info "✅ Marketplace metrics: AVAILABLE"
    else
        log_warn "⚠️  Marketplace metrics: NOT AVAILABLE (start server first)"
    fi
}

# =============================================================================
# Main Execution
# =============================================================================

main() {
    log_info "Starting monitoring setup for Monero Marketplace..."

    install_dependencies
    configure_prometheus
    configure_grafana
    configure_tor
    import_dashboard
    validate_setup

    log_info ""
    log_info "=========================================="
    log_info "Monitoring Setup Complete!"
    log_info "=========================================="
    log_info "Prometheus: http://127.0.0.1:9090"
    log_info "Grafana: http://$(sudo cat /var/lib/tor/grafana/hostname)"
    log_info "Default Grafana credentials: admin/admin"
    log_info ""
    log_info "⚠️  IMPORTANT: Change Grafana password immediately!"
    log_info "⚠️  Access Grafana only via Tor Browser for OPSEC"
    log_info "=========================================="
}

main "$@"
```

#### 3.4 Grafana Dashboard JSON

**New File: `monitoring/grafana-dashboard.json`**

```json
{
  "dashboard": {
    "title": "Monero Marketplace - Production Monitoring",
    "tags": ["monero", "escrow", "production"],
    "timezone": "utc",
    "panels": [
      {
        "id": 1,
        "title": "RPC Health Status",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(rpc_failures_total[5m])",
            "legendFormat": "Failure Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                { "value": 0, "color": "green" },
                { "value": 0.01, "color": "yellow" },
                { "value": 0.1, "color": "red" }
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "title": "Escrows by Status",
        "type": "piechart",
        "targets": [
          {
            "expr": "escrows_by_status",
            "legendFormat": "{{status}}"
          }
        ]
      },
      {
        "id": 3,
        "title": "Stuck Transactions",
        "type": "stat",
        "targets": [
          {
            "expr": "stuck_transactions_total"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                { "value": 0, "color": "green" },
                { "value": 1, "color": "red" }
              ]
            }
          }
        }
      },
      {
        "id": 4,
        "title": "Multisig Setup Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(multisig_setup_duration_seconds_bucket[5m]))",
            "legendFormat": "p95 setup time"
          },
          {
            "expr": "histogram_quantile(0.50, rate(multisig_setup_duration_seconds_bucket[5m]))",
            "legendFormat": "p50 setup time"
          }
        ]
      },
      {
        "id": 5,
        "title": "Open Recovery Cases",
        "type": "stat",
        "targets": [
          {
            "expr": "recovery_cases_open"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                { "value": 0, "color": "green" },
                { "value": 1, "color": "yellow" },
                { "value": 5, "color": "red" }
              ]
            }
          }
        }
      }
    ]
  }
}
```

### 3.5 Implementation Checklist

**Day 1 (4-8 hours):**
- [ ] Create `monitoring/metrics.rs` with all Prometheus metrics
- [ ] Add metrics collection to critical code paths:
  - [ ] `WalletManager::register_client_wallet_rpc()` → RPC_HEALTH_CHECKS_TOTAL
  - [ ] Multisig setup → MULTISIG_SETUP_DURATION_SECONDS
  - [ ] Database queries → DB_QUERY_DURATION_SECONDS
  - [ ] Release/refund → FUNDS_RELEASED_TOTAL / FUNDS_REFUNDED_TOTAL
- [ ] Add `/metrics` endpoint to `handlers/monitoring.rs`
- [ ] Create `scripts/setup-monitoring.sh`
- [ ] Make script executable: `chmod +x scripts/setup-monitoring.sh`
- [ ] Create `monitoring/grafana-dashboard.json`
- [ ] Test locally: `./scripts/setup-monitoring.sh`
- [ ] Verify Prometheus scrapes metrics
- [ ] Verify Grafana displays data
- [ ] Verify .onion address works

**Validation Criteria:**
- ✅ `/metrics` endpoint returns valid Prometheus format
- ✅ Prometheus scrapes every 15s
- ✅ Grafana dashboard shows real-time data
- ✅ Tor hidden service accessible
- ✅ No metrics leak sensitive data (addresses, keys, amounts)

---

## 📋 PHASE 4: BETA LAUNCH PROTOCOL
**Duration:** 1 day (validation)
**Priority:** P0 - Final gate before production

### 4.1 Pre-Launch Testing Checklist

**Testnet Validation (100 transactions minimum):**

```bash
# Test 1: Concurrent Escrow Creation (10 simultaneous)
for i in {1..10}; do
    cargo run --bin cli -- create-escrow \
        --buyer-id "buyer_$i" \
        --vendor-id "vendor_$i" \
        --amount 0.5 &
done
wait

# Validation: All 10 escrows have unique wallet mappings
sqlite3 marketplace.db "SELECT COUNT(DISTINCT escrow_id) FROM escrow_wallet_mappings;"
# Expected: 10

# Test 2: RPC Failure Simulation
# Kill buyer RPC during prepare_multisig
pkill -9 -f "monero-wallet-rpc.*buyer"
# Expected: Multisig setup fails gracefully, timeout at T+1h

# Test 3: Stuck Transaction Recovery
# Create escrow, fund it, kill vendor RPC during release
# Expected: Alert at T+6h, manual recovery available

# Test 4: Dispute Resolution
# Initiate dispute, verify arbiter can resolve
# Expected: Arbiter signs, funds released

# Test 5: Load Test
# 50 escrows in 10 minutes
# Expected: All succeed, no database locks, metrics show <5s per escrow
```

### 4.2 Production Limits (Beta)

**Hardcode limits in `server/src/config/mod.rs`:**

```rust
pub struct BetaLimits {
    pub max_escrow_amount_atomic: u64,  // 5 XMR = 5_000_000_000_000
    pub max_total_volume_atomic: u64,   // 50 XMR = 50_000_000_000_000
    pub max_concurrent_escrows: usize,  // 20
}

impl Default for BetaLimits {
    fn default() -> Self {
        Self {
            max_escrow_amount_atomic: 5_000_000_000_000,  // 5 XMR
            max_total_volume_atomic: 50_000_000_000_000,  // 50 XMR
            max_concurrent_escrows: 20,
        }
    }
}
```

**Enforce in handlers:**
```rust
// In create_escrow handler
let beta_limits = BetaLimits::default();

if escrow.amount > beta_limits.max_escrow_amount_atomic as i64 {
    return HttpResponse::BadRequest().json(serde_json::json!({
        "error": format!(
            "Beta limit: Max {} XMR per escrow",
            beta_limits.max_escrow_amount_atomic as f64 / 1e12
        )
    }));
}
```

### 4.3 Monitoring Checklist (24/7 for first 2 weeks)

**Alerting Rules (Prometheus `alert.rules`):**
```yaml
groups:
  - name: critical
    interval: 30s
    rules:
      - alert: StuckTransaction
        expr: stuck_transactions_total > 0
        for: 1m
        annotations:
          summary: "Transaction stuck for >6h"

      - alert: RecoveryCaseOpen
        expr: recovery_cases_open > 0
        for: 5m
        annotations:
          summary: "Manual recovery required"

      - alert: RPCFailureSpike
        expr: rate(rpc_failures_total[5m]) > 0.1
        for: 2m
        annotations:
          summary: "RPC failure rate >10%"
```

### 4.4 Rollback Plan

**If critical issue detected:**
```bash
# 1. Disable new escrow creation
curl -X POST http://localhost:8080/api/admin/maintenance-mode \
    -d '{"enabled": true, "reason": "Critical issue detected"}'

# 2. List all active escrows
sqlite3 marketplace.db "SELECT id, status FROM escrows WHERE status NOT IN ('completed', 'refunded', 'cancelled');"

# 3. Manual resolution for each active escrow
# (Use admin dashboard)

# 4. Database backup
cp marketplace.db "marketplace.db.backup_$(date +%s)"

# 5. Restart with fix
git checkout hotfix/issue-xxx
cargo build --release
killall -9 server
./target/release/server &
```

---

## 📅 TIMELINE & MILESTONES

### Week 1: Critical Blockers

**Day 1-2: Phase 1 (Race Condition Fix)**
- Migration + schema
- WalletManager refactor
- Tests
- **Milestone:** 10 concurrent escrows succeed

**Day 3-4: Phase 2 (Manual Recovery)**
- TimeoutMonitor enhancement
- Admin dashboard
- Retry logic
- **Milestone:** Stuck transaction can be recovered via dashboard

**Day 5: Phase 3 (Monitoring)**
- Metrics collection
- Prometheus + Grafana setup
- Tor hidden service
- **Milestone:** Dashboard shows real-time escrow states

**Day 6: Phase 4 (Beta Validation)**
- 100 testnet transactions
- Load testing
- Security audit
- **Milestone:** All tests pass, audit score 100/100

**Day 7: BETA LAUNCH** 🚀
- Deploy to mainnet with 5 XMR limit
- 24/7 monitoring
- Ready for first real users

---

## 🎯 SUCCESS CRITERIA

**Phase 1 Complete:**
- [ ] `temp-escrow-needs-refactor` TODO eliminated
- [ ] Database migration applied
- [ ] 10 concurrent escrows tested
- [ ] No race conditions detected

**Phase 2 Complete:**
- [ ] 4-stage escalation workflow implemented
- [ ] Admin dashboard functional
- [ ] Manual retry tested successfully
- [ ] Recovery documentation written

**Phase 3 Complete:**
- [ ] Prometheus metrics collected
- [ ] Grafana dashboard accessible via .onion
- [ ] No sensitive data in metrics
- [ ] Alerts configured

**Phase 4 Complete:**
- [ ] 100+ testnet transactions
- [ ] 5 XMR limit enforced
- [ ] Rollback plan documented
- [ ] Team trained on recovery procedures

---

## 🚨 CRITICAL NOTES

### What This Plan Does NOT Include

**Out of Scope for v1 Beta:**
- Bonded arbiters (requires governance system)
- DAO voting for stuck transactions
- Automatic arbiter replacement
- Cross-chain escrow (Monero only for now)
- Mobile app (web only)

**Future Roadmap (v2):**
- Decentralized arbiter pool with bonding
- Community governance for edge cases
- Multi-sig arbiter rotation
- Enhanced privacy (Dandelion++, etc.)

### Philosophy: 100% Non-Custodial

**This plan maintains non-custodial purity:**
- ✅ Server does NOT control buyer/vendor keys
- ✅ Server does NOT control arbiter keys
- ✅ Server CANNOT force transactions without signatures
- ⚠️ Tradeoff: Slower recovery, but more decentralized

**If you change your mind and want auto-refund:**
- Server must control arbiter wallet
- Becomes quasi-custodial
- Easier recovery, but violates philosophy
- **Not recommended for v1**

---

## 📚 APPENDIX A: File Structure

```
monero.marketplace/
├── DOX/
│   ├── DangerZone/
│   │   └── PRODUCTION-READINESS-ROADMAP.md  ← THIS FILE
│   └── guides/
│       ├── MANUAL-RECOVERY-GUIDE.md  ← Created in Phase 2
│       └── MONITORING-GUIDE.md  ← Created in Phase 3
├── migrations/
│   └── {timestamp}_create_escrow_wallet_mappings/
│       ├── up.sql
│       └── down.sql
├── monitoring/
│   └── grafana-dashboard.json
├── scripts/
│   └── setup-monitoring.sh
├── server/
│   └── src/
│       ├── config/
│       │   └── beta_limits.rs  ← New
│       ├── handlers/
│       │   ├── admin_recovery.rs  ← New
│       │   └── escrow.rs  ← Modified
│       ├── models/
│       │   └── escrow_wallet_mapping.rs  ← New
│       ├── monitoring/
│       │   └── metrics.rs  ← New
│       ├── services/
│       │   ├── escrow.rs  ← Modified
│       │   └── timeout_monitor.rs  ← Modified
│       └── wallet_manager.rs  ← Modified
└── server/tests/
    └── race_condition_tests.rs  ← New
```

---

## 📝 APPENDIX B: Key Decisions Summary

| Decision | Chosen Option | Rationale |
|----------|---------------|-----------|
| **Arbiter Control** | Server does NOT control | 100% non-custodial, aligns with philosophy |
| **Auto-Refund** | NO (manual escalation only) | Requires arbiter signature, no force |
| **Implementation Order** | TODO → Recovery → Monitoring | Sequential reduces complexity |
| **Monitoring Stack** | Prometheus + Grafana + Tor | Industry standard, Tor for OPSEC |
| **Beta Limit** | 5 XMR per escrow | Conservative risk management (~€1000) |
| **Recovery Escalation** | 4 stages (6h/12h/24h/48h) | Graduated response, time for resolution |

---

## 🔐 APPENDIX C: Security Considerations

**OPSEC Rules for Monitoring:**
- ❌ NEVER log wallet addresses in metrics
- ❌ NEVER log .onion addresses
- ❌ NEVER log escrow amounts (use counts only)
- ❌ NEVER expose Grafana on public IP
- ✅ Access Grafana ONLY via Tor
- ✅ Change Grafana password immediately
- ✅ Use strong encryption_key for DB

**Data Retention:**
- Prometheus: 15 days retention
- Grafana: 30 days retention
- Logs: 7 days retention (rotate daily)
- Database: Indefinite (encrypted)

---

**END OF ROADMAP**

**Next Steps:** Begin Phase 1 implementation immediately.

**Questions?** Review this document, then execute.

**Timeline:** 5-6 days to production-ready beta.

**Let's build.** 🚀
