//! Router composition for HTTP endpoints and shared middleware.
//!
//! This module wires handlers into URL paths and applies cross-cutting concerns
//! such as CORS and HTTP tracing.

use axum::{
    Router,
    routing::{get, patch},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

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
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
