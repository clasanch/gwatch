use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileDiff {
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
    pub is_new_file: bool,
    pub is_deleted: bool,
    pub is_binary: bool,
    pub is_truncated: bool,
    pub omitted_lines: usize,
    pub truncation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffStats {
    pub added_count: usize,
    pub deleted_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
    pub kind: DiffKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffKind {
    Added,
    Deleted,
    Context,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_mode_default() {
        let mode = DiffMode::default();
        assert_eq!(mode, DiffMode::All);
    }

    #[test]
    fn test_diff_mode_cycle() {
        let mode = DiffMode::All;
        assert_eq!(mode.next(), DiffMode::Unstaged);
        assert_eq!(DiffMode::Unstaged.next(), DiffMode::Staged);
        assert_eq!(DiffMode::Staged.next(), DiffMode::All);
    }

    #[test]
    fn test_diff_mode_display() {
        assert_eq!(DiffMode::All.label(), "All Changes");
        assert_eq!(DiffMode::Unstaged.label(), "Unstaged");
        assert_eq!(DiffMode::Staged.label(), "Staged");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiffMode {
    #[default]
    All, // Working tree vs HEAD
    Unstaged, // Working tree vs Index
    Staged,   // Index vs HEAD
}

impl DiffMode {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Unstaged,
            Self::Unstaged => Self::Staged,
            Self::Staged => Self::All,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Changes",
            Self::Unstaged => "Unstaged",
            Self::Staged => "Staged",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedEvent {
    pub file_path: PathBuf,
    pub relative_path: String,
    pub timestamp: DateTime<Utc>,
    pub diff: FileDiff,
}

#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub timestamp: std::time::SystemTime,
}
