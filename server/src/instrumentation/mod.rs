// server/src/instrumentation/mod.rs
//! Multisig Instrumentation System
//!
//! Provides comprehensive tracing and state capture for multisig escrow operations
//! to enable root cause analysis of race conditions, RPC cache pollution, and
//! state corruption issues.
//!
//! # Architecture
//!
//! - **Events**: Structured event definitions (RPC calls, state changes, errors)
//! - **Snapshots**: Complete wallet state capture at critical points
//! - **Collector**: Thread-safe event aggregation and JSON export
//!
//! # Usage
//!
//! ## Enable Instrumentation
//!
//! Set the `ENABLE_INSTRUMENTATION` environment variable:
//!
//! ```bash
//! export ENABLE_INSTRUMENTATION=1
//! cargo run --bin server
//! ```
//!
//! ## Instrument Your Code
//!
//! ```rust,ignore
//! use crate::instrumentation::{InstrumentationCollector, EventType, WalletSnapshot};
//! use serde_json::json;
//!
//! // Create collector for escrow
//! let collector = InstrumentationCollector::new(escrow_id);
//!
//! // Record events
//! collector.record_rpc_start("prepare_multisig", "buyer", Some(18082)).await;
//!
//! // Capture state snapshots
//! let snapshot = WalletSnapshot::capture(
//!     wallet_id,
//!     "buyer",
//!     &rpc_client,
//!     Some("/path/to/wallet"),
//!     Some(18082),
//! ).await?;
//!
//! collector.record_snapshot(
//!     EventType::SnapshotPreRound1,
//!     "buyer",
//!     snapshot,
//! ).await;
//!
//! // At the end, dump to JSON
//! collector.dump_json(&format!("escrow_{}.json", escrow_id)).await?;
//! ```
//!
//! ## Analyze Results
//!
//! ```bash
//! # View events timeline
//! python tools/analyze_escrow_json.py escrow_abc123.json
//!
//! # Compare failed vs successful escrows
//! diff <(python tools/analyze_escrow_json.py escrow_success.json) \
//!      <(python tools/analyze_escrow_json.py escrow_failed.json)
//! ```
//!
//! # Event Flow Example
//!
//! For a successful 2-of-3 multisig setup:
//!
//! ```text
//! SNAPSHOT_PRE_ROUND1          → Capture initial state (3 wallets)
//! RPC_CALL_START               → prepare_multisig (buyer)
//! RPC_CALL_END                 → prepare_multisig success
//! SNAPSHOT_POST_MAKE_MULTISIG  → Buyer wallet state after prepare
//! RPC_CALL_START               → make_multisig (buyer)
//! RPC_CALL_END                 → make_multisig success
//! SNAPSHOT_PRE_ROUND2          → State before export phase
//! RPC_CALL_START               → export_multisig_info (buyer)
//! RPC_CALL_END                 → export success
//! ... (continues for rounds 2-3)
//! SNAPSHOT_FINAL               → Final multisig wallet ready
//! ```
//!
//! # Troubleshooting
//!
//! If events are not being recorded:
//! - Check `ENABLE_INSTRUMENTATION` is set
//! - Verify collector.is_enabled() returns true
//! - Check file permissions for output directory
//! - Look for errors in server logs

pub mod collector;
pub mod events;
pub mod snapshots;

pub use collector::InstrumentationCollector;
pub use events::{EventType, MultisigEvent};
pub use snapshots::WalletSnapshot;

/// Helper macro to instrument RPC calls
///
/// # Example
///
/// ```rust,ignore
/// instrument_rpc_call!(
///     collector,
///     "prepare_multisig",
///     "buyer",
///     Some(18082),
///     {
///         rpc_client.multisig().prepare_multisig().await?
///     }
/// );
/// ```
#[macro_export]
macro_rules! instrument_rpc_call {
    ($collector:expr, $method:expr, $role:expr, $port:expr, $call:block) => {{
        let start = std::time::Instant::now();

        $collector
            .record_rpc_start($method, $role, $port)
            .await;

        let result = $call;

        let duration_ms = start.elapsed().as_millis() as u64;
        let success = result.is_ok();

        $collector
            .record_rpc_end($method, $role, duration_ms, success, $port)
            .await;

        result
    }};
}
