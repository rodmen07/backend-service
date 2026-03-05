//! IP-based sliding-window rate limiter middleware.
//!
//! Two limiters are provided:
//! - General: applied to all non-health API requests (default 60 req/60 s).
//! - Plan:    applied only to the AI planning endpoint (default 5 req/300 s).
//!
//! Health and readiness probes are excluded so that platform health-checks
//! are never throttled.

use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{ConnectInfo, Request},
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

/// Stricter limits for the expensive AI planning endpoint.
static PLAN_MAX_REQUESTS: LazyLock<usize> =
    LazyLock::new(|| env_or("PLAN_RATE_LIMIT_MAX_REQUESTS", 5));

static PLAN_WINDOW_SECS: LazyLock<u64> =
    LazyLock::new(|| env_or("PLAN_RATE_LIMIT_WINDOW_SECONDS", 300));

// ---------------------------------------------------------------------------
// Limiter state
// ---------------------------------------------------------------------------

struct Bucket {
    timestamps: Vec<Instant>,
}

static BUCKETS: LazyLock<Mutex<HashMap<String, Bucket>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PLAN_BUCKETS: LazyLock<Mutex<HashMap<String, Bucket>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn check_bucket(
    map: &mut HashMap<String, Bucket>,
    key: &str,
    max: usize,
    window: Duration,
) -> bool {
    let now = Instant::now();
    let cutoff = now - window;

    let bucket = map.entry(key.to_owned()).or_insert_with(|| Bucket {
        timestamps: Vec::new(),
    });

    bucket.timestamps.retain(|t| *t > cutoff);

    if bucket.timestamps.len() >= max {
        return false;
    }

    bucket.timestamps.push(now);
    true
}

fn is_allowed(key: &str) -> bool {
    let mut map = BUCKETS.lock().unwrap_or_else(|e| e.into_inner());
    check_bucket(&mut map, key, *MAX_REQUESTS, Duration::from_secs(*WINDOW_SECS))
}

/// Check and consume one token from the plan-specific rate limit bucket.
/// Returns `true` if the request should be allowed.
pub fn is_plan_allowed(key: &str) -> bool {
    let mut map = PLAN_BUCKETS.lock().unwrap_or_else(|e| e.into_inner());
    check_bucket(&mut map, key, *PLAN_MAX_REQUESTS, Duration::from_secs(*PLAN_WINDOW_SECS))
}

// ---------------------------------------------------------------------------
// IP extraction
// ---------------------------------------------------------------------------

/// Best-effort client IP: X-Forwarded-For first hop → direct peer address.
pub fn client_ip(request: &Request) -> String {
    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_owned())
    {
        return forwarded;
    }

    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
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
