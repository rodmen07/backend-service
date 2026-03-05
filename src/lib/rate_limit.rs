//! IP-based sliding-window rate limiter middleware.
//!
//! Limits the number of requests a single client IP can make within a
//! configurable time window.  Health and readiness probes are excluded so
//! that platform health-checks are never throttled.

use std::collections::HashMap;
use std::env;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::models::ApiError;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

fn env_or<T: std::str::FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

static MAX_REQUESTS: LazyLock<usize> =
    LazyLock::new(|| env_or("RATE_LIMIT_MAX_REQUESTS", 60));

static WINDOW_SECS: LazyLock<u64> =
    LazyLock::new(|| env_or("RATE_LIMIT_WINDOW_SECONDS", 60));

// ---------------------------------------------------------------------------
// Limiter state
// ---------------------------------------------------------------------------

struct Bucket {
    timestamps: Vec<Instant>,
}

static BUCKETS: LazyLock<Mutex<HashMap<String, Bucket>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn is_allowed(key: &str) -> bool {
    let now = Instant::now();
    let window = Duration::from_secs(*WINDOW_SECS);
    let cutoff = now - window;

    let mut map = BUCKETS.lock().unwrap_or_else(|e| e.into_inner());
    let bucket = map.entry(key.to_owned()).or_insert_with(|| Bucket {
        timestamps: Vec::new(),
    });

    bucket.timestamps.retain(|t| *t > cutoff);

    if bucket.timestamps.len() >= *MAX_REQUESTS {
        return false;
    }

    bucket.timestamps.push(now);
    true
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn client_ip(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned())
}

// ---------------------------------------------------------------------------
// Axum middleware
// ---------------------------------------------------------------------------

/// Middleware that enforces per-IP rate limiting on API routes.
///
/// Health (`/`, `/health`) and readiness (`/ready`) endpoints are exempt so
/// that platform probes are never rejected.
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path();

    // Exempt health / readiness probes from rate limiting.
    if path == "/" || path == "/health" || path == "/ready" {
        return next.run(request).await;
    }

    let ip = client_ip(&request);

    if !is_allowed(&ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiError {
                code: "RATE_LIMIT_EXCEEDED".to_string(),
                message: "too many requests — try again later".to_string(),
                details: None,
            }),
        )
            .into_response();
    }

    next.run(request).await
}
