use axum::http::StatusCode;
use axum::{Json, extract::State, response::IntoResponse};

use crate::app_state::AppState;
use crate::models::HealthResponse;

use super::shared::error_response;

pub(crate) async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

pub(crate) async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let readiness_check = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await;

    match readiness_check {
        Ok(_) => Json(HealthResponse { status: "ready" }).into_response(),
        Err(_) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_NOT_READY",
            "database is not ready",
            None,
        ),
    }
}
