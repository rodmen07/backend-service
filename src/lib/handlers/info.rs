//! Service information endpoint.
//!
//! Returns build-time metadata and advertised feature flags so that
//! clients (and operators) can discover what this instance supports.

use axum::{Json, response::IntoResponse};
use serde_json::json;

pub(crate) async fn info() -> impl IntoResponse {
    Json(json!({
        "service":  env!("CARGO_PKG_NAME"),
        "version":  env!("CARGO_PKG_VERSION"),
        "features": [
            "task-crud",
            "llm-goal-planning",
            "jwt-auth",
            "admin-metrics",
            "request-audit-log",
            "sqlite-persistence",
        ],
    }))
}
