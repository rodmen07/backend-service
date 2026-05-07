use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::app_state::AppState;
use crate::auth::{AUTH_HEADER, validate_authorization_header};
use crate::models::{CreateCommentRequest, TaskComment, UpdateCommentRequest};

use super::shared::error_response;

const COMMENT_MAX_BODY_LEN: usize = 2000;

pub(crate) async fn list_comments(
    Path(task_id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let task_exists = sqlx::query_scalar::<_, i64>("SELECT id FROM tasks WHERE id = $1")
        .bind(task_id)
        .fetch_optional(&state.pool)
        .await;

    match task_exists {
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                "TASK_NOT_FOUND",
                "task not found",
                None,
            );
        }
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "database error",
                None,
            );
        }
        Ok(Some(_)) => {}
    }

    match sqlx::query_as::<_, TaskComment>(
        "SELECT id, task_id, author_id, body, created_at, updated_at FROM task_comments WHERE task_id = $1 ORDER BY created_at ASC",
    )
    .bind(task_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(comments) => Json(comments).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_LIST_COMMENTS_FAILED",
            "failed to list comments",
            None,
        ),
    }
}

pub(crate) async fn create_comment(
    Path(task_id): Path<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    let task_exists = sqlx::query_scalar::<_, i64>("SELECT id FROM tasks WHERE id = $1")
        .bind(task_id)
        .fetch_optional(&state.pool)
        .await;

    match task_exists {
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                "TASK_NOT_FOUND",
                "task not found",
                None,
            );
        }
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "database error",
                None,
            );
        }
        Ok(Some(_)) => {}
    }

    let body = payload.body.trim().to_string();
    if body.is_empty() {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "COMMENT_BODY_REQUIRED",
            "comment body is required",
            None,
        );
    }
    if body.len() > COMMENT_MAX_BODY_LEN {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "COMMENT_BODY_TOO_LONG",
            &format!(
                "comment body must not exceed {} characters",
                COMMENT_MAX_BODY_LEN
            ),
            None,
        );
    }

    let author_id =
        validate_authorization_header(headers.get(AUTH_HEADER).and_then(|v| v.to_str().ok()))
            .ok()
            .map(|c| c.sub);

    match sqlx::query_as::<_, TaskComment>(
        "INSERT INTO task_comments (task_id, author_id, body) VALUES ($1, $2, $3) RETURNING id, task_id, author_id, body, created_at, updated_at",
    )
    .bind(task_id)
    .bind(&author_id)
    .bind(&body)
    .fetch_one(&state.pool)
    .await
    {
        Ok(comment) => (StatusCode::CREATED, Json(comment)).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_CREATE_COMMENT_FAILED",
            "failed to create comment",
            None,
        ),
    }
}

pub(crate) async fn update_comment(
    Path(comment_id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateCommentRequest>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, TaskComment>(
        "SELECT id, task_id, author_id, body, created_at, updated_at FROM task_comments WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_optional(&state.pool)
    .await;

    let Ok(Some(_)) = existing else {
        return error_response(
            StatusCode::NOT_FOUND,
            "COMMENT_NOT_FOUND",
            "comment not found",
            None,
        );
    };

    let Some(raw_body) = payload.body else {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "COMMENT_BODY_REQUIRED",
            "comment body is required",
            None,
        );
    };

    let body = raw_body.trim().to_string();
    if body.is_empty() {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "COMMENT_BODY_REQUIRED",
            "comment body is required",
            None,
        );
    }
    if body.len() > COMMENT_MAX_BODY_LEN {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "COMMENT_BODY_TOO_LONG",
            &format!(
                "comment body must not exceed {} characters",
                COMMENT_MAX_BODY_LEN
            ),
            None,
        );
    }

    let update_result =
        sqlx::query("UPDATE task_comments SET body = $1, updated_at = NOW() WHERE id = $2")
            .bind(&body)
            .bind(comment_id)
            .execute(&state.pool)
            .await;

    if update_result.is_err() {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_UPDATE_COMMENT_FAILED",
            "failed to update comment",
            None,
        );
    }

    match sqlx::query_as::<_, TaskComment>(
        "SELECT id, task_id, author_id, body, created_at, updated_at FROM task_comments WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(comment) => Json(comment).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_FETCH_COMMENT_FAILED",
            "failed to load updated comment",
            None,
        ),
    }
}

pub(crate) async fn delete_comment(
    Path(comment_id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM task_comments WHERE id = $1")
        .bind(comment_id)
        .execute(&state.pool)
        .await;

    let Ok(result) = result else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_DELETE_COMMENT_FAILED",
            "failed to delete comment",
            None,
        );
    };

    if result.rows_affected() == 0 {
        return error_response(
            StatusCode::NOT_FOUND,
            "COMMENT_NOT_FOUND",
            "comment not found",
            None,
        );
    }

    StatusCode::NO_CONTENT.into_response()
}
