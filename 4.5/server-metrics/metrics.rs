//! Prometheus metrics instrumentation
//!
//! This module exports metrics for:
//! - HTTP request latency/throughput
//! - Escrow state transitions
//! - Database operations
//! - WebSocket connections
//! - Monero RPC calls

use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec,
    HistogramVec, IntCounterVec, IntGaugeVec, TextEncoder, Encoder,
};
use actix_web::{HttpResponse, Result as ActixResult};

lazy_static! {
    // ========================================================================
    // HTTP Metrics
    // ========================================================================

    /// Total HTTP requests by method, path, status
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total HTTP requests received",
        &["method", "path", "status"]
    )
    .expect("Failed to register HTTP_REQUESTS_TOTAL");

    /// HTTP request duration histogram (seconds)
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latency in seconds",
        &["method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Failed to register HTTP_REQUEST_DURATION");

    // ========================================================================
    // Escrow Metrics
    // ========================================================================

    /// Current escrows by state
    pub static ref ESCROW_TOTAL: IntGaugeVec = register_int_gauge_vec!(
        "escrow_total",
        "Total escrows by state",
        &["state"]
    )
    .expect("Failed to register ESCROW_TOTAL");

    /// Escrow state transitions
    pub static ref ESCROW_STATE_TRANSITIONS: IntCounterVec = register_int_counter_vec!(
        "escrow_state_transitions_total",
        "Total escrow state transitions",
        &["from_state", "to_state"]
    )
    .expect("Failed to register ESCROW_STATE_TRANSITIONS");

    /// Last escrow update timestamp (Unix epoch)
    pub static ref ESCROW_LAST_UPDATE: IntGaugeVec = register_int_gauge_vec!(
        "escrow_last_update_timestamp",
        "Timestamp of last escrow update",
        &["escrow_id"]
    )
    .expect("Failed to register ESCROW_LAST_UPDATE");

    // ========================================================================
    // Database Metrics
    // ========================================================================

    /// Database operation duration (seconds)
    pub static ref DB_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "db_operation_duration_seconds",
        "Database operation latency",
        &["operation", "table"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    )
    .expect("Failed to register DB_OPERATION_DURATION");

    /// Database lock wait time (seconds)
    pub static ref DB_LOCK_WAIT_DURATION: HistogramVec = register_histogram_vec!(
        "db_lock_wait_seconds",
        "Time spent waiting for database locks",
        &["operation"],
        vec![0.001, 0.01, 0.1, 1.0, 10.0]
    )
    .expect("Failed to register DB_LOCK_WAIT_DURATION");

    /// Database errors by type
    pub static ref DB_ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "db_errors_total",
        "Total database errors",
        &["error_type"]
    )
    .expect("Failed to register DB_ERRORS_TOTAL");

    // ========================================================================
    // WebSocket Metrics
    // ========================================================================

    /// Active WebSocket connections
    pub static ref WEBSOCKET_CONNECTIONS: IntGaugeVec = register_int_gauge_vec!(
        "websocket_connections_active",
        "Number of active WebSocket connections",
        &["user_type"]
    )
    .expect("Failed to register WEBSOCKET_CONNECTIONS");

    /// Total WebSocket connections (counter)
    pub static ref WEBSOCKET_CONNECTIONS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "websocket_connections_total",
        "Total WebSocket connections opened",
        &["user_type"]
    )
    .expect("Failed to register WEBSOCKET_CONNECTIONS_TOTAL");

    /// WebSocket messages sent
    pub static ref WEBSOCKET_MESSAGES_SENT: IntCounterVec = register_int_counter_vec!(
        "websocket_messages_sent_total",
        "Total WebSocket messages sent",
        &["message_type"]
    )
    .expect("Failed to register WEBSOCKET_MESSAGES_SENT");

    // ========================================================================
    // Monero RPC Metrics
    // ========================================================================

    /// Monero RPC call duration (seconds)
    pub static ref MONERO_RPC_DURATION: HistogramVec = register_histogram_vec!(
        "monero_rpc_duration_seconds",
        "Monero RPC call latency",
        &["method", "wallet"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
    )
    .expect("Failed to register MONERO_RPC_DURATION");

    /// Monero RPC errors
    pub static ref MONERO_RPC_ERRORS: IntCounterVec = register_int_counter_vec!(
        "monero_rpc_errors_total",
        "Total Monero RPC errors",
        &["method", "error_type"]
    )
    .expect("Failed to register MONERO_RPC_ERRORS");

    // ========================================================================
    // System Metrics
    // ========================================================================

    /// Application uptime (seconds)
    pub static ref UPTIME_SECONDS: IntGaugeVec = register_int_gauge_vec!(
        "uptime_seconds",
        "Application uptime in seconds",
        &[]
    )
    .expect("Failed to register UPTIME_SECONDS");
}

/// Actix-Web handler to expose Prometheus metrics
pub async fn metrics_handler() -> ActixResult<HttpResponse> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Metrics encoding failed: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer))
}

/// Record HTTP request
pub fn record_http_request(method: &str, path: &str, status: u16, duration_secs: f64) {
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, path, &status.to_string()])
        .inc();

    HTTP_REQUEST_DURATION
        .with_label_values(&[method, path])
        .observe(duration_secs);
}

/// Update escrow state gauge
pub fn update_escrow_gauge(state: &str, count: i64) {
    ESCROW_TOTAL
        .with_label_values(&[state])
        .set(count);
}

/// Record escrow state transition
pub fn record_escrow_transition(from: &str, to: &str) {
    ESCROW_STATE_TRANSITIONS
        .with_label_values(&[from, to])
        .inc();
}

/// Record database operation
pub fn record_db_operation(operation: &str, table: &str, duration_secs: f64) {
    DB_OPERATION_DURATION
        .with_label_values(&[operation, table])
        .observe(duration_secs);
}

/// Increment WebSocket connection counter
pub fn increment_websocket_connections(user_type: &str) {
    WEBSOCKET_CONNECTIONS
        .with_label_values(&[user_type])
        .inc();

    WEBSOCKET_CONNECTIONS_TOTAL
        .with_label_values(&[user_type])
        .inc();
}

/// Decrement WebSocket connection counter
pub fn decrement_websocket_connections(user_type: &str) {
    WEBSOCKET_CONNECTIONS
        .with_label_values(&[user_type])
        .dec();
}

/// Record Monero RPC call
pub fn record_monero_rpc_call(method: &str, wallet: &str, duration_secs: f64) {
    MONERO_RPC_DURATION
        .with_label_values(&[method, wallet])
        .observe(duration_secs);
}

/// Record Monero RPC error
pub fn record_monero_rpc_error(method: &str, error_type: &str) {
    MONERO_RPC_ERRORS
        .with_label_values(&[method, error_type])
        .inc();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_http_request() {
        record_http_request("GET", "/api/listings", 200, 0.123);

        let metric = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/api/listings", "200"])
            .get();

        assert!(metric > 0);
    }

    #[test]
    fn test_escrow_metrics() {
        update_escrow_gauge("pending", 5);
        record_escrow_transition("pending", "funded");

        let gauge = ESCROW_TOTAL
            .with_label_values(&["pending"])
            .get();

        assert_eq!(gauge, 5);
    }
}