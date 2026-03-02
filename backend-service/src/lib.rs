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
