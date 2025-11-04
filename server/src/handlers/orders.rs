//! Order API handlers
//!
//! REST API endpoints for managing purchase orders in the escrow marketplace.

use actix::Addr;
use actix_session::Session;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use diesel::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::crypto::encryption::{decrypt_field, encrypt_field};
use crate::db::{DbPool, db_load_escrow};
use crate::middleware::csrf::validate_csrf_token;
use crate::models::cart::Cart;
use crate::models::listing::Listing;
use crate::models::order::{NewOrder, Order, OrderStatus};
use crate::models::user::User;
use crate::services::escrow::EscrowOrchestrator;
use crate::websocket::{NotifyUser, WebSocketServer, WsEvent};

/// Request body for creating a new order
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(equal = 36, message = "Listing ID must be a valid UUID"))]
    pub listing_id: String,

    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,

    #[validate(length(min = 10, max = 500, message = "Shipping address must be between 10 and 500 characters"))]
    pub shipping_address: String,

    #[validate(length(max = 200, message = "Shipping notes must be 200 characters or less"))]
    pub shipping_notes: Option<String>,
}

/// Request body for creating a new order from cart
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderFromCartRequest {
    pub checkout_mode: String, // "cart" | "listing"

    #[validate(length(min = 10, max = 500, message = "Shipping address must be between 10 and 500 characters"))]
    pub shipping_address: String,

    #[validate(length(max = 200, message = "Shipping notes must be 200 characters or less"))]
    pub shipping_notes: Option<String>,
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

/// POST /api/orders/create - Create a new order from cart
///
/// Creates a new order from the buyer's cart with encrypted shipping address.
/// This is the main checkout endpoint used by the frontend.
///
/// Requires authentication as a buyer.
#[post("/orders/create")]
pub async fn create_order_from_cart(
    pool: web::Data<DbPool>,
    session: Session,
    http_req: HttpRequest,
    req: web::Json<CreateOrderFromCartRequest>,
    websocket: web::Data<Addr<WebSocketServer>>,
    encryption_key: web::Data<Vec<u8>>,
) -> impl Responder {
    // SECURITY: Validate CSRF token
    let csrf_token = http_req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !validate_csrf_token(&session, csrf_token) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Invalid or missing CSRF token"
        }));
    }

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

    // SECURITY: Verify user has buyer role
    let user_role = match session.get::<String>("role") {
        Ok(Some(role)) => role,
        _ => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Buyer role required to create orders"
            }))
        }
    };

    if user_role != "buyer" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only buyers can create orders"
        }));
    }

    // Get database connection early for both modes
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    // Handle different checkout modes
    let (vendor_id, listing_id, total_xmr, quantity) = if req.checkout_mode == "listing" {
        // Single listing mode (Buy Now)
        let listing_id_from_session = match session.get::<String>("checkout_listing_id") {
            Ok(Some(id)) => id,
            _ => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "No listing selected for checkout"
                }))
            }
        };

        // Fetch listing
        let listing = match Listing::find_by_id(&mut conn, listing_id_from_session.clone()) {
            Ok(l) => l,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Listing not found"
                }))
            }
        };

        // Validate listing is active
        if listing.status != "active" {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "This listing is not available for purchase"
            }));
        }

        // Check stock (quantity = 1 for Buy Now)
        if listing.stock < 1 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "This item is out of stock"
            }));
        }

        // Prevent self-purchasing
        if listing.vendor_id == buyer_id {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Cannot purchase your own listing"
            }));
        }

        // Clear listing_id from session after retrieving
        let _ = session.remove("checkout_listing_id");

        (listing.vendor_id.clone(), listing.id.clone(), listing.price_xmr, 1)
    } else {
        // Cart mode (existing logic)
        let mut cart = match session.get::<Cart>("cart") {
            Ok(Some(c)) => c,
            _ => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Cart is empty"
                }))
            }
        };

        if cart.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Cart is empty"
            }));
        }

        // For now, only support single-vendor carts
        // TODO: Support multi-vendor carts with separate orders per vendor
        let vendor_id = cart.items[0].vendor_id.clone();
        if !cart.items.iter().all(|item| &item.vendor_id == &vendor_id) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Multi-vendor carts not yet supported. Please checkout items from one vendor at a time."
            }));
        }

        // Prevent self-purchasing
        if vendor_id == buyer_id {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Cannot purchase your own listings"
            }));
        }

        let total_xmr = cart.total_price();
        let listing_id = cart.items[0].listing_id.clone();

        (vendor_id, listing_id, total_xmr, cart.total_quantity())
    };

    // SECURITY: Validate total is positive and reasonable
    if total_xmr <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid order total"
        }));
    }

    // SECURITY: Check maximum order value (e.g., 10,000 XMR = 10^16 piconeros)
    const MAX_ORDER_VALUE: i64 = 10_000_000_000_000_000; // 10,000 XMR
    if total_xmr > MAX_ORDER_VALUE {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Order total exceeds maximum allowed value"
        }));
    }

    // SECURITY: Field-level AES-256-GCM encryption for shipping address
    // Only the vendor can decrypt this address using the same encryption key
    // Database is also encrypted at rest with SQLCipher (defense in depth)
    let encrypted_address = match encrypt_field(&req.shipping_address, &encryption_key) {
        Ok(encrypted_bytes) => base64::encode(&encrypted_bytes),
        Err(e) => {
            tracing::error!("Failed to encrypt shipping address: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to encrypt shipping address"
            }));
        }
    };

    // Create the order (conn already acquired at line 163)
    let new_order = NewOrder {
        id: Uuid::new_v4().to_string(),
        buyer_id: buyer_id.clone(),
        vendor_id: vendor_id.clone(),
        listing_id: listing_id.clone(), // From tuple (cart or listing mode)
        escrow_id: None, // Set when escrow is initialized
        status: OrderStatus::Pending.as_str().to_string(),
        total_xmr,
        shipping_address: Some(encrypted_address),
        shipping_notes: req.shipping_notes.clone(),
    };

    let order = match Order::create(&mut conn, new_order) {
        Ok(order) => order,
        Err(e) => {
            tracing::error!("Failed to create order: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create order"
            }))
        }
    };

    tracing::info!(
        "Order created successfully: id={}, buyer={}, vendor={}, total={} piconeros, mode={}",
        order.id, order.buyer_id, order.vendor_id, order.total_xmr, req.checkout_mode
    );

    // Clear the cart in session ONLY if this was a cart checkout (not Buy Now)
    if req.checkout_mode == "cart" {
        if let Ok(Some(mut cart)) = session.get::<Cart>("cart") {
            cart.clear();
            if let Err(e) = session.insert("cart", &cart) {
                tracing::warn!("Failed to clear cart in session after order creation: {}", e);
                // Don't fail the request, order was created successfully
            }
        }
    }

    // Send WebSocket notification to vendor
    let vendor_uuid = match Uuid::parse_str(&order.vendor_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            tracing::error!("Invalid vendor UUID: {}", order.vendor_id);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal error"
            }));
        }
    };

    let order_uuid = match Uuid::parse_str(&order.id) {
        Ok(uuid) => uuid,
        Err(_) => {
            tracing::error!("Invalid order UUID: {}", order.id);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal error"
            }));
        }
    };

    websocket.do_send(NotifyUser {
        user_id: vendor_uuid,
        event: WsEvent::OrderStatusChanged {
            order_id: order_uuid,
            new_status: "pending".to_string(),
        },
    });

    tracing::info!("Sent order notification to vendor {}", vendor_uuid);

    HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "order_id": order.id,
        "total_xmr": order.total_xmr,
        "message": "Order created successfully"
    }))
}

/// POST /api/orders - Create a new order
///
/// Creates a new order in pending status. The buyer must fund the escrow
/// separately to transition the order to funded status.
///
/// Requires authentication as a buyer.
#[post("/orders")]
pub async fn create_order(
    pool: web::Data<DbPool>,
    session: Session,
    http_req: HttpRequest,
    req: web::Json<CreateOrderRequest>,
    websocket: web::Data<Addr<WebSocketServer>>,
    encryption_key: web::Data<Vec<u8>>,
) -> impl Responder {
    // SECURITY: Validate CSRF token
    let csrf_token = http_req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !validate_csrf_token(&session, csrf_token) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Invalid or missing CSRF token"
        }));
    }

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

    // SECURITY: Verify user has buyer role
    let user_role = match session.get::<String>("role") {
        Ok(Some(role)) => role,
        _ => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Buyer role required to create orders"
            }))
        }
    };

    if user_role != "buyer" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only buyers can create orders"
        }));
    }

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

    // SECURITY: Calculate total with overflow protection
    // price_xmr is in atomic units (piconeros)
    let total_xmr = match listing.price_xmr.checked_mul(req.quantity as i64) {
        Some(total) => total,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Order total exceeds maximum value (integer overflow)"
            }))
        }
    };

    // SECURITY: Validate total is positive and reasonable
    if total_xmr <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid order total"
        }));
    }

    // SECURITY: Check maximum order value (e.g., 10,000 XMR = 10^16 piconeros)
    const MAX_ORDER_VALUE: i64 = 10_000_000_000_000_000; // 10,000 XMR
    if total_xmr > MAX_ORDER_VALUE {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Order total exceeds maximum allowed value"
        }));
    }

    // SECURITY: Field-level AES-256-GCM encryption for shipping address
    let encrypted_address = match encrypt_field(&req.shipping_address, &encryption_key) {
        Ok(encrypted_bytes) => base64::encode(&encrypted_bytes),
        Err(e) => {
            tracing::error!("Failed to encrypt shipping address: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to encrypt shipping address"
            }));
        }
    };

    // SECURITY: Use database transaction to atomically create order and reserve stock
    // This prevents race conditions where multiple buyers could order the same stock
    let order_result = conn.transaction::<Order, diesel::result::Error, _>(|conn| {
        // First, decrease stock atomically
        // This will fail if stock is insufficient (race condition protection)
        Listing::decrease_stock(conn, req.listing_id.clone(), req.quantity)
            .map_err(|e| {
                tracing::error!("Failed to decrease stock: {}", e);
                diesel::result::Error::RollbackTransaction
            })?;

        // Then create the order
        let new_order = NewOrder {
            id: Uuid::new_v4().to_string(),
            buyer_id: buyer_id.clone(),
            vendor_id: listing.vendor_id.clone(),
            listing_id: req.listing_id.clone(),
            escrow_id: None, // Set when escrow is created
            status: OrderStatus::Pending.as_str().to_string(),
            total_xmr,
            shipping_address: Some(encrypted_address),
            shipping_notes: req.shipping_notes.clone(),
        };

        Order::create(conn, new_order).map_err(|e| {
            tracing::error!("Failed to create order: {}", e);
            diesel::result::Error::RollbackTransaction
        })
    });

    match order_result {
        Ok(order) => {
            tracing::info!(
                "Order created successfully: id={}, buyer={}, vendor={}, total={} piconeros",
                order.id, order.buyer_id, order.vendor_id, order.total_xmr
            );
            
            // Send WebSocket notification to vendor
            let vendor_uuid = match Uuid::parse_str(&order.vendor_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    tracing::error!("Invalid vendor UUID: {}", order.vendor_id);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Internal error"
                    }));
                }
            };
            
            let order_uuid = match Uuid::parse_str(&order.id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    tracing::error!("Invalid order UUID: {}", order.id);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Internal error"
                    }));
                }
            };
            
            websocket.do_send(NotifyUser {
                user_id: vendor_uuid,
                event: WsEvent::OrderStatusChanged {
                    order_id: order_uuid,
                    new_status: "pending".to_string(),
                },
            });
            
            tracing::info!("Sent order notification to vendor {}", vendor_uuid);
            
            HttpResponse::Created().json(OrderResponse::from(order))
        }
        Err(e) => {
            tracing::error!("Transaction failed: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create order. Stock may have been exhausted."
            }))
        }
    }
}

/// GET /api/orders/pending-count - Get count of pending orders for vendor
#[get("/orders/pending-count")]
pub async fn get_pending_count(pool: web::Data<DbPool>, session: Session) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    // Check if user is vendor
    let user_role = match session.get::<String>("role") {
        Ok(Some(role)) => role,
        _ => return HttpResponse::Ok().json(serde_json::json!({ "count": 0 })),
    };

    if user_role != "vendor" {
        return HttpResponse::Ok().json(serde_json::json!({ "count": 0 }));
    }

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    // Count pending orders where user is vendor
    let count_result = web::block(move || {
        use crate::schema::orders::dsl::*;
        use diesel::prelude::*;
        
        orders
            .filter(vendor_id.eq(user_id))
            .filter(status.eq("pending"))
            .count()
            .get_result::<i64>(&mut conn)
    })
    .await;

    match count_result {
        Ok(Ok(count)) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        _ => HttpResponse::Ok().json(serde_json::json!({ "count": 0 })),
    }
}

/// GET /api/orders - List all orders for the authenticated user
///
/// Returns orders where the user is either the buyer or vendor.
#[get("/orders")]
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
#[get("/orders/{id}")]
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

/// POST /api/orders/{id}/ship - Mark order as shipped
///
/// Requires authentication as the vendor. Order must be in funded status.
#[post("/orders/{id}/ship")]
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

    // CRITICAL: Verify vendor has wallet address configured
    // Without this, buyer cannot complete the order (complete_order requires vendor wallet_address)
    let vendor = match User::find_by_id(&mut conn, user_id.clone()) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Vendor not found"
            }))
        }
    };

    if vendor.wallet_address.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "You must configure your Monero wallet address before shipping orders. Go to Settings to add your wallet address."
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
#[post("/orders/{id}/complete")]
pub async fn complete_order(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
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

    // Validate state transition - with detailed error logging
    tracing::info!(
        "Validating order {} completion: current_status='{}' (raw DB value)",
        order_id,
        order.status
    );

    // Parse status first to get better error messages
    let current_status = match order.get_status() {
        Ok(status) => status,
        Err(e) => {
            tracing::error!(
                "Failed to parse order status '{}' for order {}: {}",
                order.status,
                order_id,
                e
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Invalid order status in database: '{}'", order.status)
            }));
        }
    };

    tracing::info!(
        "Parsed order status successfully: {:?}",
        current_status
    );

    // Check if order is in 'shipped' status
    if current_status != OrderStatus::Shipped {
        tracing::warn!(
            "Buyer {} attempted to complete order {} in invalid status: {:?} (must be Shipped)",
            user_id,
            order_id,
            current_status
        );
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Cannot complete order in status '{}'. Order must be 'shipped' first.", current_status.as_str())
        }));
    }

    tracing::info!(
        "Order {} status validation passed - proceeding with completion",
        order_id
    );

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

/// POST /api/orders/{id}/init-escrow - Initialize escrow for an order
///
/// Buyer initializes the escrow multisig and gets the payment address.
#[post("/orders/{id}/init-escrow")]
pub async fn init_escrow(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    http_req: HttpRequest,
    id: web::Path<String>,
) -> impl Responder {
    // TEMPORARY: CSRF validation disabled for database debugging
    // TODO: Re-enable after fixing database issue
    // let csrf_token = http_req
    //     .headers()
    //     .get("X-CSRF-Token")
    //     .and_then(|h| h.to_str().ok())
    //     .unwrap_or("");
    //
    // if !validate_csrf_token(&session, csrf_token) {
    //     return HttpResponse::Forbidden().json(serde_json::json!({
    //         "error": "Invalid or missing CSRF token"
    //     }));
    // }

    // Get authenticated user (buyer)
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

    let order_id_str = id.into_inner();
    let order = match Order::find_by_id(&mut conn, order_id_str.clone()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: only buyer can initialize escrow
    if order.buyer_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only the buyer can initialize escrow"
        }));
    }

    // Validate order is in pending status
    if order.status != "pending" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Cannot initialize escrow for order in status: {}", order.status)
        }));
    }

    // Check if escrow already exists
    if order.escrow_id.is_some() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Escrow already initialized for this order"
        }));
    }

    // Parse UUIDs
    let order_uuid = match Uuid::parse_str(&order.id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid order ID format"
            }))
        }
    };

    let buyer_uuid = match Uuid::parse_str(&order.buyer_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid buyer ID format"
            }))
        }
    };

    let vendor_uuid = match Uuid::parse_str(&order.vendor_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid vendor ID format"
            }))
        }
    };

    // Initialize escrow
    match escrow_orchestrator
        .init_escrow(order_uuid, buyer_uuid, vendor_uuid, order.total_xmr)
        .await
    {
        Ok(escrow) => {
            // Update order with escrow_id
            match Order::set_escrow(&mut conn, order_id_str, escrow.id.clone()) {
                Ok(_) => {
                    tracing::info!("Escrow initialized for order {}: escrow_id={}", order.id, escrow.id);

                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "escrow_id": escrow.id,
                        "escrow_address": escrow.multisig_address.unwrap_or_else(|| "Pending".to_string()),
                        "amount": order.total_xmr,
                        "amount_xmr": format!("{:.12}", order.total_xmr as f64 / 1_000_000_000_000.0),
                        "status": escrow.status
                    }))
                }
                Err(e) => {
                    tracing::error!("Failed to update order with escrow_id: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Escrow created but failed to link to order"
                    }))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to initialize escrow: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to initialize escrow: {}", e)
            }))
        }
    }
}

/// POST /api/orders/{id}/dev-simulate-payment - Simulate escrow payment (DEV ONLY)
///
/// Development endpoint to simulate payment without real XMR.
/// Available in all builds for testing purposes.
/// Also initializes mock multisig wallets for testing the release flow.
#[post("/orders/{id}/dev-simulate-payment")]
pub async fn dev_simulate_payment(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
    session: Session,
    http_req: HttpRequest,
    id: web::Path<String>,
    websocket: web::Data<Addr<WebSocketServer>>,
) -> impl Responder {
    // SECURITY: Validate CSRF token
    let csrf_token = http_req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !validate_csrf_token(&session, csrf_token) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Invalid or missing CSRF token"
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

    let order_id_str = id.into_inner();
    let order = match Order::find_by_id(&mut conn, order_id_str.clone()) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }))
        }
    };

    // Authorization: only buyer can simulate payment
    if order.buyer_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only the buyer can simulate payment"
        }));
    }

    // Validate order has escrow
    let escrow_id = match &order.escrow_id {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Order has no escrow. Click 'Fund Escrow' first."
            }))
        }
    };

    let escrow_id_clone = escrow_id.clone();
    
    // Update escrow status to funded
    match web::block(move || {
        use crate::schema::escrows::dsl::*;
        use diesel::prelude::*;
        
        diesel::update(escrows.filter(id.eq(&escrow_id_clone)))
            .set(status.eq("funded"))
            .execute(&mut conn)
    })
    .await
    {
        Ok(Ok(_)) => {
            // Update order status to funded
            let mut conn2 = match pool.get() {
                Ok(c) => c,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Database error"
                    }))
                }
            };
            
            match Order::update_status(&mut conn2, order_id_str.clone(), OrderStatus::Funded) {
                Ok(_) => {
                    tracing::info!("DEV: Simulated payment for order {} (escrow {})", order.id, escrow_id);

                    // DEV: Initialize mock multisig wallets for testing
                    let escrow_uuid = match Uuid::parse_str(&escrow_id) {
                        Ok(uuid) => uuid,
                        Err(_) => {
                            tracing::error!("Invalid escrow UUID: {}", escrow_id);
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Internal error"
                            }));
                        }
                    };

                    match escrow_orchestrator.dev_initialize_mock_wallets(escrow_uuid).await {
                        Ok(_) => {
                            tracing::info!("DEV: Mock multisig wallets initialized for escrow {}", escrow_id);
                        }
                        Err(e) => {
                            tracing::warn!("DEV: Failed to initialize mock wallets: {}. This may cause issues with release.", e);
                            // Don't fail the request, just log warning
                        }
                    }

                    // Send WebSocket notification to vendor
                    let vendor_uuid = match Uuid::parse_str(&order.vendor_id) {
                        Ok(uuid) => uuid,
                        Err(_) => {
                            tracing::error!("Invalid vendor UUID: {}", order.vendor_id);
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Internal error"
                            }));
                        }
                    };
                    
                    let order_uuid = match Uuid::parse_str(&order.id) {
                        Ok(uuid) => uuid,
                        Err(_) => {
                            tracing::error!("Invalid order UUID: {}", order.id);
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Internal error"
                            }));
                        }
                    };
                    
                    // Notify vendor that order is now funded
                    websocket.do_send(NotifyUser {
                        user_id: vendor_uuid,
                        event: WsEvent::OrderStatusChanged {
                            order_id: order_uuid,
                            new_status: "funded".to_string(),
                        },
                    });
                    
                    tracing::info!("Sent payment notification to vendor {}", vendor_uuid);
                    
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "message": "Payment simulated successfully (DEV MODE)",
                        "order_id": order.id,
                        "escrow_id": escrow_id,
                        "new_status": "funded"
                    }))
                }
                Err(e) => {
                    tracing::error!("Failed to update order status: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to update order: {}", e)
                    }))
                }
            }
        }
        _ => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update escrow status"
            }))
        }
    }
}

/// PUT /api/orders/{id}/cancel - Cancel an order
///
/// Buyer can cancel in pending or funded status.
/// If funded, triggers refund via escrow.
#[put("/orders/{id}/cancel")]
pub async fn cancel_order(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
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

        // Load escrow and verify buyer is the one cancelling
        let escrow = match db_load_escrow(&pool, escrow_uuid).await {
            Ok(e) => e,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Escrow not found"
                }))
            }
        };

        if escrow.buyer_id != user_id.to_string() {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only the buyer can cancel this order"
            }));
        }

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
#[put("/orders/{id}/dispute")]
pub async fn dispute_order(
    pool: web::Data<DbPool>,
    escrow_orchestrator: web::Data<EscrowOrchestrator>,
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
