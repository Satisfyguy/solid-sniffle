//! API handlers for the reputation system
//!
//! This module provides REST endpoints for submitting and retrieving
//! cryptographically-signed vendor reviews. All endpoints implement:
//! - CSRF protection
//! - Rate limiting
//! - Input validation
//! - Proper error handling
//! - Audit logging

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use reputation_common::types::{SignedReview, VendorReputation};
use reputation_crypto::reputation::{calculate_stats, verify_review_signature};

use crate::db::reputation::{
    db_get_vendor_reviews, db_get_vendor_stats, db_insert_review, db_review_exists,
};
use crate::db::DbPool;

/// Maximum comment length (defense in depth)
const MAX_COMMENT_LENGTH: usize = 500;

/// Minimum rating value
const MIN_RATING: u8 = 1;

/// Maximum rating value
const MAX_RATING: u8 = 5;

// ============================================================================
// Security Helpers
// ============================================================================

/// Hash a transaction ID for logging to prevent blockchain correlation
///
/// SECURITY: Never log raw transaction IDs - they can be used to correlate
/// on-chain activity with application logs. Always hash before logging.
///
/// Uses SHA-256 to create a one-way hash that preserves uniqueness for
/// debugging while preventing reverse lookup.
fn hash_txid_for_logging(txid: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(txid.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string() // First 16 chars of hex for brevity
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to submit a new review
#[derive(Debug, Deserialize)]
pub struct SubmitReviewRequest {
    /// Signed review with cryptographic proof
    pub review: SignedReview,

    /// Vendor UUID being reviewed
    pub vendor_id: String,
}

/// Response after submitting a review
#[derive(Debug, Serialize)]
pub struct SubmitReviewResponse {
    pub status: String,
    pub message: String,
    pub review_id: Option<String>,
}

/// Response with vendor reputation data
#[derive(Debug, Serialize)]
pub struct ReputationResponse {
    pub vendor_id: String,
    pub reputation: VendorReputation,
}

/// Quick stats response (without full review list)
#[derive(Debug, Serialize)]
pub struct QuickStatsResponse {
    pub vendor_id: String,
    pub total_reviews: i64,
    pub average_rating: f64,
}

// ============================================================================
// API Handlers
// ============================================================================

/// POST /api/reviews
///
/// Submit a cryptographically-signed review after escrow completion.
///
/// # Security
/// - Session authentication required (SameSite=Strict cookies provide CSRF protection)
/// - Signature verification (ed25519)
/// - Duplicate detection (same txid + reviewer)
/// - Rate limiting: 10 requests/hour per user
///
/// # Request Body
/// ```json
/// {
///   "review": {
///     "txid": "monero_transaction_hash",
///     "rating": 5,
///     "comment": "Excellent service!",
///     "timestamp": "2025-10-22T12:00:00Z",
///     "buyer_pubkey": "base64_encoded_pubkey",
///     "signature": "base64_encoded_signature"
///   },
///   "vendor_id": "uuid"
/// }
/// ```
///
/// # Response (201 Created)
/// ```json
/// {
///   "status": "success",
///   "message": "Review submitted successfully",
///   "review_id": "uuid"
/// }
/// ```
///
/// # Errors
/// - 401: Not authenticated
/// - 400: Invalid signature, duplicate review, validation error
/// - 500: Database error
pub async fn submit_review(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<SubmitReviewRequest>,
) -> impl Responder {
    // 1. Authentication check
    let reviewer_id = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            tracing::warn!("Unauthenticated review submission attempt");
            return HttpResponse::Unauthorized().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Authentication required".to_string(),
                review_id: None,
            });
        }
        Err(e) => {
            tracing::error!(error = %e, "Session error during review submission");
            return HttpResponse::InternalServerError().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Session error".to_string(),
                review_id: None,
            });
        }
    };

    // 2. Validate vendor_id format
    let vendor_uuid = match Uuid::parse_str(&req.vendor_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Invalid vendor ID format".to_string(),
                review_id: None,
            });
        }
    };

    // 4. Validate review fields
    if let Err(e) = validate_review_input(&req.review) {
        tracing::warn!(
            reviewer_id = %reviewer_id,
            vendor_id = %vendor_uuid,
            error = %e,
            "Review validation failed"
        );
        return HttpResponse::BadRequest().json(SubmitReviewResponse {
            status: "error".to_string(),
            message: format!("Validation error: {}", e),
            review_id: None,
        });
    }

    // 5. Verify cryptographic signature
    match verify_review_signature(&req.review) {
        Ok(true) => {
            tracing::debug!(
                txid_hash = %hash_txid_for_logging(&req.review.txid),
                reviewer_id = %reviewer_id,
                "Review signature verified successfully"
            );
        }
        Ok(false) => {
            tracing::warn!(
                txid_hash = %hash_txid_for_logging(&req.review.txid),
                reviewer_id = %reviewer_id,
                "Invalid cryptographic signature"
            );
            return HttpResponse::BadRequest().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Cryptographic signature verification failed".to_string(),
                review_id: None,
            });
        }
        Err(e) => {
            tracing::error!(
                txid_hash = %hash_txid_for_logging(&req.review.txid),
                reviewer_id = %reviewer_id,
                error = %e,
                "Signature verification error"
            );
            return HttpResponse::InternalServerError().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Signature verification error".to_string(),
                review_id: None,
            });
        }
    }

    // 6. Check for duplicate review
    match db_review_exists(&pool, &req.review.txid, &reviewer_id).await {
        Ok(true) => {
            tracing::warn!(
                txid_hash = %hash_txid_for_logging(&req.review.txid),
                reviewer_id = %reviewer_id,
                "Duplicate review attempt detected"
            );
            return HttpResponse::BadRequest().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Review already exists for this transaction".to_string(),
                review_id: None,
            });
        }
        Ok(false) => {
            // Continue - no duplicate
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error checking duplicate review");
            return HttpResponse::InternalServerError().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Database error".to_string(),
                review_id: None,
            });
        }
    }

    // 7. TODO: Verify transaction exists on blockchain
    // This will be implemented when blockchain_monitor integration is complete
    // For now, we trust the signature verification

    // 8. Store review in database
    match db_insert_review(&pool, &req.review, &reviewer_id, &req.vendor_id).await {
        Ok(created_review) => {
            tracing::info!(
                review_id = %created_review.id,
                txid_hash = %hash_txid_for_logging(&req.review.txid),
                reviewer_id = %reviewer_id,
                vendor_id = %vendor_uuid,
                rating = req.review.rating,
                "Review submitted successfully"
            );

            HttpResponse::Created().json(SubmitReviewResponse {
                status: "success".to_string(),
                message: "Review submitted successfully".to_string(),
                review_id: Some(created_review.id),
            })
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                txid_hash = %hash_txid_for_logging(&req.review.txid),
                reviewer_id = %reviewer_id,
                vendor_id = %vendor_uuid,
                "Database error inserting review"
            );
            HttpResponse::InternalServerError().json(SubmitReviewResponse {
                status: "error".to_string(),
                message: "Failed to store review".to_string(),
                review_id: None,
            })
        }
    }
}

/// GET /api/reputation/{vendor_id}
///
/// Retrieve complete reputation file for a vendor (all signed reviews + stats).
/// This endpoint is public and does not require authentication.
///
/// # Performance
/// - Uses database indexes for fast lookup
/// - Returns cached stats (calculated server-side)
/// - Rate limiting: 100 requests/minute per IP
///
/// # Response (200 OK)
/// ```json
/// {
///   "vendor_id": "uuid",
///   "reputation": {
///     "format_version": "1.0",
///     "vendor_pubkey": "uuid",
///     "generated_at": "2025-10-22T12:00:00Z",
///     "reviews": [...],
///     "stats": {
///       "total_reviews": 42,
///       "average_rating": 4.7,
///       "rating_distribution": [0, 1, 2, 10, 29],
///       "oldest_review": "2025-01-01T00:00:00Z",
///       "newest_review": "2025-10-22T12:00:00Z"
///     }
///   }
/// }
/// ```
///
/// # Errors
/// - 400: Invalid vendor ID format
/// - 404: Vendor not found or has no reviews
/// - 500: Database error
pub async fn get_vendor_reputation(
    pool: web::Data<DbPool>,
    vendor_id: web::Path<String>,
) -> impl Responder {
    // 1. Validate vendor_id format
    let vendor_uuid = match Uuid::parse_str(vendor_id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid vendor ID format"
            }));
        }
    };

    // 2. Load all reviews from database
    let reviews = match db_get_vendor_reviews(&pool, vendor_uuid).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                vendor_id = %vendor_uuid,
                error = %e,
                "Error loading vendor reviews"
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load reputation data"
            }));
        }
    };

    // 3. Return 404 if no reviews found
    if reviews.is_empty() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "No reviews found for this vendor"
        }));
    }

    // 4. Calculate statistics
    let stats = calculate_stats(&reviews);

    // 5. Build reputation file
    let reputation = VendorReputation {
        format_version: "1.0".to_string(),
        vendor_pubkey: vendor_uuid.to_string(),
        generated_at: chrono::Utc::now(),
        reviews,
        stats,
    };

    tracing::debug!(
        vendor_id = %vendor_uuid,
        total_reviews = reputation.stats.total_reviews,
        average_rating = reputation.stats.average_rating,
        "Reputation file generated"
    );

    HttpResponse::Ok().json(ReputationResponse {
        vendor_id: vendor_uuid.to_string(),
        reputation,
    })
}

/// GET /api/reputation/{vendor_id}/stats
///
/// Retrieve quick statistics for a vendor without full review list.
/// Optimized for performance (uses database aggregation).
///
/// # Response (200 OK)
/// ```json
/// {
///   "vendor_id": "uuid",
///   "total_reviews": 42,
///   "average_rating": 4.7
/// }
/// ```
pub async fn get_vendor_stats(
    pool: web::Data<DbPool>,
    vendor_id: web::Path<String>,
) -> impl Responder {
    let vendor_uuid = match Uuid::parse_str(vendor_id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid vendor ID format"
            }));
        }
    };

    match db_get_vendor_stats(&pool, vendor_uuid).await {
        Ok((total, average)) => HttpResponse::Ok().json(QuickStatsResponse {
            vendor_id: vendor_uuid.to_string(),
            total_reviews: total,
            average_rating: average,
        }),
        Err(e) => {
            tracing::error!(
                vendor_id = %vendor_uuid,
                error = %e,
                "Error loading vendor stats"
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load statistics"
            }))
        }
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate review input fields (defense in depth)
///
/// Even though SignedReview has validation, we double-check here
/// to provide clear error messages and prevent database corruption.
fn validate_review_input(review: &SignedReview) -> Result<()> {
    // Validate rating range
    if review.rating < MIN_RATING || review.rating > MAX_RATING {
        anyhow::bail!(
            "Rating must be between {} and {}, got {}",
            MIN_RATING,
            MAX_RATING,
            review.rating
        );
    }

    // Validate comment length
    if let Some(ref comment) = review.comment {
        if comment.len() > MAX_COMMENT_LENGTH {
            anyhow::bail!(
                "Comment exceeds maximum length: {} chars (max {})",
                comment.len(),
                MAX_COMMENT_LENGTH
            );
        }

        // Prevent empty comments (use None instead)
        if comment.trim().is_empty() {
            anyhow::bail!("Comment cannot be empty (use null instead)");
        }
    }

    // Validate txid format (basic check)
    if review.txid.is_empty() {
        anyhow::bail!("Transaction ID cannot be empty");
    }

    if review.txid.len() < 32 {
        anyhow::bail!("Transaction ID too short (minimum 32 characters)");
    }

    // Validate buyer_pubkey is not empty
    if review.buyer_pubkey.is_empty() {
        anyhow::bail!("Buyer public key cannot be empty");
    }

    // Validate signature is not empty
    if review.signature.is_empty() {
        anyhow::bail!("Signature cannot be empty");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_validate_review_rating_bounds() {
        let review = SignedReview {
            txid: "a".repeat(64),
            rating: 0,
            comment: None,
            timestamp: Utc::now(),
            buyer_pubkey: "valid_pubkey".to_string(),
            signature: "valid_sig".to_string(),
        };

        assert!(validate_review_input(&review).is_err());

        let review = SignedReview {
            txid: "a".repeat(64),
            rating: 6,
            comment: None,
            timestamp: Utc::now(),
            buyer_pubkey: "valid_pubkey".to_string(),
            signature: "valid_sig".to_string(),
        };

        assert!(validate_review_input(&review).is_err());
    }

    #[test]
    fn test_validate_review_comment_length() {
        let review = SignedReview {
            txid: "a".repeat(64),
            rating: 5,
            comment: Some("x".repeat(501)),
            timestamp: Utc::now(),
            buyer_pubkey: "valid_pubkey".to_string(),
            signature: "valid_sig".to_string(),
        };

        assert!(validate_review_input(&review).is_err());
    }

    #[test]
    fn test_validate_review_txid_length() {
        let review = SignedReview {
            txid: "short".to_string(),
            rating: 5,
            comment: None,
            timestamp: Utc::now(),
            buyer_pubkey: "valid_pubkey".to_string(),
            signature: "valid_sig".to_string(),
        };

        assert!(validate_review_input(&review).is_err());
    }
}
