use crate::types::{DiffHunk, DiffKind};

/// Calculate display lines count for a list of hunks
pub fn calculate_display_line_count(
    hunks: &[DiffHunk],
    collapsed_hunks: &std::collections::HashSet<usize>,
    collapse_context: bool,
) -> usize {
    let mut count = 0;
    for (idx, hunk) in hunks.iter().enumerate() {
        count += 1; // Hunk header
        if collapsed_hunks.contains(&idx) {
            count += 1; // Summary line
        } else {
            let lines = if collapse_context {
                hunk.lines
                    .iter()
                    .filter(|l| l.kind != DiffKind::Context)
                    .count()
            } else {
                hunk.lines.len()
            };
            count += lines;
        }
    }
    count
}

/// Get style info for a diff line kind
pub fn get_line_style_info(kind: Option<&DiffKind>, is_flashing: bool) -> LineStyleInfo {
    match kind {
        Some(DiffKind::Deleted) => LineStyleInfo {
            is_change: true,
            prefix: "-",
            invert_on_flash: is_flashing,
        },
        Some(DiffKind::Added) => LineStyleInfo {
            is_change: true,
            prefix: "+",
            invert_on_flash: is_flashing,
        },
        Some(DiffKind::Context) => LineStyleInfo {
            is_change: false,
            prefix: " ",
            invert_on_flash: false,
        },
        None => LineStyleInfo {
            is_change: false,
            prefix: " ",
            invert_on_flash: false,
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineStyleInfo {
    pub is_change: bool,
    pub prefix: &'static str,
    pub invert_on_flash: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DiffLine;

    #[test]
    fn test_line_style_deleted() {
        let info = get_line_style_info(Some(&DiffKind::Deleted), false);
        assert_eq!(info.prefix, "-");
        assert!(info.is_change);
        assert!(!info.invert_on_flash);
    }

    #[test]
    fn test_line_style_deleted_flashing() {
        let info = get_line_style_info(Some(&DiffKind::Deleted), true);
        assert!(info.invert_on_flash);
    }

    #[test]
    fn test_line_style_added() {
        let info = get_line_style_info(Some(&DiffKind::Added), false);
        assert_eq!(info.prefix, "+");
        assert!(info.is_change);
    }

    #[test]
    fn test_line_style_context() {
        let info = get_line_style_info(Some(&DiffKind::Context), false);
        assert_eq!(info.prefix, " ");
        assert!(!info.is_change);
    }

    #[test]
    fn test_line_style_none() {
        let info = get_line_style_info(None, false);
        assert_eq!(info.prefix, " ");
        assert!(!info.is_change);
    }

    #[test]
    fn test_calculate_display_empty() {
        let count = calculate_display_line_count(&[], &std::collections::HashSet::new(), false);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_calculate_display_one_hunk() {
        let hunks = vec![DiffHunk {
            old_start: 1,
            old_count: 3,
            new_start: 1,
            new_count: 3,
            lines: vec![
                DiffLine {
                    old_line_number: Some(1),
                    new_line_number: Some(1),
                    kind: DiffKind::Context,
                    content: "a".to_string(),
                },
                DiffLine {
                    old_line_number: Some(2),
                    new_line_number: None,
                    kind: DiffKind::Deleted,
                    content: "b".to_string(),
                },
                DiffLine {
                    old_line_number: None,
                    new_line_number: Some(2),
                    kind: DiffKind::Added,
                    content: "c".to_string(),
                },
            ],
        }];

        let count = calculate_display_line_count(&hunks, &std::collections::HashSet::new(), false);
        assert_eq!(count, 4); // 1 header + 3 lines
    }

    #[test]
    fn test_calculate_display_collapsed_hunk() {
        let hunks = vec![DiffHunk {
            old_start: 1,
            old_count: 3,
            new_start: 1,
            new_count: 3,
            lines: vec![DiffLine {
                old_line_number: Some(1),
                new_line_number: Some(1),
                kind: DiffKind::Context,
                content: "a".to_string(),
            }],
        }];

        let mut collapsed = std::collections::HashSet::new();
        collapsed.insert(0);

        let count = calculate_display_line_count(&hunks, &collapsed, false);
        assert_eq!(count, 2); // 1 header + 1 summary
    }

    #[test]
    fn test_calculate_display_collapse_context() {
        let hunks = vec![DiffHunk {
            old_start: 1,
            old_count: 3,
            new_start: 1,
            new_count: 3,
            lines: vec![
                DiffLine {
                    old_line_number: Some(1),
                    new_line_number: Some(1),
                    kind: DiffKind::Context,
                    content: "a".to_string(),
                },
                DiffLine {
                    old_line_number: Some(2),
                    new_line_number: None,
                    kind: DiffKind::Deleted,
                    content: "b".to_string(),
                },
            ],
        }];

        let count = calculate_display_line_count(&hunks, &std::collections::HashSet::new(), true);
        assert_eq!(count, 2); // 1 header + 1 non-context line
    }
}
