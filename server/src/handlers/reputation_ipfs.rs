//! IPFS export handler for reputation system
//!
//! This module provides the endpoint to export vendor reputation to IPFS.

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use reputation_common::types::VendorReputation;
use reputation_crypto::reputation::calculate_stats;

use crate::db::reputation::db_get_vendor_reviews;
use crate::db::DbPool;
use crate::ipfs::client::IpfsClient;

/// Request to export reputation to IPFS
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    /// Vendor UUID to export
    pub vendor_id: String,
}

/// Response after exporting to IPFS
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub status: String,
    pub ipfs_hash: String,
    pub file_size: usize,
    pub total_reviews: u32,
    pub gateway_url: String,
}

/// POST /api/reputation/export
///
/// Export vendor reputation to IPFS for portable storage.
/// Only the vendor themselves can export their own reputation.
///
/// # Security
/// - Session authentication required (SameSite=Strict cookies provide CSRF protection)
/// - Authorization: Only vendor can export their own reputation
///
/// # Request Body
/// ```json
/// {
///   "vendor_id": "uuid"
/// }
/// ```
///
/// # Response (200 OK)
/// ```json
/// {
///   "status": "success",
///   "ipfs_hash": "Qm...",
///   "file_size": 12345,
///   "total_reviews": 42,
///   "gateway_url": "http://localhost:8080/ipfs/Qm..."
/// }
/// ```
///
/// # Errors
/// - 401: Not authenticated
/// - 403: Not authorized (not your reputation)
/// - 400: Invalid vendor ID, no reviews found
/// - 500: Database error, IPFS error
pub async fn export_to_ipfs(
    pool: web::Data<DbPool>,
    ipfs: web::Data<IpfsClient>,
    session: Session,
    req: web::Json<ExportRequest>,
) -> impl Responder {
    // 1. Authentication check
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            tracing::warn!("Unauthenticated IPFS export attempt");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication required"
            }));
        }
        Err(e) => {
            tracing::error!(error = %e, "Session error during IPFS export");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Session error"
            }));
        }
    };

    // 2. Validate vendor_id format
    let vendor_uuid = match Uuid::parse_str(&req.vendor_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid vendor ID format"
            }));
        }
    };

    // 4. Authorization check: Only vendor can export their own reputation
    if user_id != req.vendor_id {
        tracing::warn!(
            user_id = %user_id,
            vendor_id = %vendor_uuid,
            "Unauthorized IPFS export attempt"
        );
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only export your own reputation"
        }));
    }

    // 5. Load all reviews from database
    let reviews = match db_get_vendor_reviews(&pool, vendor_uuid).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                vendor_id = %vendor_uuid,
                error = %e,
                "Error loading vendor reviews for IPFS export"
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load reputation data"
            }));
        }
    };

    // 6. Validate that vendor has reviews
    if reviews.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No reviews found to export"
        }));
    }

    // 7. Calculate statistics
    let stats = calculate_stats(&reviews);

    // 8. Build reputation file
    let reputation = VendorReputation {
        format_version: "1.0".to_string(),
        vendor_pubkey: vendor_uuid.to_string(),
        generated_at: chrono::Utc::now(),
        reviews,
        stats,
    };

    // 9. Serialize to JSON
    let json_bytes = match serde_json::to_vec_pretty(&reputation) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!(
                error = %e,
                vendor_id = %vendor_uuid,
                "Failed to serialize reputation to JSON"
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Serialization error"
            }));
        }
    };

    let file_size = json_bytes.len();

    // 10. Upload to IPFS
    let ipfs_hash = match ipfs.add(json_bytes, "reputation.json", "application/json").await {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!(
                error = %e,
                vendor_id = %vendor_uuid,
                file_size = file_size,
                "IPFS upload failed"
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "IPFS upload failed",
                "details": e.to_string()
            }));
        }
    };

    // 11. Build gateway URL
    let gateway_url = format!("http://127.0.0.1:8080/ipfs/{}", ipfs_hash);

    tracing::info!(
        vendor_id = %vendor_uuid,
        ipfs_hash = %ipfs_hash,
        file_size = file_size,
        total_reviews = reputation.stats.total_reviews,
        "Reputation exported to IPFS successfully"
    );

    HttpResponse::Ok().json(ExportResponse {
        status: "success".to_string(),
        ipfs_hash,
        file_size,
        total_reviews: reputation.stats.total_reviews,
        gateway_url,
    })
}
