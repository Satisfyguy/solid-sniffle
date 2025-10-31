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
use server::handlers::{auth, cart, escrow, frontend, listings, monitoring, multisig_challenge, orders, reputation, reputation_ipfs};
use server::middleware::{
    admin_auth::AdminAuth,
    rate_limit::{global_rate_limiter, protected_rate_limiter},
    security_headers::SecurityHeaders,
};
use hex;
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
    session: actix_session::Session,
) -> Result<HttpResponse, Error> {
    // Get authenticated user ID from session
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => match Uuid::parse_str(&uid) {
            Ok(uuid) => uuid,
            Err(_) => {
                tracing::error!("Invalid user_id UUID in session: {}", uid);
                return Ok(HttpResponse::Unauthorized().body("Invalid session"));
            }
        },
        _ => {
            tracing::warn!("WebSocket connection attempted without authentication");
            return Ok(HttpResponse::Unauthorized().body("Authentication required"));
        }
    };

    tracing::info!("WebSocket connection established for user: {}", user_id);

    ws::start(
        WebSocketSession {
            id: Uuid::new_v4(),
            user_id,
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

    // TM-002 MITIGATION: Shamir 3-of-5 secret sharing for DB encryption key
    // If DB_ENCRYPTION_KEY is NOT set in .env → interactive Shamir reconstruction
    // If DB_ENCRYPTION_KEY IS set in .env → development mode (insecure, warns user)
    let db_encryption_key = server::crypto::shamir_startup::get_db_encryption_key()
        .context("Failed to get DB encryption key (Shamir or .env)")?;

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

    // 6. Initialize Wallet Manager with persistence and automatic recovery
    let encryption_key = hex::decode(&db_encryption_key).context("Failed to hex decode DB_ENCRYPTION_KEY")?;
    let wallet_manager = {
        let mut wm = WalletManager::new_with_persistence(
            vec![MoneroConfig::default()],
            pool.clone(),
            encryption_key.clone(),
        )?;

        // Attempt automatic recovery of active escrows
        info!("Attempting automatic recovery of active escrows...");
        match wm.recover_active_escrows().await {
            Ok(recovered_escrows) => {
                if recovered_escrows.is_empty() {
                    info!("No escrows found for recovery");
                } else {
                    info!("✅ Successfully recovered {} escrow wallet(s): {:?}",
                          recovered_escrows.len(), recovered_escrows);

                    // Emit MultisigRecovered WebSocket events for each recovered escrow
                    for escrow_id_str in &recovered_escrows {
                        if let Ok(escrow_id) = Uuid::parse_str(escrow_id_str) {
                            use server::websocket::WsEvent;
                            websocket_server.do_send(WsEvent::MultisigRecovered {
                                escrow_id,
                                recovered_wallets: vec!["buyer".to_string(), "vendor".to_string(), "arbiter".to_string()],
                                phase: "Recovered from persistence".to_string(),
                                recovered_at: chrono::Utc::now().timestamp(),
                            });
                            info!("Sent MultisigRecovered event for escrow {}", escrow_id);
                        }
                    }
                }
            }
            Err(e) => {
                // Log error but don't fail startup - recovery is best-effort
                tracing::error!("⚠️  Escrow recovery encountered errors: {}", e);
                info!("Server will continue with fresh wallet state");
            }
        }

        Arc::new(Mutex::new(wm))
    };

    // 7. Ensure system arbiter exists
    {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };
        use server::models::user::{NewUser, User};
        
        let mut conn = pool.get().context("Failed to get DB connection")?;
        let arbiter_exists = web::block(move || {
            use diesel::prelude::*;
            use server::schema::users::dsl::*;
            users.filter(role.eq("arbiter")).first::<User>(&mut conn).optional()
        })
        .await
        .context("Failed to check for arbiter")??;
        
        if arbiter_exists.is_none() {
            info!("No arbiter found, creating system arbiter...");
            let password = "arbiter_system_2024";
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .context("Failed to hash password")?
                .to_string();
            
            let mut conn = pool.get().context("Failed to get DB connection")?;
            let new_arbiter = NewUser {
                id: Uuid::new_v4().to_string(),
                username: "arbiter_system".to_string(),
                password_hash,
                wallet_address: None,
                wallet_id: None,
                role: "arbiter".to_string(),
            };
            
            web::block(move || User::create(&mut conn, new_arbiter))
                .await
                .context("Failed to create arbiter")??;
            
            info!("✅ System arbiter created successfully (username: arbiter_system, password: arbiter_system_2024)");
        } else {
            info!("System arbiter already exists");
        }
    }

    // 8. Initialize Escrow Orchestrator
    let escrow_orchestrator = Arc::new(EscrowOrchestrator::new(
        wallet_manager.clone(),
        pool.clone(),
        websocket_server.clone(),
        encryption_key.clone(),
    ));

    // 9. Initialize and start Timeout Monitor (background service)
    use server::config::TimeoutConfig;
    use server::services::timeout_monitor::TimeoutMonitor;

    let timeout_config = TimeoutConfig::from_env();
    info!(
        "TimeoutConfig loaded: multisig_setup={}s, funding={}s, tx_confirmation={}s",
        timeout_config.multisig_setup_timeout_secs,
        timeout_config.funding_timeout_secs,
        timeout_config.transaction_confirmation_timeout_secs
    );

    let timeout_monitor = Arc::new(TimeoutMonitor::new_with_persistence(
        pool.clone(),
        websocket_server.clone(),
        timeout_config,
        encryption_key.clone(),
    ));

    // Spawn TimeoutMonitor in background
    let timeout_monitor_handle = timeout_monitor.clone();
    tokio::spawn(async move {
        timeout_monitor_handle.start_monitoring().await;
    });
    info!("TimeoutMonitor background service started");

    // 10. Initialize Tera template engine
    let tera = Tera::new("templates/**/*.html").context("Failed to initialize Tera templates")?;
    info!("Tera template engine initialized");

    // 11. Initialize IPFS client for reputation export
    use server::ipfs::client::IpfsClient;
    let ipfs_client = IpfsClient::new_local().context("Failed to initialize IPFS client")?;
    info!("IPFS client initialized (local node at 127.0.0.1:5001)");

    info!("Starting HTTP server on http://127.0.0.1:8080");

    // 12. Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Security headers (CSP, X-Frame-Options, etc.)
            .wrap(SecurityHeaders)
            // Logging middleware (logs all requests)
            .wrap(Logger::default())
            // Global rate limiter (100 req/min per IP)
            // .wrap(global_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ
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
            .route("/new-home", web::get().to(frontend::new_index))
            .route("/", web::get().to(frontend::index))
            .route("/login", web::get().to(frontend::show_login))
            .route("/register", web::get().to(frontend::show_register))
            .route("/logout", web::post().to(frontend::logout))
            .route("/listings", web::get().to(frontend::show_listings))
            .route(
                "/listings/new",
                web::get().to(frontend::show_create_listing),
            )
            .route(
                "/listings/create",
                web::get().to(frontend::show_create_listing),
            )
            .route("/listings/{id}", web::get().to(frontend::show_listing))
            .route("/listings/{id}/edit", web::get().to(frontend::show_edit_listing))
            .route("/vendor/listings", web::get().to(frontend::show_vendor_listings))
            .route("/orders", web::get().to(frontend::show_orders))
            .route("/orders/{id}", web::get().to(frontend::show_order))
            .route("/escrow/{id}", web::get().to(frontend::show_escrow))
            // Cart frontend route
            .route("/cart", web::get().to(frontend::show_cart))
            // New V2 Listings route
            .route("/v2/listings", web::get().to(frontend::show_v2_listings))
            .route("/fr/home", web::get().to(frontend::new_index_french))
            // Reputation frontend routes
            .route("/vendor/{vendor_id}", web::get().to(frontend::vendor_profile))
            .route("/review/submit", web::get().to(frontend::submit_review_form))
            // Settings frontend routes (non-custodial wallet)
            .route("/settings", web::get().to(frontend::show_settings))
            .route("/settings/wallet", web::get().to(frontend::show_wallet_settings))
            .route("/docs/wallet-setup", web::get().to(frontend::show_wallet_guide))
            .route("/profile", web::get().to(frontend::show_profile))
            // API Routes
            .route("/api/health", web::get().to(health_check))
            // WebSocket route
            .route("/ws/", web::get().to(ws_route))
            // Auth endpoints with stricter rate limiting (5 req/15min per IP)
            .service(
                web::scope("/api/auth")
                    // .wrap(auth_rate_limiter()) // Temporarily disabled for testing
                    .service(auth::register)
                    .service(auth::login)
                    .service(auth::whoami)
                    .service(auth::logout),
            )
            // Settings endpoints
            .service(
                web::scope("/api/settings")
                    .service(auth::update_wallet_address),
            )
            // Protected endpoints (listings + orders + escrow) with moderate rate limiting (60 req/min per IP)
            // Single scope ensures shared rate limit quota across all protected operations
            .service(
                web::scope("/api")
                    // .wrap(protected_rate_limiter()) // TEMPORAIREMENT DÉSACTIVÉ
                    // Listings
                    .service(listings::create_listing)
                    .service(listings::create_listing_with_images)
                    .service(listings::list_listings)
                    .service(listings::get_listing)
                    .service(listings::get_vendor_listings)
                    .service(listings::search_listings)
                    .service(listings::update_listing)
                    .service(listings::delete_listing)
                    .service(listings::upload_listing_images)
                    .service(listings::get_listing_image)
                    .service(listings::remove_listing_image)
                    // Orders
                    .service(orders::create_order)
                    .service(orders::get_pending_count)
                    .service(orders::list_orders)
                    .service(orders::get_order)
                    .service(orders::init_escrow)
                    .service(orders::dev_simulate_payment)
                    .service(orders::ship_order)
                    .service(orders::complete_order)
                    .service(orders::cancel_order)
                    .service(orders::dispute_order)
                    // Escrow
                    .route("/escrow/{id}", web::get().to(escrow::get_escrow))
                    .service(escrow::get_escrow_status)
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
                    // TM-003: Challenge-Response multisig validation
                    .service(multisig_challenge::request_multisig_challenge)
                    .service(multisig_challenge::submit_multisig_info_with_signature)
                    .service(multisig_challenge::cleanup_expired_challenges)
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
                    )
                    // Cart endpoints
                    .route("/cart/add", web::post().to(cart::add_to_cart))
                    .route("/cart/remove", web::post().to(cart::remove_from_cart))
                    .route("/cart/update", web::post().to(cart::update_cart))
                    .route("/cart/clear", web::post().to(cart::clear_cart))
                    .route("/cart", web::get().to(cart::get_cart))
                    .route("/cart/count", web::get().to(cart::get_cart_count)),
            )
            // Admin-only endpoints (requires admin role)
            .service(
                web::scope("/admin")
                    .wrap(AdminAuth)
                    .service(monitoring::get_escrow_health)
                    .service(monitoring::get_escrow_status),
            )
    })
    .bind(("127.0.0.1", 8080))
    .context("Failed to bind to 127.0.0.1:8080")?
    .run()
    .await
    .context("HTTP server error")?;

    Ok(())
}
