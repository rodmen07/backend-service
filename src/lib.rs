use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTasksQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub completed: Option<bool>,
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiMessage {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

impl AppState {
    pub async fn from_database_url(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/{id}", patch(update_task).delete(delete_task))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);

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

async fn create_task(
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

async fn update_task(
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

async fn delete_task(Path(id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
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

fn normalize_title(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_search_query(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_search_query, normalize_title};

    #[test]
    fn normalize_title_rejects_blank() {
        assert_eq!(normalize_title("   \n"), None);
    }

    #[test]
    fn normalize_title_trims_content() {
        assert_eq!(normalize_title("  hello  "), Some("hello".to_string()));
    }

    #[test]
    fn normalize_search_query_rejects_blank() {
        assert_eq!(normalize_search_query("   \n"), None);
    }
}
