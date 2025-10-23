//! Listing API handlers
//!
//! REST API endpoints for managing product listings on the marketplace.

use actix_session::Session;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::models::listing::{Listing, ListingStatus, NewListing, UpdateListing};

/// Request body for creating a new listing
#[derive(Debug, Deserialize, Validate)]
pub struct CreateListingRequest {
    #[validate(length(min = 3, max = 200, message = "Title must be between 3-200 characters"))]
    pub title: String,

    #[validate(length(
        min = 10,
        max = 5000,
        message = "Description must be between 10-5000 characters"
    ))]
    pub description: String,

    #[validate(range(min = 1, message = "Price must be positive"))]
    pub price_xmr: i64,

    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: i32,
}

/// Request body for updating a listing
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateListingRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: Option<String>,

    #[validate(length(min = 10, max = 5000))]
    pub description: Option<String>,

    #[validate(range(min = 1))]
    pub price_xmr: Option<i64>,

    #[validate(range(min = 0))]
    pub stock: Option<i32>,

    pub status: Option<String>,
}

/// Response for listing operations
#[derive(Debug, Serialize)]
pub struct ListingResponse {
    pub id: String,
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    pub price_xmr: i64,
    pub price_display: String, // XMR with formatting
    pub stock: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Listing> for ListingResponse {
    fn from(listing: Listing) -> Self {
        Self {
            id: listing.id.clone(),
            vendor_id: listing.vendor_id.clone(),
            title: listing.title.clone(),
            description: listing.description.clone(),
            price_xmr: listing.price_xmr,
            price_display: format!("{:.12} XMR", listing.price_as_xmr()),
            stock: listing.stock,
            status: listing.status.clone(),
            created_at: listing.created_at.to_string(),
            updated_at: listing.updated_at.to_string(),
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

/// POST /api/listings - Create a new listing
///
/// Requires authentication and vendor role.
#[post("/api/listings")]
pub async fn create_listing(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Json<CreateListingRequest>,
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

    // Create listing
    let new_listing = NewListing {
        id: Uuid::new_v4().to_string(),
        vendor_id: user_id,
        title: req.title.clone(),
        description: req.description.clone(),
        price_xmr: req.price_xmr,
        stock: req.stock,
        status: ListingStatus::Active.as_str().to_string(),
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    match Listing::create(&mut conn, new_listing) {
        Ok(listing) => HttpResponse::Created().json(ListingResponse::from(listing)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create listing: {}", e)
        })),
    }
}

/// GET /api/listings - List all active listings (paginated)
#[get("/api/listings")]
pub async fn list_listings(
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(20)
        .min(100); // Max 100 per page

    let offset = query
        .get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    match Listing::list_active(&mut conn, limit, offset) {
        Ok(listings) => {
            let responses: Vec<ListingResponse> =
                listings.into_iter().map(ListingResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to list listings: {}", e)
        })),
    }
}

/// GET /api/listings/{id} - Get a single listing by ID
#[get("/api/listings/{id}")]
pub async fn get_listing(pool: web::Data<DbPool>, id: web::Path<String>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    match Listing::find_by_id(&mut conn, id.into_inner()) {
        Ok(listing) => HttpResponse::Ok().json(ListingResponse::from(listing)),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Listing not found"
        })),
    }
}

/// GET /api/listings/vendor/{vendor_id} - Get all listings by a vendor
#[get("/api/listings/vendor/{vendor_id}")]
pub async fn get_vendor_listings(
    pool: web::Data<DbPool>,
    vendor_id: web::Path<String>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    match Listing::find_by_vendor(&mut conn, vendor_id.into_inner()) {
        Ok(listings) => {
            let responses: Vec<ListingResponse> =
                listings.into_iter().map(ListingResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to get vendor listings: {}", e)
        })),
    }
}

/// GET /api/listings/search?q=query - Search listings by title
#[get("/api/listings/search")]
pub async fn search_listings(
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let search_query = match query.get("q") {
        Some(q) if !q.is_empty() => q,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Search query parameter 'q' is required"
            }))
        }
    };

    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(20)
        .min(100);

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    match Listing::search_by_title(&mut conn, search_query, limit) {
        Ok(listings) => {
            let responses: Vec<ListingResponse> =
                listings.into_iter().map(ListingResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Search failed: {}", e)
        })),
    }
}

/// PUT /api/listings/{id} - Update a listing
///
/// Requires authentication. Only the vendor who created the listing can update it.
#[put("/api/listings/{id}")]
pub async fn update_listing(
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
    req: web::Json<UpdateListingRequest>,
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

    let listing_id = id.into_inner();

    // Check listing exists and user owns it
    let existing_listing = match Listing::find_by_id(&mut conn, listing_id.clone()) {
        Ok(listing) => listing,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Listing not found"
            }))
        }
    };

    if existing_listing.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only update your own listings"
        }));
    }

    // Build update
    let update_data = UpdateListing {
        title: req.title.clone(),
        description: req.description.clone(),
        price_xmr: req.price_xmr,
        stock: req.stock,
        status: req.status.clone(),
    };

    match Listing::update(&mut conn, listing_id, update_data) {
        Ok(listing) => HttpResponse::Ok().json(ListingResponse::from(listing)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update listing: {}", e)
        })),
    }
}

/// DELETE /api/listings/{id} - Delete a listing (soft delete)
///
/// Requires authentication. Only the vendor who created the listing can delete it.
#[delete("/api/listings/{id}")]
pub async fn delete_listing(
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

    let listing_id = id.into_inner();

    // Check listing exists and user owns it
    let existing_listing = match Listing::find_by_id(&mut conn, listing_id.clone()) {
        Ok(listing) => listing,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Listing not found"
            }))
        }
    };

    if existing_listing.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only delete your own listings"
        }));
    }

    match Listing::delete(&mut conn, listing_id) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete listing: {}", e)
        })),
    }
}
