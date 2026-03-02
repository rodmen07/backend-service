//! Application state and database bootstrap utilities.
//!
//! This module owns shared runtime state that handlers depend on.

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub(crate) pool: SqlitePool,
}

impl AppState {
    /// Builds application state from a SQLite connection string.
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

        Ok(Self { pool })
    }
}
