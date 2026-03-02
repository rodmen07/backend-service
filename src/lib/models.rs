use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
