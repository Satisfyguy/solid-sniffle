// server/src/instrumentation/events.rs
//! Event definitions for multisig instrumentation
//!
//! This module defines structured events to track every step of the multisig
//! setup process, enabling post-mortem analysis of race conditions and state corruption.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::{SystemTime, UNIX_EPOCH};

/// Event types for multisig instrumentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    /// RPC call started
    RpcCallStart,
    /// RPC call completed successfully
    RpcCallEnd,
    /// RPC call failed
    RpcCallError,
    /// Wallet state snapshot taken
    SnapshotPreRound1,
    SnapshotPostMakeMultisig,
    SnapshotPreRound2,
    SnapshotPostExportMultisig,
    SnapshotPreRound3,
    SnapshotPostImportMultisig,
    SnapshotFinal,
    /// State change detected
    StateChange,
    /// File operation (copy, delete, chmod)
    FileOperation,
    /// Cache pollution detected
    CachePollutionDetected,
    /// Final error with full context
    ErrorFinal,
    /// Custom event
    Custom,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EventType::RpcCallStart => "RPC_CALL_START",
            EventType::RpcCallEnd => "RPC_CALL_END",
            EventType::RpcCallError => "RPC_CALL_ERROR",
            EventType::SnapshotPreRound1 => "SNAPSHOT_PRE_ROUND1",
            EventType::SnapshotPostMakeMultisig => "SNAPSHOT_POST_MAKE_MULTISIG",
            EventType::SnapshotPreRound2 => "SNAPSHOT_PRE_ROUND2",
            EventType::SnapshotPostExportMultisig => "SNAPSHOT_POST_EXPORT_MULTISIG",
            EventType::SnapshotPreRound3 => "SNAPSHOT_PRE_ROUND3",
            EventType::SnapshotPostImportMultisig => "SNAPSHOT_POST_IMPORT_MULTISIG",
            EventType::SnapshotFinal => "SNAPSHOT_FINAL",
            EventType::StateChange => "STATE_CHANGE",
            EventType::FileOperation => "FILE_OPERATION",
            EventType::CachePollutionDetected => "CACHE_POLLUTION_DETECTED",
            EventType::ErrorFinal => "ERROR_FINAL",
            EventType::Custom => "CUSTOM",
        };
        write!(f, "{}", s)
    }
}

/// A structured event in the multisig instrumentation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigEvent {
    /// Unique trace ID for this escrow (format: "escrow_id-timestamp_ms")
    pub trace_id: String,

    /// Event timestamp (milliseconds since Unix epoch)
    pub timestamp: u64,

    /// Event type
    pub event_type: EventType,

    /// Role involved (buyer, vendor, arbiter, or "coordinator")
    pub role: String,

    /// RPC port if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_port: Option<u16>,

    /// Additional event-specific details (JSON object)
    pub details: JsonValue,
}

impl MultisigEvent {
    /// Create a new event
    pub fn new(
        trace_id: impl Into<String>,
        event_type: EventType,
        role: impl Into<String>,
        details: JsonValue,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            timestamp: now_ms(),
            event_type,
            role: role.into(),
            rpc_port: None,
            details,
        }
    }

    /// Create a new event with RPC port
    pub fn with_rpc_port(
        trace_id: impl Into<String>,
        event_type: EventType,
        role: impl Into<String>,
        rpc_port: u16,
        details: JsonValue,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            timestamp: now_ms(),
            event_type,
            role: role.into(),
            rpc_port: Some(rpc_port),
            details,
        }
    }
}

/// Get current timestamp in milliseconds since Unix epoch
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_creation() {
        let event = MultisigEvent::new(
            "escrow_123-1699999999999",
            EventType::RpcCallStart,
            "buyer",
            json!({ "method": "prepare_multisig" }),
        );

        assert_eq!(event.trace_id, "escrow_123-1699999999999");
        assert_eq!(event.role, "buyer");
        assert!(event.rpc_port.is_none());
        assert_eq!(event.details["method"], "prepare_multisig");
    }

    #[test]
    fn test_event_with_rpc_port() {
        let event = MultisigEvent::with_rpc_port(
            "escrow_456-1699999999999",
            EventType::RpcCallEnd,
            "vendor",
            18083,
            json!({ "duration_ms": 150 }),
        );

        assert_eq!(event.rpc_port, Some(18083));
    }

    #[test]
    fn test_event_serialization() {
        let event = MultisigEvent::new(
            "trace_789",
            EventType::CachePollutionDetected,
            "arbiter",
            json!({ "reason": "wallet already in multisig mode" }),
        );

        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("CACHE_POLLUTION_DETECTED"));
        assert!(json.contains("arbiter"));
    }
}
