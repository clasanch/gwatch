use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewState {
    pub reviewed_files: HashMap<PathBuf, ReviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEntry {
    pub reviewed_at: chrono::DateTime<chrono::Utc>,
}

impl ReviewState {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state_path() -> PathBuf {
        Config::config_dir().join("review_state.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::state_path()).unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let state: Self = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::state_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_reviewed(&self, path: &Path) -> bool {
        self.reviewed_files.contains_key(path)
    }

    pub fn mark_reviewed(&mut self, path: &Path) {
        self.reviewed_files.insert(
            path.to_path_buf(),
            ReviewEntry {
                reviewed_at: chrono::Utc::now(),
            },
        );
    }

    pub fn unmark_reviewed(&mut self, path: &Path) {
        self.reviewed_files.remove(path);
    }

    pub fn toggle_reviewed(&mut self, path: &Path) {
        if self.is_reviewed(path) {
            self.unmark_reviewed(path);
        } else {
            self.mark_reviewed(path);
        }
    }

    pub fn clear_all(&mut self) {
        self.reviewed_files.clear();
    }

    pub fn reviewed_count(&self) -> usize {
        self.reviewed_files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_review_state_new() {
        let state = ReviewState::new();
        assert!(state.reviewed_files.is_empty());
    }

    #[test]
    fn test_mark_reviewed() {
        let mut state = ReviewState::new();
        let path = PathBuf::from("/test/file.rs");

        assert!(!state.is_reviewed(&path));
        state.mark_reviewed(&path);
        assert!(state.is_reviewed(&path));
    }

    #[test]
    fn test_unmark_reviewed() {
        let mut state = ReviewState::new();
        let path = PathBuf::from("/test/file.rs");

        state.mark_reviewed(&path);
        assert!(state.is_reviewed(&path));

        state.unmark_reviewed(&path);
        assert!(!state.is_reviewed(&path));
    }

    #[test]
    fn test_clear_all() {
        let mut state = ReviewState::new();
        state.mark_reviewed(&PathBuf::from("/test/file1.rs"));
        state.mark_reviewed(&PathBuf::from("/test/file2.rs"));

        assert_eq!(state.reviewed_count(), 2);
        state.clear_all();
        assert_eq!(state.reviewed_count(), 0);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("review_state.json");

        let mut state = ReviewState::new();
        state.mark_reviewed(&PathBuf::from("/test/file.rs"));
        state.save_to(&state_path).unwrap();

        let loaded = ReviewState::load_from(&state_path).unwrap();
        assert!(loaded.is_reviewed(&PathBuf::from("/test/file.rs")));
    }
}
