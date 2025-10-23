//! Rate limiting middleware using actix-governor
//!
//! Provides multi-tier rate limiting:
//! - Global: 100 requests/minute per IP
//! - Auth endpoints: 5 requests/15 minutes per IP
//! - Protected endpoints: 60 requests/minute per IP
//!
//! This prevents:
//! - DDoS attacks
//! - Brute-force login attempts
//! - Account enumeration
//! - Resource exhaustion

use actix_governor::{
    governor::middleware::NoOpMiddleware, Governor, GovernorConfigBuilder, KeyExtractor,
};
use actix_web::dev::ServiceRequest;

/// Custom key extractor that works in both test and production environments
///
/// In production: Extracts peer IP address
/// In test: Uses a fixed test key to avoid peer address requirement
#[derive(Clone, Default)]
pub struct TestCompatibleKeyExtractor;

impl KeyExtractor for TestCompatibleKeyExtractor {
    type Key = String;
    type KeyExtractionError = Box<dyn std::error::Error + 'static>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // Extract peer IP if available, otherwise use default test key
        // This allows tests to work without peer addresses while still
        // providing IP-based rate limiting in production
        Ok(req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "test-127.0.0.1".to_string()))
    }
}

/// Create global rate limiter (100 requests/minute per IP)
///
/// Applied to all endpoints as baseline DDoS protection.
///
/// # Returns
/// Governor middleware configured for 100 req/min
///
/// # Panics
/// This function will panic if rate limiter configuration is invalid.
/// This is acceptable as it's a startup-time configuration error.
pub fn global_rate_limiter() -> Governor<TestCompatibleKeyExtractor, NoOpMiddleware> {
    let config = GovernorConfigBuilder::default()
        .per_second(2) // ~120 per minute
        .burst_size(100) // Allow bursts up to 100
        .key_extractor(TestCompatibleKeyExtractor)
        .finish()
        .unwrap(); // Safe: static configuration, panics are acceptable at startup

    Governor::new(&config)
}

/// Create auth rate limiter (5 requests/15 minutes per IP)
///
/// Applied to login/register endpoints to prevent brute-force attacks.
///
/// # Security Considerations
/// - 5 attempts = reasonable for legitimate users (typos, forgotten password)
/// - 15 minute window = long enough to frustrate attackers
/// - Per-IP tracking = prevents distributed attacks from single source
///
/// # Note
/// For production, consider:
/// - Tracking failed attempts per username (requires database)
/// - CAPTCHA after 3 failed attempts
/// - Email notifications on repeated failures
///
/// # Returns
/// Governor middleware configured for 5 req/15min
///
/// # Panics
/// This function will panic if rate limiter configuration is invalid.
/// This is acceptable as it's a startup-time configuration error.
pub fn auth_rate_limiter() -> Governor<TestCompatibleKeyExtractor, NoOpMiddleware> {
    let config = GovernorConfigBuilder::default()
        .burst_size(5) // Maximum 5 requests
        .period(std::time::Duration::from_secs(900)) // Per 15-minute window
        .key_extractor(TestCompatibleKeyExtractor)
        .finish()
        .unwrap(); // Safe: static configuration, panics are acceptable at startup

    Governor::new(&config)
}

/// Create protected endpoint rate limiter (60 requests/minute per IP)
///
/// Applied to authenticated endpoints that perform expensive operations.
///
/// # Returns
/// Governor middleware configured for 60 req/min
///
/// # Panics
/// This function will panic if rate limiter configuration is invalid.
/// This is acceptable as it's a startup-time configuration error.
pub fn protected_rate_limiter() -> Governor<TestCompatibleKeyExtractor, NoOpMiddleware> {
    let config = GovernorConfigBuilder::default()
        .per_second(1) // ~60 per minute
        .burst_size(60)
        .key_extractor(TestCompatibleKeyExtractor)
        .finish()
        .unwrap(); // Safe: static configuration, panics are acceptable at startup

    Governor::new(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        // Verify rate limiters can be created without panicking
        let _global = global_rate_limiter();
        let _auth = auth_rate_limiter();
        let _protected = protected_rate_limiter();
    }
}
