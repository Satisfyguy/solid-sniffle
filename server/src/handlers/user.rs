//! User API handlers for profile and escrow management

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use serde::Serialize;
use tracing::{error, info};

use crate::db::DbPool;
use crate::models::escrow::Escrow;

/// Response struct for user escrow list
#[derive(Debug, Serialize)]
struct EscrowResponse {
    id: String,
    order_id: String,
    amount: i64,
    status: String,
    user_role: String,
    multisig_phase: String,
    created_at: String,
}

/// GET /api/user/escrows - Get all escrows for authenticated user
///
/// Returns list of escrows where the user is buyer, vendor, or arbiter
pub async fn get_user_escrows(
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    // Fetch escrows where user is buyer, vendor, or arbiter
    let buyer_escrows = Escrow::find_by_buyer(&mut conn, user_id.clone())
        .unwrap_or_default();
    let vendor_escrows = Escrow::find_by_vendor(&mut conn, user_id.clone())
        .unwrap_or_default();
    let arbiter_escrows = Escrow::find_by_arbiter(&mut conn, user_id.clone())
        .unwrap_or_default();

    // Combine all escrows
    let mut all_escrows = Vec::new();

    for escrow in buyer_escrows {
        all_escrows.push(EscrowResponse {
            id: escrow.id.clone(),
            order_id: escrow.order_id.clone(),
            amount: escrow.amount,
            status: escrow.status.clone(),
            user_role: "Buyer".to_string(),
            multisig_phase: escrow.multisig_phase.clone(),
            created_at: escrow.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        });
    }

    for escrow in vendor_escrows {
        all_escrows.push(EscrowResponse {
            id: escrow.id.clone(),
            order_id: escrow.order_id.clone(),
            amount: escrow.amount,
            status: escrow.status.clone(),
            user_role: "Vendor".to_string(),
            multisig_phase: escrow.multisig_phase.clone(),
            created_at: escrow.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        });
    }

    for escrow in arbiter_escrows {
        all_escrows.push(EscrowResponse {
            id: escrow.id.clone(),
            order_id: escrow.order_id.clone(),
            amount: escrow.amount,
            status: escrow.status.clone(),
            user_role: "Arbiter".to_string(),
            multisig_phase: escrow.multisig_phase.clone(),
            created_at: escrow.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        });
    }

    // Sort by created_at (most recent first)
    all_escrows.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    info!("Retrieved {} escrows for user {}", all_escrows.len(), user_id);

    HttpResponse::Ok().json(all_escrows)
}
