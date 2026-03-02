use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use projects::{AppState, build_router};
use serde_json::Value;
use std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use tower::ServiceExt;

struct TestApp {
    app: axum::Router,
    database_path: PathBuf,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.database_path);
    }
}

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
}

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
