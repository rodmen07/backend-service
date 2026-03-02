//! Library facade for the backend tutorial project.
//!
//! This file exposes the high-level public API used by the binary entrypoint,
//! while implementation details live in focused submodules under `src/lib/`.

#[path = "lib/app_state.rs"]
mod app_state;
#[path = "lib/auth.rs"]
mod auth;
#[path = "lib/handlers.rs"]
mod handlers;
#[path = "lib/models.rs"]
mod models;
#[path = "lib/router.rs"]
mod router;
#[path = "lib/validation.rs"]
mod validation;

pub use app_state::AppState;
pub use auth::{AUTH_HEADER, AUTH_SCHEME, frozen_auth_header_format};
pub use router::build_router;
