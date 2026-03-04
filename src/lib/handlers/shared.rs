use std::{env, time::Duration};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;

use crate::models::{ApiError, ListTasksQuery};

pub(super) fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> Response {
    (
        status,
        Json(ApiError {
            code: code.to_string(),
            message: message.to_string(),
            details,
        }),
    )
        .into_response()
}

pub(super) fn resolved_pagination(params: &ListTasksQuery) -> (u32, u32) {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);
    (limit, offset)
}

pub(super) fn orchestrator_timeout() -> Duration {
    let seconds = env::var("AI_ORCHESTRATOR_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(15.0);

    Duration::from_secs_f64(seconds)
}

#[cfg(test)]
mod tests {
    use super::{orchestrator_timeout, resolved_pagination};
    use crate::models::ListTasksQuery;
    use std::env;

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
            status: None,
            q: None,
        };

        assert_eq!(resolved_pagination(&params), (100, 3));
    }

    #[test]
    fn orchestrator_timeout_uses_default_for_invalid_values() {
        unsafe {
            env::remove_var("AI_ORCHESTRATOR_TIMEOUT_SECONDS");
        }
        assert_eq!(orchestrator_timeout().as_secs_f64(), 15.0);

        unsafe {
            env::set_var("AI_ORCHESTRATOR_TIMEOUT_SECONDS", "0");
        }
        assert_eq!(orchestrator_timeout().as_secs_f64(), 15.0);

        unsafe {
            env::set_var("AI_ORCHESTRATOR_TIMEOUT_SECONDS", "invalid");
        }
        assert_eq!(orchestrator_timeout().as_secs_f64(), 15.0);
    }

    #[test]
    fn orchestrator_timeout_accepts_positive_values() {
        unsafe {
            env::set_var("AI_ORCHESTRATOR_TIMEOUT_SECONDS", "2.5");
        }
        assert_eq!(orchestrator_timeout().as_secs_f64(), 2.5);
    }
}
