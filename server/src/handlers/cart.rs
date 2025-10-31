//! Shopping cart API handlers
//!
//! Provides REST API endpoints for cart operations.
//! All cart data is stored in session storage (JSON serialized).
//!
//! # Endpoints
//! - POST /api/cart/add - Add item to cart
//! - POST /api/cart/remove - Remove item from cart
//! - POST /api/cart/update - Update item quantity
//! - POST /api/cart/clear - Clear entire cart
//! - GET /api/cart - Get current cart state
//! - GET /api/cart/count - Get cart item count

use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::middleware::csrf::validate_csrf_token;
use crate::models::cart::{Cart, CartItem};
use crate::models::listing::Listing;
use crate::db::DbPool;

/// Request to add item to cart
#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub listing_id: String,
    pub quantity: i32,
    pub csrf_token: String,
}

/// Request to update item quantity
#[derive(Debug, Deserialize)]
pub struct UpdateCartRequest {
    pub listing_id: String,
    pub quantity: i32,
    pub csrf_token: String,
}

/// Request to remove item from cart
#[derive(Debug, Deserialize)]
pub struct RemoveFromCartRequest {
    pub listing_id: String,
    pub csrf_token: String,
}

/// Standard API response
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cart: Option<Cart>,
}

/// POST /api/cart/add - Add item to cart
///
/// Fetches listing from database, validates availability, and adds to session cart.
/// If item already exists in cart, increments quantity.
///
/// # Authentication
/// - No authentication required (cart is session-based)
///
/// # Request Body
/// ```json
/// {
///   "listing_id": "abc123",
///   "quantity": 2
/// }
/// ```
///
/// # Returns
/// - 200 OK: Item added successfully
/// - 400 Bad Request: Invalid quantity, listing not found, or out of stock
/// - 500 Internal Server Error: Database or session error
pub async fn add_to_cart(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<AddToCartRequest>,
) -> impl Responder {
    // CSRF protection
    if !validate_csrf_token(&session, &req.csrf_token) {
        return HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            message: "Invalid CSRF token".to_string(),
            cart: None,
        });
    }

    // Validate quantity
    if req.quantity <= 0 {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Quantity must be positive".to_string(),
            cart: None,
        });
    }

    // Fetch listing from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                message: "Database connection error".to_string(),
                cart: None,
            });
        }
    };

    let listing_id = req.listing_id.clone();
    let listing_result = web::block(move || Listing::find_by_id(&mut conn, listing_id)).await;

    let listing = match listing_result {
        Ok(Ok(l)) => l,
        Ok(Err(e)) => {
            error!("Listing not found: {}", e);
            return HttpResponse::BadRequest().json(ApiResponse {
                success: false,
                message: "Listing not found".to_string(),
                cart: None,
            });
        }
        Err(e) => {
            error!("Database query error: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                message: "Database error".to_string(),
                cart: None,
            });
        }
    };

    // Check if listing is active
    if listing.status != "active" {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "This listing is not available".to_string(),
            cart: None,
        });
    }

    // Check stock availability
    if listing.stock < req.quantity {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Only {} items available in stock", listing.stock),
            cart: None,
        });
    }

    // Get or create cart from session
    let mut cart = match session.get::<Cart>("cart") {
        Ok(Some(c)) => c,
        _ => Cart::new(),
    };

    // Extract first image CID from JSON array
    let image_cid = listing.images_ipfs_cids.as_ref().and_then(|cids_json| {
        serde_json::from_str::<Vec<String>>(cids_json)
            .ok()
            .and_then(|cids| cids.first().cloned())
    });

    // TODO: Fetch vendor username from users table via vendor_id
    // For now, use vendor_id as placeholder
    let vendor_username = listing.vendor_id.clone();

    // Create cart item
    let cart_item = CartItem {
        listing_id: listing.id.clone(),
        title: listing.title.clone(),
        vendor_id: listing.vendor_id.clone(),
        vendor_username,
        unit_price_xmr: listing.price_xmr,
        quantity: req.quantity,
        image_cid,
    };

    // Add to cart
    if let Err(e) = cart.add_item(cart_item) {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: e,
            cart: None,
        });
    }

    // Save cart to session
    if let Err(e) = session.insert("cart", &cart) {
        error!("Failed to save cart to session: {}", e);
        return HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: "Failed to save cart".to_string(),
            cart: None,
        });
    }

    info!("Added item {} to cart (quantity: {})", listing.id, req.quantity);

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Item added to cart".to_string(),
        cart: Some(cart),
    })
}

/// POST /api/cart/remove - Remove item from cart
///
/// # Request Body
/// ```json
/// {
///   "listing_id": "abc123"
/// }
/// ```
///
/// # Returns
/// - 200 OK: Item removed successfully
/// - 400 Bad Request: Item not found in cart
/// - 500 Internal Server Error: Session error
pub async fn remove_from_cart(
    session: Session,
    req: web::Json<RemoveFromCartRequest>,
) -> impl Responder {
    // CSRF protection
    if !validate_csrf_token(&session, &req.csrf_token) {
        return HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            message: "Invalid CSRF token".to_string(),
            cart: None,
        });
    }

    // Get cart from session
    let mut cart = match session.get::<Cart>("cart") {
        Ok(Some(c)) => c,
        _ => {
            return HttpResponse::BadRequest().json(ApiResponse {
                success: false,
                message: "Cart is empty".to_string(),
                cart: None,
            })
        }
    };

    // Remove item
    if !cart.remove_item(&req.listing_id) {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Item not found in cart".to_string(),
            cart: Some(cart),
        });
    }

    // Save updated cart
    if let Err(e) = session.insert("cart", &cart) {
        error!("Failed to save cart to session: {}", e);
        return HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: "Failed to update cart".to_string(),
            cart: None,
        });
    }

    info!("Removed item {} from cart", req.listing_id);

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Item removed from cart".to_string(),
        cart: Some(cart),
    })
}

/// POST /api/cart/update - Update item quantity
///
/// # Request Body
/// ```json
/// {
///   "listing_id": "abc123",
///   "quantity": 5
/// }
/// ```
///
/// # Returns
/// - 200 OK: Quantity updated successfully
/// - 400 Bad Request: Invalid quantity or item not found
/// - 500 Internal Server Error: Session error
pub async fn update_cart(
    session: Session,
    req: web::Json<UpdateCartRequest>,
) -> impl Responder {
    // CSRF protection
    if !validate_csrf_token(&session, &req.csrf_token) {
        return HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            message: "Invalid CSRF token".to_string(),
            cart: None,
        });
    }

    // Validate quantity
    if req.quantity <= 0 {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Quantity must be positive".to_string(),
            cart: None,
        });
    }

    // Get cart from session
    let mut cart = match session.get::<Cart>("cart") {
        Ok(Some(c)) => c,
        _ => {
            return HttpResponse::BadRequest().json(ApiResponse {
                success: false,
                message: "Cart is empty".to_string(),
                cart: None,
            })
        }
    };

    // Update quantity
    if let Err(e) = cart.update_quantity(&req.listing_id, req.quantity) {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: e,
            cart: Some(cart),
        });
    }

    // Save updated cart
    if let Err(e) = session.insert("cart", &cart) {
        error!("Failed to save cart to session: {}", e);
        return HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: "Failed to update cart".to_string(),
            cart: None,
        });
    }

    info!("Updated item {} quantity to {}", req.listing_id, req.quantity);

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Cart updated".to_string(),
        cart: Some(cart),
    })
}

/// POST /api/cart/clear - Clear entire cart
///
/// Requires CSRF token in X-CSRF-Token header
///
/// # Returns
/// - 200 OK: Cart cleared successfully
/// - 403 Forbidden: Invalid CSRF token
/// - 500 Internal Server Error: Session error
pub async fn clear_cart(req: HttpRequest, session: Session) -> impl Responder {
    // CSRF protection - read from header
    let csrf_token = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !validate_csrf_token(&session, csrf_token) {
        return HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            message: "Invalid CSRF token".to_string(),
            cart: None,
        });
    }

    // Remove cart from session
    session.remove("cart");

    info!("Cart cleared");

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Cart cleared".to_string(),
        cart: Some(Cart::new()),
    })
}

/// GET /api/cart - Get current cart state
///
/// # Returns
/// - 200 OK: Cart data (may be empty)
pub async fn get_cart(session: Session) -> impl Responder {
    let cart = match session.get::<Cart>("cart") {
        Ok(Some(c)) => c,
        _ => Cart::new(),
    };

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Cart retrieved".to_string(),
        cart: Some(cart),
    })
}

/// GET /api/cart/count - Get cart item count (for badge)
///
/// Returns the total number of distinct items in cart.
///
/// # Returns
/// ```json
/// {
///   "count": 3
/// }
/// ```
#[derive(Serialize)]
pub struct CartCountResponse {
    pub count: usize,
}

pub async fn get_cart_count(session: Session) -> impl Responder {
    let cart = match session.get::<Cart>("cart") {
        Ok(Some(c)) => c,
        _ => Cart::new(),
    };

    HttpResponse::Ok().json(CartCountResponse {
        count: cart.item_count(),
    })
}
