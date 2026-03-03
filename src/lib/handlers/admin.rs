use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use sqlx::FromRow;

use crate::app_state::AppState;
use crate::models::{
    AdminListQuery, AdminMetricsResponse, AdminRequestLog, AdminUserActivity,
};

use super::shared::error_response;

#[derive(Debug, FromRow)]
struct TaskCounts {
    total_tasks: i64,
    completed_tasks: i64,
}

#[derive(Debug, FromRow)]
struct RequestCounts {
    total_requests: i64,
    unique_subjects: i64,
}

pub(crate) async fn admin_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let task_counts = sqlx::query_as::<_, TaskCounts>(
        "SELECT COUNT(*) as total_tasks, SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed_tasks FROM tasks",
    )
    .fetch_one(&state.pool)
    .await;

    let Ok(task_counts) = task_counts else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ADMIN_METRICS_TASKS_FAILED",
            "failed to load admin task metrics",
            None,
        );
    };

    let request_counts = sqlx::query_as::<_, RequestCounts>(
        "SELECT COUNT(*) as total_requests, COUNT(DISTINCT subject) as unique_subjects FROM api_request_logs",
    )
    .fetch_one(&state.pool)
    .await;

    let Ok(request_counts) = request_counts else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ADMIN_METRICS_REQUESTS_FAILED",
            "failed to load admin request metrics",
            None,
        );
    };

    let pending_tasks = task_counts.total_tasks - task_counts.completed_tasks;

    Json(AdminMetricsResponse {
        total_tasks: task_counts.total_tasks,
        completed_tasks: task_counts.completed_tasks,
        pending_tasks,
        total_requests: request_counts.total_requests,
        unique_subjects: request_counts.unique_subjects,
    })
    .into_response()
}

pub(crate) async fn admin_request_logs(
    State(state): State<AppState>,
    Query(params): Query<AdminListQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);

    let records = sqlx::query_as::<_, AdminRequestLog>(
        "SELECT id, occurred_at, subject, method, path, status_code, duration_ms, user_agent
         FROM api_request_logs
         ORDER BY id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    match records {
        Ok(rows) => Json(rows).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ADMIN_REQUEST_LOGS_FAILED",
            "failed to list request logs",
            None,
        ),
    }
}

pub(crate) async fn admin_user_activity(
    State(state): State<AppState>,
    Query(params): Query<AdminListQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);

    let records = sqlx::query_as::<_, AdminUserActivity>(
        "SELECT
            subject,
            COUNT(*) as request_count,
            MIN(occurred_at) as first_seen_at,
            MAX(occurred_at) as last_seen_at
         FROM api_request_logs
         WHERE subject IS NOT NULL AND TRIM(subject) <> ''
         GROUP BY subject
         ORDER BY last_seen_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    match records {
        Ok(rows) => Json(rows).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ADMIN_USER_ACTIVITY_FAILED",
            "failed to list user activity",
            None,
        ),
    }
}