//! Listing API handlers
//!
//! REST API endpoints for managing product listings on the marketplace.

use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use anyhow::Context;
use diesel::prelude::*;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use infer;

use crate::db::DbPool;
use crate::ipfs::client::IpfsClient;
use crate::models::listing::{Listing, ListingStatus, NewListing, UpdateListing};
use crate::models::order::Order;
use crate::schema::{listings, orders};
use chrono::{Datelike, Timelike, Utc};

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

    #[validate(length(min = 2, max = 50, message = "Category must be between 2-50 characters"))]
    pub category: String,
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

    #[validate(length(min = 2, max = 50))]
    pub category: Option<String>,
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
    pub images: Vec<String>, // IPFS CIDs for images
    pub category: String,
}

impl From<Listing> for ListingResponse {
    fn from(listing: Listing) -> Self {
        // Parse images from JSON string
        let images = listing.images_ipfs_cids
            .as_ref()
            .and_then(|json| serde_json::from_str::<Vec<String>>(json).ok())
            .unwrap_or_default();

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
            images,
            category: listing.category.clone(),
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

/// Upload images to IPFS and return CIDs
///
/// # Arguments
/// * `multipart` - Multipart form data containing image files
/// * `ipfs_client` - IPFS client instance
///
/// # Returns
/// Vector of IPFS CIDs for uploaded images
async fn upload_images_to_ipfs(
    mut multipart: Multipart,
    ipfs_client: &IpfsClient,
) -> Result<Vec<String>, HttpResponse> {
    let mut image_cids = Vec::new();
    let mut image_count = 0;
    const MAX_IMAGES: usize = 10;
    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB per image

    while let Some(item) = multipart.try_next().await.map_err(|e| {
        tracing::error!("Multipart parsing error: {}", e);
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid multipart data"
        }))
    })? {
        if image_count >= MAX_IMAGES {
            return Err(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Maximum {} images allowed", MAX_IMAGES)
            })));
        }

        let field = item;
        if field.name() == "images" {
            let mut data = Vec::new();
            let mut stream = field;

            while let Some(chunk) = stream.try_next().await.map_err(|e| {
                tracing::error!("Stream reading error: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to read file data"
                }))
            })? {
                data.extend_from_slice(&chunk);
                
                // Check file size limit
                if data.len() > MAX_FILE_SIZE {
                    return Err(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": format!("File too large. Maximum size: {}MB", MAX_FILE_SIZE / 1024 / 1024)
                    })));
                }
            }

            if !data.is_empty() {
                // Determine mime type and generate a filename
                let kind = infer::get(&data).ok_or_else(|| {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Failed to determine image type"
                    }))
                })?;
                
                if !["image/jpeg", "image/png", "image/gif"].contains(&kind.mime_type()) {
                    return Err(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid image format. Only JPEG, PNG, and GIF are supported"
                    })));
                }

                let file_name = format!("{}.{}", Uuid::new_v4(), kind.extension());

                // Upload to IPFS
                match ipfs_client.add(data, &file_name, kind.mime_type()).await {
                    Ok(cid) => {
                        tracing::info!("Image uploaded to IPFS: {}", cid);
                        image_cids.push(cid);
                        image_count += 1;
                    }
                    Err(e) => {
                        tracing::error!("IPFS upload failed: {}", e);
                        return Err(HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to upload image to IPFS"
                        })));
                    }
                }
            }
        }
    }

    Ok(image_cids)
}



/// POST /api/listings - Create a new listing (JSON)
///
/// Requires authentication and vendor role.
#[post("/listings")]
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
        images_ipfs_cids: Some("[]".to_string()), // Default to empty JSON array
        category: req.category.clone(),
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let listing_result = web::block(move || Listing::create(&mut conn, new_listing)).await;

    match listing_result {
        Ok(Ok(listing)) => {
            // Redirect to the listing page after creation (HTMX will follow this)
            HttpResponse::Created()
                .insert_header(("HX-Redirect", format!("/listings/{}", listing.id)))
                .json(ListingResponse::from(listing))
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create listing: {}", e)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Async task failed: {}", e)
        })),
    }
}

/// POST /api/listings/with-images - Create a new listing with images (multipart)
///
/// Requires authentication and vendor role.
#[post("/listings/with-images")]
pub async fn create_listing_with_images(
    pool: web::Data<DbPool>,
    session: Session,
    mut multipart: Multipart,
    ipfs_client: web::Data<IpfsClient>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    // Parse multipart form data
    let mut title = String::new();
    let mut description = String::new();
    let mut price_xmr: i64 = 0;
    let mut stock: i32 = 0;
    let mut category = String::from("other"); // Default category
    let mut image_files = Vec::new();

    while let Some(item) = multipart.try_next().await.map_err(|e| {
        tracing::error!("Multipart parsing error: {}", e);
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid multipart data"
        }))
    }).transpose() {
        match item {
            Ok(field) => {
                let field_name = field.name().to_string();
                
                if field_name == "images" {
                    // Collect image data
                    let mut data = Vec::new();
                    let mut stream = field;
                    while let Some(chunk) = stream.try_next().await.map_err(|e| {
                        tracing::error!("Stream reading error: {}", e);
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to read file data"
                        }))
                    }).transpose() {
                        if let Ok(chunk) = chunk {
                            data.extend_from_slice(&chunk);
                        }
                    }
                    if !data.is_empty() {
                        image_files.push(data);
                    }
                } else {
                    // Parse text fields
                    let mut stream = field;
                    let mut data = Vec::new();
                    while let Some(chunk) = stream.try_next().await.map_err(|_| {
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to read field data"
                        }))
                    }).transpose() {
                        if let Ok(chunk) = chunk {
                            data.extend_from_slice(&chunk);
                        }
                    }
                    let value = String::from_utf8_lossy(&data).to_string();
                    
                    match field_name.as_str() {
                        "title" => title = value,
                        "description" => description = value,
                        "price_xmr" => price_xmr = value.parse().unwrap_or(0),
                        "stock" => stock = value.parse().unwrap_or(0),
                        "category" => category = value,
                        _ => {}
                    }
                }
            }
            Err(e) => return e,
        }
    }

    // Validate required fields
    if title.len() < 3 || title.len() > 200 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Title must be between 3-200 characters"
        }));
    }
    if description.len() < 10 || description.len() > 5000 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Description must be between 10-5000 characters"
        }));
    }
    if price_xmr < 1 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Price must be positive"
        }));
    }

    // Upload images to IPFS
    let mut image_cids = Vec::new();
    for (idx, image_data) in image_files.iter().enumerate() {
        let kind = match infer::get(image_data) {
            Some(k) if ["image/jpeg", "image/png", "image/gif"].contains(&k.mime_type()) => k,
            _ => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid image format. Only JPEG, PNG, and GIF are supported"
                }));
            }
        };

        let file_name = format!("{}.{}", Uuid::new_v4(), kind.extension());
        match ipfs_client.add(image_data.clone(), &file_name, kind.mime_type()).await {
            Ok(cid) => {
                tracing::info!("Image {} uploaded to IPFS: {}", idx + 1, cid);
                image_cids.push(cid);
            }
            Err(e) => {
                tracing::error!("IPFS upload failed: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to upload image to IPFS"
                }));
            }
        }
    }

    let images_json = serde_json::to_string(&image_cids).unwrap_or_else(|_| "[]".to_string());

    // Create listing
    let new_listing = NewListing {
        id: Uuid::new_v4().to_string(),
        vendor_id: user_id,
        title,
        description,
        price_xmr,
        stock,
        status: ListingStatus::Active.as_str().to_string(),
        images_ipfs_cids: Some(images_json),
        category,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let listing_result = web::block(move || Listing::create(&mut conn, new_listing)).await;

    match listing_result {
        Ok(Ok(listing)) => {
            HttpResponse::Created()
                .insert_header(("HX-Redirect", format!("/listings/{}", listing.id)))
                .json(ListingResponse::from(listing))
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create listing: {}", e)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Async task failed: {}", e)
        })),
    }
}

/// POST /api/listings/{id}/images - Upload images for a listing
///
/// Requires authentication. Only the vendor who created the listing can upload images.
#[post("/listings/{id}/images")]
pub async fn upload_listing_images(
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
    multipart: Multipart,
    ipfs_client: web::Data<IpfsClient>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let listing_id = id.into_inner();



    // Check listing ownership
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let listing_id_clone = listing_id.clone();
    let existing_listing = match web::block(move || Listing::find_by_id(&mut conn, listing_id_clone)).await {
        Ok(Ok(listing)) => listing,
        Ok(Err(_)) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Listing not found"
            }))
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }))
        }
    };

    // Check ownership
    if existing_listing.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only upload images to your own listings"
        }));
    }

    // Upload images to IPFS
    let image_cids = match upload_images_to_ipfs(multipart, &ipfs_client).await {
        Ok(cids) => cids,
        Err(response) => return response,
    };

    // Update listing with new images
    let existing_images: Vec<String> = existing_listing.images_ipfs_cids
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();

    let mut updated_images = existing_images;
    updated_images.extend(image_cids);

    let images_json = serde_json::to_string(&updated_images).unwrap_or_else(|_| "[]".to_string());

    let update_result = web::block(move || {
        let mut conn = pool.get().with_context(|| "Database connection failed")?;
        diesel::update(listings::table.filter(listings::id.eq(listing_id)))
            .set(listings::images_ipfs_cids.eq(images_json))
            .execute(&mut conn)
            .context("Failed to update listing images")?;
        Ok::<(), anyhow::Error>(())
    }).await;

    match update_result {
        Ok(Ok(_)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Images uploaded successfully",
                "image_count": updated_images.len()
            }))
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update listing: {}", e)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Async task failed: {}", e)
        })),
    }
}

/// GET /api/listings/{id}/images/{cid} - Get image from IPFS
///
/// Serves images from IPFS through the server to avoid CORS issues.
#[get("/listings/{id}/images/{cid}")]
pub async fn get_listing_image(
    path: web::Path<(String, String)>,
    ipfs_client: web::Data<IpfsClient>,
) -> impl Responder {
    let (_listing_id, image_cid) = path.into_inner();



    // Download image from IPFS
    match ipfs_client.cat(&image_cid).await {
        Ok(image_data) => {
            // Determine content type based on image data
            let content_type = if image_data.len() >= 4 {
                if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    "image/jpeg"
                } else if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png"
                } else if image_data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
                    "image/gif"
                } else {
                    "application/octet-stream"
                }
            } else {
                "application/octet-stream"
            };

            HttpResponse::Ok()
                .content_type(content_type)
                .body(image_data)
        }
        Err(e) => {
            tracing::error!("Failed to retrieve image from IPFS: {:?}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Image not found"
            }))
        }
    }
}

/// DELETE /api/listings/{id}/images/{cid} - Remove an image from a listing
///
/// Requires authentication. Only the vendor who created the listing can remove images.
#[delete("/listings/{id}/images/{cid}")]
pub async fn remove_listing_image(
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<(String, String)>,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let (listing_id, image_cid) = path.into_inner();

    // Check listing ownership and remove image
    let remove_result = web::block(move || {
        let mut conn = pool.get().with_context(|| "Database connection failed")?;

        // 1. Find the listing to check ownership
        let existing_listing = Listing::find_by_id(&mut conn, listing_id.clone())?;

        // 2. Check if the authenticated user is the vendor
        if existing_listing.vendor_id != user_id {
            return Err(anyhow::anyhow!("Permission denied"));
        }

        // 3. Parse existing images
        let mut existing_images: Vec<String> = existing_listing.images_ipfs_cids
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default();

        // 4. Remove the specified image CID
        existing_images.retain(|cid| cid != &image_cid);

        // 5. Update the listing with the new image list
        let images_json = serde_json::to_string(&existing_images).unwrap_or_else(|_| "[]".to_string());
        
        diesel::update(listings::table.filter(listings::id.eq(listing_id)))
            .set(listings::images_ipfs_cids.eq(images_json))
            .execute(&mut conn)
            .context("Failed to update listing images")?;

        Ok::<(), anyhow::Error>(())
    }).await;

    match remove_result {
        Ok(Ok(_)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Image removed successfully"
            }))
        }
        Ok(Err(e)) => {
            if e.to_string().contains("Permission denied") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only remove images from your own listings"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to remove image: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Async task failed: {}", e)
        })),
    }
}

/// GET /api/listings - List all active listings (paginated)
#[get("/listings")]
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

    let listings_result = web::block(move || Listing::list_active(&mut conn, limit, offset)).await;

    match listings_result {
        Ok(Ok(listings)) => {
            let responses: Vec<ListingResponse> = listings.into_iter().map(ListingResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to list listings: {}", e)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Async task failed: {}", e)
        })),
    }
}

/// GET /api/listings/{id} - Get a single listing by ID
#[get("/listings/{id}")]
pub async fn get_listing(pool: web::Data<DbPool>, id: web::Path<String>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }))
        }
    };

    let listing_result = web::block(move || Listing::find_by_id(&mut conn, id.into_inner())).await;

    match listing_result {
        Ok(Ok(listing)) => HttpResponse::Ok().json(ListingResponse::from(listing)),
        Ok(Err(_)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Listing not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Async task failed"
        })),
    }
}

/// GET /api/listings/vendor/{vendor_id} - Get all listings by a vendor
#[get("/listings/vendor/{vendor_id}")]
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

    let listings_result = web::block(move || Listing::find_by_vendor(&mut conn, vendor_id.into_inner())).await;

    match listings_result {
        Ok(Ok(listings)) => {
            let responses: Vec<ListingResponse> =
                listings.into_iter().map(ListingResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to get vendor listings: {}", e)
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Async task failed"
        })),
    }
}

/// GET /api/listings/search?q=query - Search listings by title
#[get("/listings/search")]
pub async fn search_listings(
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let search_query = match query.get("q") {
        Some(q) if !q.is_empty() => q.clone(),
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

    let search_pattern = format!("%{}%", search_query);
    let listings_result = web::block(move || Listing::search_by_title(&mut conn, &search_pattern, limit)).await;

    match listings_result {
        Ok(Ok(listings)) => {
            let responses: Vec<ListingResponse> = listings.into_iter().map(ListingResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Search failed: {}", e)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Async task failed: {}", e)
        })),
    }
}

/// PUT /api/listings/{id} - Update a listing
///
/// Requires authentication. Only the vendor who created the listing can update it.
#[put("/listings/{id}")]
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

    let listing_id = id.into_inner();

    // Build update data from request
    let update_data = UpdateListing {
        title: req.title.clone(),
        description: req.description.clone(),
        price_xmr: req.price_xmr,
        stock: req.stock,
        status: req.status.clone(),
        category: req.category.clone(),
    };

            let update_result = web::block(move || {
                let mut conn = pool.get().with_context(|| "Database connection failed")?;
            // 1. Find the listing to check ownership
        let existing_listing = Listing::find_by_id(&mut conn, listing_id.clone())?;

        // 2. Check if the authenticated user is the vendor
        if existing_listing.vendor_id != user_id {
            return Err(anyhow::anyhow!("Permission denied"));
        }

        // 3. Perform the update
            Listing::update(&mut conn, listing_id, update_data)
        })
        .await;

        match update_result {
            Ok(Ok(listing)) => HttpResponse::Ok()
                .insert_header(("HX-Redirect", format!("/listings/{}", listing.id)))
                .json(ListingResponse::from(listing)),
            Ok(Err(e)) => {
            if e.to_string().contains("Permission denied") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only update your own listings"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to update listing: {}", e)
                }))
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An unexpected async task error occurred"
        })),
    }
}

/// DELETE /api/listings/{id} - Delete a listing (soft delete)
///
/// Requires authentication. Only the vendor who created the listing can delete it.
#[delete("/listings/{id}")]
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

    let listing_id = id.into_inner();

    let delete_result = web::block(move || {
        let mut conn = pool.get().with_context(|| "Database connection failed")?;

        // 1. Find the listing to check ownership
        let existing_listing = Listing::find_by_id(&mut conn, listing_id.clone())?;

        // 2. Check if the authenticated user is the vendor
        if existing_listing.vendor_id != user_id {
            return Err(anyhow::anyhow!("Permission denied"));
        }

        // 3. Perform the delete (soft delete)
        Listing::delete(&mut conn, listing_id)
    })
    .await;

    match delete_result {
        Ok(Ok(_)) => HttpResponse::NoContent().finish(),
        Ok(Err(e)) => {
            if e.to_string().contains("Permission denied") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only delete your own listings"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to delete listing: {}", e)
                }))
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An unexpected async task error occurred"
        })),
    }
}

/// Response body for vendor dashboard stats
#[derive(Debug, Serialize)]
pub struct VendorDashboardStats {
    pub active_listings: i64,
    pub pending_orders: i64,
    pub total_revenue_xmr: String,
    pub revenue_this_month_xmr: String,
    pub total_sales: i64,
    pub sales_this_month: i64,
}

/// GET /api/vendor/dashboard/stats - Get vendor dashboard statistics
///
/// Requires authentication. Returns statistics about vendor's listings and orders.
#[get("/vendor/dashboard/stats")]
pub async fn get_vendor_dashboard_stats(
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let stats_result = web::block(move || -> anyhow::Result<VendorDashboardStats> {
        let mut conn = pool.get().with_context(|| "Database connection failed")?;

        // Count active listings
        let active_listings = listings::table
            .filter(listings::vendor_id.eq(&user_id))
            .filter(listings::status.eq("active"))
            .count()
            .get_result::<i64>(&mut conn)
            .context("Failed to count active listings")?;

        // Count pending orders (pending or funded status)
        let pending_orders = orders::table
            .filter(orders::vendor_id.eq(&user_id))
            .filter(
                orders::status
                    .eq("pending")
                    .or(orders::status.eq("funded"))
                    .or(orders::status.eq("shipped")),
            )
            .count()
            .get_result::<i64>(&mut conn)
            .context("Failed to count pending orders")?;

        // Calculate total revenue from completed orders
        let completed_orders: Vec<Order> = orders::table
            .filter(orders::vendor_id.eq(&user_id))
            .filter(orders::status.eq("completed"))
            .load::<Order>(&mut conn)
            .context("Failed to load completed orders")?;

        let total_revenue_atomic: i64 = completed_orders.iter().map(|o| o.total_xmr).sum();
        let total_revenue_xmr = format!("{:.12}", total_revenue_atomic as f64 / 1_000_000_000_000.0);

        // Calculate total sales count
        let total_sales = completed_orders.len() as i64;

        // Calculate this month's revenue and sales
        let now = Utc::now();
        let month_start = now
            .with_day(1)
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .ok_or_else(|| anyhow::anyhow!("Failed to calculate month start"))?
            .naive_utc();

        let this_month_orders: Vec<Order> = orders::table
            .filter(orders::vendor_id.eq(&user_id))
            .filter(orders::status.eq("completed"))
            .filter(orders::created_at.ge(month_start))
            .load::<Order>(&mut conn)
            .context("Failed to load this month's orders")?;

        let revenue_this_month_atomic: i64 = this_month_orders.iter().map(|o| o.total_xmr).sum();
        let revenue_this_month_xmr =
            format!("{:.12}", revenue_this_month_atomic as f64 / 1_000_000_000_000.0);

        let sales_this_month = this_month_orders.len() as i64;

        Ok(VendorDashboardStats {
            active_listings,
            pending_orders,
            total_revenue_xmr,
            revenue_this_month_xmr,
            total_sales,
            sales_this_month,
        })
    })
    .await;

    match stats_result {
        Ok(Ok(stats)) => HttpResponse::Ok().json(stats),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to retrieve dashboard stats: {}", e)
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An unexpected async task error occurred"
        })),
    }
}
