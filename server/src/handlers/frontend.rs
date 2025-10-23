//! Frontend handlers for template rendering
//!
//! Serves HTML pages using Tera templates with HTMX for dynamic interactions.

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};
use tracing::{error, info};

use crate::db::DbPool;
use crate::middleware::csrf::get_csrf_token;

/// GET / - Homepage (listings index)
pub async fn index(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Check if user is logged in
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Empty listings for now (will be populated by DB query later)
    ctx.insert("listings", &Vec::<String>::new());

    match tera.render("listings/index.html", &ctx) {
        Ok(html) => {
            info!("Rendered homepage");
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering homepage: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /login - Login page
pub async fn show_login(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Redirect if already logged in
    if let Ok(Some(_username)) = session.get::<String>("username") {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut ctx = Context::new();

    // Add CSRF token to template context
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("auth/login.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering login: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /register - Registration page
pub async fn show_register(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Redirect if already logged in
    if let Ok(Some(_username)) = session.get::<String>("username") {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut ctx = Context::new();

    // Add CSRF token to template context
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("auth/register.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering register: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// POST /logout - Logout user
pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    info!("User logged out");

    HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish()
}

/// GET /listings - Listings index page
pub async fn show_listings(
    tera: web::Data<Tera>,
    _pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    let mut ctx = Context::new();

    // Check auth
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Empty listings vector - listings functionality implemented in Milestone 2.1
    ctx.insert("listings", &Vec::<String>::new());

    match tera.render("listings/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering listings: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /listings/{id} - Listing detail page
pub async fn show_listing(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    listing_id: web::Path<String>,
) -> impl Responder {
    use crate::models::listing::Listing;

    let mut ctx = Context::new();

    // Check auth
    let _user_id = if let Ok(Some(uid)) = session.get::<String>("user_id") {
        ctx.insert("user_id", &uid);
        ctx.insert("logged_in", &true);

        if let Ok(Some(username)) = session.get::<String>("username") {
            ctx.insert("username", &username);
        }
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
        Some(uid)
    } else {
        ctx.insert("logged_in", &false);
        None
    };

    // Fetch listing from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let listing_id_str = listing_id.into_inner();
    let listing_result = web::block(move || Listing::find_by_id(&mut conn, listing_id_str)).await;

    let listing = match listing_result {
        Ok(Ok(l)) => l,
        Ok(Err(e)) => {
            error!("Listing not found: {}", e);
            return HttpResponse::NotFound().body("Listing not found");
        }
        Err(e) => {
            error!("Database query error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    ctx.insert("listing", &listing);

    match tera.render("listings/show.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering listing detail: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /listings/new - Create listing page (vendor only)
pub async fn show_create_listing(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Check auth and role
    if let Ok(Some(role)) = session.get::<String>("role") {
        if role != "vendor" {
            return HttpResponse::Forbidden().body("Only vendors can create listings");
        }
    } else {
        return HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish();
    }

    let mut ctx = Context::new();

    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
        ctx.insert("role", &"vendor");
    }

    match tera.render("listings/create.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering create listing: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /orders - Orders index page
pub async fn show_orders(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    use crate::models::order::Order;

    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    }

    // Fetch orders from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let orders_result = web::block(move || Order::find_by_user(&mut conn, user_id)).await;

    let orders = match orders_result {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            error!("Error fetching orders: {}", e);
            Vec::new()
        }
        Err(e) => {
            error!("Database query error: {}", e);
            Vec::new()
        }
    };

    ctx.insert("orders", &orders);

    match tera.render("orders/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering orders: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /orders/{id} - Order detail page
pub async fn show_order(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    order_id: web::Path<String>,
) -> impl Responder {
    use crate::models::order::Order;

    // Require authentication
    let _user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    }

    // Fetch order from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let order_id_str = order_id.into_inner();
    let order_result = web::block(move || Order::find_by_id(&mut conn, order_id_str)).await;

    let order = match order_result {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            error!("Order not found: {}", e);
            return HttpResponse::NotFound().body("Order not found");
        }
        Err(e) => {
            error!("Database query error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    ctx.insert("order", &order);

    match tera.render("orders/show.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering order detail: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /escrow/{id} - Escrow detail page
pub async fn show_escrow(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    escrow_id: web::Path<String>,
) -> impl Responder {
    use crate::models::escrow::Escrow;

    // Require authentication
    let _user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    }

    // Fetch escrow from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let escrow_id_str = escrow_id.into_inner();
    let escrow_result = web::block(move || Escrow::find_by_id(&mut conn, escrow_id_str)).await;

    let escrow = match escrow_result {
        Ok(Ok(e)) => e,
        Ok(Err(e)) => {
            error!("Escrow not found: {}", e);
            return HttpResponse::NotFound().body("Escrow not found");
        }
        Err(e) => {
            error!("Database query error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    ctx.insert("escrow", &escrow);

    match tera.render("escrow/show.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering escrow detail: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

// ============================================================================
// Reputation Frontend Handlers
// ============================================================================

/// GET /vendor/{vendor_id} - Vendor profile page with reputation display
///
/// Displays vendor reputation with WASM-verified reviews, rating distribution,
/// and IPFS export functionality.
pub async fn vendor_profile(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    vendor_id: web::Path<String>,
) -> impl Responder {
    use crate::db::reputation::db_get_vendor_reviews;
    use reputation_common::types::VendorReputation;
    use uuid::Uuid;

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(user_id)) = session.get::<String>("user_id") {
            ctx.insert("user_id", &user_id);
        }

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Validate vendor_id format
    let vendor_uuid = match Uuid::parse_str(&vendor_id.into_inner()) {
        Ok(uuid) => uuid,
        Err(_) => {
            error!("Invalid vendor ID format");
            return HttpResponse::BadRequest().body("Invalid vendor ID");
        }
    };

    // Fetch vendor reviews from database
    let reviews = match db_get_vendor_reviews(&pool, vendor_uuid).await {
        Ok(r) => r,
        Err(e) => {
            error!("Database error fetching reviews: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    // Calculate stats from reviews using reputation-crypto
    use reputation_crypto::reputation::calculate_stats;
    let stats = calculate_stats(&reviews);

    // Build VendorReputation structure
    let reputation = VendorReputation {
        format_version: "1.0".to_string(),
        vendor_pubkey: vendor_uuid.to_string(),
        generated_at: chrono::Utc::now(),
        reviews,
        stats,
    };

    // Serialize reputation to JSON for WASM verification
    let reputation_json = match serde_json::to_string(&reputation) {
        Ok(json) => json,
        Err(e) => {
            error!("JSON serialization error: {}", e);
            return HttpResponse::InternalServerError().body("Serialization error");
        }
    };

    ctx.insert("vendor_id", &vendor_uuid.to_string());
    ctx.insert("reputation", &reputation);
    ctx.insert("reputation_json", &reputation_json);

    // Check if current user is the vendor (for IPFS export button)
    if let Ok(Some(user_id)) = session.get::<String>("user_id") {
        ctx.insert("is_vendor", &(user_id == vendor_uuid.to_string()));
    } else {
        ctx.insert("is_vendor", &false);
    }

    match tera.render("reputation/vendor_profile.html", &ctx) {
        Ok(html) => {
            info!("Rendered vendor profile for {}", vendor_uuid);
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering vendor profile: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /review/submit - Review submission form
///
/// Displays form for submitting a cryptographically-signed review.
/// Requires authentication.
pub async fn submit_review_form(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    // Check authentication
    if session.get::<String>("username").unwrap_or(None).is_none() {
        return HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish();
    }

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    }

    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("reputation/submit_review.html", &ctx) {
        Ok(html) => {
            info!("Rendered review submission form");
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering submit review form: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
