//! Integration tests for the task API routes.
//!
//! These tests exercise request/response behavior against a real SQLite database
//! and the fully wired Axum router.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use task_api_service::{AppState, build_router};
use serde_json::Value;
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tower::ServiceExt;

/// Holds test resources for one integration test case.
///
/// # Fields
/// - `app`: Configured router used for one-shot requests.
/// - `database_path`: On-disk SQLite file path that gets cleaned up on drop.
struct TestApp {
    app: axum::Router,
    database_path: PathBuf,
}

impl Drop for TestApp {
    /// Cleans up the temporary SQLite file once test resources are dropped.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to the test fixture.
    ///
    /// # Returns
    /// - No return value.
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.database_path);
    }
}

/// Builds a test application with an isolated SQLite database file.
///
/// # Parameters
/// - None.
///
/// # Returns
/// - `TestApp` containing a configured router and unique DB path.
async fn test_app() -> TestApp {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();

    let database_path = std::env::temp_dir().join(format!("backend_service_test_{timestamp}.db"));
    let database_url = format!("sqlite://{}?mode=rwc", database_path.display());

    let state = AppState::from_database_url(&database_url)
        .await
        .expect("failed to initialize test app state");

    TestApp {
        app: build_router(state),
        database_path,
    }
}

/// Verifies that creating a task makes it available in subsequent list calls.
#[tokio::test]
async fn create_task_then_list_tasks() {
    let test_app = test_app().await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"title":"Write integration tests"}"#))
        .expect("failed to build create request");

    let create_response = test_app
        .app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create request failed");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let list_request = Request::builder()
        .method("GET")
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("failed to build list request");

    let list_response = test_app
        .app
        .clone()
        .oneshot(list_request)
        .await
        .expect("list request failed");

    assert_eq!(list_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("failed reading response body");
    let payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse list response body");

    let tasks = payload.as_array().expect("list response must be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Write integration tests");
    assert_eq!(tasks[0]["completed"], false);
    assert_eq!(tasks[0]["difficulty"], 1);
    assert!(tasks[0]["goal"].is_null());
}

/// Verifies list endpoint filtering and pagination behavior.
#[tokio::test]
async fn list_tasks_respects_filters_and_pagination() {
    let test_app = test_app().await;

    for title in ["Design API", "Write docs", "Design UI"] {
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tasks")
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"title":"{title}"}}"#)))
            .expect("failed to build seed task request");

        let response = test_app
            .app
            .clone()
            .oneshot(request)
            .await
            .expect("seed request failed");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let filtered_request = Request::builder()
        .method("GET")
        .uri("/api/v1/tasks?limit=1&offset=1&q=Design")
        .body(Body::empty())
        .expect("failed to build filtered list request");

    let filtered_response = test_app
        .app
        .clone()
        .oneshot(filtered_request)
        .await
        .expect("filtered list request failed");

    assert_eq!(filtered_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(filtered_response.into_body(), usize::MAX)
        .await
        .expect("failed reading filtered response body");
    let payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse filtered response body");

    let tasks = payload
        .as_array()
        .expect("filtered list response must be an array");

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Design UI");
}

/// Verifies standardized error envelope for empty-title validation failures.
#[tokio::test]
async fn create_task_rejects_empty_title_with_error_code() {
    let test_app = test_app().await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"title":"   "}"#))
        .expect("failed to build create request");

    let create_response = test_app
        .app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create request failed");

    assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("failed reading response body");
    let payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse error response body");

    assert_eq!(payload["code"], "VALIDATION_TITLE_REQUIRED");
    assert_eq!(payload["message"], "title is required");
}

/// Verifies title-length invariant and details payload for overly long titles.
#[tokio::test]
async fn create_task_rejects_too_long_title_with_details() {
    let test_app = test_app().await;
    let long_title = "x".repeat(121);

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"title":"{long_title}"}}"#)))
        .expect("failed to build create request");

    let create_response = test_app
        .app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create request failed");

    assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("failed reading response body");
    let payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse error response body");

    assert_eq!(payload["code"], "VALIDATION_TITLE_TOO_LONG");
    assert_eq!(payload["details"]["max"], 120);
    assert_eq!(payload["details"]["actual"], 121);
}

/// Verifies difficulty must be between 1 and 5.
#[tokio::test]
async fn create_task_rejects_out_of_range_difficulty() {
    let test_app = test_app().await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"title":"Calibrate forge","difficulty":7}"#))
        .expect("failed to build create request");

    let create_response = test_app
        .app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create request failed");

    assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("failed reading response body");
    let payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse error response body");

    assert_eq!(payload["code"], "VALIDATION_DIFFICULTY_OUT_OF_RANGE");
}

/// Verifies task difficulty updates persist and round-trip through API responses.
#[tokio::test]
async fn update_task_allows_difficulty_changes() {
    let test_app = test_app().await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"title":"Temper blade"}"#))
        .expect("failed to build create request");

    let create_response = test_app
        .app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create request failed");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("failed reading create response body");
    let created_payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse create response body");
    let task_id = created_payload["id"].as_i64().expect("task id missing");

    let patch_request = Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/tasks/{task_id}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"difficulty":5}"#))
        .expect("failed to build patch request");

    let patch_response = test_app
        .app
        .clone()
        .oneshot(patch_request)
        .await
        .expect("patch request failed");

    assert_eq!(patch_response.status(), StatusCode::OK);

    let patch_body = to_bytes(patch_response.into_body(), usize::MAX)
        .await
        .expect("failed reading patch response body");
    let patched_payload: Value =
        serde_json::from_slice(&patch_body).expect("failed to parse patch response body");

    assert_eq!(patched_payload["difficulty"], 5);
}

/// Verifies readiness endpoint reports ready when DB is reachable.
#[tokio::test]
async fn ready_endpoint_reports_ready() {
    let test_app = test_app().await;

    let ready_request = Request::builder()
        .method("GET")
        .uri("/ready")
        .body(Body::empty())
        .expect("failed to build ready request");

    let ready_response = test_app
        .app
        .clone()
        .oneshot(ready_request)
        .await
        .expect("ready request failed");

    assert_eq!(ready_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(ready_response.into_body(), usize::MAX)
        .await
        .expect("failed reading ready response body");
    let payload: Value =
        serde_json::from_slice(&body_bytes).expect("failed to parse ready response body");

    assert_eq!(payload["status"], "ready");
}

/// Verifies v1 stance: API remains accessible without authentication headers.
#[tokio::test]
async fn v1_allows_requests_without_auth() {
    let test_app = test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .expect("failed to build health request");

    let response = test_app
        .app
        .clone()
        .oneshot(request)
        .await
        .expect("health request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

/// Verifies future auth-header shape does not break v1 while auth is not enforced.
#[tokio::test]
async fn v1_accepts_reserved_future_auth_header_shape() {
    let test_app = test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .header("Authorization", "Bearer tutorial-token")
        .body(Body::empty())
        .expect("failed to build health request");

    let response = test_app
        .app
        .clone()
        .oneshot(request)
        .await
        .expect("health request failed");

    assert_eq!(response.status(), StatusCode::OK);
}
