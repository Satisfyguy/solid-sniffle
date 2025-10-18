//! Custom error types for the API

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Blocking operation failed: {0}")]
    Blocking(#[from] actix_web::error::BlockingError),
    
    #[error("Error from anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Argon2 hashing error: {0}")]
    Argon2(#[from] argon2::Error),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    status: u16,
    error: String,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Blocking(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Argon2(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_response = ErrorResponse {
            status: status.as_u16(),
            error: self.to_string(),
        };
        HttpResponse::build(status).json(error_response)
    }
}
