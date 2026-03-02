use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::{QueryBuilder, Sqlite};

use crate::app_state::AppState;
use crate::models::{
    ApiMessage, CreateTaskRequest, HealthResponse, ListTasksQuery, Task, UpdateTaskRequest,
};
use crate::validation::{normalize_search_query, normalize_title};

pub(crate) async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

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
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to list tasks".to_string(),
            }),
        )
            .into_response(),
    }
}

pub(crate) async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let Some(title) = normalize_title(&payload.title) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "title is required".to_string(),
            }),
        )
            .into_response();
    };

    let insert_result = sqlx::query("INSERT INTO tasks (title, completed) VALUES (?, ?)")
        .bind(&title)
        .bind(false)
        .execute(&state.pool)
        .await;

    let Ok(result) = insert_result else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to create task".to_string(),
            }),
        )
            .into_response();
    };

    let task_id = result.last_insert_rowid();
    let fetch_result =
        sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(&state.pool)
            .await;

    match fetch_result {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to load created task".to_string(),
            }),
        )
            .into_response(),
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to update task".to_string(),
            }),
        )
            .into_response();
    };

    let Some(mut task) = existing_task else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "task not found".to_string(),
            }),
        )
            .into_response();
    };

    if let Some(title) = payload.title.as_deref() {
        let Some(trimmed) = normalize_title(title) else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiMessage {
                    message: "title cannot be empty".to_string(),
                }),
            )
                .into_response();
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
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to update task".to_string(),
            }),
        )
            .into_response(),
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                message: "failed to delete task".to_string(),
            }),
        )
            .into_response();
    };

    if result.rows_affected() == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "task not found".to_string(),
            }),
        )
            .into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

fn resolved_pagination(params: &ListTasksQuery) -> (u32, u32) {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);
    (limit, offset)
}

#[cfg(test)]
mod tests {
    use super::resolved_pagination;
    use crate::models::ListTasksQuery;

    #[test]
    fn resolved_pagination_defaults() {
        let params = ListTasksQuery::default();
        assert_eq!(resolved_pagination(&params), (50, 0));
    }

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
