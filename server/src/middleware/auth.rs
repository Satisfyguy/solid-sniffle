//! Authentication middleware for protected endpoints
//!
//! Provides RequireAuth middleware that:
//! - Validates session exists
//! - Loads user from database
//! - Attaches user to request extensions
//! - Returns 401 if not authenticated
//! - Optionally checks user role

use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use anyhow::Context;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use tracing::warn;

use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::user::User;

/// Middleware that requires authentication
///
/// # Usage
/// ```rust
/// use actix_web::web;
/// use server::middleware::auth::RequireAuth;
///
/// web::resource("/api/orders")
///     .wrap(RequireAuth)
///     .route(web::get().to(get_orders))
/// ```
///
/// # Behavior
/// 1. Extracts session from request
/// 2. Validates user_id exists in session
/// 3. Loads user from database
/// 4. Attaches user to request extensions
/// 5. Calls next service
/// 6. Returns 401 Unauthorized if any step fails
///
/// # Access User in Handler
/// ```rust
/// use actix_web::{web, HttpRequest, HttpResponse};
/// use server::models::user::User;
///
/// async fn protected_handler(req: HttpRequest) -> Result<HttpResponse, ApiError> {
///     let user = req.extensions()
///         .get::<User>()
///         .ok_or_else(|| ApiError::Internal("User not found in extensions".to_string()))?
///         .clone();
///     Ok(HttpResponse::Ok().json(user))
/// }
/// ```
pub struct RequireAuth;

impl<S, B> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // 1. Extract session from request
            let session = req.get_session();

            // 2. Get user_id from session
            let user_id: String = session
                .get("user_id")
                .context("Failed to read session")
                .map_err(|e| {
                    warn!(error = %e, "Session read error");
                    ApiError::Internal("Session error".to_string())
                })?
                .ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

            // 3. Get database pool from app data
            let pool = req
                .app_data::<actix_web::web::Data<DbPool>>()
                .ok_or_else(|| {
                    warn!("Database pool not found in app data");
                    ApiError::Internal("Database configuration error".to_string())
                })?;

            // 4. Load user from database
            let mut conn = pool
                .get()
                .context("Failed to get database connection")
                .map_err(|e| {
                    warn!(error = %e, "Database connection error");
                    ApiError::Internal("Database error".to_string())
                })?;

            let user_id_for_lookup = user_id.clone();
            let user_id_for_warn = user_id.clone();
            let user =
                actix_web::web::block(move || User::find_by_id(&mut conn, user_id_for_lookup))
                    .await
                    .context("Database query failed")
                    .map_err(|e| {
                        warn!(error = %e, "User lookup failed");
                        ApiError::Internal("Database error".to_string())
                    })?
                    .map_err(|_| {
                        warn!(user_id = %user_id_for_warn, "Session refers to non-existent user");
                        ApiError::Unauthorized("Invalid session".to_string())
                    })?;

            // 5. Attach user to request extensions
            req.extensions_mut().insert(user);

            // 6. Call next service
            svc.call(req).await
        })
    }
}

/// Middleware that requires specific role
///
/// # Usage
/// ```rust
/// use actix_web::web;
/// use server::middleware::auth::RequireRole;
///
/// web::resource("/api/admin/users")
///     .wrap(RequireRole::new("admin"))
///     .route(web::get().to(list_users))
/// ```
pub struct RequireRole {
    required_role: String,
}

impl RequireRole {
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            required_role: role.into(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireRoleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireRoleMiddleware {
            service: Rc::new(service),
            required_role: self.required_role.clone(),
        }))
    }
}

pub struct RequireRoleMiddleware<S> {
    service: Rc<S>,
    required_role: String,
}

impl<S, B> Service<ServiceRequest> for RequireRoleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let required_role = self.required_role.clone();

        Box::pin(async move {
            // First, run RequireAuth logic
            let session = req.get_session();

            let user_id: String = session
                .get("user_id")
                .context("Failed to read session")
                .map_err(|e| {
                    warn!(error = %e, "Session read error");
                    ApiError::Internal("Session error".to_string())
                })?
                .ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

            let pool = req
                .app_data::<actix_web::web::Data<DbPool>>()
                .ok_or_else(|| {
                    warn!("Database pool not found in app data");
                    ApiError::Internal("Database configuration error".to_string())
                })?;

            let mut conn = pool
                .get()
                .context("Failed to get database connection")
                .map_err(|e| {
                    warn!(error = %e, "Database connection error");
                    ApiError::Internal("Database error".to_string())
                })?;

            let user_id_for_lookup = user_id.clone();
            let user_id_for_warn = user_id.clone();
            let user =
                actix_web::web::block(move || User::find_by_id(&mut conn, user_id_for_lookup))
                    .await
                    .context("Database query failed")
                    .map_err(|e| {
                        warn!(error = %e, "User lookup failed");
                        ApiError::Internal("Database error".to_string())
                    })?
                    .map_err(|_| {
                        warn!(user_id = %user_id_for_warn, "Session refers to non-existent user");
                        ApiError::Unauthorized("Invalid session".to_string())
                    })?;

            // Check role
            if user.role != required_role {
                warn!(
                    user_id = %user.id,
                    user_role = %user.role,
                    required_role = %required_role,
                    "Insufficient permissions"
                );
                return Err(ApiError::Forbidden(format!("Requires {} role", required_role)).into());
            }

            // Attach user to request extensions
            req.extensions_mut().insert(user);

            // Call next service
            svc.call(req).await
        })
    }
}
