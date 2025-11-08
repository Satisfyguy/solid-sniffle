//! Non-custodial escrow handlers (Haveno-inspired)
//!
//! This module provides API endpoints for the non-custodial escrow flow
//! where clients run their own monero-wallet-rpc instances and the server
//! acts as a pure coordinator for multisig info exchange.
//!
//! **Architecture Difference:**
//! - Old (handlers/escrow.rs): Server manages wallets via WalletManager
//! - New (this file): Server coordinates client wallets via EscrowCoordinator
//!
//! **Key Principle:**
//! The server NEVER creates, opens, or manages wallets. It only stores RPC URLs
//! and coordinates the exchange of public multisig info.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::coordination::{
    CoordinationState, EscrowCoordinator, EscrowRole, MultisigExchangeResult,
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to register a client's local wallet RPC
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterClientWalletRequest {
    /// Escrow ID this wallet is for
    #[validate(length(min = 1, max = 100))]
    pub escrow_id: String,

    /// Role in escrow (buyer, seller, or arbiter)
    #[validate(length(min = 1, max = 20))]
    pub role: String,

    /// Client's local wallet RPC URL (must be localhost)
    /// Example: "http://127.0.0.1:18083"
    #[validate(url, length(min = 10, max = 200))]
    pub rpc_url: String,
}

/// Response after successful wallet registration
#[derive(Debug, Serialize)]
pub struct RegisterClientWalletResponse {
    pub success: bool,
    pub message: String,
    pub escrow_id: String,
    pub role: String,
    pub coordination_state: String,
    pub awaiting: Vec<String>, // List of roles still needed
}

/// Request to coordinate multisig exchange
#[derive(Debug, Deserialize, Validate)]
pub struct CoordinateExchangeRequest {
    /// Escrow ID to coordinate
    #[validate(length(min = 1, max = 100))]
    pub escrow_id: String,
}

/// Response with multisig infos for each participant
#[derive(Debug, Serialize)]
pub struct CoordinateExchangeResponse {
    pub success: bool,
    pub message: String,
    pub escrow_id: String,
    pub exchange_result: MultisigExchangeResult,
}

/// Request to get coordination status
#[derive(Debug, Deserialize)]
pub struct GetCoordinationStatusRequest {
    pub escrow_id: String,
}

/// Response with current coordination state
#[derive(Debug, Serialize)]
pub struct GetCoordinationStatusResponse {
    pub success: bool,
    pub escrow_id: String,
    pub state: String,
    pub buyer_registered: bool,
    pub seller_registered: bool,
    pub arbiter_registered: bool,
    pub ready_for_exchange: bool,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// POST /api/v2/escrow/register-wallet
///
/// Register a client's local wallet RPC URL for non-custodial escrow.
///
/// **Flow:**
/// 1. Client starts local monero-wallet-rpc (e.g., on port 18083)
/// 2. Client calls this endpoint with escrow_id, role, and RPC URL
/// 3. Server validates localhost and checks RPC connectivity
/// 4. Server stores URL (NOT the wallet itself)
/// 5. When all 3 participants register, state → AllRegistered
///
/// **Example Request:**
/// ```json
/// {
///   "escrow_id": "escrow_abc123",
///   "role": "buyer",
///   "rpc_url": "http://127.0.0.1:18083"
/// }
/// ```
pub async fn register_client_wallet(
    coordinator: web::Data<Arc<EscrowCoordinator>>,
    req: web::Json<RegisterClientWalletRequest>,
) -> impl Responder {
    use tracing::{error, info};

    // Validate request
    if let Err(e) = req.validate() {
        error!("Validation error in register_client_wallet: {}", e);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": format!("Validation failed: {}", e)
        }));
    }

    // Parse role
    let role = match EscrowRole::from_str(&req.role) {
        Ok(r) => r,
        Err(e) => {
            error!("Invalid role '{}': {}", req.role, e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid role: {}", e)
            }));
        }
    };

    info!(
        "📝 Registering {} wallet for escrow {} at {}",
        role.as_str(),
        req.escrow_id,
        req.rpc_url
    );

    // Register with coordinator
    match coordinator
        .register_client_wallet(&req.escrow_id, role.clone(), req.rpc_url.clone())
        .await
    {
        Ok(()) => {
            // Get updated status to see what's still needed
            let status = coordinator
                .get_coordination_status(&req.escrow_id)
                .await
                .unwrap();

            let awaiting = vec![
                (!status.buyer_rpc_url.is_some()).then_some("buyer"),
                (!status.seller_rpc_url.is_some()).then_some("seller"),
                (!status.arbiter_rpc_url.is_some()).then_some("arbiter"),
            ]
            .into_iter()
            .filter_map(|x| x.map(String::from))
            .collect::<Vec<_>>();

            let message = if awaiting.is_empty() {
                "✅ All participants registered! Ready to coordinate multisig exchange.".to_string()
            } else {
                format!(
                    "✅ {} wallet registered. Waiting for: {:?}",
                    role.as_str(),
                    awaiting
                )
            };

            info!(
                "✅ {} wallet registered for escrow {}, state: {:?}",
                role.as_str(),
                req.escrow_id,
                status.state
            );

            HttpResponse::Ok().json(RegisterClientWalletResponse {
                success: true,
                message,
                escrow_id: req.escrow_id.clone(),
                role: req.role.clone(),
                coordination_state: format!("{:?}", status.state),
                awaiting,
            })
        }
        Err(e) => {
            error!(
                "Failed to register {} wallet for escrow {}: {}",
                role.as_str(),
                req.escrow_id,
                e
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Registration failed: {}", e)
            }))
        }
    }
}

/// POST /api/v2/escrow/coordinate-exchange
///
/// Coordinate multisig info exchange between all participants.
///
/// **Prerequisites:**
/// - All 3 wallets must be registered (buyer, seller, arbiter)
/// - Each wallet must have executed prepare_multisig locally
///
/// **Flow:**
/// 1. Server requests prepare_multisig from each client wallet
/// 2. Server validates all multisig_info formats
/// 3. Server exchanges infos (each gets the other 2)
/// 4. Clients finalize with make_multisig locally
///
/// **Example Request:**
/// ```json
/// {
///   "escrow_id": "escrow_abc123"
/// }
/// ```
///
/// **Example Response:**
/// ```json
/// {
///   "success": true,
///   "message": "Multisig exchange coordinated",
///   "exchange_result": {
///     "buyer_receives": ["MultisigV1...", "MultisigV1..."],
///     "seller_receives": ["MultisigV1...", "MultisigV1..."],
///     "arbiter_receives": ["MultisigV1...", "MultisigV1..."]
///   }
/// }
/// ```
pub async fn coordinate_multisig_exchange(
    coordinator: web::Data<Arc<EscrowCoordinator>>,
    req: web::Json<CoordinateExchangeRequest>,
) -> impl Responder {
    use tracing::{error, info};

    // Validate request
    if let Err(e) = req.validate() {
        error!("Validation error in coordinate_exchange: {}", e);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": format!("Validation failed: {}", e)
        }));
    }

    info!("🔄 Coordinating multisig exchange for escrow {}", req.escrow_id);

    // Coordinate exchange
    match coordinator.coordinate_multisig_exchange(&req.escrow_id).await {
        Ok(exchange_result) => {
            info!(
                "✅ Multisig exchange coordinated for escrow {}",
                req.escrow_id
            );

            HttpResponse::Ok().json(CoordinateExchangeResponse {
                success: true,
                message: "✅ Multisig info exchange coordinated successfully. Clients can now finalize with make_multisig.".to_string(),
                escrow_id: req.escrow_id.clone(),
                exchange_result,
            })
        }
        Err(e) => {
            error!(
                "Failed to coordinate multisig exchange for escrow {}: {}",
                req.escrow_id, e
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Coordination failed: {}", e)
            }))
        }
    }
}

/// GET /api/v2/escrow/coordination-status/{escrow_id}
///
/// Get current coordination status for an escrow.
///
/// **Returns:**
/// - Current state (AwaitingRegistrations, AllRegistered, Prepared, etc.)
/// - Which participants are registered
/// - Whether ready for multisig exchange
pub async fn get_coordination_status(
    coordinator: web::Data<Arc<EscrowCoordinator>>,
    escrow_id: web::Path<String>,
) -> impl Responder {
    use tracing::{error, info};

    let escrow_id = escrow_id.into_inner();
    info!("📊 Getting coordination status for escrow {}", escrow_id);

    match coordinator.get_coordination_status(&escrow_id).await {
        Ok(status) => {
            let buyer_registered = status.buyer_rpc_url.is_some();
            let seller_registered = status.seller_rpc_url.is_some();
            let arbiter_registered = status.arbiter_rpc_url.is_some();
            let ready_for_exchange = buyer_registered && seller_registered && arbiter_registered;

            HttpResponse::Ok().json(GetCoordinationStatusResponse {
                success: true,
                escrow_id,
                state: format!("{:?}", status.state),
                buyer_registered,
                seller_registered,
                arbiter_registered,
                ready_for_exchange,
            })
        }
        Err(e) => {
            error!("Failed to get coordination status for escrow {}: {}", escrow_id, e);
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Escrow not found: {}", e)
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_client_wallet_request_validation() {
        // Valid request
        let valid_req = RegisterClientWalletRequest {
            escrow_id: "escrow_123".to_string(),
            role: "buyer".to_string(),
            rpc_url: "http://127.0.0.1:18083".to_string(),
        };
        assert!(valid_req.validate().is_ok());

        // Invalid: empty escrow_id
        let invalid_req = RegisterClientWalletRequest {
            escrow_id: "".to_string(),
            role: "buyer".to_string(),
            rpc_url: "http://127.0.0.1:18083".to_string(),
        };
        assert!(invalid_req.validate().is_err());

        // Invalid: malformed URL
        let invalid_req = RegisterClientWalletRequest {
            escrow_id: "escrow_123".to_string(),
            role: "buyer".to_string(),
            rpc_url: "not-a-url".to_string(),
        };
        assert!(invalid_req.validate().is_err());
    }
}
