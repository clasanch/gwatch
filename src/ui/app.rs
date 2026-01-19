use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::review_state::ReviewState;
use crate::types::{DiffMode, DisplayedEvent};

use super::diff_view::build_side_by_side_lines;
use super::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Running,
    Paused,
    ThemeSelector,
    HelpPanel,
    SettingsEditor,
}

#[derive(Debug, Clone, Default)]
pub struct HunkViewState {
    pub focused_hunk: usize,
    pub collapsed_hunks: HashSet<usize>,
    pub collapse_context: bool,
}

impl HunkViewState {
    pub fn is_collapsed(&self, hunk_index: usize) -> bool {
        self.collapsed_hunks.contains(&hunk_index)
    }

    pub fn toggle_collapsed(&mut self, hunk_index: usize) {
        if self.collapsed_hunks.contains(&hunk_index) {
            self.collapsed_hunks.remove(&hunk_index);
        } else {
            self.collapsed_hunks.insert(hunk_index);
        }
    }

    pub fn focus_next(&mut self, total_hunks: usize) {
        if total_hunks == 0 {
            return;
        }
        self.focused_hunk = (self.focused_hunk + 1) % total_hunks;
    }

    pub fn focus_prev(&mut self, total_hunks: usize) {
        if total_hunks == 0 {
            return;
        }
        if self.focused_hunk == 0 {
            self.focused_hunk = total_hunks - 1;
        } else {
            self.focused_hunk -= 1;
        }
    }

    pub fn toggle_collapse_context(&mut self) {
        self.collapse_context = !self.collapse_context;
    }

    pub fn reset(&mut self) {
        self.focused_hunk = 0;
        self.collapsed_hunks.clear();
    }
}

pub struct App {
    pub events: VecDeque<DisplayedEvent>,
    pub state: AppState,
    pub scroll_offset: usize,
    pub diff_scroll_offset: usize,
    pub diff_horizontal_offset: usize,
    pub config: Config,
    pub theme: Theme,
    pub hunk_state: HunkViewState,
    pub review_state: ReviewState,
    pub diff_mode: DiffMode,
    pub max_events: usize,
    pub selected_theme_index: usize,
    pub should_quit: bool,
    #[allow(dead_code)]
    pub repo_root: PathBuf,
    pub settings_editor: SettingsEditorState,
    pub flash_until: Option<Instant>,
}

#[derive(Debug, Clone, Default)]
pub struct SettingsEditorState {
    pub content: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub error_message: Option<String>,
}

impl App {
    pub fn new(config: Config, repo_root: PathBuf, review_state: ReviewState) -> Self {
        let theme = Theme::by_name(&config.theme.name);
        let max_events = config.watcher.max_events_buffer;

        Self {
            events: VecDeque::with_capacity(max_events),
            state: AppState::Running,
            scroll_offset: 0,
            diff_scroll_offset: 0,
            diff_horizontal_offset: 0,
            config,
            theme,
            hunk_state: HunkViewState::default(),
            review_state,
            diff_mode: DiffMode::default(),
            max_events,
            selected_theme_index: 0,
            should_quit: false,
            repo_root,
            settings_editor: SettingsEditorState::default(),
            flash_until: None,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.state == AppState::Paused
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            AppState::Running => AppState::Paused,
            AppState::Paused => {
                self.scroll_offset = 0;
                AppState::Running
            }
            _ => self.state.clone(),
        };
    }

    pub fn cycle_diff_mode(&mut self) {
        self.diff_mode = self.diff_mode.next();
        tracing::info!("Diff mode changed to: {:?}", self.diff_mode);
    }

    pub fn get_current_hunk_count(&self) -> usize {
        self.get_current_event()
            .map(|e| e.diff.hunks.len())
            .unwrap_or(0)
    }

    pub fn next_hunk(&mut self) {
        let count = self.get_current_hunk_count();
        self.hunk_state.focus_next(count);
        self.scroll_to_focused_hunk();
    }

    pub fn prev_hunk(&mut self) {
        let count = self.get_current_hunk_count();
        self.hunk_state.focus_prev(count);
        self.scroll_to_focused_hunk();
    }

    pub fn toggle_current_hunk_collapsed(&mut self) {
        self.hunk_state
            .toggle_collapsed(self.hunk_state.focused_hunk);
    }

    pub fn toggle_context_collapsed(&mut self) {
        self.hunk_state.toggle_collapse_context();
    }

    pub fn toggle_current_reviewed(&mut self) {
        if let Some(event) = self.get_current_event() {
            let path = event.file_path.clone();
            self.review_state.toggle_reviewed(&path);
            if let Err(e) = self.review_state.save() {
                tracing::warn!("Failed to save review state: {}", e);
            }
        }
    }

    pub fn clear_all_reviewed(&mut self) {
        self.review_state.clear_all();
        if let Err(e) = self.review_state.save() {
            tracing::warn!("Failed to save review state: {}", e);
        }
    }

    fn scroll_to_focused_hunk(&mut self) {
        if let Some(event) = self.get_current_event() {
            let mut line_offset = 0;
            for (i, hunk) in event.diff.hunks.iter().enumerate() {
                if i == self.hunk_state.focused_hunk {
                    self.diff_scroll_offset = line_offset;
                    return;
                }
                if !self.hunk_state.is_collapsed(i) {
                    line_offset += hunk.lines.len();
                } else {
                    line_offset += 1; // Collapsed shows 1 summary line
                }
            }
        }
    }

    pub fn add_event(&mut self, event: DisplayedEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_back();
        }

        // Calculate scroll offset to focus on first actual change (skip context lines)
        let first_change_offset = self.find_first_change_offset(&event);

        self.events.push_front(event);

        // Always set flash effect for visual feedback
        self.flash_until = Some(Instant::now() + Duration::from_millis(1500));

        if !self.is_paused() {
            self.scroll_offset = 0;
            self.diff_scroll_offset = first_change_offset;
            self.diff_horizontal_offset = 0;
            self.hunk_state.reset();
            tracing::info!(
                "New event: scroll to line {}, flash enabled",
                first_change_offset
            );
        } else {
            tracing::info!("New event (paused): flash only, no scroll");
        }
    }

    pub fn is_flashing(&self) -> bool {
        self.flash_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }

    fn find_first_change_offset(&self, event: &DisplayedEvent) -> usize {
        use crate::types::DiffKind;

        let all_lines: Vec<_> = event
            .diff
            .hunks
            .iter()
            .flat_map(|h| h.lines.clone())
            .collect();

        tracing::debug!("find_first_change_offset: {} raw lines", all_lines.len());

        let side_by_side = build_side_by_side_lines(&all_lines);

        tracing::debug!(
            "find_first_change_offset: {} side-by-side lines",
            side_by_side.len()
        );

        // Find first non-context line (actual change)
        for (idx, line) in side_by_side.iter().enumerate() {
            let is_change = matches!(line.left_kind, Some(DiffKind::Deleted))
                || matches!(line.right_kind, Some(DiffKind::Added));
            if is_change {
                tracing::info!(
                    "First change at line {}, scrolling to {}",
                    idx,
                    idx.saturating_sub(2)
                );
                // Return a few lines before to show context
                return idx.saturating_sub(2);
            }
        }

        tracing::warn!("No changes found in diff, returning 0");
        0
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset < self.events.len().saturating_sub(1) {
            self.scroll_offset += 1;
            self.diff_scroll_offset = 0;
            self.diff_horizontal_offset = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.diff_scroll_offset = 0;
            self.diff_horizontal_offset = 0;
        }
    }

    pub fn diff_scroll_up(&mut self, lines: usize) {
        self.diff_scroll_offset = self.diff_scroll_offset.saturating_sub(lines);
    }

    pub fn diff_scroll_down(&mut self, lines: usize, max_lines: usize) {
        if max_lines == 0 {
            self.diff_scroll_offset = 0;
            return;
        }
        let new_offset = self.diff_scroll_offset.saturating_add(lines);
        // Don't scroll past the last line
        self.diff_scroll_offset = new_offset.min(max_lines.saturating_sub(1));
    }

    pub fn diff_scroll_left(&mut self) {
        self.diff_horizontal_offset = self.diff_horizontal_offset.saturating_sub(10);
    }

    pub fn diff_scroll_right(&mut self) {
        self.diff_horizontal_offset += 10;
    }

    pub fn clear_history(&mut self) {
        self.events.clear();
        self.scroll_offset = 0;
    }

    pub fn open_theme_selector(&mut self) {
        let themes = Theme::available_themes();
        self.selected_theme_index = themes
            .iter()
            .position(|t| t.to_lowercase() == self.config.theme.name.to_lowercase())
            .unwrap_or(0);
        self.state = AppState::ThemeSelector;
    }

    pub fn close_overlay(&mut self) {
        self.state = if self.is_paused() {
            AppState::Paused
        } else {
            AppState::Running
        };
    }

    pub fn open_help(&mut self) {
        self.state = AppState::HelpPanel;
    }

    pub fn open_settings_editor(&mut self) {
        let json = serde_json::to_string_pretty(&self.config).unwrap_or_default();
        self.settings_editor = SettingsEditorState {
            content: json,
            cursor_line: 0,
            cursor_col: 0,
            error_message: None,
        };
        self.state = AppState::SettingsEditor;
    }

    pub fn save_settings(&mut self) -> bool {
        match serde_json::from_str::<Config>(&self.settings_editor.content) {
            Ok(new_config) => {
                self.theme = Theme::by_name(&new_config.theme.name);
                self.max_events = new_config.watcher.max_events_buffer;
                self.config = new_config;
                if let Err(e) = self.config.save() {
                    self.settings_editor.error_message = Some(format!("Save failed: {e}"));
                    return false;
                }
                self.settings_editor.error_message = None;
                true
            }
            Err(e) => {
                self.settings_editor.error_message = Some(format!("Invalid JSON: {e}"));
                false
            }
        }
    }

    pub fn select_theme(&mut self, index: usize) {
        let themes = Theme::available_themes();
        if index < themes.len() {
            self.config.theme.name = themes[index].to_string();
            self.theme = Theme::by_name(themes[index]);
            let _ = self.config.save();
        }
    }

    pub fn theme_selector_up(&mut self) {
        let count = Theme::available_themes().len();
        if self.selected_theme_index > 0 {
            self.selected_theme_index -= 1;
        } else {
            self.selected_theme_index = count - 1;
        }
    }

    pub fn theme_selector_down(&mut self) {
        let count = Theme::available_themes().len();
        self.selected_theme_index = (self.selected_theme_index + 1) % count;
    }

    pub fn get_current_event(&self) -> Option<&DisplayedEvent> {
        self.events.get(self.scroll_offset)
    }

    pub fn get_first_changed_line(&self) -> Option<usize> {
        self.get_current_event().and_then(|event| {
            event.diff.hunks.first().and_then(|hunk| {
                hunk.lines
                    .first()
                    .and_then(|line| line.new_line_number.or(line.old_line_number))
            })
        })
    }

    pub fn get_current_diff_line_count(&self) -> usize {
        self.get_current_event()
            .map(|event| {
                let all_lines: Vec<_> = event
                    .diff
                    .hunks
                    .iter()
                    .flat_map(|h| h.lines.clone())
                    .collect();
                build_side_by_side_lines(&all_lines).len()
            })
            .unwrap_or(0)
    }

    pub fn reload_config(&mut self) {
        match Config::load() {
            Ok(new_config) => {
                tracing::info!("Config reloaded: theme={}", new_config.theme.name);
                self.theme = Theme::by_name(&new_config.theme.name);
                self.max_events = new_config.watcher.max_events_buffer;
                self.config = new_config;
            }
            Err(e) => {
                tracing::warn!("Failed to reload config: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::{DiffHunk, FileDiff};
    use std::path::PathBuf;

    fn test_app() -> App {
        App::new(
            Config::default(),
            PathBuf::from("/tmp/test"),
            ReviewState::default(),
        )
    }

    #[test]
    fn test_hunk_view_state_default() {
        let state = HunkViewState::default();
        assert_eq!(state.focused_hunk, 0);
        assert!(state.collapsed_hunks.is_empty());
        assert!(!state.collapse_context);
    }

    #[test]
    fn test_toggle_hunk_collapsed() {
        let mut state = HunkViewState::default();

        assert!(!state.is_collapsed(0));
        state.toggle_collapsed(0);
        assert!(state.is_collapsed(0));
        state.toggle_collapsed(0);
        assert!(!state.is_collapsed(0));
    }

    #[test]
    fn test_focus_next_hunk() {
        let mut state = HunkViewState::default();
        state.focus_next(3); // 3 total hunks
        assert_eq!(state.focused_hunk, 1);
        state.focus_next(3);
        assert_eq!(state.focused_hunk, 2);
        state.focus_next(3); // wrap around
        assert_eq!(state.focused_hunk, 0);
    }

    #[test]
    fn test_focus_prev_hunk() {
        let mut state = HunkViewState::default();
        state.focus_prev(3); // wrap to last
        assert_eq!(state.focused_hunk, 2);
        state.focus_prev(3);
        assert_eq!(state.focused_hunk, 1);
    }

    #[test]
    fn test_collapse_all_context() {
        let mut state = HunkViewState::default();
        assert!(!state.collapse_context);
        state.toggle_collapse_context();
        assert!(state.collapse_context);
    }

    #[test]
    fn test_app_hunk_navigation() {
        let mut app = test_app();
        // Add event with multiple hunks
        let diff = FileDiff {
            hunks: vec![
                DiffHunk {
                    old_start: 1,
                    old_count: 3,
                    new_start: 1,
                    new_count: 4,
                    lines: vec![],
                },
                DiffHunk {
                    old_start: 10,
                    old_count: 2,
                    new_start: 11,
                    new_count: 3,
                    lines: vec![],
                },
                DiffHunk {
                    old_start: 20,
                    old_count: 1,
                    new_start: 22,
                    new_count: 2,
                    lines: vec![],
                },
            ],
            ..Default::default()
        };
        let event = DisplayedEvent {
            file_path: PathBuf::from("/test/file.rs"),
            relative_path: "file.rs".to_string(),
            timestamp: chrono::Utc::now(),
            diff,
        };
        app.events.push_front(event);

        assert_eq!(app.get_current_hunk_count(), 3);
        assert_eq!(app.hunk_state.focused_hunk, 0);

        app.next_hunk();
        assert_eq!(app.hunk_state.focused_hunk, 1);

        app.toggle_current_hunk_collapsed();
        assert!(app.hunk_state.is_collapsed(1));
    }

    #[test]
    fn test_app_has_review_state() {
        let app = test_app();
        assert_eq!(app.review_state.reviewed_count(), 0);
    }

    #[test]
    fn test_toggle_current_reviewed() {
        let mut app = test_app();
        // Add a mock event
        let event = DisplayedEvent {
            file_path: PathBuf::from("/test/file.rs"),
            relative_path: "file.rs".to_string(),
            timestamp: chrono::Utc::now(),
            diff: FileDiff::default(),
        };
        let path = event.file_path.clone();
        app.events.push_front(event);

        assert!(!app.review_state.is_reviewed(&path));
        app.toggle_current_reviewed();
        assert!(app.review_state.is_reviewed(&path));
    }

    #[test]
    fn test_diff_mode_default() {
        let app = test_app();
        assert_eq!(app.diff_mode, DiffMode::All);
    }

    #[test]
    fn test_cycle_diff_mode() {
        let mut app = test_app();
        assert_eq!(app.diff_mode, DiffMode::All);

        app.cycle_diff_mode();
        assert_eq!(app.diff_mode, DiffMode::Unstaged);

        app.cycle_diff_mode();
        assert_eq!(app.diff_mode, DiffMode::Staged);

        app.cycle_diff_mode();
        assert_eq!(app.diff_mode, DiffMode::All);
    }
}
