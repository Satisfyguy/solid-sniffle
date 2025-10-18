use actix_web::{web, App, HttpServer, Responder};
use anyhow::Result;
use std::env;

use monero_marketplace_server::db::create_pool;
use monero_marketplace_server::handlers::auth;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url)?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/api/health", web::get().to(health_check))
            .service(auth::register)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;Ok(())
}
