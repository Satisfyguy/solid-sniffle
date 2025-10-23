//! Honeypot endpoints for attack detection
//!
//! These endpoints are intentionally fake and designed to attract attackers.
//! Any access to these endpoints is logged with high threat scores.
//!
//! OPSEC: Never log real IPs - always hash with salt

use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use std::collections::HashMap;
use tracing::warn;

use crate::security::threat_scorer::{ThreatEvent, ThreatScorer};

/// Fake admin panel endpoint
/// Attackers looking for admin access will hit this
pub async fn fake_admin(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(&ip, ThreatEvent::HoneypotAccess("/api/admin".to_string()))
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/api/admin",
        "Honeypot triggered - fake admin panel accessed"
    );

    // Return plausible fake response to waste attacker's time
    Ok(HttpResponse::Ok().json(json!({
        "admin": true,
        "version": "1.2.3",
        "endpoints": ["/api/admin/users", "/api/admin/config", "/api/admin/logs"]
    })))
}

/// Fake debug endpoint
pub async fn fake_debug(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(&ip, ThreatEvent::HoneypotAccess("/api/debug".to_string()))
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/api/debug",
        "Honeypot triggered - fake debug endpoint accessed"
    );

    Ok(HttpResponse::Ok().json(json!({
        "debug_mode": true,
        "database": "postgresql://localhost:5432/marketplace",
        "redis": "127.0.0.1:6379",
        "environment": "production"
    })))
}

/// Fake user enumeration endpoint
pub async fn fake_users(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(
            &ip,
            ThreatEvent::HoneypotAccess("/api/v1/users".to_string()),
        )
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/api/v1/users",
        "Honeypot triggered - fake user enumeration"
    );

    Ok(HttpResponse::Ok().json(json!({
        "users": [
            {"id": 1, "username": "admin", "role": "admin"},
            {"id": 2, "username": "vendor1", "role": "vendor"},
            {"id": 3, "username": "buyer1", "role": "buyer"}
        ]
    })))
}

/// Fake backup download endpoint
pub async fn fake_backup(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(&ip, ThreatEvent::HoneypotAccess("/api/backup".to_string()))
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/api/backup",
        "Honeypot triggered - fake backup download attempt"
    );

    // Return fake SQL dump
    Ok(HttpResponse::Ok()
        .content_type("application/sql")
        .body("-- Monero Marketplace Database Backup\n-- Generated: 2025-10-22\nSELECT * FROM users;"))
}

/// Fake environment file endpoint
pub async fn fake_env(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(&ip, ThreatEvent::HoneypotAccess("/.env".to_string()))
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/.env",
        "Honeypot triggered - .env file access attempt"
    );

    // Return fake environment variables
    Ok(HttpResponse::Ok().content_type("text/plain").body(
        "DATABASE_URL=postgresql://admin:password123@localhost:5432/marketplace\n\
         SESSION_SECRET=insecure_secret_key_do_not_use\n\
         MONERO_RPC_URL=http://127.0.0.1:18082\n\
         ADMIN_PASSWORD=admin123",
    ))
}

/// Fake config.json endpoint
pub async fn fake_config(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(
            &ip,
            ThreatEvent::HoneypotAccess("/config.json".to_string()),
        )
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/config.json",
        "Honeypot triggered - config.json access"
    );

    Ok(HttpResponse::Ok().json(json!({
        "database": {
            "host": "localhost",
            "port": 5432,
            "username": "marketplace_user",
            "password": "fake_password_123"
        },
        "monero": {
            "rpc_url": "http://127.0.0.1:18082",
            "wallet_path": "/var/lib/monero/wallets"
        }
    })))
}

/// Fake internal metrics endpoint
pub async fn fake_metrics(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(
            &ip,
            ThreatEvent::HoneypotAccess("/api/internal/metrics".to_string()),
        )
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/api/internal/metrics",
        "Honeypot triggered - internal metrics access"
    );

    Ok(HttpResponse::Ok().content_type("text/plain").body(
        "# HELP http_requests_total Total HTTP requests\n\
         http_requests_total{method=\"GET\",endpoint=\"/api/listings\"} 1234\n\
         # HELP active_users Current active users\n\
         active_users 42",
    ))
}

/// Fake phpMyAdmin endpoint (common scanner target)
pub async fn fake_phpmyadmin(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(
            &ip,
            ThreatEvent::HoneypotAccess("/phpMyAdmin".to_string()),
        )
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/phpMyAdmin",
        "Honeypot triggered - phpMyAdmin scanner detected"
    );

    Ok(HttpResponse::Ok().content_type("text/html").body(
        "<html><head><title>phpMyAdmin</title></head>\
         <body><h1>Welcome to phpMyAdmin 4.9.5</h1></body></html>",
    ))
}

/// Fake WordPress admin endpoint (common scanner target)
pub async fn fake_wp_admin(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(&ip, ThreatEvent::HoneypotAccess("/wp-admin".to_string()))
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/wp-admin",
        "Honeypot triggered - WordPress scanner detected"
    );

    Ok(HttpResponse::Ok().content_type("text/html").body(
        "<html><head><title>WordPress Login</title></head>\
         <body><h1>WordPress Admin</h1><form><!-- fake login --></form></body></html>",
    ))
}

/// Fake wallet balance endpoint
pub async fn fake_wallet_balance(
    req: HttpRequest,
    scorer: web::Data<ThreatScorer>,
) -> actix_web::Result<HttpResponse> {
    let ip = extract_peer_ip(&req);

    scorer
        .record_event(
            &ip,
            ThreatEvent::HoneypotAccess("/api/wallet/balance".to_string()),
        )
        .await;

    warn!(
        ip_hash = %scorer.hash_ip(&ip),
        endpoint = "/api/wallet/balance",
        "Honeypot triggered - fake wallet endpoint"
    );

    Ok(HttpResponse::Ok().json(json!({
        "balance": "123.456789",
        "unlocked_balance": "100.000000",
        "currency": "XMR"
    })))
}

/// Extract peer IP from request
/// In production, this should check X-Forwarded-For if behind reverse proxy
fn extract_peer_ip(req: &HttpRequest) -> String {
    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_fake_admin_returns_plausible_response() {
        let scorer = web::Data::new(ThreatScorer::new());
        let app = test::init_service(
            App::new()
                .app_data(scorer.clone())
                .route("/api/admin", web::get().to(fake_admin)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/admin").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["admin"], true);
    }

    #[actix_web::test]
    async fn test_fake_env_returns_fake_credentials() {
        let scorer = web::Data::new(ThreatScorer::new());
        let app = test::init_service(
            App::new()
                .app_data(scorer.clone())
                .route("/.env", web::get().to(fake_env)),
        )
        .await;

        let req = test::TestRequest::get().uri("/.env").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let content = String::from_utf8(body.to_vec()).unwrap();

        assert!(content.contains("DATABASE_URL"));
        assert!(content.contains("password123"));
    }
}
