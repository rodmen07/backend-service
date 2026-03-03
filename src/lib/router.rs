//! Router composition for HTTP endpoints and shared middleware.
//!
//! This module wires handlers into URL paths and applies cross-cutting concerns
//! such as CORS and HTTP tracing.

use std::env;

use axum::{
    Json,
    Router,
    extract::Request,
    response::IntoResponse,
    middleware::{Next, from_fn},
    http::{HeaderValue, Method},
    response::Response,
    routing::{get, patch},
};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::auth::{AUTH_HEADER, validate_authorization_header};
use crate::app_state::AppState;
use crate::handlers::{
    create_task, delete_task, health, list_tasks, plan_tasks, ready, update_task,
};
use crate::models::ApiError;

/// Builds the application router with routes and middleware.
///
/// # Parameters
/// - `state`: Shared application state injected into handlers.
///
/// # Returns
/// - Configured `Router` with task, health, and readiness endpoints.
pub fn build_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/api/v1/tasks/plan", axum::routing::post(plan_tasks))
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/{id}", patch(update_task).delete(delete_task))
        .layer(from_fn(require_auth));

    Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/ready", get(ready))
        .merge(protected_routes)
        .with_state(state)
        .layer(build_cors_layer())
        .layer(TraceLayer::new_for_http())
}

async fn require_auth(request: Request, next: Next) -> Response {
    let auth_enforced = env::var("AUTH_ENFORCED")
        .ok()
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !auth_enforced {
        return next.run(request).await;
    }

    let auth_header = request
        .headers()
        .get(AUTH_HEADER)
        .and_then(|value| value.to_str().ok());

    let claims = match validate_authorization_header(auth_header) {
        Ok(value) => value,
        Err(error) => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: error.code().to_string(),
                    message: error.message().to_string(),
                    details: Some(json!({ "required": "Authorization: Bearer <token>" })),
                }),
            )
                .into_response();
        }
    };

    if claims.sub.trim().is_empty() {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "AUTH_INVALID_TOKEN".to_string(),
                message: "token subject is missing".to_string(),
                details: None,
            }),
        )
            .into_response();
    }

    next.run(request).await
}

fn build_cors_layer() -> CorsLayer {
    let configured_origins = env::var("ALLOWED_ORIGINS").ok().unwrap_or_default();

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
