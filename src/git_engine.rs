use anyhow::{Context, Result};
use git2::Repository;
use similar::{ChangeTag, TextDiff};
use std::path::Path;

use crate::types::{DiffHunk, DiffKind, DiffLine, DiffStats, FileDiff};

const LARGE_FILE_WARN_SIZE: u64 = 1024 * 1024; // 1MB
const LARGE_FILE_SKIP_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_DIFF_LINES: usize = 5000;
const TRUNCATE_KEEP_LINES: usize = 100;

pub struct GitEngine {
    repo: Repository,
    repo_root: std::path::PathBuf,
}

impl GitEngine {
    pub fn new(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path)
            .context("Current directory is not a Git repository. gwatch requires Git.")?;

        let repo_root = repo
            .workdir()
            .context("Repository has no working directory")?
            .to_path_buf();

        Ok(Self { repo, repo_root })
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    pub fn compute_diff(&self, file_path: &Path) -> Result<FileDiff> {
        let relative_path = self.to_relative_path(file_path);

        let metadata = match std::fs::metadata(file_path) {
            Ok(m) => m,
            Err(_) => {
                return Ok(FileDiff {
                    is_deleted: true,
                    ..Default::default()
                });
            }
        };

        let file_size = metadata.len();

        if file_size > LARGE_FILE_SKIP_SIZE {
            tracing::info!(
                "Skipping file {:?} ({:.2} MB) - exceeds 10MB limit",
                file_path,
                file_size as f64 / 1024.0 / 1024.0
            );
            return Ok(FileDiff {
                is_truncated: true,
                truncation_reason: Some(format!(
                    "File too large ({:.1} MB) - skipped",
                    file_size as f64 / 1024.0 / 1024.0
                )),
                ..Default::default()
            });
        }

        let mut warn_large_file = false;
        if file_size > LARGE_FILE_WARN_SIZE {
            tracing::warn!(
                "Large file {:?} ({:.2} MB) - diff may be truncated",
                file_path,
                file_size as f64 / 1024.0 / 1024.0
            );
            warn_large_file = true;
        }

        let current_content = match std::fs::read(file_path) {
            Ok(bytes) => {
                if bytes.contains(&0) {
                    return Ok(FileDiff {
                        is_binary: true,
                        ..Default::default()
                    });
                }
                String::from_utf8_lossy(&bytes).to_string()
            }
            Err(_) => {
                return Ok(FileDiff {
                    is_deleted: true,
                    ..Default::default()
                });
            }
        };

        let head_content = self.get_head_content(&relative_path)?;

        let diff = match head_content {
            Some(old_content) => self.diff_strings(&old_content, &current_content)?,
            None => {
                let mut diff = self.diff_strings("", &current_content)?;
                diff.is_new_file = true;
                diff
            }
        };

        self.finalize_diff(diff, file_size, warn_large_file)
    }

    pub fn compute_staged_diff(&self, file_path: &Path) -> Result<FileDiff> {
        let relative_path = self.to_relative_path(file_path);

        let index_content = self.get_index_content(&relative_path)?;
        let head_content = self.get_head_content(&relative_path)?;

        let diff = match (head_content, index_content) {
            (Some(old), Some(new)) => self.diff_strings(&old, &new)?,
            (None, Some(new)) => {
                let mut diff = self.diff_strings("", &new)?;
                diff.is_new_file = true;
                diff
            }
            (Some(_), None) => FileDiff {
                is_deleted: true,
                ..Default::default()
            },
            (None, None) => FileDiff::default(),
        };

        Ok(diff)
    }

    pub fn compute_unstaged_diff(&self, file_path: &Path) -> Result<FileDiff> {
        let relative_path = self.to_relative_path(file_path);

        let metadata = match std::fs::metadata(file_path) {
            Ok(m) => m,
            Err(_) => {
                return Ok(FileDiff {
                    is_deleted: true,
                    ..Default::default()
                });
            }
        };
        let file_size = metadata.len();

        let current_content = match std::fs::read(file_path) {
            Ok(bytes) => {
                if bytes.contains(&0) {
                    return Ok(FileDiff {
                        is_binary: true,
                        ..Default::default()
                    });
                }
                String::from_utf8_lossy(&bytes).to_string()
            }
            Err(_) => {
                return Ok(FileDiff {
                    is_deleted: true,
                    ..Default::default()
                });
            }
        };

        let index_content = self.get_index_content(&relative_path)?;

        let diff = match index_content {
            Some(old) => self.diff_strings(&old, &current_content)?,
            None => {
                // Not in index, check if it's in HEAD
                let head_content = self.get_head_content(&relative_path)?;
                match head_content {
                    Some(old) => self.diff_strings(&old, &current_content)?,
                    None => {
                        let mut diff = self.diff_strings("", &current_content)?;
                        diff.is_new_file = true;
                        diff
                    }
                }
            }
        };

        self.finalize_diff(diff, file_size, false)
    }

    fn to_relative_path(&self, path: &Path) -> std::path::PathBuf {
        match path.strip_prefix(&self.repo_root) {
            Ok(p) => p.to_path_buf(),
            Err(_) => {
                // Try canonicalizing both if strip_prefix fails (handles symlinks on macOS)
                if let (Ok(p_can), Ok(root_can)) =
                    (path.canonicalize(), self.repo_root.canonicalize())
                {
                    p_can
                        .strip_prefix(root_can)
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|_| path.to_path_buf())
                } else {
                    path.to_path_buf()
                }
            }
        }
    }

    fn finalize_diff(
        &self,
        mut diff: FileDiff,
        file_size: u64,
        warn_large_file: bool,
    ) -> Result<FileDiff> {
        let total_lines: usize = diff.hunks.iter().map(|h| h.lines.len()).sum();
        if total_lines > MAX_DIFF_LINES {
            diff = self.truncate_diff(diff, total_lines);
        } else if warn_large_file {
            diff.truncation_reason = Some(format!(
                "Large file ({:.1} MB)",
                file_size as f64 / 1024.0 / 1024.0
            ));
        }

        Ok(diff)
    }

    fn truncate_diff(&self, mut diff: FileDiff, total_lines: usize) -> FileDiff {
        let mut all_lines: Vec<DiffLine> = Vec::new();
        for hunk in &diff.hunks {
            all_lines.extend(hunk.lines.clone());
        }

        let omitted = total_lines.saturating_sub(TRUNCATE_KEEP_LINES * 2);

        let first_lines: Vec<DiffLine> = all_lines
            .iter()
            .take(TRUNCATE_KEEP_LINES)
            .cloned()
            .collect();
        let last_lines: Vec<DiffLine> = all_lines
            .iter()
            .rev()
            .take(TRUNCATE_KEEP_LINES)
            .rev()
            .cloned()
            .collect();

        let first_hunk = DiffHunk {
            old_start: diff.hunks.first().map(|h| h.old_start).unwrap_or(1),
            old_count: first_lines
                .iter()
                .filter(|l| l.old_line_number.is_some())
                .count(),
            new_start: diff.hunks.first().map(|h| h.new_start).unwrap_or(1),
            new_count: first_lines
                .iter()
                .filter(|l| l.new_line_number.is_some())
                .count(),
            lines: first_lines,
        };

        let last_hunk = DiffHunk {
            old_start: last_lines
                .first()
                .and_then(|l| l.old_line_number)
                .unwrap_or(1),
            old_count: last_lines
                .iter()
                .filter(|l| l.old_line_number.is_some())
                .count(),
            new_start: last_lines
                .first()
                .and_then(|l| l.new_line_number)
                .unwrap_or(1),
            new_count: last_lines
                .iter()
                .filter(|l| l.new_line_number.is_some())
                .count(),
            lines: last_lines,
        };

        diff.hunks = vec![first_hunk, last_hunk];
        diff.is_truncated = true;
        diff.omitted_lines = omitted;
        diff.truncation_reason = Some(format!("{omitted} lines omitted"));

        diff
    }

    fn get_head_content(&self, relative_path: &Path) -> Result<Option<String>> {
        let head = match self.repo.head() {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        let tree = head.peel_to_tree()?;
        let entry = match tree.get_path(relative_path) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };

        let blob = self.repo.find_blob(entry.id())?;

        if blob.is_binary() {
            return Ok(None);
        }

        let content = String::from_utf8_lossy(blob.content()).to_string();
        Ok(Some(content))
    }

    fn get_index_content(&self, relative_path: &Path) -> Result<Option<String>> {
        let mut index = self.repo.index()?;
        index.read(true)?; // Force reload from disk
        let entry = match index.get_path(relative_path, 0) {
            Some(e) => e,
            None => return Ok(None),
        };

        let blob = self.repo.find_blob(entry.id)?;

        if blob.is_binary() {
            return Ok(None);
        }

        let content = String::from_utf8_lossy(blob.content()).to_string();
        Ok(Some(content))
    }

    fn diff_strings(&self, old: &str, new: &str) -> Result<FileDiff> {
        let text_diff = TextDiff::from_lines(old, new);

        let mut hunks = Vec::new();
        let mut stats = DiffStats::default();

        for group in text_diff.grouped_ops(3) {
            let mut hunk_lines = Vec::new();
            let mut old_start = 0;
            let mut old_count = 0;
            let mut new_start = 0;
            let mut new_count = 0;
            let mut first = true;

            for op in group {
                for change in text_diff.iter_changes(&op) {
                    let old_ln = change.old_index().map(|i| i + 1);
                    let new_ln = change.new_index().map(|i| i + 1);

                    let kind = match change.tag() {
                        ChangeTag::Equal => DiffKind::Context,
                        ChangeTag::Insert => {
                            stats.added_count += 1;
                            DiffKind::Added
                        }
                        ChangeTag::Delete => {
                            stats.deleted_count += 1;
                            DiffKind::Deleted
                        }
                    };

                    if first {
                        old_start = old_ln.unwrap_or(1);
                        new_start = new_ln.unwrap_or(1);
                        first = false;
                    }

                    if old_ln.is_some() {
                        old_count += 1;
                    }
                    if new_ln.is_some() {
                        new_count += 1;
                    }

                    hunk_lines.push(DiffLine {
                        old_line_number: old_ln,
                        new_line_number: new_ln,
                        kind,
                        content: change.value().trim_end_matches('\n').to_string(),
                    });
                }
            }

            if !hunk_lines.is_empty() {
                hunks.push(DiffHunk {
                    old_start,
                    old_count,
                    new_start,
                    new_count,
                    lines: hunk_lines,
                });
            }
        }

        Ok(FileDiff {
            hunks,
            stats,
            is_new_file: false,
            is_deleted: false,
            is_binary: false,
            is_truncated: false,
            omitted_lines: 0,
            truncation_reason: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("Failed to init git repo");
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .output()
            .expect("Failed to set email");
        Command::new("git")
            .args(["config", "user.name", "test"])
            .current_dir(dir)
            .output()
            .expect("Failed to set name");
    }

    #[test]
    fn test_compute_staged_diff() {
        let temp = TempDir::new().unwrap();
        init_git_repo(temp.path());
        let engine = GitEngine::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "old line\n").unwrap();

        // Commit initial version
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Modify and stage
        fs::write(&file_path, "new line\n").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        let diff = engine.compute_staged_diff(&file_path).unwrap();
        assert_eq!(diff.stats.added_count, 1);
        assert_eq!(diff.stats.deleted_count, 1);
        assert_eq!(diff.hunks[0].lines[0].kind, DiffKind::Deleted);
        assert_eq!(diff.hunks[0].lines[1].kind, DiffKind::Added);
    }

    #[test]
    fn test_compute_unstaged_diff() {
        let temp = TempDir::new().unwrap();
        init_git_repo(temp.path());
        let engine = GitEngine::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "old line\n").unwrap();

        // Commit initial version
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Modify but DON'T stage
        fs::write(&file_path, "new line\n").unwrap();

        let diff = engine.compute_unstaged_diff(&file_path).unwrap();
        assert_eq!(diff.stats.added_count, 1);
        assert_eq!(diff.stats.deleted_count, 1);

        // Stage the change
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Now unstaged diff should be empty
        let diff = engine.compute_unstaged_diff(&file_path).unwrap();
        assert_eq!(diff.stats.added_count, 0);
        assert_eq!(diff.stats.deleted_count, 0);
    }
}
