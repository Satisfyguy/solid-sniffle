//! Frontend handlers for template rendering
//!
//! Serves HTML pages using Tera templates with HTMX for dynamic interactions.

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tera::{Context, Tera};
use tracing::{error, info};

use crate::db::DbPool;
use crate::middleware::csrf::get_csrf_token;
use crate::models::user::{NewUser, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use uuid::Uuid;


/// GET /new-home - New V2 Homepage
pub async fn new_index(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Add session data to context
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        } else {
            ctx.insert("role", &"buyer");
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("v2_index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error rendering new_index: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET / - Homepage
pub async fn index(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Add session data to context
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        } else {
            ctx.insert("role", &"buyer");
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
        match tera.render("index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error rendering index: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /auth - Authentication page (Login & Register)
pub async fn show_auth_page(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Redirect if already logged in
    if session.get::<String>("username").unwrap_or(None).is_some() {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut ctx = Context::new();

    // Add CSRF token to template context
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("auth/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering auth page: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// POST /auth/login - Handle user login
pub async fn handle_login(
    session: Session,
    pool: web::Data<DbPool>,
    form: web::Form<LoginForm>,
) -> impl Responder {
    info!("Login attempt for user: {}", form.username);

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection from pool: {}", e);
            return HttpResponse::InternalServerError().body("Database error.");
        }
    };

    let username = form.username.clone();
    let password = form.password.clone();

    // Find user by username in a blocking thread
    let user_result = web::block(move || User::find_by_username(&mut conn, &username)).await;

    let user = match user_result {
        Ok(Ok(user)) => user,
        _ => {
            // User not found or DB error
            info!("Login failed for user: {}. User not found or DB error.", &form.username);
            // Redirect back to auth page, maybe with an error query param in a real app
            return HttpResponse::Found()
                .append_header(("Location", "/auth?error=invalid_credentials"))
                .finish();
        }
    };

    // Verify password in a blocking thread
    let password_hash = user.password_hash.clone();
    let password_verification = web::block(move || {
        let parsed_hash = argon2::PasswordHash::new(&password_hash)
            .map_err(|_| "Failed to parse password hash".to_string())?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| "Password verification failed".to_string())
    }).await;

    match password_verification {
        Ok(Ok(())) => {
            // Password is correct, create session
            if session.insert("user_id", user.id).is_err()
                || session.insert("username", user.username.clone()).is_err()
                || session.insert("role", user.role).is_err()
            {
                error!("Failed to create session for user: {}", user.username);
                return HttpResponse::InternalServerError().body("Failed to create session.");
            }

            info!("User {} logged in successfully.", user.username);
            HttpResponse::Found()
                .append_header(("Location", "/profile"))
                .finish()
        }
        _ => {
            // Password incorrect or verification error
            info!("Login failed for user: {}. Incorrect password.", user.username);
            HttpResponse::Found()
                .append_header(("Location", "/auth?error=invalid_credentials"))
                .finish()
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    password_confirm: String,
    role: String,
}

/// POST /auth/register - Handle user registration
pub async fn handle_register(
    session: Session,
    pool: web::Data<DbPool>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    info!("Registration attempt for user: {} with role: {}", form.username, form.role);

    // 1. Validate passwords match
    if form.password != form.password_confirm {
        // In a real app, you'd redirect back with an error message
        return HttpResponse::BadRequest().body("Passwords do not match.");
    }

    // Clone username and role before moving `form` into the closure
    let username_for_new_user = form.username.clone();
    let role_for_new_user = form.role.clone();

    // 2. Hash the password
    let password_hash = match web::block(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2.hash_password(form.password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| e.to_string())
    }).await {
        Ok(Ok(hash)) => hash,
        _ => {
            error!("Failed to hash password during registration");
            return HttpResponse::InternalServerError().body("Error processing registration.");
        }
    };

    // 3. Create the new user
    let new_user = NewUser {
        id: Uuid::new_v4().to_string(),
        username: username_for_new_user,
        password_hash,
        wallet_address: None,
        wallet_id: None,
        role: role_for_new_user,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection from pool: {}", e);
            return HttpResponse::InternalServerError().body("Database error.");
        }
    };

    // Check if username already exists
    let username_clone = new_user.username.clone();
    let user_exists = web::block(move || User::find_by_username(&mut conn, &username_clone)).await;

    if let Ok(Ok(_)) = user_exists {
        return HttpResponse::BadRequest().body("Username already taken.");
    }

    // Re-acquire connection as it was moved
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection from pool: {}", e);
            return HttpResponse::InternalServerError().body("Database error.");
        }
    };

    let user = match web::block(move || User::create(&mut conn, new_user)).await {
        Ok(Ok(user)) => user,
        Ok(Err(e)) => {
            error!("Failed to create user in database: {}", e);
            return HttpResponse::InternalServerError().body("Failed to create user.");
        }
        Err(_) => {
            error!("A web::block error occurred during user creation.");
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
    };

    // 4. Create a session for the new user
    if session.insert("user_id", user.id).is_err() 
        || session.insert("username", user.username).is_err() 
        || session.insert("role", user.role).is_err() {
        error!("Failed to create session for new user");
        return HttpResponse::InternalServerError().body("Failed to create session.");
    }

    info!("User successfully registered and session created.");

    // 5. Redirect to profile page
    HttpResponse::Found()
        .append_header(("Location", "/profile"))
        .finish()
}

/// POST /logout - Logout user
pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    info!("User logged out");

    HttpResponse::Found()
        .append_header(("Location", "/auth"))
        .finish()
}



#[derive(serde::Serialize)]
struct ListingForTemplate {
    id: String,
    title: String,
    description: String, // New
    price: String, // New
    vendor: String, // New
    rating: f32, // New
    sales: i32, // New
    category: String, // New
    first_image_cid: Option<String>,
}

/// GET /listings - Listings index page
pub async fn show_listings(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    use crate::models::listing::Listing;
    let mut ctx = Context::new();

    // Check auth
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role); // For base template user menu
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for logout form in navigation
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Fetch listings from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let listings_result = web::block(move || Listing::list_active(&mut conn, 20, 0)).await;

    let listings = match listings_result {
        Ok(Ok(l)) => l,
        Ok(Err(e)) => {
            error!("Error fetching listings: {}", e);
            Vec::new()
        }
        Err(e) => {
            error!("Database query error: {}", e);
            Vec::new()
        }
    };

    let mut listings_for_template = Vec::new();
    for listing in listings {
        let mut conn2 = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                error!("Database connection error: {}", e);
                return HttpResponse::InternalServerError().body("Database error");
            }
        };
        let vendor_id = listing.vendor_id.clone();
        let vendor_result = web::block(move || User::find_by_id(&mut conn2, vendor_id)).await;
        let vendor_username = match vendor_result {
            Ok(Ok(v)) => v.username,
            _ => "Unknown".to_string(),
        };

        // Parse first image CID from JSON
        let first_image_cid = listing.images_ipfs_cids
            .as_ref()
            .and_then(|json| serde_json::from_str::<Vec<String>>(json).ok())
            .and_then(|images| images.into_iter().next());

        listings_for_template.push(ListingForTemplate {
            id: listing.id,
            title: listing.title,
            description: "High-quality digital asset with complete anonymity".to_string(), // Mock
            price: format!("{:.4} XMR", listing.price_xmr as f64 / 1_000_000_000_000.0),
            vendor: vendor_username,
            rating: 4.8, // Mock
            sales: 142, // Mock
            category: "Digital".to_string(), // Mock
            first_image_cid,
        });
    }

    ctx.insert("listings", &listings_for_template);


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
    use crate::models::user::User;

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
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
        Some(uid)
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
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

    // Fetch vendor information
    let mut conn2 = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };
    let vendor_id_str = listing.vendor_id.clone();
    let vendor_result = web::block(move || User::find_by_id(&mut conn2, vendor_id_str)).await;
    let vendor = match vendor_result {
        Ok(Ok(v)) => v,
        _ => {
            error!("Failed to fetch vendor for listing {}", listing.id);
            return HttpResponse::InternalServerError().body("Could not load vendor data");
        }
    };

    info!("Rendering listing: {:?}", listing);
    info!("With vendor: {:?}", vendor);

    // Parse images from JSON string
    let images: Vec<String> = listing.images_ipfs_cids
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();

    ctx.insert("listing", &listing);
    ctx.insert("vendor", &vendor);
    ctx.insert("price_display", &listing.price_as_xmr());
    ctx.insert("images", &images);
    
    // Add CSRF token for order creation
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

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
            .append_header(("Location", "/auth"))
            .finish();
    }

    let mut ctx = Context::new();

    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);
        ctx.insert("role", &"vendor");
        ctx.insert("user_role", &"vendor");
        ctx.insert("is_vendor", &true);
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for form submission and logout
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

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

/// GET /listings/{id}/edit - Edit listing page (vendor only)
pub async fn show_edit_listing(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    listing_id: web::Path<String>,
) -> impl Responder {
    use crate::models::listing::Listing;

    let mut ctx = Context::new();

    // Check auth and role
    let user_id = if let Ok(Some(uid)) = session.get::<String>("user_id") {
        ctx.insert("logged_in", &true);
        if let Ok(Some(username)) = session.get::<String>("username") {
            ctx.insert("username", &username);
            ctx.insert("user_name", &username);
        }
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
            if role != "vendor" {
                return HttpResponse::Forbidden().body("Only vendors can edit listings");
            }
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
        uid
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
        return HttpResponse::Found()
            .append_header(("Location", "/auth"))
            .finish();
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

    // Check if authenticated user is the vendor of this listing
    if listing.vendor_id != user_id {
        return HttpResponse::Forbidden().body("You can only edit your own listings");
    }

    ctx.insert("listing", &listing);

    // Add CSRF token for form submission
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("listings/edit.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering edit listing: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /vendor/listings - Vendor's listings management page (vendor only)
///
/// Displays all listings created by the authenticated vendor with management options.
/// Provides quick stats, filters, and actions (edit, delete, toggle active status).
///
/// # Authentication
/// - Requires active session with vendor role
/// - Redirects to /auth if not authenticated
/// - Returns 403 Forbidden if user is not a vendor
///
/// # Returns
/// - 200 OK: HTML page with vendor's listings
/// - 302 Found: Redirect to /auth (not authenticated)
/// - 403 Forbidden: User is not a vendor
/// - 500 Internal Server Error: Database or template error
pub async fn show_vendor_listings(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    use crate::models::listing::Listing;

    // Check authentication and vendor role
    let vendor_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/auth"))
                .finish()
        }
    };

    // Verify vendor role
    if let Ok(Some(role)) = session.get::<String>("role") {
        if role != "vendor" {
            return HttpResponse::Forbidden()
                .body("Access denied: Vendor role required");
        }
    } else {
        return HttpResponse::Found()
            .append_header(("Location", "/auth"))
            .finish();
    }

    let mut ctx = Context::new();

    // Insert session data (required by base-marketplace.html)
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username);
        ctx.insert("logged_in", &true);
        ctx.insert("role", &"vendor");
        ctx.insert("user_role", &"vendor");
        ctx.insert("is_vendor", &true);
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for actions
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Fetch vendor's listings from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError()
                .body("Database connection error");
        }
    };

    let vendor_id_clone = vendor_id.clone();
    let listings_result = web::block(move || {
        Listing::find_by_vendor(&mut conn, vendor_id_clone)
    })
    .await;

    let listings = match listings_result {
        Ok(Ok(l)) => l,
        Ok(Err(e)) => {
            error!("Error fetching vendor listings: {}", e);
            Vec::new()
        }
        Err(e) => {
            error!("Database query error: {}", e);
            Vec::new()
        }
    };

    // Calculate stats
    let total_listings = listings.len();
    let active_listings = listings.iter().filter(|l| l.status == "active").count();
    let inactive_listings = total_listings - active_listings;
    let total_stock: i32 = listings.iter().map(|l| l.stock).sum();

    ctx.insert("listings", &listings);
    ctx.insert("total_listings", &total_listings);
    ctx.insert("active_listings", &active_listings);
    ctx.insert("inactive_listings", &inactive_listings);
    ctx.insert("total_stock", &total_stock);

    // Render template
    match tera.render("vendor/listings.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering vendor listings: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Template error: {}", e))
        }
    }
}

/// GET /orders - Orders index page
pub async fn show_orders(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    use crate::models::listing::Listing;
    use crate::models::order::Order;
    use crate::models::user::User;

    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/auth"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    let user_role = if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
            Some(role)
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
            None
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
        None
    };

    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Fetch orders from database
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let user_id_clone = user_id.clone();
    let orders_result = web::block(move || Order::find_by_user(&mut conn, user_id_clone)).await;

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

    // Enrich orders with listing and user data
    let mut enriched_orders = Vec::new();
    let mut pending_count = 0;

    for order in orders {
        // Fetch listing
        let mut conn2 = match pool.get() {
            Ok(c) => c,
            Err(_) => continue,
        };
        let listing_id = order.listing_id.clone();
        let listing = match web::block(move || Listing::find_by_id(&mut conn2, listing_id)).await {
            Ok(Ok(l)) => l,
            _ => continue,
        };

        // Fetch other party (buyer or vendor)
        let mut conn3 = match pool.get() {
            Ok(c) => c,
            Err(_) => continue,
        };
        let other_user_id = if order.buyer_id == user_id {
            order.vendor_id.clone()
        } else {
            order.buyer_id.clone()
        };
        let other_username = match web::block(move || User::find_by_id(&mut conn3, other_user_id)).await {
            Ok(Ok(u)) => u.username,
            _ => "Unknown".to_string(),
        };

        // Count pending orders for vendor
        if user_role.as_deref() == Some("vendor") && order.vendor_id == user_id && order.status == "pending" {
            pending_count += 1;
        }

        // Get first image
        let first_image = listing.images_ipfs_cids
            .as_ref()
            .and_then(|json| serde_json::from_str::<Vec<String>>(json).ok())
            .and_then(|images| images.into_iter().next());

        enriched_orders.push(serde_json::json!({
            "id": order.id,
            "buyer_id": order.buyer_id,
            "vendor_id": order.vendor_id,
            "listing_id": order.listing_id,
            "listing_title": listing.title,
            "first_image_cid": first_image,
            "status": order.status,
            "total_xmr": order.total_xmr,
            "total_price_xmr": format!("{:.12}", order.total_xmr as f64 / 1_000_000_000_000.0),
            "unit_price_xmr": format!("{:.12}", listing.price_xmr as f64 / 1_000_000_000_000.0),
            "quantity": 1,
            "buyer_username": if order.buyer_id == user_id { "You".to_string() } else { other_username.clone() },
            "vendor_username": if order.vendor_id == user_id { "You".to_string() } else { other_username },
            "created_at": order.created_at,
        }));
    }

    ctx.insert("orders", &enriched_orders);
    ctx.insert("pending_count", &pending_count);
    ctx.insert("csrf_token", &get_csrf_token(&session));

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
    use crate::models::listing::Listing;
    use crate::models::order::Order;
    use crate::models::user::User;

    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/auth"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

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

    // Authorization: only buyer or vendor can view
    if order.buyer_id != user_id && order.vendor_id != user_id {
        return HttpResponse::Forbidden().body("You can only view your own orders");
    }

    // Fetch listing details
    let mut conn2 = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };
    let listing_id = order.listing_id.clone();
    let listing_result = web::block(move || Listing::find_by_id(&mut conn2, listing_id)).await;
    let listing = match listing_result {
        Ok(Ok(l)) => l,
        _ => {
            error!("Failed to fetch listing for order");
            return HttpResponse::InternalServerError().body("Failed to load order details");
        }
    };

    // Fetch buyer details
    let mut conn3 = match pool.get() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };
    let buyer_id = order.buyer_id.clone();
    let buyer_result = web::block(move || User::find_by_id(&mut conn3, buyer_id)).await;
    let buyer_username = match buyer_result {
        Ok(Ok(u)) => u.username,
        _ => "Unknown".to_string(),
    };

    // Fetch vendor details
    let mut conn4 = match pool.get() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };
    let vendor_id = order.vendor_id.clone();
    let vendor_result = web::block(move || User::find_by_id(&mut conn4, vendor_id)).await;
    let vendor_username = match vendor_result {
        Ok(Ok(u)) => u.username,
        _ => "Unknown".to_string(),
    };

    // Create enriched order data for template
    let order_data = serde_json::json!({
        "id": order.id,
        "buyer_id": order.buyer_id,
        "vendor_id": order.vendor_id,
        "listing_id": order.listing_id,
        "listing_title": listing.title,
        "escrow_id": order.escrow_id,
        "status": order.status,
        "total_xmr": order.total_xmr,
        "total_price_xmr": format!("{:.12}", order.total_xmr as f64 / 1_000_000_000_000.0),
        "unit_price_xmr": format!("{:.12}", listing.price_xmr as f64 / 1_000_000_000_000.0),
        "quantity": 1, // TODO: Add quantity field to Order model
        "buyer_username": buyer_username,
        "vendor_username": vendor_username,
        "created_at": order.created_at,
        "updated_at": order.updated_at,
        "funded_at": None::<String>,
        "shipped_at": None::<String>,
        "completed_at": None::<String>,
        "shipping_address": order.shipping_address,
        "shipping_notes": order.shipping_notes,
    });

    ctx.insert("order", &order_data);
    
    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

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
                .append_header(("Location", "/auth"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

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
            .append_header(("Location", "/auth"))
            .finish();
    }

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

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
// ============================================================================
// Settings Frontend Handlers (Non-Custodial Wallet)
// ============================================================================

/// GET /settings - Settings page
pub async fn show_settings(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    use crate::models::user::User;

    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/auth"))
                .finish()
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Fetch user from database to get wallet_address
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let user_result = web::block(move || User::find_by_id(&mut conn, user_id)).await;

    match user_result {
        Ok(Ok(user)) => {
            // Insert wallet_address if present
            if let Some(ref addr) = user.wallet_address {
                ctx.insert("wallet_address", addr);
            }
        }
        _ => {
            error!("Failed to fetch user for settings page");
        }
    }

    match tera.render("settings.html", &ctx) {
        Ok(html) => {
            info!("Rendered settings page");
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering settings: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /settings/wallet - Wallet settings page (non-custodial)
pub async fn show_wallet_settings(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Require authentication
    if session.get::<String>("username").unwrap_or(None).is_none() {
        return HttpResponse::Found()
            .append_header(("Location", "/auth"))
            .finish();
    }

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username); // For nav template
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", "buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("settings/wallet.html", &ctx) {
        Ok(html) => {
            info!("Rendered wallet settings page (non-custodial)");
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering wallet settings: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /docs/wallet-setup - Wallet setup documentation
pub async fn show_wallet_guide(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Insert session data (optional for docs)
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    match tera.render("docs/wallet-setup.html", &ctx) {
        Ok(html) => {
            info!("Rendered wallet setup guide");
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering wallet guide: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}


/// GET /cart - Shopping cart page
///
/// Displays the user's shopping cart with all items, quantities, and total.
/// Cart is stored in session, so no authentication required (guest carts allowed).
///
/// # Returns
/// - 200 OK: HTML cart page
/// - 500 Internal Server Error: Template error
pub async fn show_cart(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    use crate::models::cart::{Cart, CartItem};

    let mut ctx = Context::new();

    // Insert session data for base template
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", &"buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
        ctx.insert("user_role", &"guest");
    }

    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // Get cart from session
    let cart = match session.get::<Cart>("cart") {
        Ok(Some(c)) => c,
        _ => Cart::new(),
    };

    // Prepare cart items for template
    #[derive(serde::Serialize)]
    struct CartItemForTemplate {
        listing_id: String,
        title: String,
        vendor_username: String,
        price: String, // Formatted price
        quantity: i32,
        image_cid: Option<String>,
    }

    let mut cart_items_for_template = Vec::new();
    for item in &cart.items {
        cart_items_for_template.push(CartItemForTemplate {
            listing_id: item.listing_id.clone(),
            title: item.title.clone(),
            vendor_username: item.vendor_username.clone(),
            price: format!("{:.4} XMR", item.unit_price_xmr as f64 / 1_000_000_000_000.0),
            quantity: item.quantity,
            image_cid: item.image_cid.clone(),
        });
    }

    let total_xmr = cart.total_price_xmr();
    let escrow_fee_xmr = total_xmr * 0.03; // 3% escrow fee
    let final_total_xmr = total_xmr + escrow_fee_xmr;

    ctx.insert("cart_items", &cart_items_for_template);
    ctx.insert("total", &format!("{:.4} XMR", total_xmr));
    ctx.insert("escrow_fee", &format!("{:.4} XMR", escrow_fee_xmr));
    ctx.insert("final_total", &format!("{:.4} XMR", final_total_xmr));
    ctx.insert("cart_count", &cart.item_count());
    ctx.insert("cart_total_quantity", &cart.total_quantity());
    ctx.insert("is_cart_empty", &cart.is_empty());


    // Render template
    match tera.render("cart/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering cart: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Template error: {}", e))
        }
    }
}

/// GET /v2/listings - New Listings page
pub async fn show_v2_listings(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Insert session data for base template
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("user_name", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
            ctx.insert("user_role", &role);
            ctx.insert("is_vendor", &(role == "vendor"));
        } else {
            ctx.insert("user_role", &"buyer");
            ctx.insert("is_vendor", &false);
        }
    } else {
        ctx.insert("logged_in", &false);
        ctx.insert("is_vendor", &false);
        ctx.insert("user_role", &"guest");
    }

    // Add CSRF token
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("v2_listings.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!("Template error rendering v2_listings: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Template error: {}", e))
        }
    }
}

/// GET /fr/home - New V2 French Homepage
pub async fn new_index_french(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Add session data to context
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        } else {
            ctx.insert("role", &"buyer");
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("v2_index_french.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error rendering new_index_french: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /home2 - New Homepage
pub async fn home2(tera: web::Data<Tera>, session: Session) -> impl Responder {
    let mut ctx = Context::new();

    // Add session data to context
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        } else {
            ctx.insert("role", &"buyer");
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    match tera.render("home2.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error rendering home2: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /profile - User profile page
pub async fn show_profile(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/auth"))
                .finish();
        }
    };

    let mut ctx = Context::new();

    // Insert session data
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        } else {
            ctx.insert("role", &"buyer");
        }
    } else {
        // This case should be unreachable due to the guard above, but we'll handle it safely.
        return HttpResponse::Found()
            .append_header(("Location", "/auth"))
            .finish();
    }

    // Add CSRF token for forms
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    // TODO: Fetch user profile data from DB (username, pgp_key, bio, stats)
    // For now, we use placeholder data in the template itself.

    match tera.render("profile/index.html", &ctx) {
        Ok(html) => {
            info!("Rendered profile page for user {}", user_id);
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Template error rendering profile page: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
