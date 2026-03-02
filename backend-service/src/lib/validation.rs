//! Validation and normalization helpers used by request handlers.

/// Maximum allowed task title length in Unicode scalar values.
pub(crate) const TITLE_MAX_LENGTH: usize = 120;

/// Represents task-title validation failures.
#[derive(Debug, PartialEq)]
pub(crate) enum TitleValidationError {
    Empty,
    TooLong { max: usize, actual: usize },
}

/// Validates and normalizes a task title by trimming whitespace.
///
/// # Parameters
/// - `input`: Raw title string from a request payload.
///
/// # Returns
/// - `Ok(String)` with trimmed content when valid.
/// - `Err(TitleValidationError)` when empty or too long.
pub(crate) fn validate_title(input: &str) -> Result<String, TitleValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(TitleValidationError::Empty);
    }

    let length = trimmed.chars().count();
    if length > TITLE_MAX_LENGTH {
        return Err(TitleValidationError::TooLong {
            max: TITLE_MAX_LENGTH,
            actual: length,
        });
    }

    Ok(trimmed.to_string())
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
    use super::{TITLE_MAX_LENGTH, TitleValidationError, normalize_search_query, validate_title};

    /// Ensures blank/whitespace-only titles are rejected.
    #[test]
    fn validate_title_rejects_blank() {
        let result = validate_title("   \n");
        assert!(matches!(result, Err(TitleValidationError::Empty)));
    }

    /// Ensures leading/trailing spaces are removed from valid titles.
    #[test]
    fn validate_title_trims_content() {
        assert_eq!(validate_title("  hello  "), Ok("hello".to_string()));
    }

    /// Ensures title length above the defined max is rejected.
    #[test]
    fn validate_title_rejects_too_long() {
        let too_long = "a".repeat(TITLE_MAX_LENGTH + 1);
        let result = validate_title(&too_long);

        assert!(matches!(
            result,
            Err(TitleValidationError::TooLong {
                max: TITLE_MAX_LENGTH,
                actual
            }) if actual == TITLE_MAX_LENGTH + 1
        ));
    }

    /// Ensures blank search query strings are rejected.
    #[test]
    fn normalize_search_query_rejects_blank() {
        assert_eq!(normalize_search_query("   \n"), None);
    }
}
