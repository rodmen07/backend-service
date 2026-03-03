use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde_json::json;
use sqlx::{QueryBuilder, Sqlite};

use crate::app_state::AppState;
use crate::models::{CreateTaskRequest, ListTasksQuery, Task, UpdateTaskRequest};
use crate::validation::{TitleValidationError, normalize_search_query, validate_title};

use super::shared::{error_response, resolved_pagination};

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
