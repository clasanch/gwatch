use crate::types::{DiffKind, DiffLine};

#[derive(Debug, Clone)]
pub struct SideBySideLine {
    pub left_num: Option<usize>,
    pub left_content: String,
    pub left_kind: Option<DiffKind>,
    pub right_num: Option<usize>,
    pub right_content: String,
    pub right_kind: Option<DiffKind>,
}

pub fn build_side_by_side_lines(diff_lines: &[DiffLine]) -> Vec<SideBySideLine> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < diff_lines.len() {
        let line = &diff_lines[i];

        match line.kind {
            DiffKind::Context => {
                result.push(SideBySideLine {
                    left_num: line.old_line_number,
                    left_content: line.content.clone(),
                    left_kind: Some(DiffKind::Context),
                    right_num: line.new_line_number,
                    right_content: line.content.clone(),
                    right_kind: Some(DiffKind::Context),
                });
                i += 1;
            }
            DiffKind::Deleted => {
                let mut deletions = Vec::new();
                while i < diff_lines.len() && diff_lines[i].kind == DiffKind::Deleted {
                    deletions.push(&diff_lines[i]);
                    i += 1;
                }

                let mut additions = Vec::new();
                while i < diff_lines.len() && diff_lines[i].kind == DiffKind::Added {
                    additions.push(&diff_lines[i]);
                    i += 1;
                }

                let max_len = deletions.len().max(additions.len());
                for j in 0..max_len {
                    let del = deletions.get(j);
                    let add = additions.get(j);

                    result.push(SideBySideLine {
                        left_num: del.and_then(|d| d.old_line_number),
                        left_content: del.map(|d| d.content.clone()).unwrap_or_default(),
                        left_kind: del.map(|_| DiffKind::Deleted),
                        right_num: add.and_then(|a| a.new_line_number),
                        right_content: add.map(|a| a.content.clone()).unwrap_or_default(),
                        right_kind: add.map(|_| DiffKind::Added),
                    });
                }
            }
            DiffKind::Added => {
                result.push(SideBySideLine {
                    left_num: None,
                    left_content: String::new(),
                    left_kind: None,
                    right_num: line.new_line_number,
                    right_content: line.content.clone(),
                    right_kind: Some(DiffKind::Added),
                });
                i += 1;
            }
        }
    }

    result
}

pub fn truncate_with_offset(s: &str, offset: usize, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if offset >= chars.len() {
        return String::new();
    }
    let end = (offset + max_len).min(chars.len());
    chars[offset..end].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context_line(old: usize, new: usize, content: &str) -> DiffLine {
        DiffLine {
            old_line_number: Some(old),
            new_line_number: Some(new),
            kind: DiffKind::Context,
            content: content.to_string(),
        }
    }

    fn make_added_line(new: usize, content: &str) -> DiffLine {
        DiffLine {
            old_line_number: None,
            new_line_number: Some(new),
            kind: DiffKind::Added,
            content: content.to_string(),
        }
    }

    fn make_deleted_line(old: usize, content: &str) -> DiffLine {
        DiffLine {
            old_line_number: Some(old),
            new_line_number: None,
            kind: DiffKind::Deleted,
            content: content.to_string(),
        }
    }

    #[test]
    fn test_context_lines() {
        let lines = vec![
            make_context_line(1, 1, "line 1"),
            make_context_line(2, 2, "line 2"),
        ];

        let result = build_side_by_side_lines(&lines);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].left_num, Some(1));
        assert_eq!(result[0].right_num, Some(1));
        assert_eq!(result[0].left_content, "line 1");
        assert_eq!(result[0].right_content, "line 1");
    }

    #[test]
    fn test_added_lines() {
        let lines = vec![make_added_line(1, "new line")];

        let result = build_side_by_side_lines(&lines);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].left_num, None);
        assert_eq!(result[0].right_num, Some(1));
        assert_eq!(result[0].left_content, "");
        assert_eq!(result[0].right_content, "new line");
    }

    #[test]
    fn test_deleted_lines() {
        let lines = vec![make_deleted_line(1, "old line")];

        let result = build_side_by_side_lines(&lines);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].left_num, Some(1));
        assert_eq!(result[0].right_num, None);
        assert_eq!(result[0].left_content, "old line");
        assert_eq!(result[0].right_content, "");
    }

    #[test]
    fn test_modification_pairs_deleted_then_added() {
        let lines = vec![make_deleted_line(1, "old"), make_added_line(1, "new")];

        let result = build_side_by_side_lines(&lines);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].left_content, "old");
        assert_eq!(result[0].right_content, "new");
    }

    #[test]
    fn test_unbalanced_deletions() {
        let lines = vec![
            make_deleted_line(1, "old1"),
            make_deleted_line(2, "old2"),
            make_added_line(1, "new1"),
        ];

        let result = build_side_by_side_lines(&lines);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].left_content, "old1");
        assert_eq!(result[0].right_content, "new1");
        assert_eq!(result[1].left_content, "old2");
        assert_eq!(result[1].right_content, "");
    }

    #[test]
    fn test_truncate_with_offset_basic() {
        let s = "Hello, World!";
        assert_eq!(truncate_with_offset(s, 0, 5), "Hello");
        assert_eq!(truncate_with_offset(s, 7, 5), "World");
    }

    #[test]
    fn test_truncate_with_offset_past_end() {
        let s = "Short";
        assert_eq!(truncate_with_offset(s, 10, 5), "");
    }

    #[test]
    fn test_truncate_with_offset_partial() {
        let s = "Hello";
        assert_eq!(truncate_with_offset(s, 3, 10), "lo");
    }
}
