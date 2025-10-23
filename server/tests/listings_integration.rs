//! Integration tests for listing endpoints
//!
//! Tests the complete listing CRUD flow with real database and authentication.
//! No mocks - production-ready testing against actual services.

use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, test, web, App};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use serde_json::json;
use time::Duration;

use server::{
    db::create_pool,
    handlers::{auth, listings},
    middleware::{
        rate_limit::{auth_rate_limiter, global_rate_limiter},
        security_headers::SecurityHeaders,
    },
};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Helper function to create test app with all routes
fn create_test_app(
    pool: DbPool,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let secret_key = Key::from(b"test_secret_key_at_least_64_bytes_long_for_security_purposes!!!!");

    App::new()
        .wrap(SecurityHeaders)
        .wrap(Logger::default())
        .wrap(global_rate_limiter())
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                .cookie_name("test_session".to_string())
                .cookie_http_only(true)
                .cookie_same_site(actix_web::cookie::SameSite::Strict)
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::hours(24)))
                .build(),
        )
        .app_data(web::Data::new(pool.clone()))
        .service(
            web::scope("/api/auth")
                .wrap(auth_rate_limiter())
                .service(auth::register)
                .service(auth::login),
        )
        .service(listings::create_listing)
        .service(listings::list_listings)
        .service(listings::get_listing)
        .service(listings::get_vendor_listings)
        .service(listings::search_listings)
        .service(listings::update_listing)
        .service(listings::delete_listing)
}

/// Test 1: Create listing (authenticated vendor)
#[actix_web::test]
async fn test_create_listing_success() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor
    let register_payload = json!({
        "username": "vendor1",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor
    let login_payload = json!({
        "username": "vendor1",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Test Product",
        "description": "This is a test product description that is long enough",
        "price_xmr": 5_000_000_000_000i64, // 5 XMR in atomic units
        "stock": 10
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Listing creation should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Test Product");
    assert_eq!(body["price_xmr"], 5_000_000_000_000i64);
    assert_eq!(body["stock"], 10);
    assert_eq!(body["status"], "active");

    Ok(())
}

/// Test 2: List active listings (public endpoint)
#[actix_web::test]
async fn test_list_listings() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor
    let register_payload = json!({
        "username": "vendor2",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor
    let login_payload = json!({
        "username": "vendor2",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create a listing first
    let listing_payload = json!({
        "title": "Test Listing for List",
        "description": "A test listing to verify list endpoint functionality",
        "price_xmr": 2_000_000_000_000i64,
        "stock": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new("test_session", cookie))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Now list all listings (no auth required)
    let req = test::TestRequest::get().uri("/api/listings").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(
        body.as_array()
            .expect("Response body should be an array")
            .len(),
        1
    );
    assert_eq!(body[0]["title"], "Test Listing for List");

    Ok(())
}

/// Test 3: Get single listing by ID
#[actix_web::test]
async fn test_get_listing_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor
    let register_payload = json!({
        "username": "vendor3",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor
    let login_payload = json!({
        "username": "vendor3",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Specific Test Listing",
        "description": "Testing get by ID functionality for this listing",
        "price_xmr": 1_500_000_000_000i64,
        "stock": 3
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new("test_session", cookie))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let created_listing: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = created_listing["id"]
        .as_str()
        .expect("Listing ID should be a string");

    // Get listing by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/listings/{}", listing_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], listing_id);
    assert_eq!(body["title"], "Specific Test Listing");

    Ok(())
}

/// Test 4: Update listing (owner only)
#[actix_web::test]
async fn test_update_listing() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor
    let register_payload = json!({
        "username": "vendor4",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor
    let login_payload = json!({
        "username": "vendor4",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Original Title",
        "description": "Original description that meets minimum length requirement",
        "price_xmr": 1_000_000_000_000i64,
        "stock": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let created_listing: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = created_listing["id"]
        .as_str()
        .expect("Listing ID should be a string");

    // Update listing
    let update_payload = json!({
        "title": "Updated Title",
        "stock": 8
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/listings/{}", listing_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", cookie))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["stock"], 8);

    Ok(())
}

/// Test 5: Search listings
#[actix_web::test]
async fn test_search_listings() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor
    let register_payload = json!({
        "username": "vendor5",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor
    let login_payload = json!({
        "username": "vendor5",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing with searchable title
    let listing_payload = json!({
        "title": "Monero Privacy Coin Guide",
        "description": "A comprehensive guide about Monero and privacy coins in general",
        "price_xmr": 500_000_000_000i64,
        "stock": 100
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new("test_session", cookie))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Search for "Monero"
    let req = test::TestRequest::get()
        .uri("/api/listings/search?q=Monero")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(
        body.as_array()
            .expect("Response body should be an array")
            .len(),
        1
    );
    assert_eq!(body[0]["title"], "Monero Privacy Coin Guide");

    Ok(())
}

/// Test 6: Delete listing (soft delete)
#[actix_web::test]
async fn test_delete_listing() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor
    let register_payload = json!({
        "username": "vendor6",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor
    let login_payload = json!({
        "username": "vendor6",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Listing to Delete",
        "description": "This listing will be deleted to test deletion endpoint",
        "price_xmr": 300_000_000_000i64,
        "stock": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let created_listing: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = created_listing["id"]
        .as_str()
        .expect("Listing ID should be a string");

    // Delete listing
    let req = test::TestRequest::delete()
        .uri(&format!("/api/listings/{}", listing_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", cookie))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204, "Delete should return 204 No Content");

    Ok(())
}

/// Test 7: Authorization - cannot update other vendor's listing
#[actix_web::test]
async fn test_cannot_update_other_vendor_listing() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor1
    let register_payload = json!({
        "username": "vendor7",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor1
    let login_payload = json!({
        "username": "vendor7",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let vendor1_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Vendor 1 creates listing
    let listing_payload = json!({
        "title": "Vendor 1 Listing",
        "description": "This listing belongs to vendor 1 and should not be editable by vendor 2",
        "price_xmr": 1_000_000_000_000i64,
        "stock": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor1_cookie,
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let created_listing: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = created_listing["id"]
        .as_str()
        .expect("Listing ID should be a string");

    // Register vendor2
    let register_payload = json!({
        "username": "vendor8",
        "password": "password123",
        "role": "vendor"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login vendor2
    let login_payload = json!({
        "username": "vendor8",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let vendor2_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Vendor 2 tries to update vendor 1's listing
    let update_payload = json!({
        "title": "Hacked Title"
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/listings/{}", listing_id))
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor2_cookie,
        ))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Should be forbidden");

    Ok(())
}
