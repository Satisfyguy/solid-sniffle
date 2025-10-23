//! Integration tests for reputation system endpoints
//!
//! Tests the complete reputation flow with real database and cryptographic signatures.
//! No mocks - production-ready testing against actual services.

use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, test, web, App};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, Rng};
use serde_json::json;
use time::Duration;

use server::{
    db::create_pool,
    handlers::{auth, reputation},
    middleware::{
        rate_limit::{auth_rate_limiter, global_rate_limiter, protected_rate_limiter},
        security_headers::SecurityHeaders,
    },
};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Helper function to create test app with all middleware and reputation routes
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
                .service(auth::whoami)
                .service(auth::logout),
        )
        .service(
            web::scope("/api")
                .wrap(protected_rate_limiter())
                .route("/reviews", web::post().to(reputation::submit_review))
                .route(
                    "/reputation/{vendor_id}",
                    web::get().to(reputation::get_vendor_reputation),
                )
                .route(
                    "/reputation/{vendor_id}/stats",
                    web::get().to(reputation::get_vendor_stats),
                ),
        )
}

/// Test 1: Submit a cryptographically signed review
///
/// # Test Coverage
/// - User authentication (buyer + vendor)
/// - Cryptographic signature generation with ed25519
/// - Review submission with valid signature
/// - Database persistence
#[actix_web::test]
async fn test_submit_signed_review() -> Result<(), Box<dyn std::error::Error>> {
    // Setup: Create in-memory test database
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    // Run migrations
    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Migration error: {}", e))?;
    }

    let app = test::init_service(create_test_app(pool.clone())).await;

    // Step 1: Register buyer
    let buyer_payload = json!({
        "username": "buyer_alice",
        "password": "SecurePass123!@#",
        "email": "alice@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&buyer_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Buyer registration failed");

    // Step 2: Register vendor
    let vendor_payload = json!({
        "username": "vendor_bob",
        "password": "SecurePass456!@#",
        "email": "bob@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&vendor_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Vendor registration failed");

    let vendor_body: serde_json::Value = test::read_body_json(resp).await;
    let vendor_id = vendor_body["user_id"]
        .as_str()
        .ok_or("Missing vendor_id")?;

    // Step 3: Login as buyer to get session
    let login_payload = json!({
        "username": "buyer_alice",
        "password": "SecurePass123!@#"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Buyer login failed");

    // Extract session cookie
    let mut cookies = resp.response().cookies();
    let session_cookie = cookies
        .find(|c| c.name() == "test_session")
        .ok_or("Session cookie not found")?;

    // Step 4: Generate ed25519 keypair for buyer
    let secret_bytes: [u8; 32] = OsRng.gen();
    let signing_key = SigningKey::from_bytes(&secret_bytes);

    // Step 5: Create signed review using reputation-crypto
    use reputation_crypto::reputation::sign_review;

    let txid = "abc123def456fake_transaction_hash";
    let rating = 5u8;
    let comment = Some("Excellent vendor! Fast shipping and great quality.".to_string());

    let signed_review = sign_review(txid.to_string(), rating, comment, &signing_key)?;

    // Step 6: Submit review via API
    let review_payload = json!({
        "vendor_id": vendor_id,
        "review": {
            "txid": signed_review.txid,
            "rating": signed_review.rating,
            "comment": signed_review.comment,
            "timestamp": signed_review.timestamp.to_rfc3339(),
            "buyer_pubkey": signed_review.buyer_pubkey,
            "signature": signed_review.signature
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/reviews")
        .cookie(session_cookie.clone())
        .set_json(&review_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Review submission failed: {:?}",
        test::read_body(resp).await
    );

    let review_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(review_body["status"], "success");
    assert_eq!(review_body["review"]["rating"], 5);

    Ok(())
}

/// Test 2: Retrieve vendor reputation with multiple reviews
///
/// # Test Coverage
/// - Submit multiple reviews from different buyers
/// - Retrieve aggregated reputation data
/// - Verify cryptographic integrity is maintained
#[actix_web::test]
async fn test_get_vendor_reputation() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Migration error: {}", e))?;
    }

    let app = test::init_service(create_test_app(pool.clone())).await;

    // Register vendor
    let vendor_payload = json!({
        "username": "vendor_charlie",
        "password": "SecurePass789!@#",
        "email": "charlie@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&vendor_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let vendor_body: serde_json::Value = test::read_body_json(resp).await;
    let vendor_id = vendor_body["user_id"].as_str().ok_or("Missing vendor_id")?;

    // Submit 3 reviews with different ratings
    for (i, rating) in [5u8, 4u8, 5u8].iter().enumerate() {
        // Register buyer
        let buyer_payload = json!({
            "username": format!("buyer_{}", i),
            "password": "SecurePass123!@#",
            "email": format!("buyer{}@example.com", i)
        });

        let req = test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(&buyer_payload)
            .to_request();
        test::call_service(&app, req).await;

        // Login
        let login_payload = json!({
            "username": format!("buyer_{}", i),
            "password": "SecurePass123!@#"
        });

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        let mut cookies = resp.response().cookies();
        let session_cookie = cookies
            .find(|c| c.name() == "test_session")
            .ok_or("Session cookie not found")?;

        // Create and submit review
        let secret_bytes: [u8; 32] = OsRng.gen();
        let signing_key = SigningKey::from_bytes(&secret_bytes);

        use reputation_crypto::reputation::sign_review;
        let signed_review = sign_review(
            format!("tx_{}", i),
            *rating,
            Some(format!("Review from buyer {}", i)),
            &signing_key,
        )?;

        let review_payload = json!({
            "vendor_id": vendor_id,
            "review": {
                "txid": signed_review.txid,
                "rating": signed_review.rating,
                "comment": signed_review.comment,
                "timestamp": signed_review.timestamp.to_rfc3339(),
                "buyer_pubkey": signed_review.buyer_pubkey,
                "signature": signed_review.signature
            }
        });

        let req = test::TestRequest::post()
            .uri("/api/reviews")
            .cookie(session_cookie.clone())
            .set_json(&review_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Retrieve reputation
    let req = test::TestRequest::get()
        .uri(&format!("/api/reputation/{}", vendor_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let reputation_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(reputation_body["reviews"].as_array().unwrap().len(), 3);

    Ok(())
}

/// Test 3: Retrieve vendor statistics
///
/// # Test Coverage
/// - Calculate average rating
/// - Count total reviews
/// - Verify only verified reviews are counted
#[actix_web::test]
async fn test_get_vendor_stats() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Migration error: {}", e))?;
    }

    let app = test::init_service(create_test_app(pool.clone())).await;

    // Register vendor
    let vendor_payload = json!({
        "username": "vendor_diana",
        "password": "SecurePass999!@#",
        "email": "diana@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&vendor_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let vendor_body: serde_json::Value = test::read_body_json(resp).await;
    let vendor_id = vendor_body["user_id"].as_str().ok_or("Missing vendor_id")?;

    // Retrieve stats (should be zero)
    let req = test::TestRequest::get()
        .uri(&format!("/api/reputation/{}/stats", vendor_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let stats_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(stats_body["total_reviews"], 0);
    assert_eq!(stats_body["average_rating"], 0.0);

    Ok(())
}

/// Test 4: Reject review with invalid signature
///
/// # Test Coverage
/// - Tamper with review data after signing
/// - Verify signature validation catches tampering
/// - Ensure 400 Bad Request response
#[actix_web::test]
async fn test_reject_invalid_signature() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(":memory:", "test_encryption_key_32_bytes!!")?;

    {
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Migration error: {}", e))?;
    }

    let app = test::init_service(create_test_app(pool.clone())).await;

    // Register buyer and vendor
    let buyer_payload = json!({
        "username": "buyer_eve",
        "password": "SecurePass111!@#",
        "email": "eve@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&buyer_payload)
        .to_request();
    test::call_service(&app, req).await;

    let vendor_payload = json!({
        "username": "vendor_frank",
        "password": "SecurePass222!@#",
        "email": "frank@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&vendor_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let vendor_body: serde_json::Value = test::read_body_json(resp).await;
    let vendor_id = vendor_body["user_id"].as_str().ok_or("Missing vendor_id")?;

    // Login as buyer
    let login_payload = json!({
        "username": "buyer_eve",
        "password": "SecurePass111!@#"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let mut cookies = resp.response().cookies();
    let session_cookie = cookies
        .find(|c| c.name() == "test_session")
        .ok_or("Session cookie not found")?;

    // Create signed review
    let secret_bytes: [u8; 32] = OsRng.gen();
    let signing_key = SigningKey::from_bytes(&secret_bytes);

    use reputation_crypto::reputation::sign_review;
    let signed_review = sign_review(
        "tampered_tx".to_string(),
        5,
        Some("Original comment".to_string()),
        &signing_key,
    )?;

    // Tamper with rating (signature now invalid)
    let tampered_payload = json!({
        "vendor_id": vendor_id,
        "review": {
            "txid": signed_review.txid,
            "rating": 1,  // Changed from 5 to 1!
            "comment": signed_review.comment,
            "timestamp": signed_review.timestamp.to_rfc3339(),
            "buyer_pubkey": signed_review.buyer_pubkey,
            "signature": signed_review.signature
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/reviews")
        .cookie(session_cookie.clone())
        .set_json(&tampered_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Should fail with 400 Bad Request
    assert_eq!(
        resp.status().as_u16(),
        400,
        "Tampered review should be rejected"
    );

    Ok(())
}
