//! Application state and database bootstrap utilities.
//!
//! This module owns shared runtime state that handlers depend on.

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Shared application state injected into handlers via Axum's `State` extractor.
///
/// Cloning is cheap - both `PgPool` and `reqwest::Client` use internal `Arc`s.
#[derive(Clone)]
pub struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) http_client: reqwest::Client,
}

impl AppState {
    /// Builds application state from a PostgreSQL connection string.
    ///
    /// Creates a connection pool, runs pending migrations, and initialises a
    /// reusable HTTP client for upstream calls (e.g. the AI orchestrator).
    ///
    /// # Parameters
    /// - `database_url`: SQLx-compatible PostgreSQL URL (for example, `postgresql://user:pass@host/db`).
    ///
    /// # Returns
    /// - `Ok(AppState)` when a pool is created and all migrations are applied.
    /// - `Err(sqlx::Error)` when connection or migration steps fail.
    pub async fn from_database_url(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        let http_client = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .expect("failed to build HTTP client");

        Ok(Self { pool, http_client })
    }
}
