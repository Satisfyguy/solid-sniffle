//! Escrow-specific API handlers

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::services::escrow::EscrowOrchestrator;

// ============================================================================
// NON-CUSTODIAL: Client Wallet Registration
// ============================================================================

/// Request body for registering client wallet RPC endpoint
///
/// This is the CORE of non-custodial architecture: clients provide their own
/// wallet RPC URLs, ensuring the server never has access to their private keys.
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterWalletRpcRequest {
    /// Client's wallet RPC URL (e.g., "http://127.0.0.1:18082/json_rpc" or "http://abc123.onion:18082/json_rpc")
    #[validate(url(message = "Invalid RPC URL format"))]
    #[validate(length(min = 10, max = 500, message = "RPC URL must be 10-500 characters"))]
    pub rpc_url: String,

    /// Optional RPC authentication username
    #[validate(length(max = 100, message = "Username max 100 characters"))]
    pub rpc_user: Option<String>,

    /// Optional RPC authentication password
    #[validate(length(max = 100, message = "Password max 100 characters"))]
    pub rpc_password: Option<String>,

    /// Role for this wallet (buyer or vendor - arbiter not allowed)
    #[validate(custom = "validate_client_role")]
    pub role: String,
}

/// Validate that role is buyer or vendor (not arbiter)
fn validate_client_role(role: &str) -> Result<(), validator::ValidationError> {
    match role.to_lowercase().as_str() {
        "buyer" | "vendor" => Ok(()),
        "arbiter" => Err(validator::ValidationError::new(
            "role_not_allowed",
        )),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

/// Response for successful wallet registration
#[derive(Debug, Serialize)]
pub struct RegisterWalletRpcResponse {
    pub success: bool,
    pub message: String,
    pub wallet_id: String,
    pub wallet_address: String,
    pub role: String,
}

/// Register client's wallet RPC endpoint (NON-CUSTODIAL)
///
/// # Non-Custodial Architecture
/// This endpoint allows buyers and vendors to provide their own wallet RPC URLs.
/// The server connects to these client-controlled wallets but NEVER has access
/// to private keys, seed phrases, or any sensitive cryptographic material.
///
/// # Security Requirements
/// - Client must run monero-wallet-rpc on their own machine
/// - Client controls private keys (never shared with server)
/// - RPC can be accessed via local network or Tor hidden service
///
/// # Endpoint
/// POST /api/escrow/register-wallet-rpc
///
/// # Request Body
/// ```json
/// {
///   "rpc_url": "http://127.0.0.1:18082/json_rpc",
///   "rpc_user": "optional_username",
///   "rpc_password": "optional_password",
///   "role": "buyer"  // or "vendor"
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "message": "Wallet RPC registered successfully",
///   "wallet_id": "uuid-of-wallet-instance",
///   "wallet_address": "monero_address",
///   "role": "buyer"
/// }
/// ```
pub async fn register_wallet_rpc(
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    payload: web::Json<RegisterWalletRpcRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Get authenticated user
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse role
    let role = match payload.role.to_lowercase().as_str() {
        "buyer" => crate::wallet_manager::WalletRole::Buyer,
        "vendor" => crate::wallet_manager::WalletRole::Vendor,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid role: must be 'buyer' or 'vendor'"
            }));
        }
    };

    // Register client wallet RPC via orchestrator
    match escrow_orchestrator
        .register_client_wallet(
            user_id,
            role.clone(),
            payload.rpc_url.clone(),
            payload.rpc_user.clone(),
            payload.rpc_password.clone(),
        )
        .await
    {
        Ok((wallet_id, wallet_address)) => {
            HttpResponse::Ok().json(RegisterWalletRpcResponse {
                success: true,
                message: "✅ Wallet RPC registered successfully. You control your private keys.".to_string(),
                wallet_id: wallet_id.to_string(),
                wallet_address,
                role: payload.role.clone(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to register wallet RPC: {}", e)
        })),
    }
}

// ============================================================================
// Multisig Preparation
// ============================================================================

/// Request body for preparing multisig
#[derive(Debug, Deserialize, Validate)]
pub struct PrepareMultisigRequest {
    #[validate(length(
        min = 100,
        max = 5000,
        message = "Multisig info must be 100-5000 characters"
    ))]
    pub multisig_info: String,
}

/// Response for successful prepare multisig
#[derive(Debug, Serialize)]
pub struct PrepareMultisigResponse {
    pub success: bool,
    pub message: String,
    pub escrow_id: String,
}

/// Collect prepare_multisig info from a party
///
/// # Flow
/// 1. User authenticates via session
/// 2. Validates they are part of this escrow (buyer, vendor, or arbiter)
/// 3. Encrypts and stores their multisig_info
/// 4. If all 3 parties have submitted, automatically triggers make_multisig
///
/// # Endpoint
/// POST /api/escrow/:id/prepare
pub async fn prepare_multisig(
    _pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,
    payload: web::Json<PrepareMultisigRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Get authenticated user
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Call orchestrator to collect prepare info
    match escrow_orchestrator
        .collect_prepare_info(escrow_id, user_id, payload.multisig_info.clone())
        .await
    {
        Ok(()) => HttpResponse::Ok().json(PrepareMultisigResponse {
            success: true,
            message: "Multisig info collected successfully".to_string(),
            escrow_id: escrow_id.to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to collect multisig info: {}", e)
        })),
    }
}

/// Request body for releasing funds
#[derive(Debug, Deserialize, Validate)]
pub struct ReleaseFundsRequest {
    #[validate(length(equal = 95, message = "Monero address must be exactly 95 characters"))]
    pub vendor_address: String,
}

/// Release funds to vendor (buyer approves transaction)
///
/// # Flow
/// 1. Verify requester is the buyer
/// 2. Validate escrow is in 'funded' state
/// 3. Create multisig transaction to vendor_address
/// 4. Sign with buyer + arbiter wallets
/// 5. Broadcast transaction
/// 6. Update escrow status to 'released'
///
/// # Endpoint
/// POST /api/escrow/:id/release
pub async fn release_funds(
    _pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,
    payload: web::Json<ReleaseFundsRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Get authenticated user (must be buyer)
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Release funds via orchestrator
    match escrow_orchestrator
        .release_funds(escrow_id, user_id, payload.vendor_address.clone())
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tx_hash": tx_hash,
            "message": "Funds released successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to release funds: {}", e)
        })),
    }
}

/// Request body for refunding funds
#[derive(Debug, Deserialize, Validate)]
pub struct RefundFundsRequest {
    #[validate(length(equal = 95, message = "Monero address must be exactly 95 characters"))]
    pub buyer_address: String,
}

/// Refund funds to buyer (vendor or arbiter initiates)
///
/// # Flow
/// 1. Verify requester is vendor or arbiter
/// 2. Validate escrow is in 'funded' state
/// 3. Create multisig transaction to buyer_address
/// 4. Sign with vendor + arbiter wallets
/// 5. Broadcast transaction
/// 6. Update escrow status to 'refunded'
///
/// # Endpoint
/// POST /api/escrow/:id/refund
pub async fn refund_funds(
    _pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,
    payload: web::Json<RefundFundsRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Get authenticated user (must be vendor or arbiter)
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Refund funds via orchestrator
    match escrow_orchestrator
        .refund_funds(escrow_id, user_id, payload.buyer_address.clone())
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tx_hash": tx_hash,
            "message": "Funds refunded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to refund funds: {}", e)
        })),
    }
}

/// Request body for initiating dispute
#[derive(Debug, Deserialize, Validate)]
pub struct InitiateDisputeRequest {
    #[validate(length(min = 10, max = 2000, message = "Reason must be 10-2000 characters"))]
    pub reason: String,
}

/// Initiate a dispute (buyer or vendor)
///
/// # Flow
/// 1. Verify requester is buyer or vendor
/// 2. Update escrow status to 'disputed'
/// 3. Notify arbiter via WebSocket
///
/// # Endpoint
/// POST /api/escrow/:id/dispute
pub async fn initiate_dispute(
    _pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,
    payload: web::Json<InitiateDisputeRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Get authenticated user
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Initiate dispute via orchestrator
    match escrow_orchestrator
        .initiate_dispute(escrow_id, user_id, payload.reason.clone())
        .await
    {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Dispute initiated successfully. Arbiter has been notified."
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initiate dispute: {}", e)
        })),
    }
}

/// Request body for resolving dispute (arbiter only)
#[derive(Debug, Deserialize, Validate)]
pub struct ResolveDisputeRequest {
    #[validate(custom = "validate_resolution")]
    pub resolution: String,
    #[validate(length(equal = 95))]
    pub recipient_address: String,
}

/// Custom validator for resolution field
fn validate_resolution(resolution: &str) -> Result<(), validator::ValidationError> {
    if resolution != "buyer" && resolution != "vendor" {
        return Err(validator::ValidationError::new(
            "resolution must be 'buyer' or 'vendor'",
        ));
    }
    Ok(())
}

/// Resolve a dispute (arbiter only)
///
/// # Flow
/// 1. Verify requester is the assigned arbiter
/// 2. Update escrow status based on resolution:
///    - "buyer" -> status: resolved_buyer (arbiter can then call refund)
///    - "vendor" -> status: resolved_vendor (arbiter can then call release)
/// 3. Notify both parties via WebSocket
///
/// # Endpoint
/// POST /api/escrow/:id/resolve
pub async fn resolve_dispute(
    _pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    path: web::Path<String>,
    payload: web::Json<ResolveDisputeRequest>,
) -> impl Responder {
    // Validate request
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Get authenticated user (must be arbiter)
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Resolve dispute via orchestrator
    match escrow_orchestrator
        .resolve_dispute(
            escrow_id,
            user_id,
            &payload.resolution,
            payload.recipient_address.clone(),
        )
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "resolution": &payload.resolution,
            "tx_hash": tx_hash,
            "message": format!("Dispute resolved in favor of {}, funds transferred", &payload.resolution)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to resolve dispute: {}", e)
        })),
    }
}

/// Get escrow details by ID
///
/// # Endpoint
/// GET /api/escrow/:id
pub async fn get_escrow(
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<String>,
) -> impl Responder {
    // Get authenticated user
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Session error: {}", e)
            }));
        }
    };

    let user_id = match user_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid user_id in session"
            }));
        }
    };

    // Parse escrow_id from path
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid escrow_id"
            }));
        }
    };

    // Load escrow from database
    match crate::db::db_load_escrow(&pool, escrow_id).await {
        Ok(escrow) => {
            // Verify user is part of this escrow
            if user_id.to_string() != escrow.buyer_id
                && user_id.to_string() != escrow.vendor_id
                && user_id.to_string() != escrow.arbiter_id
            {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You are not authorized to view this escrow"
                }));
            }

            HttpResponse::Ok().json(escrow)
        }
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Escrow not found: {}", e)
        })),
    }
}
