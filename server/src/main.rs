use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use dotenvy::dotenv;
use std::env;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use monero_marketplace_common::MoneroConfig;

mod crypto;
mod db;
mod models;
mod schema;
mod services;
mod wallet_manager;
mod websocket;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "monero-marketplace-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env
    dotenv().ok();

    // Initialize tracing subscriber for structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,diesel=warn,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Monero Marketplace Server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize MoneroConfig and WalletManager
    let monero_config = MoneroConfig::default();
    let wallet_manager = Arc::new(
        wallet_manager::WalletManager::new(monero_config)
            .map_err(|e| {
                error!("Failed to create WalletManager: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?
    );
    info!("✓ Wallet Manager initialized");

    // Initialize DbPool
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            info!("DATABASE_URL not set, using default: ./marketplace.db");
            "marketplace.db".to_string()
        });

    let db_pool = db::create_pool(&database_url)
        .map_err(|e| {
            error!("Failed to create database pool: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
    info!("✓ Database pool created: {}", database_url);

    // Run database migrations
    {
        let mut conn = db_pool.get()
            .map_err(|e| {
                error!("Failed to get DB connection for migrations: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;

        db::run_migrations(&mut conn)
            .map_err(|e| {
                error!("Failed to run database migrations: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;
        info!("✓ Database migrations applied");
    }

    // Initialize WebSocketServer
    let websocket_server = Arc::new(websocket::WebSocketServer::new());
    info!("✓ WebSocket server initialized");

    // Generate or load encryption key
    // WARNING: In production, load this from a secure key management system
    // For now, we generate a random key (data will be lost on restart)
    let encryption_key = crypto::encryption::generate_key();
    info!("✓ Encryption key generated (ephemeral - data will be lost on restart)");

    // Initialize EscrowOrchestrator
    let escrow_orchestrator = Arc::new(services::escrow::EscrowOrchestrator::new(
        wallet_manager.clone(),
        db_pool.clone(),
        websocket_server.clone(),
        encryption_key,
    ));
    info!("✓ Escrow Orchestrator initialized");

    let bind_addr = ("127.0.0.1", 8080);
    info!("🌐 Server binding to http://{}:{}", bind_addr.0, bind_addr.1);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(escrow_orchestrator.clone()))
            .route("/api/health", web::get().to(health_check))
    })
    .bind(bind_addr)?
    .run()
    .await
}
