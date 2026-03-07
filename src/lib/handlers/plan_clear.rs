use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde_json::json;

use crate::app_state::AppState;
use crate::models::ClearPlanQuery;

use super::shared::error_response;

/// Delete all AI-generated tasks (`source = 'ai_generated'`) associated with
/// the given goal.  Manually created tasks are never touched.
///
/// Query parameter: `goal` (required, non-empty).
pub(crate) async fn clear_plan_tasks(
    State(state): State<AppState>,
    Query(params): Query<ClearPlanQuery>,
) -> impl IntoResponse {
    let goal = params.goal.trim().to_string();

    if goal.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_GOAL_REQUIRED",
            "goal query parameter is required",
            None,
        );
    }

    let result = sqlx::query(
        "DELETE FROM tasks WHERE source = 'ai_generated' AND goal = ?",
    )
    .bind(&goal)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) => Json(json!({ "deleted": r.rows_affected(), "goal": goal })).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_CLEAR_PLAN_FAILED",
            "failed to clear AI-generated tasks",
            None,
        ),
    }
}
