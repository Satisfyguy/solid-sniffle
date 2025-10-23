//! Order API handlers
//!
//! REST API endpoints for managing purchase orders in the escrow marketplace.

use actix_session::Session;
use actix_web::{get, post, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::models::listing::Listing;
use crate::models::order::{NewOrder, Order, OrderStatus};
use crate::models::user::User;
use crate::services::escrow::EscrowOrchestrator;

/// Request body for creating a new order
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(equal = 36, message = "Listing ID must be a valid UUID"))]
    pub listing_id: String,

    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,
}

/// Request body for updating order status
#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

/// Response for order operations
#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub listing_id: String,
    pub escrow_id: Option<String>,
    pub status: String,
    pub total_xmr: i64,
    pub total_display: String, // XMR with formatting
    pub created_at: String,
    pub updated_at: String,
}

impl From<Order> for OrderResponse {
    fn from(order: Order) -> Self {
        Self {
            id: order.id.clone(),
            buyer_id: order.buyer_id.clone(),
            vendor_id: order.vendor_id.clone(),
            listing_id: order.listing_id.clone(),
            escrow_id: order.escrow_id.clone(),
            status: order.status.clone(),
            total_xmr: order.total_xmr,
            total_display: format!("{:.12} XMR", order.total_as_xmr()),
            created_at: order.created_at.to_string(),
            updated_at: order.updated_at.to_string(),
        }
    }
}

/// Helper to get authenticated user ID from session
fn get_user_id_from_session(session: &Session) -> Result<String, HttpResponse> {
    session
        .get::<String>("user_id")
        .map_err(|_| {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to read session"
            }))
        })?
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }))
        })
}

/// POST /api/orders - Create a new order
///
/// Creates a new order in pending status. The buyer must fund the escrow
/// separately to transition the order to funded status.
///
/// Requires authentication as a buyer.
#[post("/api/orders")]
pub async fn create_order(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<CreateOrderRequest>,
) -> impl Responder {
    // Validate input
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation error: {}", e)
        }));
    }

    // Get authenticated user (buyer)
    let buyer_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    // Verify listing exists and is active
    let listing = match Listing::find_by_id(&mut conn, req.listing_id.clone()) {
        Ok(listing) => listing,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Listing not found"
            }))
        }
    };

    if listing.status != "active" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Listing is not active"
        }));
    }

    // Check stock availability
    if listing.stock < req.quantity {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Insufficient stock. Available: {}, requested: {}",
                listing.stock, req.quantity)
        }));
    }

    // Prevent self-purchasing
    if listing.vendor_id == buyer_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot purchase your own listing"
        }));
    }

    // Calculate total (price_xmr is already in atomic units)
    let total_xmr = listing.price_xmr * req.quantity as i64;

    // Create order
    let new_order = NewOrder {
        id: Uuid::new_v4().to_string(),
        buyer_id,
        vendor_id: listing.vendor_id.clone(),
        listing_id: req.listing_id.clone(),
        escrow_id: None, // Set when escrow is created
        status: OrderStatus::Pending.as_str().to_string(),
        total_xmr,
    };

    match Order::create(&mut conn, new_order) {
        Ok(order) => HttpResponse::Created().json(OrderResponse::from(order)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create order: {}", e)
        })),
    }
}

/// GET /api/orders - List all orders for the authenticated user
///
/// Returns orders where the user is either the buyer or vendor.
#[get("/api/orders")]
pub async fn list_orders(pool: web::Data<DbPool>, session: Session) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    // Get orders where user is buyer
    let buyer_orders = match Order::find_by_buyer(&mut conn, user_id.clone()) {
        Ok(orders) => orders,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch buyer orders: {}", e)
            }))
        }
    };

    // Get orders where user is vendor
    let vendor_orders = match Order::find_by_vendor(&mut conn, user_id) {
        Ok(orders) => orders,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch vendor orders: {}", e)
            }))
        }
    };

    // Combine and deduplicate
    let mut all_orders = buyer_orders;
    all_orders.extend(vendor_orders);

    let responses: Vec<OrderResponse> = all_orders.into_iter().map(OrderResponse::from).collect();

    HttpResponse::Ok().json(responses)
}

/// GET /api/orders/{id} - Get a single order by ID
///
/// Requires authentication. Only buyer or vendor can view the order.
#[get("/api/orders/{id}")]
pub async fn get_order(
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let order = match Order::find_by_id(&mut conn, id.into_inner()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: only buyer or vendor can view
    if order.buyer_id != user_id && order.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only view your own orders"
        }));
    }

    HttpResponse::Ok().json(OrderResponse::from(order))
}

/// PUT /api/orders/{id}/ship - Mark order as shipped
///
/// Requires authentication as the vendor. Order must be in funded status.
#[put("/api/orders/{id}/ship")]
pub async fn ship_order(
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let order_id = id.into_inner();
    let order = match Order::find_by_id(&mut conn, order_id.clone()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: only vendor can mark as shipped
    if order.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only the vendor can mark order as shipped"
        }));
    }

    // Validate state transition
    if !order.can_mark_shipped() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Cannot ship order in status: {}", order.status)
        }));
    }

    match Order::update_status(&mut conn, order_id, OrderStatus::Shipped) {
        Ok(updated_order) => HttpResponse::Ok().json(OrderResponse::from(updated_order)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update order status: {}", e)
        })),
    }
}

/// PUT /api/orders/{id}/complete - Confirm receipt and release funds
///
/// Requires authentication as the buyer. Order must be in shipped status.
/// This triggers the escrow release to the vendor.
#[put("/api/orders/{id}/complete")]
pub async fn complete_order(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<Arc<EscrowOrchestrator>>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let order_id = id.into_inner();
    let order = match Order::find_by_id(&mut conn, order_id.clone()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: only buyer can confirm receipt
    if order.buyer_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only the buyer can confirm receipt"
        }));
    }

    // Validate state transition
    if !order.can_confirm_receipt() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Cannot complete order in status: {}", order.status)
        }));
    }

    // Validate escrow exists for this order
    let escrow_id_str = match &order.escrow_id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Order has no associated escrow"
            }))
        }
    };

    let escrow_uuid = match Uuid::parse_str(escrow_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid escrow ID format"
            }))
        }
    };

    // Get vendor's wallet address
    let vendor = match User::find_by_id(&mut conn, order.vendor_id.clone()) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Vendor not found"
            }))
        }
    };

    let vendor_address = match vendor.wallet_address {
        Some(addr) => addr,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Vendor has no wallet address configured"
            }))
        }
    };

    // Parse buyer UUID
    let buyer_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid buyer ID format"
            }))
        }
    };

    // Release funds via EscrowOrchestrator
    match escrow_orchestrator
        .release_funds(escrow_uuid, buyer_uuid, vendor_address)
        .await
    {
        Ok(tx_hash) => {
            // Update order status to completed
            match Order::update_status(&mut conn, order_id, OrderStatus::Completed) {
                Ok(updated_order) => {
                    let response = OrderResponse::from(updated_order);
                    HttpResponse::Ok().json(serde_json::json!({
                        "order": response,
                        "tx_hash": tx_hash,
                        "message": "Funds released to vendor successfully"
                    }))
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Funds released but failed to update order status: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to release funds: {}", e)
        })),
    }
}

/// PUT /api/orders/{id}/cancel - Cancel an order
///
/// Buyer can cancel in pending or funded status.
/// If funded, triggers refund via escrow.
#[put("/api/orders/{id}/cancel")]
pub async fn cancel_order(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<Arc<EscrowOrchestrator>>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let order_id = id.into_inner();
    let order = match Order::find_by_id(&mut conn, order_id.clone()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: only buyer can cancel
    if order.buyer_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only the buyer can cancel the order"
        }));
    }

    // Validate cancellation is allowed
    if !order.can_buyer_cancel() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Cannot cancel order in status: {}", order.status)
        }));
    }

    // Check if order is funded (needs refund)
    let needs_refund = order.status == "funded";

    if needs_refund {
        // Validate escrow exists for this order
        let escrow_id_str = match &order.escrow_id {
            Some(id) => id,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Order is funded but has no associated escrow"
                }))
            }
        };

        let escrow_uuid = match Uuid::parse_str(escrow_id_str) {
            Ok(uuid) => uuid,
            Err(_) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Invalid escrow ID format"
                }))
            }
        };

        // Get buyer's wallet address for refund
        let buyer = match User::find_by_id(&mut conn, order.buyer_id.clone()) {
            Ok(user) => user,
            Err(_) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Buyer not found"
                }))
            }
        };

        let buyer_address = match buyer.wallet_address {
            Some(addr) => addr,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Buyer has no wallet address configured for refund"
                }))
            }
        };

        // Parse user UUID (buyer is the requester)
        let requester_uuid = match Uuid::parse_str(&user_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Invalid user ID format"
                }))
            }
        };

        // Refund funds via EscrowOrchestrator
        match escrow_orchestrator
            .refund_funds(escrow_uuid, requester_uuid, buyer_address)
            .await
        {
            Ok(tx_hash) => {
                // Update order status to cancelled
                match Order::update_status(&mut conn, order_id, OrderStatus::Cancelled) {
                    Ok(updated_order) => HttpResponse::Ok().json(serde_json::json!({
                        "order": OrderResponse::from(updated_order),
                        "tx_hash": tx_hash,
                        "message": "Order cancelled and funds refunded successfully"
                    })),
                    Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Funds refunded but failed to update order status: {}", e)
                    })),
                }
            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to refund funds: {}", e)
            })),
        }
    } else {
        // Order not funded yet, just cancel without refund
        match Order::update_status(&mut conn, order_id, OrderStatus::Cancelled) {
            Ok(updated_order) => HttpResponse::Ok().json(serde_json::json!({
                "order": OrderResponse::from(updated_order),
                "message": "Order cancelled successfully"
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to cancel order: {}", e)
            })),
        }
    }
}

/// Request body for raising a dispute
#[derive(Debug, Deserialize, Validate)]
pub struct DisputeRequest {
    #[validate(length(
        min = 10,
        max = 1000,
        message = "Reason must be between 10 and 1000 characters"
    ))]
    pub reason: String,
}

/// PUT /api/orders/{id}/dispute - Raise a dispute
///
/// Either buyer or vendor can raise a dispute on funded or shipped orders.
/// This involves the arbiter to resolve the issue.
#[put("/api/orders/{id}/dispute")]
pub async fn dispute_order(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<Arc<EscrowOrchestrator>>,
    session: Session,
    id: web::Path<String>,
    req: web::Json<DisputeRequest>,
) -> impl Responder {
    // Validate input
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation error: {}", e)
        }));
    }

    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let order_id = id.into_inner();
    let order = match Order::find_by_id(&mut conn, order_id.clone()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: buyer or vendor can dispute
    if order.buyer_id != user_id && order.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only buyer or vendor can raise a dispute"
        }));
    }

    // Validate dispute is allowed
    if !order.can_dispute() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Cannot dispute order in status: {}", order.status)
        }));
    }

    // Validate escrow exists for this order
    let escrow_id_str = match &order.escrow_id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Order has no associated escrow"
            }))
        }
    };

    let escrow_uuid = match Uuid::parse_str(escrow_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid escrow ID format"
            }))
        }
    };

    // Parse requester UUID
    let requester_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid user ID format"
            }))
        }
    };

    // Initiate dispute via EscrowOrchestrator
    match escrow_orchestrator
        .initiate_dispute(escrow_uuid, requester_uuid, req.reason.clone())
        .await
    {
        Ok(_) => {
            // Update order status to disputed
            match Order::update_status(&mut conn, order_id, OrderStatus::Disputed) {
                Ok(updated_order) => HttpResponse::Ok().json(serde_json::json!({
                    "order": OrderResponse::from(updated_order),
                    "message": "Dispute raised successfully. An arbiter will review your case."
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Dispute initiated but failed to update order status: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initiate dispute: {}", e)
        })),
    }
}
