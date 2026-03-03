//! Request/response DTOs and domain models for the task API.
//!
//! These types define the contract between clients, handlers, and the database layer.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub difficulty: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub difficulty: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub difficulty: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTasksQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub completed: Option<bool>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoalPlanRequest {
    pub goal: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoalPlanResponse {
    pub goal: String,
    pub tasks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}
