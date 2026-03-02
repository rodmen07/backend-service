use std::{env, net::SocketAddr};

use projects::{AppState, build_router};

#[tokio::main]
async fn main() {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("invalid HOST/PORT combination");

    let app = build_router(AppState::new());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("API server listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("server failed unexpectedly");
}
