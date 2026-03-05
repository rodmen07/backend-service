use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;

use crate::app_state::AppState;
use crate::models::{GoalPlanRequest, GoalPlanResponse, Task};
use crate::rate_limit::{client_ip, is_plan_allowed};
use crate::validation::{GoalValidationError, GOAL_MAX_LENGTH, TITLE_MAX_LENGTH, validate_goal};

use super::shared::{error_response, orchestrator_timeout};

#[derive(Debug, serde::Serialize)]
struct OrchestratorPlanRequest {
    goal: String,
    /// Tasks already assigned to this goal — the AI will avoid suggesting duplicates.
    existing_tasks: Vec<String>,
    /// Tasks from other goals — gives the AI broader context about ongoing work.
    context_tasks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OrchestratorErrorPayload {
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlannedTasksPayload {
    tasks: Vec<String>,
}

pub(crate) async fn plan_tasks(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
    Json(payload): Json<GoalPlanRequest>,
) -> impl IntoResponse {
    // --- Plan-specific rate limit (stricter than the global limiter) ---
    let ip = {
        // Synthesise a minimal Request-like view for IP extraction.
        let forwarded = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim().to_owned());

        forwarded
            .or_else(|| connect_info.map(|ci| ci.0.ip().to_string()))
            .unwrap_or_else(|| "unknown".to_owned())
    };

    if !is_plan_allowed(&ip) {
        return error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "PLAN_RATE_LIMIT_EXCEEDED",
            "AI planning limit reached — try again later",
            Some(json!({ "window_seconds": 300, "max_requests": 5 })),
        );
    }

    // --- Goal validation ---
    let goal = match validate_goal(&payload.goal) {
        Ok(g) => g,
        Err(GoalValidationError::Empty) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "VALIDATION_GOAL_REQUIRED",
                "goal is required",
                None,
            );
        }
        Err(GoalValidationError::TooLong { max, actual }) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "VALIDATION_GOAL_TOO_LONG",
                "goal exceeds maximum length",
                Some(json!({ "max": max, "actual": actual })),
            );
        }
    };

    // --- Fetch existing tasks for this goal (to avoid duplicate suggestions) ---
    let existing_tasks: Vec<String> = sqlx::query_scalar(
        "SELECT title FROM tasks WHERE goal = ? ORDER BY id ASC",
    )
    .bind(&goal)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // --- Fetch tasks from other goals (broader context) ---
    let context_tasks: Vec<String> = sqlx::query_scalar(
        "SELECT title FROM tasks \
         WHERE goal IS NOT NULL AND goal != '' AND goal != ? \
         ORDER BY id DESC LIMIT 20",
    )
    .bind(&goal)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // --- Call the AI orchestrator ---
    let plan_url = std::env::var("AI_ORCHESTRATOR_PLAN_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8081/plan".to_string());

    let request_body = OrchestratorPlanRequest {
        goal: goal.clone(),
        existing_tasks,
        context_tasks,
    };

    let response = match state
        .http_client
        .post(plan_url)
        .timeout(orchestrator_timeout())
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(value) => value,
        Err(_) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                "LLM_UPSTREAM_REQUEST_FAILED",
                "failed to contact LLM provider",
                None,
            );
        }
    };

    if response.status() == StatusCode::SERVICE_UNAVAILABLE {
        let detail_message = response
            .json::<OrchestratorErrorPayload>()
            .await
            .ok()
            .and_then(|payload| payload.detail)
            .unwrap_or_else(|| {
                "LLM planning is not configured. Set OPENROUTER_API_KEY.".to_string()
            });

        if detail_message.contains("OPENROUTER_API_KEY") {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "LLM_API_KEY_MISSING",
                "LLM planning is not configured. Set OPENROUTER_API_KEY.",
                None,
            );
        }

        return error_response(
            StatusCode::BAD_GATEWAY,
            "LLM_UPSTREAM_RESPONSE_FAILED",
            "LLM provider returned an error",
            Some(json!({ "detail": detail_message })),
        );
    }

    if !response.status().is_success() {
        return error_response(
            StatusCode::BAD_GATEWAY,
            "LLM_UPSTREAM_RESPONSE_FAILED",
            "LLM provider returned an error",
            Some(json!({ "status": response.status().as_u16() })),
        );
    }

    let planned = match response.json::<PlannedTasksPayload>().await {
        Ok(value) => value,
        Err(_) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                "LLM_RESPONSE_INVALID",
                "invalid LLM response payload",
                None,
            );
        }
    };

    // --- Persist AI-generated tasks into the database ---
    let mut inserted_tasks: Vec<Task> = Vec::new();

    for raw_title in planned.tasks.into_iter().take(20) {
        let title = raw_title.trim().to_string();
        if title.is_empty() || title.chars().count() > TITLE_MAX_LENGTH {
            continue;
        }

        let insert_result = sqlx::query(
            "INSERT INTO tasks (title, completed, difficulty, goal, status, source) \
             VALUES (?, 0, 1, ?, 'todo', 'ai_generated')",
        )
        .bind(&title)
        .bind(&goal)
        .execute(&state.pool)
        .await;

        if let Ok(result) = insert_result {
            let task_id = result.last_insert_rowid();
            if let Ok(task) = sqlx::query_as::<_, Task>(
                "SELECT id, title, completed, difficulty, goal, status, source \
                 FROM tasks WHERE id = ?",
            )
            .bind(task_id)
            .fetch_one(&state.pool)
            .await
            {
                inserted_tasks.push(task);
            }
        }
    }

    if inserted_tasks.is_empty() {
        return error_response(
            StatusCode::BAD_GATEWAY,
            "LLM_TASKS_EMPTY",
            "LLM did not return any actionable tasks",
            None,
        );
    }

    (
        StatusCode::OK,
        Json(GoalPlanResponse {
            goal,
            tasks: inserted_tasks,
        }),
    )
        .into_response()
}
