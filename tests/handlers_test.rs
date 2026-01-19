use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gwatch::config::Config;
use gwatch::review_state::ReviewState;
use gwatch::types::{DiffHunk, DiffKind, DiffLine, DiffMode, DisplayedEvent, FileDiff};
use gwatch::ui::app::{App, AppState};
use gwatch::ui::handlers::{get_cursor_position, handle_key_event};
use std::path::PathBuf;

fn test_app() -> App {
    App::new(
        Config::default(),
        PathBuf::from("/tmp/test"),
        ReviewState::default(),
    )
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// Helper to create an app with a valid diff for scrolling
fn app_with_diff() -> App {
    let mut app = test_app();
    let diff = FileDiff {
        hunks: vec![DiffHunk {
            old_start: 1,
            old_count: 50,
            new_start: 1,
            new_count: 50,
            lines: (0..50)
                .map(|i| DiffLine {
                    old_line_number: Some(i),
                    new_line_number: Some(i),
                    kind: DiffKind::Context,
                    content: format!("line {i}"),
                })
                .collect(),
        }],
        ..Default::default()
    };
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff,
    });
    app
}

// === Main keys ===

#[test]
fn test_quit_q() {
    let mut app = test_app();
    handle_key_event(&mut app, key(KeyCode::Char('q'))).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_quit_esc() {
    let mut app = test_app();
    handle_key_event(&mut app, key(KeyCode::Esc)).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_toggle_pause_space() {
    let mut app = test_app();
    assert!(!app.is_paused());
    handle_key_event(&mut app, key(KeyCode::Char(' '))).unwrap();
    assert!(app.is_paused());
    handle_key_event(&mut app, key(KeyCode::Char(' '))).unwrap();
    assert!(!app.is_paused());
}

#[test]
fn test_scroll_up_arrow() {
    let mut app = app_with_diff();
    app.diff_scroll_offset = 5;
    handle_key_event(&mut app, key(KeyCode::Up)).unwrap();
    assert_eq!(app.diff_scroll_offset, 4);
}

#[test]
fn test_scroll_up_k() {
    let mut app = app_with_diff();
    app.diff_scroll_offset = 5;
    handle_key_event(&mut app, key(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.diff_scroll_offset, 4);
}

#[test]
fn test_scroll_down_arrow() {
    let mut app = app_with_diff();
    handle_key_event(&mut app, key(KeyCode::Down)).unwrap();
    assert_eq!(app.diff_scroll_offset, 1);
}

#[test]
fn test_scroll_down_j() {
    let mut app = app_with_diff();
    handle_key_event(&mut app, key(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.diff_scroll_offset, 1);
}

#[test]
fn test_page_up() {
    let mut app = app_with_diff();
    app.diff_scroll_offset = 15;
    handle_key_event(&mut app, key(KeyCode::PageUp)).unwrap();
    assert_eq!(app.diff_scroll_offset, 5);
}

#[test]
fn test_page_down() {
    let mut app = app_with_diff();
    handle_key_event(&mut app, key(KeyCode::PageDown)).unwrap();
    assert_eq!(app.diff_scroll_offset, 10);
}

#[test]
fn test_horizontal_scroll_left() {
    let mut app = app_with_diff();
    app.diff_horizontal_offset = 15;
    handle_key_event(&mut app, key(KeyCode::Left)).unwrap();
    assert_eq!(app.diff_horizontal_offset, 5);
}

#[test]
fn test_horizontal_scroll_right() {
    let mut app = app_with_diff();
    handle_key_event(&mut app, key(KeyCode::Right)).unwrap();
    assert_eq!(app.diff_horizontal_offset, 10);
}

#[test]
fn test_hunk_navigation() {
    let mut app = test_app();
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
        ],
        ..Default::default()
    };
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff,
    });

    handle_key_event(&mut app, key(KeyCode::Char(']'))).unwrap();
    assert_eq!(app.hunk_state.focused_hunk, 1);

    handle_key_event(&mut app, key(KeyCode::Char('['))).unwrap();
    assert_eq!(app.hunk_state.focused_hunk, 0);
}

#[test]
fn test_toggle_hunk_collapsed_z() {
    let mut app = test_app();
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff {
            hunks: vec![DiffHunk::default()],
            ..Default::default()
        },
    });

    assert!(!app.hunk_state.is_collapsed(0));
    handle_key_event(&mut app, key(KeyCode::Char('z'))).unwrap();
    assert!(app.hunk_state.is_collapsed(0));
}

#[test]
fn test_toggle_context_collapsed_shift_z() {
    let mut app = test_app();
    assert!(!app.hunk_state.collapse_context);
    handle_key_event(&mut app, key(KeyCode::Char('Z'))).unwrap();
    assert!(app.hunk_state.collapse_context);
}

#[test]
fn test_clear_history_c() {
    let mut app = test_app();
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff::default(),
    });

    assert!(!app.events.is_empty());
    handle_key_event(&mut app, key(KeyCode::Char('c'))).unwrap();
    assert!(app.events.is_empty());
}

#[test]
fn test_open_theme_selector_t() {
    let mut app = test_app();
    handle_key_event(&mut app, key(KeyCode::Char('t'))).unwrap();
    assert_eq!(app.state, AppState::ThemeSelector);
}

#[test]
fn test_cycle_diff_mode_m() {
    let mut app = test_app();
    assert_eq!(app.diff_mode, DiffMode::All);
    handle_key_event(&mut app, key(KeyCode::Char('m'))).unwrap();
    assert_eq!(app.diff_mode, DiffMode::Unstaged);
}

#[test]
fn test_open_help() {
    let mut app = test_app();
    handle_key_event(&mut app, key(KeyCode::Char('?'))).unwrap();
    assert_eq!(app.state, AppState::HelpPanel);
}

#[test]
fn test_open_settings_s() {
    let mut app = test_app();
    handle_key_event(&mut app, key(KeyCode::Char('s'))).unwrap();
    assert_eq!(app.state, AppState::SettingsEditor);
}

// === Theme selector keys ===

#[test]
fn test_theme_selector_close_esc() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;
    handle_key_event(&mut app, key(KeyCode::Esc)).unwrap();
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_theme_selector_navigate() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;

    handle_key_event(&mut app, key(KeyCode::Down)).unwrap();
    assert_eq!(app.selected_theme_index, 1);

    handle_key_event(&mut app, key(KeyCode::Up)).unwrap();
    assert_eq!(app.selected_theme_index, 0);
}

#[test]
fn test_theme_selector_select_enter() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;
    app.selected_theme_index = 4; // monochrome

    handle_key_event(&mut app, key(KeyCode::Enter)).unwrap();
    assert_eq!(app.state, AppState::Running);
    assert_eq!(app.theme.name, "Monochrome");
}

// === Help panel ===

#[test]
fn test_help_panel_any_key_closes() {
    let mut app = test_app();
    app.state = AppState::HelpPanel;
    handle_key_event(&mut app, key(KeyCode::Char('x'))).unwrap();
    assert_eq!(app.state, AppState::Running);
}

// === Settings editor ===

#[test]
fn test_settings_editor_esc_closes() {
    let mut app = test_app();
    app.state = AppState::SettingsEditor;
    handle_key_event(&mut app, key(KeyCode::Esc)).unwrap();
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_settings_editor_navigation() {
    let mut app = test_app();
    app.open_settings_editor();

    // Move cursor
    handle_key_event(&mut app, key(KeyCode::Down)).unwrap();
    assert_eq!(app.settings_editor.cursor_line, 1);

    handle_key_event(&mut app, key(KeyCode::Right)).unwrap();
    assert_eq!(app.settings_editor.cursor_col, 1);

    handle_key_event(&mut app, key(KeyCode::Home)).unwrap();
    assert_eq!(app.settings_editor.cursor_col, 0);
}

#[test]
fn test_settings_editor_typing() {
    let mut app = test_app();
    app.open_settings_editor();
    let original_len = app.settings_editor.content.len();

    handle_key_event(&mut app, key(KeyCode::Char('x'))).unwrap();
    assert_eq!(app.settings_editor.content.len(), original_len + 1);
}

#[test]
fn test_settings_editor_backspace() {
    let mut app = test_app();
    app.settings_editor.content = "abc".to_string();
    app.settings_editor.cursor_col = 3;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Backspace)).unwrap();
    assert_eq!(app.settings_editor.content, "ab");
}

#[test]
fn test_settings_editor_tab() {
    let mut app = test_app();
    app.settings_editor.content = "".to_string();
    app.settings_editor.cursor_col = 0;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Tab)).unwrap();
    assert_eq!(app.settings_editor.content, "  ");
    assert_eq!(app.settings_editor.cursor_col, 2);
}

// === get_cursor_position tests ===

#[test]
fn test_get_cursor_position_first_line() {
    assert_eq!(get_cursor_position("hello\nworld", 0, 2), 2);
}

#[test]
fn test_get_cursor_position_second_line() {
    assert_eq!(get_cursor_position("hello\nworld", 1, 2), 8);
}

#[test]
fn test_get_cursor_position_end_of_content() {
    assert_eq!(get_cursor_position("abc", 5, 0), 3);
}

#[test]
fn test_settings_editor_delete() {
    let mut app = test_app();
    app.settings_editor.content = "abc".to_string();
    app.settings_editor.cursor_col = 1;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Delete)).unwrap();
    assert_eq!(app.settings_editor.content, "ac");
}

#[test]
fn test_settings_editor_ctrl_s_success() {
    let mut app = test_app();
    app.open_settings_editor();
    app.state = AppState::SettingsEditor;

    // Content is already valid JSON from Config::default()
    handle_key_event(
        &mut app,
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
    )
    .unwrap();

    // Should return to Running state
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_settings_editor_ctrl_s_invalid_json() {
    let mut app = test_app();
    app.open_settings_editor();
    app.settings_editor.content = "{ invalid }".to_string();
    app.state = AppState::SettingsEditor;

    handle_key_event(
        &mut app,
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
    )
    .unwrap();

    // Should stay in SettingsEditor due to error
    assert_eq!(app.state, AppState::SettingsEditor);
}

#[test]
fn test_settings_editor_home_end() {
    let mut app = test_app();
    app.settings_editor.content = "line1".to_string();
    app.settings_editor.cursor_col = 3;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Home)).unwrap();
    assert_eq!(app.settings_editor.cursor_col, 0);

    handle_key_event(&mut app, key(KeyCode::End)).unwrap();
    assert_eq!(app.settings_editor.cursor_col, 5);
}

#[test]
fn test_handle_main_keys_unhandled() {
    let mut app = test_app();
    // Should not panic or change state for random keys
    let original_state = app.state.clone();
    handle_key_event(&mut app, key(KeyCode::F(1))).unwrap();
    assert_eq!(app.state, original_state);
}

// === Additional handler tests for coverage ===

#[test]
fn test_settings_editor_enter_newline() {
    let mut app = test_app();
    app.settings_editor.content = "ab".to_string();
    app.settings_editor.cursor_col = 1;
    app.settings_editor.cursor_line = 0;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Enter)).unwrap();
    assert!(app.settings_editor.content.contains('\n'));
    assert_eq!(app.settings_editor.cursor_line, 1);
    assert_eq!(app.settings_editor.cursor_col, 0);
}

#[test]
fn test_settings_editor_cursor_up_at_top() {
    let mut app = test_app();
    app.settings_editor.content = "line1\nline2".to_string();
    app.settings_editor.cursor_line = 0;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Up)).unwrap();
    assert_eq!(app.settings_editor.cursor_line, 0); // Should stay at 0
}

#[test]
fn test_settings_editor_cursor_down_at_bottom() {
    let mut app = test_app();
    app.settings_editor.content = "line1\nline2".to_string();
    app.settings_editor.cursor_line = 1;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Down)).unwrap();
    assert_eq!(app.settings_editor.cursor_line, 1); // Should stay at bottom
}

#[test]
fn test_settings_editor_left_wraps_to_prev_line() {
    let mut app = test_app();
    app.settings_editor.content = "line1\nline2".to_string();
    app.settings_editor.cursor_line = 1;
    app.settings_editor.cursor_col = 0;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Left)).unwrap();
    assert_eq!(app.settings_editor.cursor_line, 0);
    assert_eq!(app.settings_editor.cursor_col, 5); // End of "line1"
}

#[test]
fn test_settings_editor_right_wraps_to_next_line() {
    let mut app = test_app();
    app.settings_editor.content = "line1\nline2".to_string();
    app.settings_editor.cursor_line = 0;
    app.settings_editor.cursor_col = 5; // At end of "line1"
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Right)).unwrap();
    assert_eq!(app.settings_editor.cursor_line, 1);
    assert_eq!(app.settings_editor.cursor_col, 0);
}

#[test]
fn test_settings_editor_backspace_joins_lines() {
    let mut app = test_app();
    app.settings_editor.content = "ab\ncd".to_string();
    app.settings_editor.cursor_line = 1;
    app.settings_editor.cursor_col = 0;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Backspace)).unwrap();
    assert_eq!(app.settings_editor.content, "abcd");
    assert_eq!(app.settings_editor.cursor_line, 0);
    assert_eq!(app.settings_editor.cursor_col, 2);
}

#[test]
fn test_settings_editor_backspace_at_start() {
    let mut app = test_app();
    app.settings_editor.content = "abc".to_string();
    app.settings_editor.cursor_line = 0;
    app.settings_editor.cursor_col = 0;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Backspace)).unwrap();
    assert_eq!(app.settings_editor.content, "abc"); // No change
}

#[test]
fn test_settings_editor_delete_at_end() {
    let mut app = test_app();
    app.settings_editor.content = "abc".to_string();
    app.settings_editor.cursor_col = 3;
    app.state = AppState::SettingsEditor;

    handle_key_event(&mut app, key(KeyCode::Delete)).unwrap();
    assert_eq!(app.settings_editor.content, "abc"); // No change
}

#[test]
fn test_toggle_review_r() {
    let mut app = test_app();
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff::default(),
    });

    handle_key_event(&mut app, key(KeyCode::Char('r'))).unwrap();
    assert!(app
        .review_state
        .is_reviewed(&PathBuf::from("/test/file.rs")));
}

#[test]
fn test_clear_all_reviewed_shift_r() {
    let mut app = test_app();
    app.review_state
        .mark_reviewed(&PathBuf::from("/test/file.rs"));
    assert!(app
        .review_state
        .is_reviewed(&PathBuf::from("/test/file.rs")));

    handle_key_event(&mut app, key(KeyCode::Char('R'))).unwrap();
    assert!(!app
        .review_state
        .is_reviewed(&PathBuf::from("/test/file.rs")));
}

#[test]
fn test_event_navigation_p_n() {
    let mut app = test_app();
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file1.rs"),
        relative_path: "file1.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff::default(),
    });
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file2.rs"),
        relative_path: "file2.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff::default(),
    });

    // 'p' scrolls to previous (older) events, 'n' scrolls to next (newer)
    assert_eq!(app.scroll_offset, 0);
    handle_key_event(&mut app, key(KeyCode::Char('p'))).unwrap();
    assert_eq!(app.scroll_offset, 1); // scrolled to older event
    handle_key_event(&mut app, key(KeyCode::Char('n'))).unwrap();
    assert_eq!(app.scroll_offset, 0); // back to newest
}

#[test]
fn test_horizontal_scroll_h_l() {
    let mut app = app_with_diff();

    handle_key_event(&mut app, key(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.diff_horizontal_offset, 10);

    handle_key_event(&mut app, key(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.diff_horizontal_offset, 0);
}

#[test]
fn test_theme_selector_close_q() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;
    handle_key_event(&mut app, key(KeyCode::Char('q'))).unwrap();
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_theme_selector_close_t() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;
    handle_key_event(&mut app, key(KeyCode::Char('t'))).unwrap();
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_theme_selector_k_j_navigation() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;

    handle_key_event(&mut app, key(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_theme_index, 1);

    handle_key_event(&mut app, key(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.selected_theme_index, 0);
}

#[test]
fn test_settings_editor_unhandled_key() {
    let mut app = test_app();
    app.state = AppState::SettingsEditor;
    let original_content = app.settings_editor.content.clone();

    handle_key_event(&mut app, key(KeyCode::F(5))).unwrap();
    assert_eq!(app.settings_editor.content, original_content);
}

#[test]
fn test_theme_selector_unhandled_key() {
    let mut app = test_app();
    app.state = AppState::ThemeSelector;
    let original_index = app.selected_theme_index;

    handle_key_event(&mut app, key(KeyCode::F(5))).unwrap();
    assert_eq!(app.selected_theme_index, original_index);
}
