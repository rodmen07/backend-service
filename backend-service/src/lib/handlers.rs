//! HTTP handlers for task and health endpoints.
//!
//! This module demonstrates common backend patterns in Rust:
//! extraction, validation, SQL composition, and response shaping.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use sqlx::{QueryBuilder, Sqlite};

use crate::app_state::AppState;
use crate::models::{
    ApiError, CreateTaskRequest, HealthResponse, ListTasksQuery, Task, UpdateTaskRequest,
};
use crate::validation::{TitleValidationError, normalize_search_query, validate_title};

/// Returns a lightweight server health response.
///
/// # Parameters
/// - None.
///
/// # Returns
/// - `200 OK` with `{ "status": "ok" }` JSON payload.
///
/// # Semantics
/// - Liveness only: this endpoint indicates process availability.
pub(crate) async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

/// Returns readiness status by checking database availability.
///
/// # Parameters
/// - `state`: Shared app state containing DB pool.
///
/// # Returns
/// - `200 OK` with `{ "status": "ready" }` when DB query succeeds.
/// - `503 Service Unavailable` with standard error envelope when DB check fails.
///
/// # Semantics
/// - Readiness: indicates service can process requests requiring the database.
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

/// Lists tasks with optional pagination and filtering.
///
/// # Parameters
/// - `state`: Shared app state containing DB pool.
/// - `params`: Query string fields:
///   - `limit` (default 50, clamped to 1..=100)
///   - `offset` (default 0)
///   - `completed` (`true`/`false`)
///   - `q` (substring search over title)
///
/// # Returns
/// - `200 OK` with a JSON array of tasks.
/// - `500 Internal Server Error` when query execution fails.
pub(crate) async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let (limit, offset) = resolved_pagination(&params);

    let mut query_builder = QueryBuilder::<Sqlite>::new("SELECT id, title, completed FROM tasks");

    let mut has_where_clause = false;

    if let Some(completed) = params.completed {
        query_builder
            .push(" WHERE completed = ")
            .push_bind(completed);
        has_where_clause = true;
    }

    if let Some(search) = params.q.as_deref().and_then(normalize_search_query) {
        if has_where_clause {
            query_builder.push(" AND ");
        } else {
            query_builder.push(" WHERE ");
        }

        query_builder
            .push("title LIKE ")
            .push_bind(format!("%{search}%"));
    }

    query_builder
        .push(" ORDER BY id ASC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    match query_builder
        .build_query_as::<Task>()
        .fetch_all(&state.pool)
        .await
    {
        Ok(tasks) => Json(tasks).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_LIST_TASKS_FAILED",
            "failed to list tasks",
            None,
        ),
    }
}

/// Creates a new task.
///
/// # Parameters
/// - `state`: Shared app state containing DB pool.
/// - `payload`: JSON body with a required `title`.
///
/// # Returns
/// - `201 Created` with the newly created task JSON.
/// - `400 Bad Request` when title is missing/blank or exceeds max length.
/// - `500 Internal Server Error` when insert/fetch fails.
pub(crate) async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let title = match validate_title(&payload.title) {
        Ok(title) => title,
        Err(TitleValidationError::Empty) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "VALIDATION_TITLE_REQUIRED",
                "title is required",
                None,
            );
        }
        Err(TitleValidationError::TooLong { max, actual }) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "VALIDATION_TITLE_TOO_LONG",
                "title exceeds maximum length",
                Some(json!({ "max": max, "actual": actual })),
            );
        }
    };

    let insert_result = sqlx::query("INSERT INTO tasks (title, completed) VALUES (?, ?)")
        .bind(&title)
        .bind(false)
        .execute(&state.pool)
        .await;

    let Ok(result) = insert_result else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_CREATE_TASK_FAILED",
            "failed to create task",
            None,
        );
    };

    let task_id = result.last_insert_rowid();
    let fetch_result =
        sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(&state.pool)
            .await;

    match fetch_result {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_FETCH_CREATED_TASK_FAILED",
            "failed to load created task",
            None,
        ),
    }
}

/// Updates an existing task by ID.
///
/// # Parameters
/// - `id`: Path ID of the task to update.
/// - `state`: Shared app state containing DB pool.
/// - `payload`: Optional `title` and/or `completed` fields.
///
/// # Returns
/// - `200 OK` with updated task JSON.
/// - `400 Bad Request` when provided title is blank or exceeds max length.
/// - `404 Not Found` when task ID does not exist.
/// - `500 Internal Server Error` on DB failures.
pub(crate) async fn update_task(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    let Ok(existing_task) = existing else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_LOAD_TASK_FOR_UPDATE_FAILED",
            "failed to update task",
            None,
        );
    };

    let Some(mut task) = existing_task else {
        return error_response(
            StatusCode::NOT_FOUND,
            "TASK_NOT_FOUND",
            "task not found",
            None,
        );
    };

    if let Some(title) = payload.title.as_deref() {
        let trimmed = match validate_title(title) {
            Ok(valid_title) => valid_title,
            Err(TitleValidationError::Empty) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_TITLE_REQUIRED",
                    "title cannot be empty",
                    None,
                );
            }
            Err(TitleValidationError::TooLong { max, actual }) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_TITLE_TOO_LONG",
                    "title exceeds maximum length",
                    Some(json!({ "max": max, "actual": actual })),
                );
            }
        };
        task.title = trimmed;
    }

    if let Some(completed) = payload.completed {
        task.completed = completed;
    }

    let update_result = sqlx::query("UPDATE tasks SET title = ?, completed = ? WHERE id = ?")
        .bind(&task.title)
        .bind(task.completed)
        .bind(task.id)
        .execute(&state.pool)
        .await;

    match update_result {
        Ok(_) => (StatusCode::OK, Json(task)).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_UPDATE_TASK_FAILED",
            "failed to update task",
            None,
        ),
    }
}

/// Deletes a task by ID.
///
/// # Parameters
/// - `id`: Path ID of the task to delete.
/// - `state`: Shared app state containing DB pool.
///
/// # Returns
/// - `204 No Content` when deletion succeeds.
/// - `404 Not Found` when task ID does not exist.
/// - `500 Internal Server Error` on DB failures.
pub(crate) async fn delete_task(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;

    let Ok(result) = result else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_DELETE_TASK_FAILED",
            "failed to delete task",
            None,
        );
    };

    if result.rows_affected() == 0 {
        return error_response(
            StatusCode::NOT_FOUND,
            "TASK_NOT_FOUND",
            "task not found",
            None,
        );
    }

    StatusCode::NO_CONTENT.into_response()
}

/// Builds a standardized JSON error envelope and converts it into an HTTP response.
///
/// # Parameters
/// - `status`: HTTP status code.
/// - `code`: Stable machine-readable error code.
/// - `message`: Human-readable message.
/// - `details`: Optional structured details object.
///
/// # Returns
/// - `Response` containing the status code and JSON body.
fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> Response {
    (
        status,
        Json(ApiError {
            code: code.to_string(),
            message: message.to_string(),
            details,
        }),
    )
        .into_response()
}

/// Resolves pagination defaults and safety bounds.
///
/// # Parameters
/// - `params`: Incoming query parameters.
///
/// # Returns
/// - Tuple `(limit, offset)` where limit is clamped to `1..=100`.
fn resolved_pagination(params: &ListTasksQuery) -> (u32, u32) {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);
    (limit, offset)
}

#[cfg(test)]
mod tests {
    use super::resolved_pagination;
    use crate::models::ListTasksQuery;

    /// Verifies default pagination values when no parameters are provided.
    #[test]
    fn resolved_pagination_defaults() {
        let params = ListTasksQuery::default();
        assert_eq!(resolved_pagination(&params), (50, 0));
    }

    /// Verifies that requested limit values above max are clamped.
    #[test]
    fn resolved_pagination_clamps_limit() {
        let params = ListTasksQuery {
            limit: Some(500),
            offset: Some(3),
            completed: None,
            q: None,
        };

        assert_eq!(resolved_pagination(&params), (100, 3));
    }
}
