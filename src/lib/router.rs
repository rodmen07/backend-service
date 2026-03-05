//! Router composition for HTTP endpoints and shared middleware.
//!
//! This module wires handlers into URL paths and applies cross-cutting concerns
//! such as CORS and HTTP tracing.

use std::{env, time::Instant};

use axum::{
    Json,
    Router,
    extract::{Request, State},
    middleware::{Next, from_fn, from_fn_with_state},
    http::{HeaderValue, Method},
    response::{IntoResponse, Response},
    routing::{get, patch},
};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::auth::{AUTH_HEADER, AuthClaims, validate_authorization_header};
use crate::app_state::AppState;
use crate::handlers::{
    admin_backup, admin_metrics, admin_request_logs, admin_user_activity, clear_plan_tasks,
    create_task, delete_task, health, info, list_tasks, plan_tasks, ready, update_task,
};
use crate::models::ApiError;
use crate::rate_limit::rate_limit_middleware;

/// Builds the application router with routes and middleware.
pub fn build_router(state: AppState) -> Router {
    let admin_routes = Router::new()
        .route("/api/v1/admin/metrics", get(admin_metrics))
        .route("/api/v1/admin/requests", get(admin_request_logs))
        .route("/api/v1/admin/users", get(admin_user_activity))
        .route("/api/v1/admin/backup", axum::routing::post(admin_backup))
        .layer(from_fn(require_admin));

    let protected_routes = Router::new()
        .route("/api/v1/tasks/plan", axum::routing::post(plan_tasks).delete(clear_plan_tasks))
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/{id}", patch(update_task).delete(delete_task))
        .merge(admin_routes)
        .layer(from_fn(require_auth));

    Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/info", get(info))
        .merge(protected_routes)
        .layer(from_fn(rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), audit_request))
        .with_state(state)
        .layer(build_cors_layer())
        .layer(TraceLayer::new_for_http())
}

fn auth_enforced() -> bool {
    env::var("AUTH_ENFORCED")
        .ok()
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn claims_from_request(request: &Request) -> Result<AuthClaims, crate::auth::AuthError> {
    let auth_header = request
        .headers()
        .get(AUTH_HEADER)
        .and_then(|value| value.to_str().ok());

    validate_authorization_header(auth_header)
}

async fn require_auth(request: Request, next: Next) -> Response {
    if !auth_enforced() {
        return next.run(request).await;
    }

    let claims = match claims_from_request(&request) {
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

async fn require_admin(request: Request, next: Next) -> Response {
    if !auth_enforced() {
        return next.run(request).await;
    }

    let claims = match claims_from_request(&request) {
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

    if !claims.has_role("admin") {
        return (
            axum::http::StatusCode::FORBIDDEN,
            Json(ApiError {
                code: "AUTH_ADMIN_REQUIRED".to_string(),
                message: "admin role is required".to_string(),
                details: Some(json!({ "required_role": "admin" })),
            }),
        )
            .into_response();
    }

    next.run(request).await
}

async fn audit_request(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let subject = claims_from_request(&request).ok().map(|claims| claims.sub);
    let started_at = Instant::now();

    let response = next.run(request).await;

    if path.starts_with("/api/") {
        let duration_ms = started_at.elapsed().as_millis() as i64;
        let status_code = i64::from(response.status().as_u16());
        let _ = sqlx::query(
            "INSERT INTO api_request_logs (subject, method, path, status_code, duration_ms, user_agent) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(subject)
        .bind(method)
        .bind(path)
        .bind(status_code)
        .bind(duration_ms)
        .bind(user_agent)
        .execute(&state.pool)
        .await;
    }

    response
}

fn build_cors_layer() -> CorsLayer {
    let configured_origins = env::var("ALLOWED_ORIGINS").ok().unwrap_or_default();

    if configured_origins.trim() == "*" {
        tracing::warn!("CORS: ALLOWED_ORIGINS is '*' — fully permissive (not recommended for production)");
        return CorsLayer::permissive();
    }

    let origins: Vec<HeaderValue> = configured_origins
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| HeaderValue::from_str(value).ok())
        .collect();

    if origins.is_empty() {
        tracing::info!("CORS: no ALLOWED_ORIGINS configured — rejecting cross-origin requests");
        return CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(Any);
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
