// server/src/instrumentation/collector.rs
//! Event collector for multisig instrumentation
//!
//! Provides a thread-safe collector that accumulates events during escrow setup
//! and can dump them to JSON for post-mortem analysis.

use super::events::{EventType, MultisigEvent};
use super::snapshots::WalletSnapshot;
use anyhow::{Context, Result};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Thread-safe collector for instrumentation events
#[derive(Clone)]
pub struct InstrumentationCollector {
    /// Unique trace ID for this escrow session
    trace_id: String,

    /// Collected events (protected by mutex for thread-safety)
    events: Arc<Mutex<Vec<MultisigEvent>>>,

    /// Whether instrumentation is enabled (controlled by env var)
    enabled: bool,
}

impl InstrumentationCollector {
    /// Create a new collector for the given escrow ID
    ///
    /// Instrumentation is enabled if `ENABLE_INSTRUMENTATION` env var is set.
    pub fn new(escrow_id: Uuid) -> Self {
        let trace_id = format!(
            "{}-{}",
            escrow_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
        );

        let enabled = std::env::var("ENABLE_INSTRUMENTATION").is_ok();

        if enabled {
            tracing::info!(
                trace_id = %trace_id,
                "Instrumentation enabled for escrow {}",
                escrow_id
            );
        }

        Self {
            trace_id,
            events: Arc::new(Mutex::new(Vec::new())),
            enabled,
        }
    }

    /// Record a generic event
    pub async fn record_event(
        &self,
        event_type: EventType,
        role: impl Into<String>,
        details: JsonValue,
    ) {
        if !self.enabled {
            return;
        }

        let event = MultisigEvent::new(self.trace_id.clone(), event_type, role, details);

        tracing::debug!(
            trace_id = %self.trace_id,
            event_type = %event.event_type,
            role = %event.role,
            "Recording instrumentation event"
        );

        self.events.lock().await.push(event);
    }

    /// Record an RPC call start event
    pub async fn record_rpc_start(
        &self,
        method: &str,
        role: impl Into<String>,
        rpc_port: Option<u16>,
    ) {
        if !self.enabled {
            return;
        }

        let details = json!({
            "method": method,
            "timestamp": super::events::now_ms(),
        });

        let event = if let Some(port) = rpc_port {
            MultisigEvent::with_rpc_port(
                self.trace_id.clone(),
                EventType::RpcCallStart,
                role,
                port,
                details,
            )
        } else {
            MultisigEvent::new(self.trace_id.clone(), EventType::RpcCallStart, role, details)
        };

        self.events.lock().await.push(event);
    }

    /// Record an RPC call end event
    pub async fn record_rpc_end(
        &self,
        method: &str,
        role: impl Into<String>,
        duration_ms: u64,
        success: bool,
        rpc_port: Option<u16>,
    ) {
        if !self.enabled {
            return;
        }

        let details = json!({
            "method": method,
            "duration_ms": duration_ms,
            "success": success,
        });

        let event_type = if success {
            EventType::RpcCallEnd
        } else {
            EventType::RpcCallError
        };

        let event = if let Some(port) = rpc_port {
            MultisigEvent::with_rpc_port(self.trace_id.clone(), event_type, role, port, details)
        } else {
            MultisigEvent::new(self.trace_id.clone(), event_type, role, details)
        };

        self.events.lock().await.push(event);
    }

    /// Record a wallet state snapshot
    pub async fn record_snapshot(
        &self,
        event_type: EventType,
        role: impl Into<String>,
        snapshot: WalletSnapshot,
    ) {
        if !self.enabled {
            return;
        }

        let details = serde_json::to_value(&snapshot).expect("Failed to serialize snapshot");

        let event = MultisigEvent::new(self.trace_id.clone(), event_type, role, details);

        self.events.lock().await.push(event);
    }

    /// Record an error event with full context
    pub async fn record_error(
        &self,
        role: impl Into<String>,
        error_msg: String,
        context: JsonValue,
    ) {
        if !self.enabled {
            return;
        }

        let details = json!({
            "error": error_msg,
            "context": context,
        });

        let event = MultisigEvent::new(self.trace_id.clone(), EventType::ErrorFinal, role, details);

        tracing::error!(
            trace_id = %self.trace_id,
            error = %error_msg,
            "Recording error event"
        );

        self.events.lock().await.push(event);
    }

    /// Get the total number of recorded events
    pub async fn event_count(&self) -> usize {
        self.events.lock().await.len()
    }

    /// Dump all collected events to a JSON file
    ///
    /// # Arguments
    /// * `output_path` - Path to write JSON file (e.g., "escrow_abc123.json")
    ///
    /// # Returns
    /// Path to the created file if instrumentation is enabled, None otherwise
    pub async fn dump_json(&self, output_path: &str) -> Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        let events = self.events.lock().await;

        let json = serde_json::to_string_pretty(&*events)
            .context("Failed to serialize events to JSON")?;

        fs::write(output_path, json)
            .await
            .with_context(|| format!("Failed to write instrumentation data to {}", output_path))?;

        tracing::info!(
            trace_id = %self.trace_id,
            event_count = events.len(),
            output_path = %output_path,
            "Dumped instrumentation events to JSON"
        );

        Ok(Some(output_path.to_string()))
    }

    /// Get a summary of events by type
    pub async fn summary(&self) -> JsonValue {
        let events = self.events.lock().await;

        let mut by_type: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for event in events.iter() {
            *by_type.entry(event.event_type.to_string()).or_insert(0) += 1;
        }

        json!({
            "trace_id": self.trace_id,
            "total_events": events.len(),
            "by_type": by_type,
            "enabled": self.enabled,
        })
    }

    /// Check if instrumentation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the trace ID for this collector
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collector_disabled_by_default() {
        let collector = InstrumentationCollector::new(Uuid::new_v4());
        assert!(!collector.is_enabled());

        collector
            .record_event(EventType::RpcCallStart, "buyer", json!({}))
            .await;

        assert_eq!(collector.event_count().await, 0);
    }

    #[tokio::test]
    async fn test_collector_records_events() {
        std::env::set_var("ENABLE_INSTRUMENTATION", "1");

        let collector = InstrumentationCollector::new(Uuid::new_v4());
        assert!(collector.is_enabled());

        collector
            .record_event(EventType::RpcCallStart, "buyer", json!({"method": "test"}))
            .await;

        collector
            .record_rpc_end("prepare_multisig", "buyer", 150, true, Some(18082))
            .await;

        assert_eq!(collector.event_count().await, 2);

        std::env::remove_var("ENABLE_INSTRUMENTATION");
    }

    #[tokio::test]
    async fn test_collector_summary() {
        std::env::set_var("ENABLE_INSTRUMENTATION", "1");

        let collector = InstrumentationCollector::new(Uuid::new_v4());

        collector
            .record_event(EventType::RpcCallStart, "buyer", json!({}))
            .await;
        collector
            .record_event(EventType::RpcCallStart, "vendor", json!({}))
            .await;
        collector
            .record_event(EventType::RpcCallEnd, "buyer", json!({}))
            .await;

        let summary = collector.summary().await;

        assert_eq!(summary["total_events"], 3);
        assert_eq!(summary["by_type"]["RPC_CALL_START"], 2);
        assert_eq!(summary["by_type"]["RPC_CALL_END"], 1);

        std::env::remove_var("ENABLE_INSTRUMENTATION");
    }
}
