use actix::{Actor, Addr};
use actix_files as fs;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;
use anyhow::{Context, Result};
use monero_marketplace_common::types::MoneroConfig;
use server::db::create_pool;
use server::handlers::{auth, escrow, frontend, listings, orders, reputation, reputation_ipfs};
use server::middleware::{
    rate_limit::{auth_rate_limiter, global_rate_limiter, protected_rate_limiter},
    security_headers::SecurityHeaders,
};
use server::services::escrow::EscrowOrchestrator;
use server::wallet_manager::WalletManager;
use server::websocket::{WebSocketServer, WebSocketSession};
use std::env;
use std::sync::Arc;
use tera::Tera;
use time::Duration;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<WebSocketServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WebSocketSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(), // This should be authenticated user
            hb: std::time::Instant::now(),
            server: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> Result<()> {
    // 1. Load environment variables
    dotenvy::dotenv().ok();

    // 2. Initialize structured logging (tracing)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,actix_web=info,actix_server=info,diesel=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Monero Marketplace Server");

    // 3. Database connection pool with SQLCipher encryption
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL must be set in environment")?;
    let db_encryption_key = env::var("DB_ENCRYPTION_KEY")
        .context("DB_ENCRYPTION_KEY must be set for SQLCipher encryption")?;
    let pool = create_pool(&database_url, &db_encryption_key)
        .context("Failed to create database connection pool")?;

    info!("Database connection pool created with SQLCipher encryption");

    // 4. Session secret key
    // IMPORTANT: In production, load from secure environment variable
    // This should be a 64-byte cryptographically random key
    let session_secret = env::var("SESSION_SECRET_KEY").unwrap_or_else(|_| {
        tracing::warn!("SESSION_SECRET_KEY not set, using development key - NOT FOR PRODUCTION");
        "development_key_do_not_use_in_production_minimum_64_bytes_required".to_string()
    });

    let secret_key = Key::from(session_secret.as_bytes());

    // 5. Initialize WebSocket server actor
    let websocket_server = WebSocketServer::default().start();

    // 6. Initialize Wallet Manager
    let wallet_manager = Arc::new(Mutex::new(WalletManager::new(vec![
        MoneroConfig::default(),
    ])?));

    // 7. Initialize Escrow Orchestrator
    let escrow_orchestrator = Arc::new(EscrowOrchestrator::new(
        wallet_manager.clone(),
        pool.clone(),
        websocket_server.clone(),
        vec![], // encryption_key - should be loaded from secure env
    ));

    // 8. Initialize Tera template engine
    let tera = Tera::new("templates/**/*.html").context("Failed to initialize Tera templates")?;
    info!("Tera template engine initialized");

    // 9. Initialize IPFS client for reputation export
    use server::ipfs::client::IpfsClient;
    let ipfs_client = IpfsClient::new_local().context("Failed to initialize IPFS client")?;
    info!("IPFS client initialized (local node at 127.0.0.1:5001)");

    info!("Starting HTTP server on http://127.0.0.1:8080");

    // 9. Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Security headers (CSP, X-Frame-Options, etc.)
            .wrap(SecurityHeaders)
            // Logging middleware (logs all requests)
            .wrap(Logger::default())
            // Global rate limiter (100 req/min per IP)
            .wrap(global_rate_limiter())
            // Session middleware
            // Security features:
            // - HttpOnly: prevents JavaScript access
            // - Secure: HTTPS only (disabled for dev on localhost)
            // - SameSite::Strict: CSRF protection
            // - Max age: 24 hours
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("monero_marketplace_session".to_string())
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::hours(24)),
                    )
                    .build(),
            )
            // Shared app state
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(escrow_orchestrator.clone()))
            .app_data(web::Data::new(websocket_server.clone()))
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(ipfs_client.clone()))
            // Static files (serve CSS, JS, images)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Frontend routes (HTML pages)
            .route("/", web::get().to(frontend::index))
            .route("/login", web::get().to(frontend::show_login))
            .route("/register", web::get().to(frontend::show_register))
            .route("/logout", web::post().to(frontend::logout))
            .route("/listings", web::get().to(frontend::show_listings))
            .route(
                "/listings/new",
                web::get().to(frontend::show_create_listing),
            )
            .route("/listings/{id}", web::get().to(frontend::show_listing))
            .route("/orders", web::get().to(frontend::show_orders))
            .route("/orders/{id}", web::get().to(frontend::show_order))
            .route("/escrow/{id}", web::get().to(frontend::show_escrow))
            // Reputation frontend routes
            .route("/vendor/{vendor_id}", web::get().to(frontend::vendor_profile))
            .route("/review/submit", web::get().to(frontend::submit_review_form))
            // Settings frontend routes (non-custodial wallet)
            .route("/settings", web::get().to(frontend::show_settings))
            .route("/settings/wallet", web::get().to(frontend::show_wallet_settings))
            .route("/docs/wallet-setup", web::get().to(frontend::show_wallet_guide))
            // API Routes
            .route("/api/health", web::get().to(health_check))
            // WebSocket route
            .route("/ws/", web::get().to(ws_route))
            // Auth endpoints with stricter rate limiting (5 req/15min per IP)
            .service(
                web::scope("/api/auth")
                    .wrap(auth_rate_limiter())
                    .service(auth::register)
                    .service(auth::login)
                    .service(auth::whoami)
                    .service(auth::logout),
            )
            // Protected endpoints (listings + orders + escrow) with moderate rate limiting (60 req/min per IP)
            // Single scope ensures shared rate limit quota across all protected operations
            .service(
                web::scope("/api")
                    .wrap(protected_rate_limiter())
                    // Listings
                    .service(listings::create_listing)
                    .service(listings::list_listings)
                    .service(listings::get_listing)
                    .service(listings::get_vendor_listings)
                    .service(listings::search_listings)
                    .service(listings::update_listing)
                    .service(listings::delete_listing)
                    // Orders
                    .service(orders::create_order)
                    .service(orders::list_orders)
                    .service(orders::get_order)
                    .service(orders::ship_order)
                    .service(orders::complete_order)
                    .service(orders::cancel_order)
                    .service(orders::dispute_order)
                    // Escrow
                    .route("/escrow/{id}", web::get().to(escrow::get_escrow))
                    // NON-CUSTODIAL: Client wallet registration
                    .route(
                        "/escrow/register-wallet-rpc",
                        web::post().to(escrow::register_wallet_rpc),
                    )
                    .route(
                        "/escrow/{id}/prepare",
                        web::post().to(escrow::prepare_multisig),
                    )
                    .route(
                        "/escrow/{id}/release",
                        web::post().to(escrow::release_funds),
                    )
                    .route("/escrow/{id}/refund", web::post().to(escrow::refund_funds))
                    .route(
                        "/escrow/{id}/dispute",
                        web::post().to(escrow::initiate_dispute),
                    )
                    .route(
                        "/escrow/{id}/resolve",
                        web::post().to(escrow::resolve_dispute),
                    )
                    // Reputation System
                    .route("/reviews", web::post().to(reputation::submit_review))
                    .route(
                        "/reputation/{vendor_id}",
                        web::get().to(reputation::get_vendor_reputation),
                    )
                    .route(
                        "/reputation/{vendor_id}/stats",
                        web::get().to(reputation::get_vendor_stats),
                    )
                    .route(
                        "/reputation/export",
                        web::post().to(reputation_ipfs::export_to_ipfs),
                    ),
            )
    })
    .bind(("127.0.0.1", 8080))
    .context("Failed to bind to 127.0.0.1:8080")?
    .run()
    .await
    .context("HTTP server error")?;

    Ok(())
}
