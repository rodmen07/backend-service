//! Router composition for HTTP endpoints and shared middleware.
//!
//! This module wires handlers into URL paths and applies cross-cutting concerns
//! such as CORS and HTTP tracing.

use std::env;

use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, patch},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::app_state::AppState;
use crate::handlers::{
    create_task, delete_task, health, list_tasks, plan_tasks, ready, update_task,
};

/// Builds the application router with routes and middleware.
///
/// # Parameters
/// - `state`: Shared application state injected into handlers.
///
/// # Returns
/// - Configured `Router` with task, health, and readiness endpoints.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/tasks/plan", axum::routing::post(plan_tasks))
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/{id}", patch(update_task).delete(delete_task))
        .with_state(state)
        .layer(build_cors_layer())
        .layer(TraceLayer::new_for_http())
}

fn build_cors_layer() -> CorsLayer {
    let configured_origins = env::var("ALLOWED_ORIGINS")
        .ok()
        .unwrap_or_default();

    if configured_origins.trim().is_empty() {
        return CorsLayer::permissive();
    }

    let origins: Vec<HeaderValue> = configured_origins
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| HeaderValue::from_str(value).ok())
        .collect();

    if origins.is_empty() {
        return CorsLayer::permissive();
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
}
