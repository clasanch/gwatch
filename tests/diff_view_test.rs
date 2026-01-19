use gwatch::types::{DiffKind, DiffLine};
use gwatch::ui::diff_view::{build_side_by_side_lines, truncate_with_offset};

fn make_deleted_line(num: usize, content: &str) -> DiffLine {
    DiffLine {
        old_line_number: Some(num),
        new_line_number: None,
        kind: DiffKind::Deleted,
        content: content.to_string(),
    }
}

fn make_added_line(num: usize, content: &str) -> DiffLine {
    DiffLine {
        old_line_number: None,
        new_line_number: Some(num),
        kind: DiffKind::Added,
        content: content.to_string(),
    }
}

fn make_context_line(old: usize, new: usize, content: &str) -> DiffLine {
    DiffLine {
        old_line_number: Some(old),
        new_line_number: Some(new),
        kind: DiffKind::Context,
        content: content.to_string(),
    }
}

#[test]
fn test_empty_diff_lines() {
    let result = build_side_by_side_lines(&[]);
    assert!(result.is_empty());
}

#[test]
fn test_unbalanced_additions() {
    let lines = vec![
        make_deleted_line(1, "old"),
        make_added_line(1, "new1"),
        make_added_line(2, "new2"),
        make_added_line(3, "new3"),
    ];

    let result = build_side_by_side_lines(&lines);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].left_content, "old");
    assert_eq!(result[0].right_content, "new1");
    assert_eq!(result[1].left_content, "");
    assert_eq!(result[1].right_content, "new2");
}

#[test]
fn test_truncate_unicode() {
    let s = "Hello 世界!";
    assert_eq!(truncate_with_offset(s, 0, 6), "Hello ");
    assert_eq!(truncate_with_offset(s, 6, 2), "世界");
}

#[test]
fn test_truncate_empty_string() {
    assert_eq!(truncate_with_offset("", 0, 10), "");
    assert_eq!(truncate_with_offset("", 5, 10), "");
}

#[test]
fn test_mixed_context_and_changes() {
    let lines = vec![
        make_context_line(1, 1, "context before"),
        make_deleted_line(2, "removed"),
        make_added_line(2, "added"),
        make_context_line(3, 3, "context after"),
    ];

    let result = build_side_by_side_lines(&lines);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].left_kind, Some(DiffKind::Context));
    assert_eq!(result[1].left_kind, Some(DiffKind::Deleted));
    assert_eq!(result[1].right_kind, Some(DiffKind::Added));
    assert_eq!(result[2].left_kind, Some(DiffKind::Context));
}
