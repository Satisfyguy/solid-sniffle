//! Error types for custodial module

use thiserror::Error;

/// Errors that can occur in the custodial module
#[derive(Error, Debug)]
pub enum CustodialError {
    /// Key management error
    #[error("Key manager error: {0}")]
    KeyManager(String),

    /// HSM error
    #[error("HSM error: {0}")]
    Hsm(String),

    /// Arbitration error
    #[error("Arbitration error: {0}")]
    Arbitration(String),

    /// Dispute not found
    #[error("Dispute not found: {0}")]
    DisputeNotFound(String),

    /// Unauthorized operation
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Audit logging error
    #[error("Audit error: {0}")]
    Audit(String),

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Insufficient evidence
    #[error("Insufficient evidence for dispute {0}")]
    InsufficientEvidence(String),
}
