//! Middleware for the Monero Marketplace API
//!
//! Provides production-grade middleware:
//! - Rate limiting (DDoS protection, brute-force prevention)
//! - Authentication (RequireAuth for protected endpoints)
//! - Admin authentication (AdminAuth for /admin/* endpoints)
//! - Security headers (CSP, X-Frame-Options, etc.)
//! - CSRF protection (token-based validation)

pub mod admin_auth;
pub mod auth;
pub mod csrf;
pub mod rate_limit;
pub mod security_headers;
