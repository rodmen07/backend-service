use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, response::IntoResponse};
use std::net::SocketAddr;

use crate::app_state::AppState;
use crate::models::{GoalPlanRequest, GoalPlanResponse};
use crate::rate_limit::is_plan_allowed;
use crate::validation::{GoalValidationError, validate_goal};

use super::shared::{error_response, orchestrator_timeout};

#[derive(Debug, serde::Serialize)]
struct OrchestratorPlanRequest {
    goal: String,
    /// Tasks already assigned to this goal — the AI will avoid suggesting duplicates.
    existing_tasks: Vec<String>,
    /// Tasks from other goals — gives the AI broader context about ongoing work.
    context_tasks: Vec<String>,
    /// Optional user-supplied refinement instructions.
    feedback: String,
    /// Desired number of tasks to generate (1–15).
    target_count: u8,
}

#[derive(Debug, serde::Deserialize)]
struct OrchestratorErrorPayload {
    detail: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PlannedTasksPayload {
    tasks: Vec<String>,
}

pub(crate) async fn plan_tasks(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<GoalPlanRequest>,
) -> impl IntoResponse {
    // --- Plan-specific rate limit (stricter than the global limiter) ---
    let ip = {
        let forwarded = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim().to_owned());

        forwarded.unwrap_or_else(|| addr.ip().to_string())
    };

    if !is_plan_allowed(&ip) {
        return error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "PLAN_RATE_LIMIT_EXCEEDED",
            "AI planning limit reached — try again later",
            Some(serde_json::json!({ "window_seconds": 300, "max_requests": 5 })),
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
                Some(serde_json::json!({ "max": max, "actual": actual })),
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

    let feedback = payload.feedback
        .as_deref()
        .unwrap_or("")
        .trim()
        .chars()
        .take(500)
        .collect::<String>();

    let target_count = payload.target_count
        .map(|n| n.clamp(1, 15))
        .unwrap_or(7);

    let request_body = OrchestratorPlanRequest {
        goal: goal.clone(),
        existing_tasks,
        context_tasks,
        feedback,
        target_count,
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
            .unwrap_or_else(|| "LLM planning is not configured.".to_string());

        if detail_message.contains("API_KEY") {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "LLM_API_KEY_MISSING",
                "LLM planning is not configured. Set ANTHROPIC_API_KEY.",
                None,
            );
        }

        return error_response(
            StatusCode::BAD_GATEWAY,
            "LLM_UPSTREAM_RESPONSE_FAILED",
            "LLM provider returned an error",
            Some(serde_json::json!({ "detail": detail_message })),
        );
    }

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "LLM_RATE_LIMIT_EXCEEDED",
            "Claude API rate limit reached — try again shortly",
            None,
        );
    }

    if !response.status().is_success() {
        return error_response(
            StatusCode::BAD_GATEWAY,
            "LLM_UPSTREAM_RESPONSE_FAILED",
            "LLM provider returned an error",
            Some(serde_json::json!({ "status": response.status().as_u16() })),
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

    if planned.tasks.is_empty() {
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
            tasks: planned.tasks,
        }),
    )
        .into_response()
}
