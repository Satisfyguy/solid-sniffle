//! Security headers middleware
//!
//! Adds production-grade security headers to all responses:
//! - Content-Security-Policy (CSP)
//! - X-Frame-Options
//! - X-Content-Type-Options
//! - X-XSS-Protection
//! - Strict-Transport-Security (HSTS)
//! - Referrer-Policy
//! - Permissions-Policy
//!
//! These headers protect against:
//! - XSS (Cross-Site Scripting)
//! - Clickjacking
//! - MIME sniffing attacks
//! - Information leakage
//! - Downgrade attacks

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

/// Security headers middleware
///
/// # Usage
/// ```rust
/// use actix_web::App;
/// use server::middleware::security_headers::SecurityHeaders;
///
/// App::new()
///     .wrap(SecurityHeaders)
/// ```
///
/// # Headers Added
/// - **Content-Security-Policy**: Restricts resource loading
/// - **X-Frame-Options**: Prevents clickjacking
/// - **X-Content-Type-Options**: Prevents MIME sniffing
/// - **X-XSS-Protection**: Enables browser XSS filter
/// - **Strict-Transport-Security**: Forces HTTPS (production only)
/// - **Referrer-Policy**: Controls referrer information
/// - **Permissions-Policy**: Restricts browser features
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
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
            let mut res = svc.call(req).await?;

            let headers = res.headers_mut();

            // Content-Security-Policy (CSP)
            // Restricts what resources can be loaded and from where
            // default-src 'self': Only load resources from same origin
            // script-src 'self': Only execute scripts from same origin
            // style-src 'self' 'unsafe-inline': Allow inline styles (needed for some frameworks)
            // img-src 'self' data:: Allow images from same origin and data URIs
            // font-src 'self': Only load fonts from same origin
            // connect-src 'self': Only allow AJAX/WebSocket to same origin
            // frame-ancestors 'none': Prevent embedding in iframes
            // base-uri 'self': Restrict base tag to same origin
            // form-action 'self': Only allow form submissions to same origin
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' https://unpkg.com 'sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC' 'sha256-N2pfIAEnpE2vmu77Xjadv5PP2CdREj1pDSq5RXaRbBs=' 'sha256-+Y6kiHqgDSeriw2bHq+k2bitwvZyfU77vQann9KE3/E=' 'sha256-wpeDrFg6FICFw4Rkwa/uu5SBi4lo380s9U/Iq7oVFXo=' 'sha256-Ie0hq8bifOs/4hYC8cXqdPZp5aWdg7bzB6ELrK7o4xs=' 'sha256-v3o0pJQ4U2jQZnR5twYmxhk78VnxMPhfoeSMFgjB4Nk=' 'sha256-u4Hkf3+nqAvSmcYNgW3j9lLBSmc53fKwsMjuc9h1cEM=' 'sha256-zch+b6GREm0jT2KehqZeGtaeiXFvwZTNIsF7SJ9cS3c=' 'sha256-pDjvET4xd5FPNbGQuIFbVoRMv+2z+bzHVQtCasp4pbk=' 'sha256-lolxUSgQkT0uB/gvibkkv3ggZX11uDt1lpP/XLCtLTs=' 'unsafe-hashes'; \
                     script-src-attr 'unsafe-hashes' 'sha256-U1dvsw2shH1iFgVktfXGZ2nSNr6jSs5Yw+MXYOZuM7g=' 'sha256-T66voDSUe+uVn4SqzIQDqfhDNbyGtVuZagmABOfVT2M='; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data: http://127.0.0.1:8081; \
                     font-src 'self'; \
                     connect-src 'self' ws://127.0.0.1:8080 https://unpkg.com; \
                     frame-ancestors 'none'; \
                     base-uri 'self'; \
                     form-action 'self'",
                ),
            );

            // X-Frame-Options: DENY
            // Prevents the page from being embedded in iframes (clickjacking protection)
            headers.insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );

            // X-Content-Type-Options: nosniff
            // Prevents browsers from MIME-sniffing (interpreting files as different type)
            headers.insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );

            // X-XSS-Protection: 1; mode=block
            // Enables browser's built-in XSS filter and blocks page if attack detected
            headers.insert(
                actix_web::http::header::X_XSS_PROTECTION,
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            // Strict-Transport-Security (HSTS)
            // Forces HTTPS for 1 year, includes subdomains
            // Note: Only enable in production with valid HTTPS
            // Commented out for development (localhost HTTP)
            // headers.insert(
            //     actix_web::http::header::STRICT_TRANSPORT_SECURITY,
            //     actix_web::http::header::HeaderValue::from_static(
            //         "max-age=31536000; includeSubDomains"
            //     ),
            // );

            // Referrer-Policy: no-referrer
            // Don't send referrer information (privacy protection)
            headers.insert(
                actix_web::http::header::REFERRER_POLICY,
                actix_web::http::header::HeaderValue::from_static("no-referrer"),
            );

            // Permissions-Policy
            // Restricts browser features (geolocation, camera, microphone, etc.)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "geolocation=(), \
                     microphone=(), \
                     camera=(), \
                     payment=(), \
                     usb=(), \
                     magnetometer=(), \
                     gyroscope=(), \
                     accelerometer=()",
                ),
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn test_security_headers_applied() {
        let app = test::init_service(App::new().wrap(SecurityHeaders).route(
            "/test",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify security headers are present
        let headers = resp.headers();

        assert!(headers.contains_key("content-security-policy"));
        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("x-xss-protection"));
        assert!(headers.contains_key("referrer-policy"));
        assert!(headers.contains_key("permissions-policy"));
    }
}
