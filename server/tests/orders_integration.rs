//! Integration tests for order endpoints
//!
//! Tests the complete order lifecycle with real database and authentication.
//! No mocks - production-ready testing against actual services.

use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, test, web, App};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use serde_json::json;
use time::Duration;

use server::{
    db::create_pool,
    handlers::{auth, listings, orders},
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
        .service(orders::create_order)
        .service(orders::list_orders)
        .service(orders::get_order)
        .service(orders::ship_order)
        .service(orders::complete_order)
        .service(orders::cancel_order)
        .service(orders::dispute_order)
}

/// Test 1: Create order (buyer creates order from listing)
#[actix_web::test]
async fn test_create_order_success() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Setup: vendor creates listing
    // Register vendor1
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

    // Login vendor1
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Test Product",
        "description": "Test listing description",
        "price_xmr": 5_000_000_000_000i64,
        "stock": 10
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Buyer creates order
    // Register buyer1
    let register_payload = json!({
        "username": "buyer1",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer1
    let login_payload = json!({
        "username": "buyer1",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 2
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Order creation should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["listing_id"], listing_id);
    assert_eq!(body["status"], "pending");
    assert_eq!(body["total_xmr"], 10_000_000_000_000i64); // 2 * 5 XMR

    Ok(())
}

/// Test 2: List orders for authenticated user
#[actix_web::test]
async fn test_list_orders() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor2
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

    // Login vendor2
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Another Product",
        "description": "Test listing description",
        "price_xmr": 3_000_000_000_000i64,
        "stock": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Register buyer2
    let register_payload = json!({
        "username": "buyer2",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer2
    let login_payload = json!({
        "username": "buyer2",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create order
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            buyer_cookie.clone(),
        ))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // List orders for buyer
    let req = test::TestRequest::get()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
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
    assert_eq!(body[0]["status"], "pending");

    Ok(())
}

/// Test 3: Get single order (authorization check)
#[actix_web::test]
async fn test_get_order_authorization() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor3
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

    // Login vendor3
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Protected Order Product",
        "description": "Test listing description",
        "price_xmr": 1_000_000_000_000i64,
        "stock": 3
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Register buyer3
    let register_payload = json!({
        "username": "buyer3",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer3
    let login_payload = json!({
        "username": "buyer3",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create order
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            buyer_cookie.clone(),
        ))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let order: serde_json::Value = test::read_body_json(resp).await;
    let order_id = order["id"].as_str().expect("Order ID should be a string");

    // Buyer can view their order
    let req = test::TestRequest::get()
        .uri(&format!("/api/orders/{}", order_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Third party cannot view
    // Register other_user
    let register_payload = json!({
        "username": "other_user",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login other_user
    let login_payload = json!({
        "username": "other_user",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let other_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    let req = test::TestRequest::get()
        .uri(&format!("/api/orders/{}", order_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", other_cookie))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Should be forbidden");

    Ok(())
}

/// Test 4: Complete order workflow (pending → shipped → completed)
#[actix_web::test]
async fn test_complete_order_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool.clone())).await;

    // Register vendor4
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

    // Login vendor4
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Workflow Product",
        "description": "Test listing description",
        "price_xmr": 2_000_000_000_000i64,
        "stock": 10
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Register buyer4
    let register_payload = json!({
        "username": "buyer4",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer4
    let login_payload = json!({
        "username": "buyer4",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create order
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            buyer_cookie.clone(),
        ))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let order: serde_json::Value = test::read_body_json(resp).await;
    let order_id = order["id"].as_str().expect("Order ID should be a string");

    // Manually update order to funded status (simulating escrow funding)
    // In real scenario, this would be done by escrow service
    {
        use server::models::order::{Order, OrderStatus};
        let mut conn = pool.get()?;
        Order::update_status(&mut conn, order_id.to_string(), OrderStatus::Funded)
            .expect("Failed to update order status");
    }

    // Vendor marks as shipped
    let req = test::TestRequest::put()
        .uri(&format!("/api/orders/{}/ship", order_id))
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie,
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "shipped");

    // Buyer confirms receipt
    let req = test::TestRequest::put()
        .uri(&format!("/api/orders/{}/complete", order_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "completed");

    Ok(())
}

/// Test 5: Cancel order
#[actix_web::test]
async fn test_cancel_order() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor5
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

    // Login vendor5
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Cancellable Product",
        "description": "Test listing description",
        "price_xmr": 1_500_000_000_000i64,
        "stock": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Register buyer5
    let register_payload = json!({
        "username": "buyer5",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer5
    let login_payload = json!({
        "username": "buyer5",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create order
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            buyer_cookie.clone(),
        ))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let order: serde_json::Value = test::read_body_json(resp).await;
    let order_id = order["id"].as_str().expect("Order ID should be a string");

    // Buyer cancels order
    let req = test::TestRequest::put()
        .uri(&format!("/api/orders/{}/cancel", order_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "cancelled");

    Ok(())
}

/// Test 6: Raise dispute
#[actix_web::test]
async fn test_dispute_order() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool.clone())).await;

    // Register vendor6
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

    // Login vendor6
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Disputable Product",
        "description": "Test listing description",
        "price_xmr": 3_500_000_000_000i64,
        "stock": 2
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Register buyer6
    let register_payload = json!({
        "username": "buyer6",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer6
    let login_payload = json!({
        "username": "buyer6",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create order
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            buyer_cookie.clone(),
        ))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let order: serde_json::Value = test::read_body_json(resp).await;
    let order_id = order["id"].as_str().expect("Order ID should be a string");

    // Update to funded status
    {
        use server::models::order::{Order, OrderStatus};
        let mut conn = pool.get()?;
        Order::update_status(&mut conn, order_id.to_string(), OrderStatus::Funded)
            .expect("Failed to update order status");
    }

    // Buyer raises dispute
    let req = test::TestRequest::put()
        .uri(&format!("/api/orders/{}/dispute", order_id))
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "disputed");

    Ok(())
}

/// Test 7: Validation - cannot purchase own listing
#[actix_web::test]
async fn test_cannot_purchase_own_listing() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor7
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

    // Login vendor7
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Own Listing",
        "description": "Test listing description",
        "price_xmr": 1_000_000_000_000i64,
        "stock": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Same vendor tries to buy their own listing
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie,
        ))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject self-purchase");

    Ok(())
}

/// Test 8: Validation - insufficient stock
#[actix_web::test]
async fn test_insufficient_stock() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    let app = test::init_service(create_test_app(pool)).await;

    // Register vendor8
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

    // Login vendor8
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

    let vendor_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Create listing
    let listing_payload = json!({
        "title": "Limited Stock",
        "description": "Test listing description",
        "price_xmr": 500_000_000_000i64,
        "stock": 2
    });

    let req = test::TestRequest::post()
        .uri("/api/listings")
        .cookie(actix_web::cookie::Cookie::new(
            "test_session",
            vendor_cookie.clone(),
        ))
        .set_json(&listing_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let listing_body: serde_json::Value = test::read_body_json(resp).await;
    let listing_id = listing_body["id"]
        .as_str()
        .expect("Listing ID should be a string")
        .to_string();

    // Register buyer7
    let register_payload = json!({
        "username": "buyer7",
        "password": "password123",
        "role": "buyer"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login buyer7
    let login_payload = json!({
        "username": "buyer7",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let buyer_cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "test_session")
        .expect("Session cookie should be set")
        .value()
        .to_string();

    // Try to order more than available
    let order_payload = json!({
        "listing_id": listing_id,
        "quantity": 5
    });

    let req = test::TestRequest::post()
        .uri("/api/orders")
        .cookie(actix_web::cookie::Cookie::new("test_session", buyer_cookie))
        .set_json(&order_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject insufficient stock");

    Ok(())
}
