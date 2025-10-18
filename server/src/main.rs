use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use dotenvy::dotenv;
use std::env;

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
        "status": "ok"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize MoneroConfig and WalletManager
    let monero_config = MoneroConfig::default();
    let wallet_manager = Arc::new(wallet_manager::WalletManager::new(monero_config).expect("Failed to create WalletManager"));

    // Initialize DbPool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = db::create_pool(&database_url).expect("Failed to create DbPool");

    // Initialize WebSocketServer (placeholder)
    let websocket_server = Arc::new(websocket::WebSocketServer::new());

    // Generate encryption key (placeholder)
    let encryption_key = crypto::encryption::generate_key();

    // Initialize EscrowOrchestrator
    let escrow_orchestrator = Arc::new(services::escrow::EscrowOrchestrator::new(
        wallet_manager.clone(),
        db_pool.clone(),
        websocket_server.clone(),
        encryption_key,
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(escrow_orchestrator.clone()))
            .route("/api/health", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
