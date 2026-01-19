/// Format event index display string
pub fn format_event_index(current: usize, total: usize) -> String {
    format!(" [{}/{}]", current + 1, total)
}

/// Format diff stats display string  
pub fn format_diff_stats(added: usize, deleted: usize) -> String {
    format!("+{added} / -{deleted} lines")
}

/// Get file indicator string based on diff state
pub fn get_file_indicator(
    is_new_file: bool,
    is_deleted: bool,
    is_binary: bool,
    is_truncated: bool,
    truncation_reason: Option<&str>,
) -> String {
    if is_new_file {
        " (new file)".to_string()
    } else if is_deleted {
        " (deleted)".to_string()
    } else if is_binary {
        " [binary]".to_string()
    } else if is_truncated {
        if let Some(reason) = truncation_reason {
            format!(" [{reason}]")
        } else {
            " [truncated]".to_string()
        }
    } else {
        String::new()
    }
}

/// Format hunk count display
pub fn format_hunk_info(current: usize, total: usize) -> String {
    format!("Hunk {}/{}", current + 1, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_event_index() {
        assert_eq!(format_event_index(0, 5), " [1/5]");
        assert_eq!(format_event_index(4, 5), " [5/5]");
    }

    #[test]
    fn test_format_diff_stats() {
        assert_eq!(format_diff_stats(10, 5), "+10 / -5 lines");
        assert_eq!(format_diff_stats(0, 0), "+0 / -0 lines");
    }

    #[test]
    fn test_file_indicator_new() {
        assert_eq!(
            get_file_indicator(true, false, false, false, None),
            " (new file)"
        );
    }

    #[test]
    fn test_file_indicator_deleted() {
        assert_eq!(
            get_file_indicator(false, true, false, false, None),
            " (deleted)"
        );
    }

    #[test]
    fn test_file_indicator_binary() {
        assert_eq!(
            get_file_indicator(false, false, true, false, None),
            " [binary]"
        );
    }

    #[test]
    fn test_file_indicator_truncated_with_reason() {
        assert_eq!(
            get_file_indicator(false, false, false, true, Some("too large")),
            " [too large]"
        );
    }

    #[test]
    fn test_file_indicator_truncated_no_reason() {
        assert_eq!(
            get_file_indicator(false, false, false, true, None),
            " [truncated]"
        );
    }

    #[test]
    fn test_file_indicator_normal() {
        assert_eq!(get_file_indicator(false, false, false, false, None), "");
    }

    #[test]
    fn test_format_hunk_info() {
        assert_eq!(format_hunk_info(0, 3), "Hunk 1/3");
        assert_eq!(format_hunk_info(2, 3), "Hunk 3/3");
    }
}
