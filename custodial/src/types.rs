//! Type definitions for custodial module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Status of a dispute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum DisputeStatus {
    /// Dispute opened, awaiting evidence
    #[serde(rename = "open")]
    Open,

    /// Evidence submitted, under review
    #[serde(rename = "under_review")]
    UnderReview,

    /// Resolved automatically
    #[serde(rename = "resolved")]
    Resolved,

    /// Requires manual arbitration
    #[serde(rename = "manual_review")]
    ManualReview,

    /// Closed (final state)
    #[serde(rename = "closed")]
    Closed,
}

/// Type of evidence submitted in a dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DisputeEvidence {
    /// Text description
    Text { content: String },

    /// Photo evidence (IPFS hash)
    Photo { ipfs_hash: String },

    /// Tracking information
    Tracking { tracking_number: String, carrier: String },

    /// Chat logs
    ChatLog { messages: Vec<ChatMessage> },

    /// Cryptographic proof
    CryptoProof { proof_type: String, data: String },
}

/// Chat message in dispute evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Timestamp of message
    pub timestamp: DateTime<Utc>,
    /// Sender (buyer or vendor)
    pub sender: String,
    /// Message content
    pub content: String,
}

/// A dispute in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dispute {
    /// Unique dispute ID
    pub id: String,

    /// Associated escrow ID
    pub escrow_id: String,

    /// Buyer username
    pub buyer_username: String,

    /// Vendor username
    pub vendor_username: String,

    /// Who opened the dispute (buyer or vendor)
    pub opened_by: String,

    /// Reason for dispute
    pub reason: String,

    /// Current status
    #[sqlx(try_from = "String")]
    pub status: DisputeStatus,

    /// Evidence (JSON array)
    #[sqlx(try_from = "String")]
    pub evidence: Vec<DisputeEvidence>,

    /// Resolution decision (if resolved)
    pub resolution: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Arbitration decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationDecision {
    /// Dispute ID
    pub dispute_id: String,

    /// Resolution type
    pub resolution: String,

    /// Reasoning
    pub reasoning: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Requires manual review?
    pub manual_review_required: bool,

    /// Decision timestamp
    pub decided_at: DateTime<Utc>,
}

impl TryFrom<String> for DisputeStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "open" => Ok(Self::Open),
            "under_review" => Ok(Self::UnderReview),
            "resolved" => Ok(Self::Resolved),
            "manual_review" => Ok(Self::ManualReview),
            "closed" => Ok(Self::Closed),
            _ => Err(format!("Invalid dispute status: {}", value)),
        }
    }
}

impl From<DisputeStatus> for String {
    fn from(status: DisputeStatus) -> Self {
        match status {
            DisputeStatus::Open => "open".to_string(),
            DisputeStatus::UnderReview => "under_review".to_string(),
            DisputeStatus::Resolved => "resolved".to_string(),
            DisputeStatus::ManualReview => "manual_review".to_string(),
            DisputeStatus::Closed => "closed".to_string(),
        }
    }
}

// SQLx converter for Vec<DisputeEvidence>
impl TryFrom<String> for Vec<DisputeEvidence> {
    type Error = String;

    fn try_from(json: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse evidence JSON: {}", e))
    }
}

impl From<Vec<DisputeEvidence>> for String {
    fn from(evidence: Vec<DisputeEvidence>) -> Self {
        serde_json::to_string(&evidence).unwrap_or_else(|_| "[]".to_string())
    }
}
