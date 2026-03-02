//! Library facade for the backend tutorial project.
//!
//! This file exposes the high-level public API used by the binary entrypoint,
//! while implementation details live in focused submodules under `src/lib/`.

#[path = "lib/app_state.rs"]
mod app_state;
#[path = "lib/handlers.rs"]
mod handlers;
#[path = "lib/models.rs"]
mod models;
#[path = "lib/router.rs"]
mod router;
#[path = "lib/validation.rs"]
mod validation;

pub use app_state::AppState;
pub use router::build_router;
