use actix_web::{web, HttpResponse, post};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::user::{NewUser, User};

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub role: String,
}

#[post("/register")]
pub async fn register(pool: web::Data<DbPool>, req: web::Json<RegisterRequest>) -> Result<HttpResponse, ApiError> {
    req.0.validate().map_err(ApiError::from)?;

    let mut conn = pool.get().map_err(|e| ApiError::Internal(e.to_string()))?;

    // 1. Check if username exists
    let req_username = req.username.clone();
    let username_exists = web::block(move || User::username_exists(&mut conn, &req_username))
        .await??;
    if username_exists {
        return Err(ApiError::Conflict("Username already taken".to_string()));
    }

    // 2. Hash password
    let password = req.password.clone();
    let password_hash = web::block(move || -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
    })
    .await??;

    // 3. Create user
    let mut conn = pool.get().map_err(|e| ApiError::Internal(e.to_string()))?;
    let new_user = NewUser {
        id: Uuid::new_v4().to_string(),
        username: req.username.clone(),
        password_hash,
        wallet_address: None,
        wallet_id: None,
        role: req.role.clone(),
    };

    let user = web::block(move || User::create(&mut conn, new_user))
        .await??;

    Ok(HttpResponse::Created().json(user))
}
