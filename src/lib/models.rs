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
    pub goal: Option<String>,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub difficulty: Option<i64>,
    pub goal: Option<String>,
    pub status: Option<String>,
    /// Caller-supplied source; only "ai_generated" is accepted; anything else defaults to "manual".
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub difficulty: Option<i64>,
    pub goal: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTasksQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub completed: Option<bool>,
    pub status: Option<String>,
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

/// Query parameters for clearing AI-generated tasks under a specific goal.
#[derive(Debug, Deserialize)]
pub struct ClearPlanQuery {
    pub goal: String,
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

#[derive(Debug, Serialize)]
pub struct AdminMetricsResponse {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub pending_tasks: i64,
    pub total_requests: i64,
    pub unique_subjects: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AdminRequestLog {
    pub id: i64,
    pub occurred_at: String,
    pub subject: Option<String>,
    pub method: String,
    pub path: String,
    pub status_code: i64,
    pub duration_ms: i64,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AdminListQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AdminUserActivity {
    pub subject: String,
    pub request_count: i64,
    pub first_seen_at: String,
    pub last_seen_at: String,
}
