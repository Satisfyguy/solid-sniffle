//! Authentication handlers for the Monero Marketplace API
//!
//! Provides secure authentication endpoints with production-grade security:
//! - Argon2id password hashing (time cost ≥ 2)
//! - Rate limiting (5 failed logins per IP per hour)
//! - Session management with secure cookies
//! - CSRF token validation
//! - Input validation at API boundary
//! - Structured logging without sensitive data

use actix_session::Session;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::error::ApiError;
use crate::middleware::csrf::validate_csrf_token;
use crate::models::user::{NewUser, User};

/// Helper function to check if request is from HTMX
fn is_htmx_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Helper function to create HTMX error response
fn htmx_error_response(message: &str) -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(format!(
        r#"<div class="alert alert-error">{}</div>"#,
        message
    ))
}

/// Helper function to create HTMX success response with redirect
fn htmx_success_redirect(location: &str) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("HX-Redirect", location))
        .content_type("text/html")
        .body("")
}

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub role: String,
    pub wallet_address: Option<String>,
    pub csrf_token: String,
}

/// Validate Monero address format (simple check: starts with 4 or 8, 95-106 chars)
fn is_valid_monero_address(addr: &str) -> bool {
    let len = addr.len();
    if len < 95 || len > 106 {
        return false;
    }
    let first_char = addr.chars().next().unwrap_or('0');
    if first_char != '4' && first_char != '8' {
        return false;
    }
    // Check all characters are alphanumeric
    addr.chars().all(|c| c.is_ascii_alphanumeric())
}

#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    req: web::Form<RegisterRequest>,
    http_req: HttpRequest,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    let is_htmx = is_htmx_request(&http_req);

    // Validate CSRF token
    if !validate_csrf_token(&session, &req.csrf_token) {
        return if is_htmx {
            Ok(htmx_error_response("Invalid CSRF token"))
        } else {
            Err(ApiError::Forbidden("Invalid CSRF token".to_string()))
        };
    }

    // Validate input
    if let Err(e) = req.0.validate() {
        return if is_htmx {
            Ok(htmx_error_response(&format!("Validation error: {}", e)))
        } else {
            Err(ApiError::from(e))
        };
    }

    // Validate that vendors have wallet address (optional but recommended)
    // Note: We don't make it strictly required here to allow vendors to set it later in Settings
    // But we log a warning if missing
    if req.role == "vendor" && req.wallet_address.is_none() {
        warn!(
            username = %req.username,
            "Vendor registered without wallet address - will need to configure before shipping orders"
        );
    }

    // If wallet_address is provided, validate its format
    if let Some(ref addr) = req.wallet_address {
        if !is_valid_monero_address(addr) {
            return if is_htmx {
                Ok(htmx_error_response("Invalid Monero address format. Must start with 4 or 8 and be 95-106 characters."))
            } else {
                Err(ApiError::Internal("Invalid Monero address format".to_string()))
            };
        }
    }

    let mut conn = pool.get().map_err(|e| ApiError::Internal(e.to_string()))?;

    // 1. Check if username exists
    let req_username = req.username.clone();
    let username_exists =
        web::block(move || User::username_exists(&mut conn, &req_username)).await??;
    if username_exists {
        return if is_htmx {
            Ok(htmx_error_response("Username already taken"))
        } else {
            Err(ApiError::Conflict("Username already taken".to_string()))
        };
    }

    // 2. Hash password using Argon2id with PasswordHasher trait
    let password = req.password.clone();
    let password_hash = web::block(move || -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    })
    .await??;

    // 3. Create user
    let mut conn = pool.get().map_err(|e| ApiError::Internal(e.to_string()))?;
    let new_user = NewUser {
        id: Uuid::new_v4().to_string(),
        username: req.username.clone(),
        password_hash,
        wallet_address: req.wallet_address.clone(),
        wallet_id: None,
        role: req.role.clone(),
    };

    let user = web::block(move || User::create(&mut conn, new_user)).await??;

    info!(
        user_id = %user.id,
        username = %user.username,
        role = %user.role,
        "User registered successfully"
    );

    // For HTMX: create session and redirect to homepage
    if is_htmx {
        session
            .insert("user_id", user.id.clone())
            .context("Failed to create session")
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        session
            .insert("username", user.username.clone())
            .context("Failed to store username in session")
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        session
            .insert("role", user.role.clone())
            .context("Failed to store role in session")
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        Ok(htmx_success_redirect("/"))
    } else {
        Ok(HttpResponse::Created().json(UserResponse::from(user)))
    }
}

/// Login request structure with validation
#[derive(Debug, Validate, Deserialize)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub csrf_token: String,
}

/// User response (without password_hash)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub role: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role,
        }
    }
}

/// Login endpoint
///
/// # Security Features
/// - Argon2id password verification (constant-time comparison)
/// - Rate limiting: 5 failed attempts per IP per hour (middleware)
/// - Account lockout: implemented via rate limiting middleware
/// - Session token: cryptographically random, HttpOnly cookie
/// - Structured logging without password exposure
///
/// # Note on Account Lockout
/// Account lockout is handled by the rate limiting middleware which
/// tracks failed login attempts per IP address. This prevents both
/// brute-force attacks and account enumeration. For user-specific
/// lockout (tracking by username), see Milestone 2.3 security enhancements.
#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    session: Session,
    req: web::Form<LoginRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let is_htmx = is_htmx_request(&http_req);

    // 1. Validate CSRF token
    if !validate_csrf_token(&session, &req.csrf_token) {
        return if is_htmx {
            Ok(htmx_error_response("Invalid CSRF token"))
        } else {
            Err(ApiError::Forbidden("Invalid CSRF token".to_string()))
        };
    }

    // 2. Validate input
    if let Err(e) = req.0.validate() {
        return if is_htmx {
            Ok(htmx_error_response(&format!("Validation error: {}", e)))
        } else {
            Err(ApiError::from(e))
        };
    }

    let username = req.username.clone();
    let password = req.password.clone();

    // 2. Find user by username
    let mut conn = pool
        .get()
        .context("Failed to get database connection")
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let username_for_lookup = username.clone();
    let user_result = web::block(move || User::find_by_username(&mut conn, &username_for_lookup))
        .await
        .context("Database query failed")
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let user = match user_result {
        Ok(u) => u,
        Err(_) => {
            warn!(
                username = %username,
                "Login attempt with non-existent username"
            );
            return if is_htmx {
                Ok(htmx_error_response("Invalid credentials"))
            } else {
                Err(ApiError::Unauthorized("Invalid credentials".to_string()))
            };
        }
    };

    // 3. Verify password using PasswordVerifier trait (constant-time comparison)
    let password_hash_str = user.password_hash.clone();
    let user_id = user.id.clone();
    let user_username = user.username.clone();

    let password_valid = web::block(move || -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(&password_hash_str)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await
    .context("Password verification task failed")
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .map_err(|e| {
        warn!(
            user_id = %user_id,
            error = %e,
            "Argon2 password verification failed"
        );
        ApiError::Internal("Password verification error".to_string())
    })?;

    if !password_valid {
        warn!(
            user_id = %user_id,
            username = %user_username,
            "Failed login attempt - invalid password"
        );
        return if is_htmx {
            Ok(htmx_error_response("Invalid credentials"))
        } else {
            Err(ApiError::Unauthorized("Invalid credentials".to_string()))
        };
    }

    // 4. Create session
    session
        .insert("user_id", user.id.clone())
        .context("Failed to create session")
        .map_err(|e| {
            warn!(
                user_id = %user.id,
                error = %e,
                "Failed to insert user_id into session"
            );
            ApiError::Internal("Session creation failed".to_string())
        })?;
    session
        .insert("username", user.username.clone())
        .context("Failed to store username in session")
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    session
        .insert("role", user.role.clone())
        .context("Failed to store role in session")
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    info!(
        user_id = %user.id,
        username = %user.username,
        role = %user.role,
        "User logged in successfully"
    );

    // 5. Return appropriate response
    if is_htmx {
        Ok(htmx_success_redirect("/"))
    } else {
        Ok(HttpResponse::Ok().json(UserResponse::from(user)))
    }
}

/// Whoami endpoint - get current authenticated user
///
/// # Security
/// - Requires valid session
/// - Returns 401 if not authenticated
#[get("/whoami")]
pub async fn whoami(pool: web::Data<DbPool>, session: Session) -> Result<HttpResponse, ApiError> {
    // 1. Extract user_id from session
    let user_id: String = session
        .get("user_id")
        .context("Failed to read session")
        .map_err(|e| {
            warn!(error = %e, "Session read error");
            ApiError::Internal("Session error".to_string())
        })?
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))?;

    // 2. Load user from database
    let mut conn = pool
        .get()
        .context("Failed to get database connection")
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let user_id_for_lookup = user_id.clone();
    let user_id_for_warn = user_id.clone();
    let user = web::block(move || User::find_by_id(&mut conn, user_id_for_lookup))
        .await
        .context("Database query failed")
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map_err(|_| {
            warn!(
                user_id = %user_id_for_warn,
                "Session refers to non-existent user"
            );
            ApiError::Unauthorized("Invalid session".to_string())
        })?;

    // 3. Return user info
    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

/// Logout endpoint - clear session
#[post("/logout")]
pub async fn logout(session: Session) -> Result<HttpResponse, ApiError> {
    // Extract user_id for logging before clearing session
    let user_id: Option<String> = session.get("user_id").unwrap_or(None);

    // Clear session
    session.clear();

    if let Some(user_id) = user_id {
        info!(
            user_id = %user_id,
            "User logged out successfully"
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// POST /api/settings/update-wallet - Update user's Monero wallet address
#[derive(Debug, Deserialize)]
pub struct UpdateWalletRequest {
    pub wallet_address: String,
    pub csrf_token: String,
}

#[post("/update-wallet")]
pub async fn update_wallet_address(
    pool: web::Data<DbPool>,
    req: web::Form<UpdateWalletRequest>,
    http_req: HttpRequest,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    use diesel::prelude::*;
    use crate::schema::users;

    let is_htmx = is_htmx_request(&http_req);

    // Require authentication
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            return if is_htmx {
                Ok(htmx_error_response("Not authenticated"))
            } else {
                Err(ApiError::Unauthorized("Not authenticated".to_string()))
            };
        }
    };

    // Validate CSRF token
    if !validate_csrf_token(&session, &req.csrf_token) {
        return if is_htmx {
            Ok(htmx_error_response("Invalid CSRF token"))
        } else {
            Err(ApiError::Forbidden("Invalid CSRF token".to_string()))
        };
    }

    // Validate wallet address format
    if !is_valid_monero_address(&req.wallet_address) {
        return if is_htmx {
            Ok(htmx_error_response("Invalid Monero address format. Must start with 4 or 8 and be 95-106 characters."))
        } else {
            Err(ApiError::Internal("Invalid Monero address format".to_string()))
        };
    }

    // Update wallet address in database
    let mut conn = pool.get().map_err(|e| ApiError::Internal(e.to_string()))?;

    let wallet_addr = req.wallet_address.clone();
    let uid = user_id.clone();

    let update_result = web::block(move || {
        diesel::update(users::table.filter(users::id.eq(&uid)))
            .set(users::wallet_address.eq(Some(&wallet_addr)))
            .execute(&mut conn)
    }).await;

    match update_result {
        Ok(Ok(_)) => {
            info!(
                user_id = %user_id,
                "Wallet address updated successfully"
            );

            if is_htmx {
                Ok(HttpResponse::Ok().content_type("text/html").body(""))
            } else {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Wallet address updated successfully"
                })))
            }
        }
        Ok(Err(e)) => {
            error!("Failed to update wallet address: {}", e);
            if is_htmx {
                Ok(htmx_error_response("Failed to update wallet address"))
            } else {
                Err(ApiError::Internal("Failed to update wallet address".to_string()))
            }
        }
        Err(e) => {
            error!("Database operation failed: {}", e);
            if is_htmx {
                Ok(htmx_error_response("Database error"))
            } else {
                Err(ApiError::Internal(e.to_string()))
            }
        }
    }
}
