//! Immutable audit logging system
//!
//! All custodial operations are logged to an append-only audit trail
//! with cryptographic integrity verification.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use sqlx::SqlitePool;

use crate::arbitration::DisputeResolution;

/// Audit logger for custodial operations
pub struct AuditLogger {
    db_pool: SqlitePool,
    /// Previous entry hash (for chain integrity)
    last_hash: Option<String>,
}

/// Type of audit event
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum AuditEventType {
    /// Arbitration attempt
    #[serde(rename = "arbitration_attempt")]
    ArbitrationAttempt,

    /// Resolution decision
    #[serde(rename = "resolution")]
    Resolution,

    /// Transaction signing
    #[serde(rename = "signing")]
    Signing,

    /// Key rotation
    #[serde(rename = "key_rotation")]
    KeyRotation,

    /// Manual review
    #[serde(rename = "manual_review")]
    ManualReview,
}

/// An audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: i64,

    /// Event type
    #[sqlx(try_from = "String")]
    pub event_type: AuditEventType,

    /// Associated escrow/dispute ID
    pub entity_id: String,

    /// Event data (JSON)
    pub data: String,

    /// Entry timestamp
    pub timestamp: DateTime<Utc>,

    /// Hash of this entry
    pub entry_hash: String,

    /// Hash of previous entry (for chain integrity)
    pub previous_hash: Option<String>,

    /// Actor (system or admin username)
    pub actor: String,
}

impl AuditLogger {
    /// Create new audit logger
    ///
    /// # Arguments
    ///
    /// * `db_pool` - Database connection pool
    ///
    /// # Errors
    ///
    /// Returns error if initialization fails
    pub async fn new(db_pool: SqlitePool) -> Result<Self> {
        // Get last hash from database
        let last_hash: Option<String> = sqlx::query_scalar(
            "SELECT entry_hash FROM audit_log ORDER BY id DESC LIMIT 1"
        )
        .fetch_optional(&db_pool)
        .await?;

        tracing::info!(
            chain_initialized = last_hash.is_some(),
            "Audit logger initialized"
        );

        Ok(Self { db_pool, last_hash })
    }

    /// Log an arbitration attempt
    ///
    /// # Arguments
    ///
    /// * `dispute_id` - Dispute being arbitrated
    ///
    /// # Errors
    ///
    /// Returns error if logging fails
    pub async fn log_arbitration_attempt(&mut self, dispute_id: &str) -> Result<()> {
        let data = serde_json::json!({
            "dispute_id": dispute_id,
            "action": "arbitration_attempt"
        });

        self.log_event(
            AuditEventType::ArbitrationAttempt,
            dispute_id,
            &data.to_string(),
            "system",
        )
        .await
    }

    /// Log a resolution decision
    ///
    /// # Arguments
    ///
    /// * `dispute_id` - Dispute ID
    /// * `resolution` - Resolution decision
    ///
    /// # Errors
    ///
    /// Returns error if logging fails
    pub async fn log_resolution(
        &mut self,
        dispute_id: &str,
        resolution: &DisputeResolution,
    ) -> Result<()> {
        let data = serde_json::json!({
            "dispute_id": dispute_id,
            "resolution": resolution
        });

        self.log_event(
            AuditEventType::Resolution,
            dispute_id,
            &data.to_string(),
            "system",
        )
        .await
    }

    /// Log a transaction signing operation
    ///
    /// # Arguments
    ///
    /// * `escrow_id` - Escrow ID
    /// * `tx_data` - Transaction data (hashed, not stored)
    ///
    /// # Errors
    ///
    /// Returns error if logging fails
    ///
    /// # Security
    ///
    /// Only the hash of tx_data is stored, not the actual data
    pub async fn log_signing_operation(&mut self, escrow_id: &str, tx_data: &[u8]) -> Result<()> {
        let tx_hash = hex::encode(Sha3_256::digest(tx_data));

        let data = serde_json::json!({
            "escrow_id": escrow_id,
            "tx_data_hash": tx_hash,
            "action": "sign_transaction"
        });

        self.log_event(
            AuditEventType::Signing,
            escrow_id,
            &data.to_string(),
            "system",
        )
        .await
    }

    /// Log a key rotation event
    ///
    /// # Arguments
    ///
    /// * `old_public_key` - Old public key hex
    /// * `new_public_key` - New public key hex
    /// * `actor` - Admin who performed rotation
    ///
    /// # Errors
    ///
    /// Returns error if logging fails
    pub async fn log_key_rotation(
        &mut self,
        old_public_key: &str,
        new_public_key: &str,
        actor: &str,
    ) -> Result<()> {
        let data = serde_json::json!({
            "old_public_key": old_public_key,
            "new_public_key": new_public_key,
            "action": "key_rotation"
        });

        self.log_event(
            AuditEventType::KeyRotation,
            "key_manager",
            &data.to_string(),
            actor,
        )
        .await
    }

    /// Get audit trail for specific escrow
    ///
    /// # Arguments
    ///
    /// * `escrow_id` - Escrow ID
    ///
    /// # Errors
    ///
    /// Returns error if query fails
    pub async fn get_trail_for_escrow(&self, escrow_id: &str) -> Result<Vec<AuditEntry>> {
        let entries = sqlx::query_as::<_, AuditEntry>(
            "SELECT * FROM audit_log WHERE entity_id = ? ORDER BY id ASC"
        )
        .bind(escrow_id)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch audit trail")?;

        Ok(entries)
    }

    /// Verify audit trail integrity
    ///
    /// # Errors
    ///
    /// Returns error if integrity check fails
    pub async fn verify_integrity(&self) -> Result<bool> {
        let entries = sqlx::query_as::<_, AuditEntry>("SELECT * FROM audit_log ORDER BY id ASC")
            .fetch_all(&self.db_pool)
            .await?;

        let mut previous_hash: Option<String> = None;

        for entry in &entries {
            // Verify hash chain
            if entry.previous_hash != previous_hash {
                tracing::error!(
                    entry_id = entry.id,
                    expected = ?previous_hash,
                    actual = ?entry.previous_hash,
                    "Audit trail integrity violated - hash mismatch"
                );
                return Ok(false);
            }

            // Verify entry hash
            let computed_hash = Self::compute_entry_hash(&entry.data, &entry.timestamp, &previous_hash);
            if computed_hash != entry.entry_hash {
                tracing::error!(
                    entry_id = entry.id,
                    "Audit trail integrity violated - entry hash mismatch"
                );
                return Ok(false);
            }

            previous_hash = Some(entry.entry_hash.clone());
        }

        tracing::info!(entries_verified = entries.len(), "Audit trail integrity verified");

        Ok(true)
    }

    /// Internal: Log an event to audit trail
    async fn log_event(
        &mut self,
        event_type: AuditEventType,
        entity_id: &str,
        data: &str,
        actor: &str,
    ) -> Result<()> {
        let timestamp = Utc::now();
        let previous_hash = self.last_hash.clone();

        // Compute hash for this entry
        let entry_hash = Self::compute_entry_hash(data, &timestamp, &previous_hash);

        // Insert into database
        sqlx::query(
            "INSERT INTO audit_log (event_type, entity_id, data, timestamp, entry_hash, previous_hash, actor)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(event_type_to_string(&event_type))
        .bind(entity_id)
        .bind(data)
        .bind(timestamp)
        .bind(&entry_hash)
        .bind(&previous_hash)
        .bind(actor)
        .execute(&self.db_pool)
        .await
        .context("Failed to insert audit log entry")?;

        // Update last hash
        self.last_hash = Some(entry_hash);

        tracing::debug!(
            event_type = ?event_type,
            entity_id = %entity_id,
            "Audit entry logged"
        );

        Ok(())
    }

    /// Compute hash for an audit entry
    fn compute_entry_hash(data: &str, timestamp: &DateTime<Utc>, previous_hash: &Option<String>) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(data.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

fn event_type_to_string(event_type: &AuditEventType) -> String {
    match event_type {
        AuditEventType::ArbitrationAttempt => "arbitration_attempt".to_string(),
        AuditEventType::Resolution => "resolution".to_string(),
        AuditEventType::Signing => "signing".to_string(),
        AuditEventType::KeyRotation => "key_rotation".to_string(),
        AuditEventType::ManualReview => "manual_review".to_string(),
    }
}

impl TryFrom<String> for AuditEventType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "arbitration_attempt" => Ok(Self::ArbitrationAttempt),
            "resolution" => Ok(Self::Resolution),
            "signing" => Ok(Self::Signing),
            "key_rotation" => Ok(Self::KeyRotation),
            "manual_review" => Ok(Self::ManualReview),
            _ => Err(format!("Invalid audit event type: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logger_initialization() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Create table manually for test
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                data TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                entry_hash TEXT NOT NULL,
                previous_hash TEXT,
                actor TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await?;

        let mut logger = AuditLogger::new(pool).await?;
        logger.log_arbitration_attempt("test_dispute").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_audit_trail_integrity() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                data TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                entry_hash TEXT NOT NULL,
                previous_hash TEXT,
                actor TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await?;

        let mut logger = AuditLogger::new(pool).await?;

        logger.log_arbitration_attempt("dispute1").await?;
        logger.log_arbitration_attempt("dispute2").await?;
        logger.log_arbitration_attempt("dispute3").await?;

        let integrity = logger.verify_integrity().await?;
        assert!(integrity, "Audit trail integrity should be valid");

        Ok(())
    }
}
