//! Arbitration engine for dispute resolution
//!
//! This module implements rule-based and ML-assisted arbitration logic.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Dispute, DisputeEvidence, DisputeStatus};

/// Resolution decision for a dispute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DisputeResolution {
    /// Release funds to vendor
    ReleaseToVendor {
        /// Reasoning
        reason: String,
        /// Confidence (0.0-1.0)
        confidence: f64,
    },

    /// Refund to buyer
    RefundToBuyer {
        /// Reasoning
        reason: String,
        /// Confidence (0.0-1.0)
        confidence: f64,
    },

    /// Split funds 50/50
    Split {
        /// Reasoning
        reason: String,
        /// Confidence (0.0-1.0)
        confidence: f64,
    },

    /// Escalate to manual review
    ManualReview {
        /// Reason for escalation
        reason: String,
    },
}

/// Arbitration engine implementing dispute resolution logic
pub struct ArbitrationEngine {
    /// Minimum confidence threshold for automated decisions
    confidence_threshold: f64,
}

impl Default for ArbitrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ArbitrationEngine {
    /// Create new arbitration engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            // Require 80% confidence for automated decisions
            confidence_threshold: 0.8,
        }
    }

    /// Resolve a dispute using arbitration rules
    ///
    /// # Arguments
    ///
    /// * `dispute` - The dispute to resolve
    ///
    /// # Errors
    ///
    /// Returns error if resolution fails
    ///
    /// # Decision Logic
    ///
    /// 1. Analyze evidence quality and quantity
    /// 2. Apply rule-based heuristics
    /// 3. Calculate confidence score
    /// 4. If confidence < threshold → Manual review
    /// 5. Otherwise → Automated decision
    pub async fn resolve(&self, dispute: &Dispute) -> Result<DisputeResolution> {
        tracing::info!(
            dispute_id = %dispute.id,
            opened_by = %dispute.opened_by,
            "Analyzing dispute for resolution"
        );

        // Check if already resolved
        if dispute.status == DisputeStatus::Resolved
            || dispute.status == DisputeStatus::Closed
        {
            anyhow::bail!("Dispute {} already resolved", dispute.id);
        }

        // Analyze evidence
        let evidence_analysis = self.analyze_evidence(&dispute.evidence);

        // Apply arbitration rules
        let resolution = self.apply_rules(dispute, &evidence_analysis);

        // Check confidence threshold
        let final_resolution = match &resolution {
            DisputeResolution::ReleaseToVendor { confidence, .. }
            | DisputeResolution::RefundToBuyer { confidence, .. }
            | DisputeResolution::Split { confidence, .. } => {
                if *confidence < self.confidence_threshold {
                    DisputeResolution::ManualReview {
                        reason: format!(
                            "Confidence {} below threshold {}",
                            confidence, self.confidence_threshold
                        ),
                    }
                } else {
                    resolution
                }
            }
            DisputeResolution::ManualReview { .. } => resolution,
        };

        tracing::info!(
            dispute_id = %dispute.id,
            resolution = ?final_resolution,
            "Dispute resolution determined"
        );

        Ok(final_resolution)
    }

    /// Analyze evidence quality and quantity
    fn analyze_evidence(&self, evidence: &[DisputeEvidence]) -> EvidenceAnalysis {
        let mut analysis = EvidenceAnalysis::default();

        for item in evidence {
            match item {
                DisputeEvidence::Text { .. } => {
                    analysis.text_count += 1;
                }
                DisputeEvidence::Photo { .. } => {
                    analysis.photo_count += 1;
                    analysis.quality_score += 0.2;
                }
                DisputeEvidence::Tracking { .. } => {
                    analysis.tracking_count += 1;
                    analysis.quality_score += 0.3;
                }
                DisputeEvidence::ChatLog { messages } => {
                    analysis.chat_log_count += 1;
                    analysis.quality_score += 0.1 * (messages.len() as f64 / 10.0).min(1.0);
                }
                DisputeEvidence::CryptoProof { .. } => {
                    analysis.crypto_proof_count += 1;
                    analysis.quality_score += 0.4;
                }
            }
        }

        analysis.quality_score = analysis.quality_score.min(1.0);

        tracing::debug!(
            text = analysis.text_count,
            photos = analysis.photo_count,
            tracking = analysis.tracking_count,
            quality = analysis.quality_score,
            "Evidence analysis complete"
        );

        analysis
    }

    /// Apply arbitration rules based on dispute and evidence
    fn apply_rules(&self, dispute: &Dispute, evidence: &EvidenceAnalysis) -> DisputeResolution {
        // Rule 1: Strong evidence from vendor (tracking + photos)
        if dispute.opened_by == "buyer"
            && evidence.tracking_count > 0
            && evidence.photo_count > 0
        {
            return DisputeResolution::ReleaseToVendor {
                reason: "Vendor provided tracking and photo proof of shipment".to_string(),
                confidence: 0.85 + evidence.quality_score * 0.15,
            };
        }

        // Rule 2: Strong evidence from buyer (crypto proof of non-delivery)
        if dispute.opened_by == "vendor" && evidence.crypto_proof_count > 0 {
            return DisputeResolution::RefundToBuyer {
                reason: "Buyer provided cryptographic proof of issue".to_string(),
                confidence: 0.9,
            };
        }

        // Rule 3: Buyer claims non-delivery, no vendor evidence
        if dispute.opened_by == "buyer"
            && dispute.reason.to_lowercase().contains("not received")
            && evidence.tracking_count == 0
        {
            return DisputeResolution::RefundToBuyer {
                reason: "No delivery proof provided by vendor".to_string(),
                confidence: 0.75,
            };
        }

        // Rule 4: Both parties have evidence → split
        if evidence.quality_score > 0.5 {
            return DisputeResolution::Split {
                reason: "Both parties provided substantial evidence".to_string(),
                confidence: 0.7,
            };
        }

        // Rule 5: Insufficient evidence → manual review
        DisputeResolution::ManualReview {
            reason: "Insufficient or unclear evidence for automated resolution".to_string(),
        }
    }

    /// Set confidence threshold for automated decisions
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        tracing::info!(threshold = self.confidence_threshold, "Confidence threshold updated");
    }
}

/// Evidence analysis result
#[derive(Debug, Default)]
struct EvidenceAnalysis {
    text_count: usize,
    photo_count: usize,
    tracking_count: usize,
    chat_log_count: usize,
    crypto_proof_count: usize,
    quality_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_dispute(opened_by: &str, reason: &str) -> Dispute {
        Dispute {
            id: "test_dispute_123".to_string(),
            escrow_id: "escrow_456".to_string(),
            buyer_username: "buyer1".to_string(),
            vendor_username: "vendor1".to_string(),
            opened_by: opened_by.to_string(),
            reason: reason.to_string(),
            status: DisputeStatus::UnderReview,
            evidence: vec![],
            resolution: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_vendor_with_tracking_wins() -> Result<()> {
        let engine = ArbitrationEngine::new();

        let mut dispute = create_test_dispute("buyer", "Item not received");
        dispute.evidence = vec![
            DisputeEvidence::Tracking {
                tracking_number: "123456".to_string(),
                carrier: "DHL".to_string(),
            },
            DisputeEvidence::Photo {
                ipfs_hash: "QmTest123".to_string(),
            },
        ];

        let resolution = engine.resolve(&dispute).await?;

        assert!(matches!(
            resolution,
            DisputeResolution::ReleaseToVendor { .. }
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_no_evidence_requires_manual_review() -> Result<()> {
        let engine = ArbitrationEngine::new();
        let dispute = create_test_dispute("buyer", "Item damaged");

        let resolution = engine.resolve(&dispute).await?;

        assert!(matches!(resolution, DisputeResolution::ManualReview { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn test_both_parties_evidence_split() -> Result<()> {
        let engine = ArbitrationEngine::new();

        let mut dispute = create_test_dispute("buyer", "Wrong item received");
        dispute.evidence = vec![
            DisputeEvidence::Photo {
                ipfs_hash: "QmBuyer".to_string(),
            },
            DisputeEvidence::Photo {
                ipfs_hash: "QmVendor".to_string(),
            },
            DisputeEvidence::Text {
                content: "Evidence from both sides".to_string(),
            },
        ];

        let resolution = engine.resolve(&dispute).await?;

        assert!(matches!(resolution, DisputeResolution::Split { .. }));

        Ok(())
    }
}
