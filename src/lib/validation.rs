//! Validation and normalization helpers used by request handlers.

/// Maximum allowed task title length in Unicode scalar values.
pub(crate) const TITLE_MAX_LENGTH: usize = 120;
pub(crate) const TASK_DIFFICULTY_MIN: i64 = 1;
pub(crate) const TASK_DIFFICULTY_MAX: i64 = 6;

/// Maximum allowed goal string length in Unicode scalar values.
pub(crate) const GOAL_MAX_LENGTH: usize = 500;

/// Valid task status values for Kanban board workflow.
pub(crate) const VALID_TASK_STATUSES: &[&str] = &["todo", "doing", "done"];

/// Represents task-title validation failures.
#[derive(Debug, PartialEq)]
pub(crate) enum TitleValidationError {
    Empty,
    TooLong { max: usize, actual: usize },
}

#[derive(Debug, PartialEq)]
pub(crate) enum DifficultyValidationError {
    OutOfRange { min: i64, max: i64, actual: i64 },
}

#[derive(Debug, PartialEq)]
pub(crate) enum StatusValidationError {
    Invalid { actual: String },
}

#[derive(Debug, PartialEq)]
pub(crate) enum GoalValidationError {
    Empty,
    TooLong { max: usize, actual: usize },
}

/// Validates and normalizes a task title by trimming whitespace.
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

/// Validates and normalizes a planning goal by trimming whitespace.
pub(crate) fn validate_goal(input: &str) -> Result<String, GoalValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(GoalValidationError::Empty);
    }

    let length = trimmed.chars().count();
    if length > GOAL_MAX_LENGTH {
        return Err(GoalValidationError::TooLong {
            max: GOAL_MAX_LENGTH,
            actual: length,
        });
    }

    Ok(trimmed.to_string())
}

/// Normalizes search text by trimming whitespace.
pub(crate) fn normalize_search_query(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn validate_difficulty(input: i64) -> Result<i64, DifficultyValidationError> {
    if (TASK_DIFFICULTY_MIN..=TASK_DIFFICULTY_MAX).contains(&input) {
        Ok(input)
    } else {
        Err(DifficultyValidationError::OutOfRange {
            min: TASK_DIFFICULTY_MIN,
            max: TASK_DIFFICULTY_MAX,
            actual: input,
        })
    }
}

/// Validates and normalizes a task status string.
///
/// Accepted values: `todo`, `doing`, `done` (case-insensitive, trimmed).
pub(crate) fn validate_status(input: &str) -> Result<String, StatusValidationError> {
    let normalized = input.trim().to_lowercase();
    if VALID_TASK_STATUSES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(StatusValidationError::Invalid {
            actual: input.to_string(),
        })
    }
}

/// Returns the `completed` boolean that should correspond to a given status value.
pub(crate) fn completed_for_status(status: &str) -> bool {
    status == "done"
}

/// Returns the `status` string that should correspond to a given completed boolean.
pub(crate) fn status_for_completed(completed: bool) -> &'static str {
    if completed { "done" } else { "todo" }
}

#[cfg(test)]
mod tests {
    use super::{
        DifficultyValidationError, GoalValidationError, StatusValidationError,
        GOAL_MAX_LENGTH, TASK_DIFFICULTY_MAX, TASK_DIFFICULTY_MIN, TITLE_MAX_LENGTH,
        TitleValidationError, completed_for_status, normalize_search_query,
        status_for_completed, validate_difficulty, validate_goal, validate_status,
        validate_title,
    };

    #[test]
    fn validate_title_rejects_blank() {
        let result = validate_title("   \n");
        assert!(matches!(result, Err(TitleValidationError::Empty)));
    }

    #[test]
    fn validate_title_trims_content() {
        assert_eq!(validate_title("  hello  "), Ok("hello".to_string()));
    }

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

    #[test]
    fn validate_goal_rejects_blank() {
        assert!(matches!(validate_goal("   "), Err(GoalValidationError::Empty)));
    }

    #[test]
    fn validate_goal_rejects_too_long() {
        let too_long = "a".repeat(GOAL_MAX_LENGTH + 1);
        assert!(matches!(
            validate_goal(&too_long),
            Err(GoalValidationError::TooLong { max: GOAL_MAX_LENGTH, .. })
        ));
    }

    #[test]
    fn validate_goal_trims_and_accepts_valid() {
        assert_eq!(validate_goal("  learn Rust  "), Ok("learn Rust".to_string()));
    }

    #[test]
    fn normalize_search_query_rejects_blank() {
        assert_eq!(normalize_search_query("   \n"), None);
    }

    #[test]
    fn validate_difficulty_accepts_range() {
        assert_eq!(validate_difficulty(TASK_DIFFICULTY_MIN), Ok(TASK_DIFFICULTY_MIN));
        assert_eq!(validate_difficulty(TASK_DIFFICULTY_MAX), Ok(TASK_DIFFICULTY_MAX));
    }

    #[test]
    fn validate_difficulty_rejects_out_of_range() {
        assert!(matches!(
            validate_difficulty(0),
            Err(DifficultyValidationError::OutOfRange {
                min: TASK_DIFFICULTY_MIN,
                max: TASK_DIFFICULTY_MAX,
                actual: 0
            })
        ));
    }

    #[test]
    fn validate_status_accepts_valid_values() {
        assert_eq!(validate_status("todo"), Ok("todo".to_string()));
        assert_eq!(validate_status("doing"), Ok("doing".to_string()));
        assert_eq!(validate_status("done"), Ok("done".to_string()));
    }

    #[test]
    fn validate_status_normalizes_case_and_whitespace() {
        assert_eq!(validate_status("  TODO  "), Ok("todo".to_string()));
        assert_eq!(validate_status("Doing"), Ok("doing".to_string()));
        assert_eq!(validate_status("DONE"), Ok("done".to_string()));
    }

    #[test]
    fn validate_status_rejects_invalid() {
        assert!(matches!(
            validate_status("invalid"),
            Err(StatusValidationError::Invalid { .. })
        ));
    }

    #[test]
    fn completed_for_status_maps_correctly() {
        assert!(completed_for_status("done"));
        assert!(!completed_for_status("todo"));
        assert!(!completed_for_status("doing"));
    }

    #[test]
    fn status_for_completed_maps_correctly() {
        assert_eq!(status_for_completed(true), "done");
        assert_eq!(status_for_completed(false), "todo");
    }
}
