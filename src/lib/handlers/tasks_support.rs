use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;
use sqlx::{QueryBuilder, Sqlite};

use crate::models::ListTasksQuery;
use crate::validation::{
    DifficultyValidationError, StatusValidationError, TitleValidationError, normalize_search_query,
};

use super::shared::error_response;

pub(crate) fn apply_list_task_filters(
    query_builder: &mut QueryBuilder<Sqlite>,
    params: &ListTasksQuery,
) {
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
            has_where_clause = true;
        }

        query_builder
            .push("title LIKE ")
            .push_bind(format!("%{search}%"));
    }

    if let Some(status) = params.status.as_deref() {
        let trimmed = status.trim().to_lowercase();
        if !trimmed.is_empty() {
            if has_where_clause {
                query_builder.push(" AND ");
            } else {
                query_builder.push(" WHERE ");
            }

            query_builder
                .push("status = ")
                .push_bind(trimmed);
        }
    }
}

pub(crate) fn title_validation_error_response(
    error: TitleValidationError,
    empty_message: &'static str,
) -> Response {
    match error {
        TitleValidationError::Empty => error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_TITLE_REQUIRED",
            empty_message,
            None,
        ),
        TitleValidationError::TooLong { max, actual } => error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_TITLE_TOO_LONG",
            "title exceeds maximum length",
            Some(json!({ "max": max, "actual": actual })),
        ),
    }
}

pub(crate) fn difficulty_validation_error_response(error: DifficultyValidationError) -> Response {
    match error {
        DifficultyValidationError::OutOfRange { min, max, actual } => error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_DIFFICULTY_OUT_OF_RANGE",
            "difficulty must be between 1 and 6",
            Some(json!({ "min": min, "max": max, "actual": actual })),
        ),
    }
}

pub(crate) fn status_validation_error_response(error: StatusValidationError) -> Response {
    match error {
        StatusValidationError::Invalid { actual } => error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_STATUS_INVALID",
            "status must be one of: todo, doing, done",
            Some(json!({ "actual": actual, "valid": ["todo", "doing", "done"] })),
        ),
    }
}
