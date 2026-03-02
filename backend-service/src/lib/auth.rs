//! Authentication contract definitions for the API.
//!
//! v1 stance: authentication is intentionally not enforced.
//! This module freezes the future-facing interface to avoid client churn.

/// HTTP header reserved for future bearer-token authentication.
pub const AUTH_HEADER: &str = "Authorization";

/// Token scheme reserved for future auth middleware.
pub const AUTH_SCHEME: &str = "Bearer";

/// Returns the canonical header format reserved for future authenticated requests.
///
/// # Parameters
/// - None.
///
/// # Returns
/// - A static format string showing the expected header shape.
pub fn frozen_auth_header_format() -> &'static str {
    "Authorization: Bearer <token>"
}
