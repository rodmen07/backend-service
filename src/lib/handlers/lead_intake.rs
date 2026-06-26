use std::env;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::{Value, json};
use sqlx::types::Json as SqlxJson;

use crate::app_state::AppState;
use crate::models::PublicLeadIntakeResponse;

use super::shared::error_response;

fn extract_email(payload: &Value) -> Option<String> {
    payload
        .get("email")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.contains('@'))
        .map(ToOwned::to_owned)
}

pub(crate) async fn create_public_lead(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if !payload.is_object() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_BODY_OBJECT_REQUIRED",
            "request body must be a JSON object",
            None,
        );
    }

    let Some(email) = extract_email(&payload) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "VALIDATION_EMAIL_REQUIRED",
            "email is required",
            Some(json!({ "field": "email" })),
        );
    };

    let event_type = payload
        .get("event_type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("consultation")
        .to_string();

    let lead_source = payload
        .get("lead_source")
        .or_else(|| payload.get("source"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("website")
        .to_string();

    let forward_url = env::var("LEAD_INTAKE_FORWARD_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let (forwarded, forward_status, forward_detail) = if let Some(url) = forward_url {
        match state.http_client.post(url).json(&payload).send().await {
            Ok(response) if response.status().is_success() => (true, "forwarded".to_string(), None),
            Ok(response) => (
                false,
                "forward-failed".to_string(),
                Some(format!(
                    "{} {}",
                    response.status().as_u16(),
                    response.status().canonical_reason().unwrap_or("")
                )),
            ),
            Err(error) => (false, "forward-failed".to_string(), Some(error.to_string())),
        }
    } else {
        (false, "not-configured".to_string(), None)
    };

    let insert_result = sqlx::query(
        "INSERT INTO public_lead_intake_submissions (email, event_type, lead_source, payload, forward_status, forward_detail) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(email)
    .bind(event_type)
    .bind(lead_source)
    .bind(SqlxJson(payload))
    .bind(forward_status)
    .bind(forward_detail)
    .execute(&state.pool)
    .await;

    match insert_result {
        Ok(_) => (
            StatusCode::ACCEPTED,
            Json(PublicLeadIntakeResponse {
                status: "accepted",
                forwarded,
            }),
        )
            .into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_CREATE_LEAD_INTAKE_FAILED",
            "failed to record lead intake",
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::extract_email;
    use serde_json::json;

    #[test]
    fn extract_email_rejects_missing_or_invalid() {
        assert!(extract_email(&json!({})).is_none());
        assert!(extract_email(&json!({ "email": "" })).is_none());
        assert!(extract_email(&json!({ "email": "not-an-email" })).is_none());
    }

    #[test]
    fn extract_email_trims_and_accepts_valid_email() {
        assert_eq!(
            extract_email(&json!({ "email": "  founder@example.com " })),
            Some("founder@example.com".to_string())
        );
    }
}
