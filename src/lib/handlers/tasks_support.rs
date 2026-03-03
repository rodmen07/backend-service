use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;
use sqlx::{QueryBuilder, Sqlite};

use crate::models::ListTasksQuery;
use crate::validation::{TitleValidationError, normalize_search_query};

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
        }

        query_builder
            .push("title LIKE ")
            .push_bind(format!("%{search}%"));
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
