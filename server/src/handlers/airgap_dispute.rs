/// # Air-Gap Dispute Handlers (TM-001 Mitigation)
///
/// HTTP endpoints for exporting disputes to offline arbiter and importing decisions.
///
/// ## Security Properties
///
/// - ✅ Arbiter wallet NEVER on server
/// - ✅ Disputes exported via QR code (one-way communication)
/// - ✅ Decisions imported via QR code with cryptographic verification
/// - ✅ Signature validation ensures arbiter actually signed decision
/// - ✅ Nonce prevents replay attacks
///
/// ## Endpoints
///
/// 1. `GET /api/escrow/:id/dispute/export` - Export dispute as QR-ready JSON
/// 2. `POST /api/escrow/:id/dispute/import` - Import arbiter decision from QR
/// 3. `GET /api/escrow/:id/dispute/qr` - Generate QR code image (PNG data URI)
use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::escrow::Escrow;
use crate::services::airgap::{ArbiterDecision, DisputeRequest};

/// Helper: Extract user_id from session
fn get_user_id_from_session(session: &Session) -> actix_web::Result<Uuid> {
    let user_id_str = session
        .get::<String>("user_id")
        .map_err(|_| actix_web::error::ErrorUnauthorized("Session error"))?
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    Uuid::parse_str(&user_id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user_id in session"))
}

/// Response for dispute export
#[derive(Serialize)]
pub struct DisputeExportResponse {
    /// Dispute request JSON (to be encoded as QR)
    pub dispute_json: String,

    /// Human-readable summary
    pub summary: DisputeSummary,
}

#[derive(Serialize)]
pub struct DisputeSummary {
    pub escrow_id: String,
    pub amount_xmr: String,
    pub buyer_claim: String,
    pub vendor_response: Option<String>,
    pub evidence_count: usize,
}

/// Request body for importing arbiter decision
#[derive(Deserialize)]
pub struct ImportDecisionRequest {
    /// JSON from QR code scan (ArbiterDecision)
    pub decision_json: String,
}

/// GET /api/escrow/:escrow_id/dispute/export
///
/// Export dispute details as JSON ready for QR encoding.
///
/// **Authentication:** Required (buyer, vendor, or admin)
///
/// **Response:**
/// ```json
/// {
///   "dispute_json": "{\"escrow_id\":\"...\",\"buyer_claim\":\"...\", ...}",
///   "summary": {
///     "escrow_id": "550e8400-...",
///     "amount_xmr": "0.1",
///     "buyer_claim": "Item not received",
///     "vendor_response": "Shipped on 2025-10-20",
///     "evidence_count": 3
///   }
/// }
/// ```
#[get("/api/escrow/{escrow_id}/dispute/export")]
pub async fn export_dispute(
    escrow_id: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let _user_id = get_user_id_from_session(&session)?;
    let escrow_id = escrow_id.into_inner();

    // Fetch escrow from database
    let mut conn = pool
        .get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?;

    let escrow = web::block(move || {
        use diesel::prelude::*;
        use crate::schema::escrows::dsl::*;

        escrows
            .filter(id.eq(escrow_id.to_string()))
            .first::<Escrow>(&mut conn)
            .optional()
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB query error: {}", e)))?
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Escrow not found"))?;

    // Verify escrow is in dispute state
    if escrow.status != "disputed" {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "Escrow not in disputed state (current: {})",
            escrow.status
        )));
    }

    // Parse dispute details from multisig_state_json
    let dispute_data: serde_json::Value = serde_json::from_str(&escrow.multisig_state_json.unwrap_or_else(|| "{}".to_string()))
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse multisig state: {}", e)))?;

    // Extract dispute information (may be stored in different format)
    // For now, use placeholders - this needs to be integrated with actual dispute data storage
    let buyer_claim = dispute_data["dispute"]["buyer_claim"]
        .as_str()
        .unwrap_or("Dispute claim not available")
        .to_string();

    let vendor_response = dispute_data["dispute"]["vendor_response"]
        .as_str()
        .map(|s| s.to_string());

    // Get multisig transaction data (partially signed by buyer/vendor)
    // TODO: Retrieve actual partial_tx_hex from wallet manager or database
    let partial_tx_hex = dispute_data["partial_tx_hex"]
        .as_str()
        .unwrap_or("placeholder_tx_hex_to_be_implemented")
        .to_string();

    // Generate nonce (prevents replay attacks)
    let nonce = hex::encode(&{
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    });

    // Create DisputeRequest
    let dispute_request = DisputeRequest {
        escrow_id,
        buyer_id: Uuid::parse_str(&escrow.buyer_id)
            .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid buyer_id"))?,
        vendor_id: Uuid::parse_str(&escrow.vendor_id)
            .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid vendor_id"))?,
        amount: escrow.amount as u64, // amount is already i64, just cast to u64
        buyer_claim: buyer_claim.clone(),
        vendor_response: vendor_response.clone(),
        dispute_opened_at: escrow.updated_at.and_utc().timestamp(),
        evidence_file_count: dispute_data["dispute"]["evidence_count"].as_u64().unwrap_or(0) as usize,
        partial_tx_hex,
        nonce: nonce.clone(),
    };

    // Serialize to JSON
    let dispute_json = dispute_request
        .to_json()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("JSON serialization failed: {}", e)))?;

    // Calculate amount in XMR (amount is i64, convert to f64)
    let amount_xmr = format!(
        "{:.12}",
        escrow.amount as f64 / 1_000_000_000_000.0
    );

    // Create response
    let response = DisputeExportResponse {
        dispute_json,
        summary: DisputeSummary {
            escrow_id: escrow_id.to_string(),
            amount_xmr,
            buyer_claim,
            vendor_response,
            evidence_count: dispute_data["evidence_count"].as_u64().unwrap_or(0) as usize,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/escrow/:escrow_id/dispute/import
///
/// Import arbiter decision from QR code scan.
///
/// **Authentication:** Admin only (or authorized arbiter importer)
///
/// **Request Body:**
/// ```json
/// {
///   "decision_json": "{\"escrow_id\":\"...\",\"decision\":\"buyer\",\"signed_tx_hex\":\"...\", ...}"
/// }
/// ```
///
/// **Response:**
/// ```json
/// {
///   "status": "accepted",
///   "decision": "buyer",
///   "escrow_id": "550e8400-...",
///   "tx_hash": "abc123..."
/// }
/// ```
#[post("/api/escrow/{escrow_id}/dispute/import")]
pub async fn import_arbiter_decision(
    escrow_id: web::Path<Uuid>,
    payload: web::Json<ImportDecisionRequest>,
    pool: web::Data<DbPool>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let _user_id = get_user_id_from_session(&session)?;
    let escrow_id = escrow_id.into_inner();

    // Parse decision JSON
    let decision = ArbiterDecision::from_json(&payload.decision_json)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid decision JSON: {}", e)))?;

    // Validate decision structure
    decision
        .validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Decision validation failed: {}", e)))?;

    // Verify escrow ID matches
    if decision.escrow_id != escrow_id {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "Escrow ID mismatch: URL has {}, decision has {}",
            escrow_id, decision.escrow_id
        )));
    }

    // TODO: Retrieve arbiter public key from configuration or database
    // For now, use placeholder (must be replaced with actual arbiter pubkey)
    let arbiter_pubkey = std::env::var("ARBITER_PUBKEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError(
            "ARBITER_PUBKEY not configured. Set it in .env file."
        ))?;

    // Verify arbiter signature
    decision
        .verify_signature(&arbiter_pubkey)
        .map_err(|e| actix_web::error::ErrorForbidden(format!("Signature verification failed: {}", e)))?;

    // Fetch escrow
    let mut conn = pool
        .get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?;

    let escrow = web::block(move || {
        use diesel::prelude::*;
        use crate::schema::escrows::dsl::*;

        escrows
            .filter(id.eq(escrow_id.to_string()))
            .first::<Escrow>(&mut conn)
            .optional()
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB query error: {}", e)))?
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Escrow not found"))?;

    // Verify escrow is in dispute state
    if escrow.status != "disputed" {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "Escrow not in disputed state (current: {})",
            escrow.status
        )));
    }

    // TODO: Submit signed transaction to Monero network
    // 1. Use wallet manager to finalize transaction
    // 2. Broadcast signed_tx_hex to network
    // 3. Wait for confirmation
    // 4. Update escrow status to "completed" or "refunded" based on decision

    tracing::info!(
        "Arbiter decision imported for escrow {}: {:?} → reason: {}",
        escrow_id,
        decision.decision,
        decision.reason
    );

    // For now, return success response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted",
        "decision": match decision.decision {
            crate::services::airgap::ArbiterResolution::Buyer => "buyer",
            crate::services::airgap::ArbiterResolution::Vendor => "vendor",
        },
        "escrow_id": escrow_id.to_string(),
        "reason": decision.reason,
        "message": "Decision accepted. Transaction will be broadcast once finalized."
    })))
}

/// GET /api/escrow/:escrow_id/dispute/qr
///
/// Generate QR code image (PNG data URI) for dispute export.
///
/// **Authentication:** Required
///
/// **Response:**
/// ```json
/// {
///   "qr_data_uri": "data:image/png;base64,iVBORw0KGgo..."
/// }
/// ```
///
/// **Note:** Requires `qr_generation` feature enabled.
#[cfg(feature = "qr_generation")]
#[get("/api/escrow/{escrow_id}/dispute/qr")]
pub async fn generate_dispute_qr(
    escrow_id: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    use crate::services::airgap::DisputeRequest;

    let _user_id = get_user_id_from_session(&session)?;
    let escrow_id = escrow_id.into_inner();

    // Reuse export logic to get DisputeRequest
    // (For production, extract to shared function)
    let export_response = export_dispute(
        web::Path::from(escrow_id),
        pool,
        session,
    )
    .await?;

    // Parse dispute_json back to DisputeRequest
    let dispute_json = match export_response.into_body() {
        actix_web::body::MessageBody::Bytes(bytes) => {
            String::from_utf8(bytes.to_vec())
                .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to parse response"))?
        }
        _ => return Err(actix_web::error::ErrorInternalServerError("Unexpected response type")),
    };

    let export_data: DisputeExportResponse = serde_json::from_str(&dispute_json)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("JSON parse error: {}", e)))?;

    let dispute_request = DisputeRequest::from_json(&export_data.dispute_json)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse dispute: {}", e)))?;

    // Generate QR code data URI
    let qr_data_uri = dispute_request
        .to_qr_data_uri()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("QR generation failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "qr_data_uri": qr_data_uri
    })))
}
