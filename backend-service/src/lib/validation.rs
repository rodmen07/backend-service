pub(crate) fn normalize_title(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

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

    #[test]
    fn normalize_title_rejects_blank() {
        assert_eq!(normalize_title("   \n"), None);
    }

    #[test]
    fn normalize_title_trims_content() {
        assert_eq!(normalize_title("  hello  "), Some("hello".to_string()));
    }

    #[test]
    fn normalize_search_query_rejects_blank() {
        assert_eq!(normalize_search_query("   \n"), None);
    }
}
