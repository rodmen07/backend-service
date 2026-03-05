//! Binary entrypoint for the backend tutorial application.
//!
//! This module bootstraps configuration from environment variables,
//! initializes shared application state, builds the router, and starts the HTTP server.

use std::{env, net::SocketAddr};

use task_api_service::{AppState, build_router};

/// Starts the backend server.
///
/// Configuration is read from environment variables:
/// - `HOST` (default: `0.0.0.0`)
/// - `PORT` (default: `3000`)
/// - `DATABASE_URL` (default: `sqlite://app.db`)
#[tokio::main]
async fn main() {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("invalid HOST/PORT combination");

    let state = AppState::from_database_url(&database_url)
        .await
        .expect("failed to initialize database state");
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("API server listening on http://{addr}");

    // `into_make_service_with_connect_info` makes the peer socket address available
    // to middleware (e.g. the rate limiter) via the ConnectInfo extractor.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("server failed unexpectedly");
}
