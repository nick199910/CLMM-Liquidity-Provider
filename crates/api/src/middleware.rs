//! Middleware components.

use crate::handlers::health::{increment_error_count, increment_request_count};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// API key authentication middleware.
pub async fn api_key_auth(
    api_keys: Arc<HashSet<String>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health endpoints
    let path = request.uri().path();
    if path.starts_with("/health") || path == "/metrics" {
        return Ok(next.run(request).await);
    }

    // Check for API key in header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if api_keys.is_empty() || api_keys.contains(key) => Ok(next.run(request).await),
        Some(_) => {
            warn!("Invalid API key");
            Err(StatusCode::UNAUTHORIZED)
        }
        None if api_keys.is_empty() => Ok(next.run(request).await),
        None => {
            warn!("Missing API key");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Request logging middleware.
pub async fn request_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    increment_request_count();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        increment_error_count();
    }

    debug!(
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

/// Rate limiting state.
#[derive(Clone)]
pub struct RateLimiter {
    /// Request counts per IP.
    counts: Arc<RwLock<std::collections::HashMap<String, (u32, std::time::Instant)>>>,
    /// Maximum requests per minute.
    max_requests: u32,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    pub fn new(max_requests: u32) -> Self {
        Self {
            counts: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_requests,
        }
    }

    /// Checks if a request is allowed.
    pub async fn check(&self, ip: &str) -> bool {
        let mut counts = self.counts.write().await;
        let now = std::time::Instant::now();

        // Clean up old entries
        counts.retain(|_, (_, time)| now.duration_since(*time).as_secs() < 60);

        let entry = counts.entry(ip.to_string()).or_insert((0, now));

        // Reset if more than a minute has passed
        if now.duration_since(entry.1).as_secs() >= 60 {
            *entry = (0, now);
        }

        if entry.0 >= self.max_requests {
            false
        } else {
            entry.0 += 1;
            true
        }
    }
}

/// Rate limiting middleware.
pub async fn rate_limit(
    rate_limiter: Arc<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get client IP (simplified - in production would check X-Forwarded-For)
    let ip = "default".to_string();

    if rate_limiter.check(&ip).await {
        Ok(next.run(request).await)
    } else {
        warn!(ip = %ip, "Rate limit exceeded");
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
