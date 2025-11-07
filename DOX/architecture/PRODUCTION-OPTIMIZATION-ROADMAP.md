# Production Optimization Roadmap

**Status:** 📋 Planned
**Current Capacity:** 1 simultaneous escrow (3 RPC instances)
**Target Capacity:** 10+ simultaneous escrows (30+ RPC instances) with auto-scaling

---

## 🎯 Overview

This roadmap outlines the production optimizations to transform the current working implementation into a highly scalable, production-grade system with:
- Auto-healing (health checks + failover)
- Real-time monitoring (Prometheus + dashboard)
- Auto-scaling (dynamic RPC instance management)
- Batch operations (parallel escrow creation)
- Sub-10s escrow creation time

**Current Metrics:**
- ⏱️ Escrow creation time: ~13s
- 📊 Simultaneous escrows: 1
- ❌ Failure rate: ~5% (no retry logic)
- 🔍 Monitoring: Log files only
- 📈 Scaling: Manual

**Target Metrics:**
- ⏱️ Escrow creation time: <10s (30% improvement)
- 📊 Simultaneous escrows: 10+ (1000% increase)
- ✅ Failure rate: <1% (with retry + failover)
- 🔍 Monitoring: Real-time dashboard
- 📈 Scaling: Automatic at 80% utilization

---

## Phase 1: Stability & Resilience (1-2h) 🚨 PRIORITY

### 1.1 RPC Health Checks with Automatic Failover

**Problem:** If an RPC instance crashes or becomes unresponsive, escrow initialization fails with no recovery.

**Solution:** Health checks before RPC assignment with automatic failover to next available instance.

**Implementation:**

**File:** `server/src/wallet_manager.rs`

```rust
use std::time::Duration;

impl WalletManager {
    /// Get healthy RPC for role with automatic failover
    async fn get_healthy_rpc_for_role(
        &self,
        role: &WalletRole
    ) -> Result<MoneroConfig, WalletManagerError> {
        let role_rpcs = self.get_rpcs_for_role(role)?;

        // Try each RPC until we find a healthy one
        for (attempt, config) in role_rpcs.iter().enumerate() {
            match self.check_rpc_health(&config).await {
                Ok(true) => {
                    info!("✅ Using healthy RPC for {:?}: {} (attempt {})",
                        role, config.rpc_url, attempt + 1);
                    return Ok(config.clone());
                }
                Ok(false) => {
                    warn!("⚠️ RPC unhealthy: {} (attempt {})", config.rpc_url, attempt + 1);
                    continue;
                }
                Err(e) => {
                    error!("❌ Health check failed for {}: {}", config.rpc_url, e);
                    continue;
                }
            }
        }

        error!("❌ No healthy RPC instances available for role={:?}", role);
        Err(WalletManagerError::NoAvailableRpc)
    }

    /// Check if RPC instance is healthy (timeout: 1s)
    async fn check_rpc_health(&self, config: &MoneroConfig) -> Result<bool, WalletManagerError> {
        match tokio::time::timeout(
            Duration::from_secs(1),
            self.ping_rpc(config)
        ).await {
            Ok(Ok(())) => Ok(true),
            Ok(Err(e)) => {
                debug!("RPC health check failed: {}", e);
                Ok(false)
            }
            Err(_) => {
                warn!("RPC health check timeout for {}", config.rpc_url);
                Ok(false)
            }
        }
    }

    /// Quick ping to RPC using get_version (lightweight call)
    async fn ping_rpc(&self, config: &MoneroConfig) -> Result<(), WalletManagerError> {
        let client = MoneroClient::new(config.clone());
        client.get_version().await
            .map(|_| ())
            .map_err(|e| WalletManagerError::RpcUnreachable(e.to_string()))
    }

    /// Get all RPCs assigned to a role
    fn get_rpcs_for_role(&self, role: &WalletRole) -> Result<Vec<MoneroConfig>, WalletManagerError> {
        let rpcs = match role {
            WalletRole::Buyer => {
                self.rpc_configs.iter()
                    .enumerate()
                    .filter(|(i, _)| i % 3 == 0)
                    .map(|(_, config)| config.clone())
                    .collect()
            }
            WalletRole::Vendor => {
                self.rpc_configs.iter()
                    .enumerate()
                    .filter(|(i, _)| i % 3 == 1)
                    .map(|(_, config)| config.clone())
                    .collect()
            }
            WalletRole::Arbiter => {
                self.rpc_configs.iter()
                    .enumerate()
                    .filter(|(i, _)| i % 3 == 2)
                    .map(|(_, config)| config.clone())
                    .collect()
            }
        };

        if rpcs.is_empty() {
            return Err(WalletManagerError::NoAvailableRpc);
        }

        Ok(rpcs)
    }
}
```

**Testing:**
```bash
# Kill one RPC instance during escrow creation
killall -9 -r "monero-wallet-rpc.*18083"

# Expected: System should failover to next vendor RPC (if available) or fail gracefully
```

**Acceptance Criteria:**
- [ ] Health check completes in <1s
- [ ] Automatic failover to next RPC instance
- [ ] Error logged with clear message
- [ ] No crash on all-RPCs-down scenario

---

### ~~1.2 Retry Logic with Exponential Backoff~~ ❌ ABANDONED

**Problem:** Transient network errors or RPC hiccups cause permanent escrow initialization failures.

**Originally Proposed Solution:** Retry failed operations with exponential backoff (3 attempts max).

**Why Abandoned:**

This approach is **incompatible with the ephemeral temporary wallet architecture**:

1. **Wallet lifecycle issue:**
   - Temporary wallets are created → used for multisig → immediately closed
   - Wallets are closed **during** multisig process to free RPC slots
   - Retry logic would trigger **after** wallets already closed

2. **Concrete failure scenario:**
   ```
   ✅ Create buyer_temp_abc123, vendor_temp_xyz789, arbiter_temp_def456
   ✅ Multisig Phase 1: prepare_multisig() succeeds
   ✅ Close wallets to free RPC slots
   ❌ Multisig Phase 2: exchange_keys() fails (network timeout)
   🔄 Retry triggers...
   ❌ CRASH: wallet_id "buyer_temp_abc123" doesn't exist anymore!
   ```

3. **Why we can't "just recreate wallets":**
   - New wallets would have **different wallet IDs**
   - Previous `multisig_info` strings are **wallet-specific** (can't be reused)
   - Would require **full restart** of multisig process with new participants
   - Breaks 2-of-3 coordination (other parties already have old info)

**Alternative Approach:**
- ✅ **Phase 1.1 Health Checks** (upstream prevention) - Keep this
- ✅ **Fail-fast** on errors (let caller handle retry at HTTP level)
- ✅ Better error messages to help diagnose issues
- 🔮 **Future:** Implement persistent wallet sessions (major refactor)

**Decision Date:** 2025-11-07

---

## Phase 2: Monitoring & Observability (2-3h) 📊

### 2.1 Prometheus Metrics Integration

**Implementation:**

**File:** `server/Cargo.toml`
```toml
[dependencies]
prometheus = "0.13"
lazy_static = "1.4"
```

**File:** `server/src/metrics.rs` (new file)

```rust
use prometheus::{
    IntGauge, IntCounter, Histogram, HistogramOpts,
    register_int_gauge, register_int_counter, register_histogram
};
use lazy_static::lazy_static;

lazy_static! {
    // Escrow metrics
    pub static ref ESCROW_CREATION_TIME: Histogram = register_histogram!(
        "escrow_creation_duration_seconds",
        "Time to create escrow including multisig setup",
        vec![5.0, 10.0, 15.0, 20.0, 30.0, 60.0]
    ).unwrap();

    pub static ref ESCROW_CREATED_TOTAL: IntCounter = register_int_counter!(
        "escrow_created_total",
        "Total number of escrows created"
    ).unwrap();

    pub static ref ESCROW_FAILURES_TOTAL: IntCounter = register_int_counter!(
        "escrow_failures_total",
        "Total number of escrow creation failures"
    ).unwrap();

    // RPC pool metrics
    pub static ref RPC_POOL_FREE: IntGauge = register_int_gauge!(
        "rpc_pool_free_slots",
        "Number of free RPC slots"
    ).unwrap();

    pub static ref RPC_POOL_BUSY: IntGauge = register_int_gauge!(
        "rpc_pool_busy_slots",
        "Number of busy RPC slots"
    ).unwrap();

    pub static ref RPC_POOL_TOTAL: IntGauge = register_int_gauge!(
        "rpc_pool_total_slots",
        "Total number of RPC slots"
    ).unwrap();

    // Multisig metrics
    pub static ref MULTISIG_SETUP_TIME: Histogram = register_histogram!(
        "multisig_setup_duration_seconds",
        "Time to complete multisig setup (3 phases)",
        vec![1.0, 2.0, 5.0, 10.0, 15.0, 30.0]
    ).unwrap();

    pub static ref MULTISIG_FAILURES_TOTAL: IntCounter = register_int_counter!(
        "multisig_setup_failures_total",
        "Total number of multisig setup failures"
    ).unwrap();

    pub static ref MULTISIG_RETRIES_TOTAL: IntCounter = register_int_counter!(
        "multisig_retries_total",
        "Total number of multisig retry attempts"
    ).unwrap();

    // Wallet metrics
    pub static ref WALLETS_CREATED_TOTAL: IntCounter = register_int_counter!(
        "wallets_created_total",
        "Total number of temporary wallets created"
    ).unwrap();

    pub static ref WALLET_CREATION_FAILURES: IntCounter = register_int_counter!(
        "wallet_creation_failures_total",
        "Total number of wallet creation failures"
    ).unwrap();
}

/// Update RPC pool metrics
pub fn update_rpc_pool_metrics(total: usize, free: usize, busy: usize) {
    RPC_POOL_TOTAL.set(total as i64);
    RPC_POOL_FREE.set(free as i64);
    RPC_POOL_BUSY.set(busy as i64);
}
```

**Usage in code:**
```rust
// In escrow.rs
use crate::metrics::{ESCROW_CREATION_TIME, ESCROW_CREATED_TOTAL, ESCROW_FAILURES_TOTAL};

pub async fn init_escrow(&self, ...) -> Result<Escrow> {
    let timer = ESCROW_CREATION_TIME.start_timer();

    match self.init_escrow_inner(...).await {
        Ok(escrow) => {
            ESCROW_CREATED_TOTAL.inc();
            timer.observe_duration();
            Ok(escrow)
        }
        Err(e) => {
            ESCROW_FAILURES_TOTAL.inc();
            Err(e)
        }
    }
}
```

**Prometheus endpoint:**

**File:** `server/src/handlers/admin.rs`
```rust
use prometheus::{Encoder, TextEncoder};

#[get("/metrics")]
async fn metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}
```

**Access:** `http://localhost:8080/metrics`

---

### 2.2 Real-Time Dashboard Endpoint

**File:** `server/src/handlers/admin.rs`

```rust
#[derive(Serialize)]
struct DashboardStats {
    rpc_pool: RpcPoolStats,
    escrows: EscrowStats,
    capacity: CapacityStats,
    health: HealthStatus,
}

#[derive(Serialize)]
struct RpcPoolStats {
    total: usize,
    free: usize,
    busy: usize,
    utilization_percent: f64,
}

#[derive(Serialize)]
struct EscrowStats {
    active: usize,
    creation_avg_time_ms: f64,
    creation_p95_time_ms: f64,
    success_rate_percent: f64,
    failures_24h: u64,
}

#[derive(Serialize)]
struct CapacityStats {
    current_max_concurrent: usize,
    recommended_rpc_instances: usize,
    recommended_action: String,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

#[get("/admin/dashboard")]
async fn dashboard(
    pool: web::Data<WalletPool>,
    escrow_service: web::Data<EscrowService>,
) -> impl Responder {
    let pool_stats = pool.stats().await;

    let utilization = (pool_stats.busy as f64 / pool_stats.total as f64) * 100.0;

    let health = if pool_stats.free >= 3 {
        HealthStatus::Healthy
    } else if pool_stats.free >= 1 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Critical
    };

    let stats = DashboardStats {
        rpc_pool: RpcPoolStats {
            total: pool_stats.total,
            free: pool_stats.free,
            busy: pool_stats.busy,
            utilization_percent: utilization,
        },
        escrows: EscrowStats {
            active: escrow_service.get_active_count().await,
            creation_avg_time_ms: get_avg_escrow_time_ms(),
            creation_p95_time_ms: get_p95_escrow_time_ms(),
            success_rate_percent: calculate_success_rate(),
            failures_24h: ESCROW_FAILURES_TOTAL.get(),
        },
        capacity: CapacityStats {
            current_max_concurrent: pool_stats.free / 3,
            recommended_rpc_instances: calculate_recommended_instances(pool_stats),
            recommended_action: if pool_stats.free < 3 {
                "⚠️ URGENT: Add more RPC instances!".to_string()
            } else if utilization > 80.0 {
                "⚡ Consider scaling up for higher load".to_string()
            } else {
                "✅ Capacity OK".to_string()
            },
        },
        health,
    };

    HttpResponse::Ok().json(stats)
}

fn calculate_recommended_instances(stats: PoolStats) -> usize {
    // Recommend 3x current load + 30% buffer
    let current_load = stats.busy;
    let buffer = (current_load as f64 * 0.3).ceil() as usize;
    (current_load + buffer).max(3) // Minimum 3 (1 escrow)
}
```

**Access:** `http://localhost:8080/admin/dashboard`

**Example Response:**
```json
{
  "rpc_pool": {
    "total": 30,
    "free": 18,
    "busy": 12,
    "utilization_percent": 40.0
  },
  "escrows": {
    "active": 4,
    "creation_avg_time_ms": 9500,
    "creation_p95_time_ms": 12000,
    "success_rate_percent": 99.2,
    "failures_24h": 2
  },
  "capacity": {
    "current_max_concurrent": 6,
    "recommended_rpc_instances": 30,
    "recommended_action": "✅ Capacity OK"
  },
  "health": "healthy"
}
```

---

## Phase 3: Auto-Scaling (4-6h) 📈

### 3.1 Hot Reload RPC Configuration

**Problem:** Adding new RPC instances requires server restart.

**Solution:** Hot reload RPC configs without downtime.

**File:** `server/src/wallet_manager.rs`

```rust
use std::sync::RwLock;

pub struct WalletManager {
    rpc_configs: Arc<RwLock<Vec<MoneroConfig>>>,
    // ... other fields
}

impl WalletManager {
    /// Reload RPC configurations from environment without restart
    pub async fn hot_reload_rpc_configs(&self) -> Result<usize, WalletManagerError> {
        let new_urls = std::env::var("MONERO_RPC_URLS")
            .map_err(|_| WalletManagerError::ConfigError("MONERO_RPC_URLS not set".into()))?;

        let new_configs: Vec<MoneroConfig> = new_urls
            .split(',')
            .map(|url| MoneroConfig {
                rpc_url: url.trim().to_string(),
                ..Default::default()
            })
            .collect();

        let mut configs = self.rpc_configs.write()
            .map_err(|_| WalletManagerError::LockError)?;

        let old_count = configs.len();

        // Only add NEW configs (don't disrupt existing)
        for config in new_configs {
            if !configs.iter().any(|c| c.rpc_url == config.rpc_url) {
                info!("➕ Adding new RPC config: {}", config.rpc_url);
                configs.push(config);
            }
        }

        let added = configs.len() - old_count;
        info!("✅ Hot reload complete: added {} new RPC instances (total: {})",
            added, configs.len());

        Ok(added)
    }
}
```

**Handler:**

**File:** `server/src/handlers/admin.rs`

```rust
#[post("/admin/reload-rpc-config")]
async fn reload_rpc_config(
    wallet_manager: web::Data<Arc<Mutex<WalletManager>>>,
) -> impl Responder {
    let manager = wallet_manager.lock().await;

    match manager.hot_reload_rpc_configs().await {
        Ok(added) => HttpResponse::Ok().json(json!({
            "status": "success",
            "added": added,
            "message": format!("Added {} new RPC instances", added)
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Hot reload failed: {}", e)
        }))
    }
}
```

---

### 3.2 Auto-Scaling Script

**File:** `scripts/auto-scale-rpc.sh`

```bash
#!/bin/bash
# Auto-scale RPC instances based on load

set -e

DASHBOARD_URL="http://localhost:8080/admin/dashboard"
RELOAD_URL="http://localhost:8080/admin/reload-rpc-config"
THRESHOLD=80  # Scale up at 80% utilization
WALLET_DIR="/var/monero/wallets"
LOG_DIR="/home/malix/Desktop/monero.marketplace"

echo "📊 Checking RPC pool utilization..."

# Get current utilization from dashboard
UTILIZATION=$(curl -s "$DASHBOARD_URL" | jq -r '.rpc_pool.utilization_percent')
FREE_SLOTS=$(curl -s "$DASHBOARD_URL" | jq -r '.rpc_pool.free')

echo "Current utilization: ${UTILIZATION}%"
echo "Free slots: $FREE_SLOTS"

# Check if scaling is needed
if (( $(echo "$UTILIZATION > $THRESHOLD" | bc -l) )); then
    echo "🚨 Utilization above ${THRESHOLD}% - Scaling up..."

    # Find next available ports (3 per escrow capacity)
    CURRENT_PORT_COUNT=$(pgrep -c "monero-wallet-rpc" || echo 0)
    BASE_PORT=18082
    NEXT_BUYER_PORT=$((BASE_PORT + CURRENT_PORT_COUNT))
    NEXT_VENDOR_PORT=$((NEXT_BUYER_PORT + 1))
    NEXT_ARBITER_PORT=$((NEXT_BUYER_PORT + 2))

    echo "Starting 3 new RPC instances:"
    echo "  - Buyer: port $NEXT_BUYER_PORT"
    echo "  - Vendor: port $NEXT_VENDOR_PORT"
    echo "  - Arbiter: port $NEXT_ARBITER_PORT"

    # Launch buyer RPC
    monero-wallet-rpc \
        --rpc-bind-port $NEXT_BUYER_PORT \
        --disable-rpc-login \
        --wallet-dir "$WALLET_DIR/buyers" \
        --testnet \
        --log-level 2 \
        --offline \
        > "$LOG_DIR/monero-wallet-rpc-$NEXT_BUYER_PORT.log" 2>&1 &

    # Launch vendor RPC
    monero-wallet-rpc \
        --rpc-bind-port $NEXT_VENDOR_PORT \
        --disable-rpc-login \
        --wallet-dir "$WALLET_DIR/vendors" \
        --testnet \
        --log-level 2 \
        --offline \
        > "$LOG_DIR/monero-wallet-rpc-$NEXT_VENDOR_PORT.log" 2>&1 &

    # Launch arbiter RPC
    monero-wallet-rpc \
        --rpc-bind-port $NEXT_ARBITER_PORT \
        --disable-rpc-login \
        --wallet-dir "$WALLET_DIR/arbiters" \
        --testnet \
        --log-level 2 \
        --offline \
        > "$LOG_DIR/monero-wallet-rpc-$NEXT_ARBITER_PORT.log" 2>&1 &

    sleep 3

    # Update environment and hot reload
    CURRENT_URLS=$(echo $MONERO_RPC_URLS)
    NEW_URLS="$CURRENT_URLS,http://127.0.0.1:$NEXT_BUYER_PORT,http://127.0.0.1:$NEXT_VENDOR_PORT,http://127.0.0.1:$NEXT_ARBITER_PORT"

    export MONERO_RPC_URLS="$NEW_URLS"

    # Trigger hot reload
    curl -X POST "$RELOAD_URL"

    echo "✅ Scaled up: Added 3 RPC instances (capacity +1 escrow)"
    echo "New RPC URLs: $NEW_URLS"

elif (( FREE_SLOTS < 3 )); then
    echo "⚠️ WARNING: Only $FREE_SLOTS free slots remaining!"
    echo "Recommendation: Scale up manually or wait for auto-scale trigger"
else
    echo "✅ Utilization OK (${UTILIZATION}%) - No scaling needed"
fi
```

**Cron Job (every 5 minutes):**
```bash
*/5 * * * * /opt/monero-marketplace/scripts/auto-scale-rpc.sh >> /var/log/auto-scale.log 2>&1
```

---

## Phase 4: Performance Optimization (3-4h) ⚡

### 4.1 Batch Escrow Creation

**Problem:** Creating multiple escrows sequentially is slow.

**Solution:** Batch API endpoint for parallel escrow creation.

**File:** `server/src/handlers/orders.rs`

```rust
#[derive(Deserialize)]
struct BatchEscrowRequest {
    orders: Vec<EscrowRequest>,
}

#[derive(Serialize)]
struct BatchEscrowResponse {
    results: Vec<EscrowResult>,
    summary: BatchSummary,
}

#[derive(Serialize)]
struct EscrowResult {
    order_id: Uuid,
    status: String,
    escrow_id: Option<Uuid>,
    multisig_address: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct BatchSummary {
    total: usize,
    successful: usize,
    failed: usize,
    duration_ms: u64,
}

#[post("/api/orders/batch-init-escrow")]
async fn batch_init_escrow(
    req: web::Json<BatchEscrowRequest>,
    escrow_service: web::Data<EscrowService>,
) -> impl Responder {
    let start = std::time::Instant::now();

    info!("📦 Batch escrow creation started: {} orders", req.orders.len());

    // Create all escrows in parallel
    let tasks: Vec<_> = req.orders
        .iter()
        .map(|order_req| {
            let service = escrow_service.clone();
            let order_req = order_req.clone();

            tokio::spawn(async move {
                service.init_escrow(
                    order_req.order_id,
                    order_req.buyer_id,
                    order_req.vendor_id,
                    order_req.amount
                ).await
            })
        })
        .collect();

    // Wait for all tasks
    let results = futures::future::join_all(tasks).await;

    // Collect results
    let mut escrow_results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for (idx, result) in results.into_iter().enumerate() {
        let order_id = req.orders[idx].order_id;

        let escrow_result = match result {
            Ok(Ok(escrow)) => {
                successful += 1;
                EscrowResult {
                    order_id,
                    status: "success".to_string(),
                    escrow_id: Some(escrow.id),
                    multisig_address: Some(escrow.multisig_address.unwrap_or_default()),
                    error: None,
                }
            }
            Ok(Err(e)) => {
                failed += 1;
                EscrowResult {
                    order_id,
                    status: "failed".to_string(),
                    escrow_id: None,
                    multisig_address: None,
                    error: Some(e.to_string()),
                }
            }
            Err(e) => {
                failed += 1;
                EscrowResult {
                    order_id,
                    status: "error".to_string(),
                    escrow_id: None,
                    multisig_address: None,
                    error: Some(format!("Task error: {}", e)),
                }
            }
        };

        escrow_results.push(escrow_result);
    }

    let duration = start.elapsed().as_millis() as u64;

    let response = BatchEscrowResponse {
        results: escrow_results,
        summary: BatchSummary {
            total: req.orders.len(),
            successful,
            failed,
            duration_ms: duration,
        },
    };

    info!("✅ Batch escrow creation complete: {}/{} succeeded in {}ms",
        successful, req.orders.len(), duration);

    HttpResponse::Ok().json(response)
}
```

**Usage:**
```bash
curl -X POST http://localhost:8080/api/orders/batch-init-escrow \
  -H "Content-Type: application/json" \
  -d '{
    "orders": [
      {"order_id": "...", "buyer_id": "...", "vendor_id": "...", "amount": 1.0},
      {"order_id": "...", "buyer_id": "...", "vendor_id": "...", "amount": 2.0},
      {"order_id": "...", "buyer_id": "...", "vendor_id": "...", "amount": 0.5}
    ]
  }'
```

---

### 4.2 Parallel Multisig Phases (Advanced)

**Optimization:** Phase 1 (prepare_multisig) can be parallelized across all 3 wallets.

**File:** `server/src/services/escrow.rs`

```rust
pub async fn setup_multisig_parallel(&self, escrow_id: Uuid) -> Result<String> {
    // Phase 1: Parallel prepare_multisig
    let (buyer_info, vendor_info, arbiter_info) = tokio::join!(
        self.wallet_manager.prepare_multisig(buyer_wallet_id),
        self.wallet_manager.prepare_multisig(vendor_wallet_id),
        self.wallet_manager.prepare_multisig(arbiter_wallet_id),
    );

    let buyer_info = buyer_info?;
    let vendor_info = vendor_info?;
    let arbiter_info = arbiter_info?;

    // Phase 2 & 3: Sequential (must wait for blockchain)
    self.wallet_manager.exchange_multisig_keys(...).await?;
    let address = self.wallet_manager.finalize_multisig(...).await?;

    Ok(address)
}
```

**Expected improvement:** 2-3s reduction in escrow creation time.

---

## Phase 5: Production Deployment (2-3h) 🚀

### 5.1 Docker Compose Configuration

**File:** `docker-compose.yml`

```yaml
version: '3.8'

services:
  # Server
  monero-marketplace:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DB_ENCRYPTION_KEY=${DB_ENCRYPTION_KEY}
      - MONERO_RPC_URLS=http://wallet-buyer-1:18082,http://wallet-vendor-1:18083,http://wallet-arbiter-1:18084,http://wallet-buyer-2:18082,http://wallet-vendor-2:18083,http://wallet-arbiter-2:18084,http://wallet-buyer-3:18082,http://wallet-vendor-3:18083,http://wallet-arbiter-3:18084
    volumes:
      - ./marketplace.db:/app/marketplace.db
    depends_on:
      - wallet-buyer-1
      - wallet-vendor-1
      - wallet-arbiter-1
    restart: always

  # Buyer RPCs (10 replicas for 10 concurrent escrows)
  wallet-buyer-1:
    image: monero-wallet-rpc:latest
    ports:
      - "18082:18082"
    volumes:
      - wallet-buyers:/wallets
    command: >
      --rpc-bind-port 18082
      --disable-rpc-login
      --wallet-dir /wallets
      --testnet
      --offline
    restart: always

  wallet-buyer-2:
    image: monero-wallet-rpc:latest
    ports:
      - "18085:18082"
    volumes:
      - wallet-buyers:/wallets
    command: >
      --rpc-bind-port 18082
      --disable-rpc-login
      --wallet-dir /wallets
      --testnet
      --offline
    restart: always

  # ... (repeat for wallet-buyer-3 through wallet-buyer-10)

  # Vendor RPCs (10 replicas)
  wallet-vendor-1:
    image: monero-wallet-rpc:latest
    ports:
      - "18083:18083"
    volumes:
      - wallet-vendors:/wallets
    command: >
      --rpc-bind-port 18083
      --disable-rpc-login
      --wallet-dir /wallets
      --testnet
      --offline
    restart: always

  # ... (repeat for vendor-2 through vendor-10)

  # Arbiter RPCs (10 replicas)
  wallet-arbiter-1:
    image: monero-wallet-rpc:latest
    ports:
      - "18084:18084"
    volumes:
      - wallet-arbiters:/wallets
    command: >
      --rpc-bind-port 18084
      --disable-rpc-login
      --wallet-dir /wallets
      --testnet
      --offline
    restart: always

  # ... (repeat for arbiter-2 through arbiter-10)

  # Prometheus (metrics)
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    restart: always

  # Grafana (dashboard)
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
    restart: always

volumes:
  wallet-buyers:
  wallet-vendors:
  wallet-arbiters:
  prometheus-data:
  grafana-data:
```

**Start:**
```bash
docker-compose up -d --scale wallet-buyer=10 --scale wallet-vendor=10 --scale wallet-arbiter=10
```

---

## 📊 Expected Results After All Phases

| Metric                      | Before | After Phase 1 | After Phase 2 | After Phase 4 | After Phase 5 |
|-----------------------------|--------|---------------|---------------|---------------|---------------|
| Escrow creation time        | ~13s   | ~12s          | ~12s          | ~9s           | ~8s           |
| Simultaneous escrows        | 1      | 1             | 1             | 10            | 10+           |
| Failure rate                | ~5%    | <1%           | <1%           | <0.5%         | <0.1%         |
| Recovery time (crash)       | Manual | Auto          | Auto          | Auto          | Auto          |
| Monitoring                  | Logs   | Logs          | Dashboard     | Dashboard     | Grafana       |
| Scaling                     | Manual | Manual        | Auto          | Auto          | Docker Scale  |
| Health checks               | None   | ✅            | ✅            | ✅            | ✅            |
| Metrics export              | None   | None          | ✅            | ✅            | ✅            |
| Auto-scaling                | None   | None          | None          | ✅            | ✅            |
| Batch operations            | None   | None          | None          | ✅            | ✅            |

---

## 🚀 Implementation Order

**Priority:**
1. ⚠️ **Phase 1.1** (Health checks) - CRITICAL for stability
2. 📊 **Phase 2.1** (Metrics) - HIGH for observability
3. 📊 **Phase 2.2** (Dashboard) - HIGH for monitoring
4. 📈 **Phase 3.1** (Hot reload) - MEDIUM for operational flexibility
5. 📈 **Phase 3.2** (Auto-scaling) - MEDIUM for growth
6. ⚡ **Phase 4.1** (Batch API) - LOW (nice-to-have)
7. ⚡ **Phase 4.2** (Parallel multisig) - LOW (optimization)
8. 🚀 **Phase 5** (Docker) - LOW (deployment sugar)

**Note:** Phase 1.2 (Retry logic) was abandoned due to architectural incompatibility with ephemeral wallet design. See section 1.2 for details.

**Start with:** Phase 1.1 (Health checks) - Most impactful for production stability.

---

**Last Updated:** 2025-11-07
**Status:** 📋 Planning Revised - Phase 1.2 Abandoned
**Next Action:** Implement Phase 1.1 (RPC health checks with failover)
