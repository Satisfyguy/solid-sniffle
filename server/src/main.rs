use actix::{Actor, Addr};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use anyhow::Result;
use monero_marketplace_common::types::MoneroConfig;
use server::db::create_pool;
use server::handlers::auth;
use server::services::escrow::EscrowOrchestrator;
use server::wallet_manager::WalletManager;
use server::websocket::{WebSocketServer, WebSocketSession};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_encryption_key = env::var("DB_ENCRYPTION_KEY")
        .expect("DB_ENCRYPTION_KEY must be set for SQLCipher encryption");
    let pool = create_pool(&database_url, &db_encryption_key)?;

    let websocket_server = WebSocketServer::default().start();

    let wallet_manager = Arc::new(Mutex::new(WalletManager::new(vec![MoneroConfig::default()])?));

    let escrow_orchestrator = Arc::new(EscrowOrchestrator::new(
        wallet_manager.clone(),
        pool.clone(),
        websocket_server.clone(),
        vec![], // encryption_key
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(escrow_orchestrator.clone()))
            .app_data(web::Data::new(websocket_server.clone()))
            .route("/api/health", web::get().to(health_check))
            .service(auth::register)
            .route("/ws/", web::get().to(ws_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}