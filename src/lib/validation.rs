//! Validation and normalization helpers used by request handlers.

/// Normalizes a task title by trimming whitespace.
///
/// # Parameters
/// - `input`: Raw title string from a request payload.
///
/// # Returns
/// - `Some(String)` with trimmed content when non-empty.
/// - `None` when the trimmed value is empty.
pub(crate) fn normalize_title(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Normalizes search text by trimming whitespace.
///
/// # Parameters
/// - `input`: Raw search query.
///
/// # Returns
/// - `Some(String)` with trimmed query when non-empty.
/// - `None` when the trimmed value is empty.
pub(crate) fn normalize_search_query(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_search_query, normalize_title};

    /// Ensures blank/whitespace-only titles are rejected.
    #[test]
    fn normalize_title_rejects_blank() {
        assert_eq!(normalize_title("   \n"), None);
    }

    /// Ensures leading/trailing spaces are removed from valid titles.
    #[test]
    fn normalize_title_trims_content() {
        assert_eq!(normalize_title("  hello  "), Some("hello".to_string()));
    }

    /// Ensures blank search query strings are rejected.
    #[test]
    fn normalize_search_query_rejects_blank() {
        assert_eq!(normalize_search_query("   \n"), None);
    }
}
