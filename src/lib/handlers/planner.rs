use axum::http::StatusCode;
use axum::{Json, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

use crate::models::{GoalPlanRequest, GoalPlanResponse};

use super::shared::{error_response, orchestrator_timeout};

#[derive(Debug, serde::Serialize)]
struct OrchestratorPlanRequest {
    goal: String,
}

#[derive(Debug, Deserialize)]
struct OrchestratorErrorPayload {
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlannedTasksPayload {
    tasks: Vec<String>,
}

pub(crate) async fn plan_tasks(Json(payload): Json<GoalPlanRequest>) -> impl IntoResponse {
    let goal = payload.goal.trim();
    if goal.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_GOAL_REQUIRED",
            "goal is required",
            None,
        );
    }

    let plan_url = std::env::var("AI_ORCHESTRATOR_PLAN_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8081/plan".to_string());

    let client = match reqwest::Client::builder()
        .timeout(orchestrator_timeout())
        .build()
    {
        Ok(value) => value,
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "HTTP_CLIENT_INIT_FAILED",
                "failed to initialize upstream HTTP client",
                None,
            );
        }
    };

    let request_body = OrchestratorPlanRequest {
        goal: goal.to_string(),
    };

    let response = match client
        .post(plan_url)
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

    let tasks: Vec<String> = planned
        .tasks
        .into_iter()
        .map(|task| task.trim().to_string())
        .filter(|task| !task.is_empty())
        .take(20)
        .collect();

    if tasks.is_empty() {
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
            goal: goal.to_string(),
            tasks,
        }),
    )
        .into_response()
}
