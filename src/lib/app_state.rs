//! Application state and database bootstrap utilities.
//!
//! This module owns shared runtime state that handlers depend on.

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

/// Shared application state injected into handlers via Axum's `State` extractor.
///
/// Cloning is cheap — both `SqlitePool` and `reqwest::Client` use internal `Arc`s.
#[derive(Clone)]
pub struct AppState {
    pub(crate) pool: SqlitePool,
    pub(crate) http_client: reqwest::Client,
}

impl AppState {
    /// Builds application state from a SQLite connection string.
    ///
    /// Creates a connection pool, runs pending migrations, and initialises a
    /// reusable HTTP client for upstream calls (e.g. the AI orchestrator).
    ///
    /// # Parameters
    /// - `database_url`: SQLx-compatible SQLite URL (for example, `sqlite://app.db`).
    ///
    /// # Returns
    /// - `Ok(AppState)` when a pool is created and all migrations are applied.
    /// - `Err(sqlx::Error)` when connection or migration steps fail.
    pub async fn from_database_url(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        let http_client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build HTTP client");

        Ok(Self { pool, http_client })
    }
}
