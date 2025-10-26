//! Integration tests for HTMX dual-mode responses
//!
//! Tests that authentication endpoints correctly detect HTMX requests
//! and return appropriate HTML fragments vs JSON responses.

use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, test, web, App};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use serde_json::json;
use time::Duration;

use server::{
    db::create_pool,
    handlers::auth,
    middleware::{
        rate_limit::{auth_rate_limiter, global_rate_limiter},
        security_headers::SecurityHeaders,
    },
};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Helper function to create a test database pool with migrations run
fn create_test_pool() -> DbPool {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")
        .expect("Failed to create test DB pool");

    // Run migrations
    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    pool
}

/// Helper function to create test app with all middleware
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
                .service(auth::login)
                .service(auth::logout)
                .service(auth::whoami),
        )
}

#[actix_web::test]
async fn test_login_returns_json_without_htmx_header() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({
            "username": "testuser",
            "password": "testpassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return JSON error (user doesn't exist)
    assert_eq!(resp.status(), 401); // Unauthorized

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[actix_web::test]
async fn test_login_returns_html_with_htmx_header() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .insert_header(("HX-Request", "true"))
        .set_json(json!({
            "username": "testuser",
            "password": "testpassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // HTMX returns 200 with error HTML fragment
    assert_eq!(resp.status(), 200);

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/html"));

    // Response body should contain alert div
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("alert alert-error"));
    assert!(body_str.contains("Invalid credentials"));
}

#[actix_web::test]
async fn test_register_validation_error_htmx() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .insert_header(("HX-Request", "true"))
        .set_json(json!({
            "username": "ab", // Too short (min 3)
            "password": "short", // Too short (min 8)
            "role": "buyer"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200); // HTMX returns 200 with error HTML

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should contain validation error message
    assert!(body_str.contains("alert alert-error"));
    assert!(body_str.contains("Validation error"));
}

#[actix_web::test]
async fn test_register_validation_error_json() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        // No HX-Request header - should return JSON
        .set_json(json!({
            "username": "ab",
            "password": "short",
            "role": "buyer"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return JSON error
    assert_eq!(resp.status(), 400); // Bad Request

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[actix_web::test]
async fn test_successful_registration_returns_redirect_for_htmx() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    let unique_username = format!("htmx_test_{}", uuid::Uuid::new_v4());

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .insert_header(("HX-Request", "true"))
        .set_json(json!({
            "username": unique_username,
            "password": "securepassword123",
            "role": "buyer"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return 200 with HX-Redirect header
    assert_eq!(resp.status(), 200);

    let redirect_header = resp.headers().get("HX-Redirect");
    assert!(redirect_header.is_some());
    assert_eq!(redirect_header.unwrap().to_str().unwrap(), "/");

    // Body should be empty for redirect
    let body = test::read_body(resp).await;
    assert!(body.is_empty());
}

#[actix_web::test]
async fn test_successful_registration_returns_json_without_htmx() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    let unique_username = format!("json_test_{}", uuid::Uuid::new_v4());

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        // No HX-Request header
        .set_json(json!({
            "username": unique_username,
            "password": "securepassword123",
            "role": "vendor"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return 201 Created with JSON user data
    assert_eq!(resp.status(), 201);

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));

    let body = test::read_body(resp).await;
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user["username"], unique_username);
    assert_eq!(user["role"], "vendor");
    assert!(user["id"].is_string());
}

#[actix_web::test]
async fn test_htmx_header_value_must_be_true() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool)).await;

    // Test with "false" value - should NOT be treated as HTMX
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .insert_header(("HX-Request", "false"))
        .set_json(json!({
            "username": "testuser",
            "password": "testpassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return JSON (HTMX header must be "true")
    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[actix_web::test]
async fn test_login_success_with_htmx_creates_session() {
    let pool = create_test_pool();
    let app = test::init_service(create_test_app(pool.clone())).await;

    // First create a user
    let username = format!("session_test_{}", uuid::Uuid::new_v4());
    let password = "securepassword123";

    let register_req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "username": username,
            "password": password,
            "role": "buyer"
        }))
        .to_request();

    let _register_resp = test::call_service(&app, register_req).await;

    // Now login with HTMX
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .insert_header(("HX-Request", "true"))
        .set_json(json!({
            "username": username,
            "password": password
        }))
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;

    // Should succeed with redirect
    assert_eq!(login_resp.status(), 200);
    assert!(login_resp.headers().get("HX-Redirect").is_some());

    // Session cookie should be set
    let cookies = login_resp.headers().get_all("set-cookie");
    assert!(cookies
        .into_iter()
        .any(|c| { c.to_str().unwrap().contains("test_session") }));
}
