use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    store: Arc<Mutex<Store>>,
}

#[derive(Default)]
struct Store {
    tasks: Vec<Task>,
    next_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
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

#[derive(Debug, Serialize)]
pub struct ApiMessage {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Store {
                tasks: Vec::new(),
                next_id: 1,
            })),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
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

async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().expect("store mutex poisoned");
    Json(store.tasks.clone())
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let title = payload.title.trim().to_string();

    if title.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "title is required".to_string(),
            }),
        )
            .into_response();
    }

    let mut store = state.store.lock().expect("store mutex poisoned");
    let task = Task {
        id: store.next_id,
        title,
        completed: false,
    };
    store.next_id += 1;
    store.tasks.push(task.clone());

    (StatusCode::CREATED, Json(task)).into_response()
}

async fn update_task(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let mut store = state.store.lock().expect("store mutex poisoned");

    let Some(task) = store.tasks.iter_mut().find(|task| task.id == id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "task not found".to_string(),
            }),
        )
            .into_response();
    };

    if let Some(title) = payload.title {
        let trimmed = title.trim().to_string();
        if trimmed.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiMessage {
                    message: "title cannot be empty".to_string(),
                }),
            )
                .into_response();
        }
        task.title = trimmed;
    }

    if let Some(completed) = payload.completed {
        task.completed = completed;
    }

    (StatusCode::OK, Json(task.clone())).into_response()
}

async fn delete_task(Path(id): Path<u64>, State(state): State<AppState>) -> impl IntoResponse {
    let mut store = state.store.lock().expect("store mutex poisoned");
    let initial_len = store.tasks.len();
    store.tasks.retain(|task| task.id != id);

    if store.tasks.len() == initial_len {
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

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn app_state_starts_empty() {
        let state = AppState::new();
        let store = state.store.lock().expect("store mutex poisoned");

        assert!(store.tasks.is_empty());
        assert_eq!(store.next_id, 1);
    }
}
