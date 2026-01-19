use gwatch::types::{DiffHunk, DiffKind, DiffStats, FileDiff};

#[test]
fn test_file_diff_default() {
    let diff = FileDiff::default();
    assert!(diff.hunks.is_empty());
    assert!(!diff.is_new_file);
    assert!(!diff.is_deleted);
    assert!(!diff.is_binary);
    assert!(!diff.is_truncated);
    assert_eq!(diff.omitted_lines, 0);
    assert!(diff.truncation_reason.is_none());
}

#[test]
fn test_diff_stats_default() {
    let stats = DiffStats::default();
    assert_eq!(stats.added_count, 0);
    assert_eq!(stats.deleted_count, 0);
}

#[test]
fn test_diff_kind_equality() {
    assert_eq!(DiffKind::Added, DiffKind::Added);
    assert_ne!(DiffKind::Added, DiffKind::Deleted);
}

#[test]
fn test_diff_hunk_default() {
    let hunk = DiffHunk::default();
    assert_eq!(hunk.old_start, 0);
    assert_eq!(hunk.old_count, 0);
    assert!(hunk.lines.is_empty());
}
