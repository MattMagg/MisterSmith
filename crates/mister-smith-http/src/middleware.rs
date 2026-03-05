//! HTTP middleware: request ID tracking and rate limiting.
//!
//! - **Request ID**: Generates a UUID v4 `X-Request-Id` header for each request,
//!   or preserves a client-provided one.
//! - **Rate limiting**: Per-IP rate limiting with configurable RPS, returning
//!   429 with `Retry-After` header when exceeded.
//! - **Security hooks**: Pass-through placeholder for Phase 5 auth enforcement.

use axum::extract::ConnectInfo;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Header name for request ID tracking.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID middleware.
///
/// If the incoming request has an `X-Request-Id` header, it is preserved.
/// Otherwise, a new UUID v4 is generated and attached to the response.
pub async fn request_id_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    response
}

/// Per-IP request tracking entry.
struct RateLimitEntry {
    /// Timestamps of recent requests within the current window.
    request_times: Vec<Instant>,
}

/// Shared rate limiter state.
pub struct RateLimiter {
    /// Maximum requests per second per IP.
    max_rps: u32,
    /// Per-IP request tracking.
    entries: Mutex<HashMap<String, RateLimitEntry>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given maximum RPS.
    pub fn new(max_rps: u32) -> Self {
        Self {
            max_rps,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a request from the given IP should be allowed.
    ///
    /// Returns `true` if the request is allowed, `false` if rate limited.
    pub async fn check(&self, ip: &str) -> bool {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();
        let window = std::time::Duration::from_secs(1);

        let entry = entries.entry(ip.to_string()).or_insert(RateLimitEntry {
            request_times: Vec::new(),
        });

        // Remove timestamps older than the window.
        entry.request_times.retain(|t| now.duration_since(*t) < window);

        if entry.request_times.len() >= self.max_rps as usize {
            return false;
        }

        entry.request_times.push(now);
        true
    }
}

/// Rate limiting middleware.
///
/// Extracts the client IP from `ConnectInfo<SocketAddr>` and checks against
/// the rate limiter. Returns 429 with `Retry-After: 1` when the limit is exceeded.
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Extract rate limiter from request extensions.
    let rate_limiter = request
        .extensions()
        .get::<Arc<RateLimiter>>()
        .cloned();

    if let Some(limiter) = rate_limiter {
        let ip = addr.ip().to_string();
        if !limiter.check(&ip).await {
            return rate_limited_response();
        }
    }

    next.run(request).await
}

/// Build a 429 Too Many Requests response.
fn rate_limited_response() -> Response {
    let body = serde_json::json!({
        "error": "rate_limited",
        "message": "Rate limit exceeded",
        "request_id": Uuid::new_v4().to_string(),
    });
    let mut response = (
        axum::http::StatusCode::TOO_MANY_REQUESTS,
        axum::Json(body),
    )
        .into_response();
    response.headers_mut().insert(
        "retry-after",
        HeaderValue::from_static("1"),
    );
    response
}

/// Security middleware placeholder for Phase 5 auth enforcement.
///
/// Currently passes all requests through without checks.
pub async fn security_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    // Phase 5: Add JWT/API key validation here.
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_header_constant() {
        assert_eq!(REQUEST_ID_HEADER, "x-request-id");
    }

    #[tokio::test]
    async fn rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(5);
        for _ in 0..5 {
            assert!(limiter.check("127.0.0.1").await);
        }
    }

    #[tokio::test]
    async fn rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3);
        for _ in 0..3 {
            assert!(limiter.check("127.0.0.1").await);
        }
        // 4th request should be blocked.
        assert!(!limiter.check("127.0.0.1").await);
    }

    #[tokio::test]
    async fn rate_limiter_per_ip_isolation() {
        let limiter = RateLimiter::new(2);
        assert!(limiter.check("10.0.0.1").await);
        assert!(limiter.check("10.0.0.1").await);
        assert!(!limiter.check("10.0.0.1").await);

        // Different IP should still be allowed.
        assert!(limiter.check("10.0.0.2").await);
    }

    #[test]
    fn rate_limited_response_status() {
        let response = rate_limited_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            response.headers().get("retry-after").unwrap(),
            "1"
        );
    }
}
