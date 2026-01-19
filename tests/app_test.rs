use gwatch::config::Config;
use gwatch::review_state::ReviewState;
use gwatch::types::{DisplayedEvent, FileDiff};
use gwatch::ui::app::{App, AppState};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn test_app() -> App {
    App::new(
        Config::default(),
        PathBuf::from("/tmp/test"),
        ReviewState::default(),
    )
}

#[test]
fn test_add_event_respects_max_events() {
    let mut app = test_app();
    app.max_events = 3;

    for i in 0..5 {
        app.add_event(DisplayedEvent {
            file_path: PathBuf::from(format!("/test/file{i}.rs")),
            relative_path: format!("file{i}.rs"),
            timestamp: chrono::Utc::now(),
            diff: FileDiff::default(),
        });
    }

    assert_eq!(app.events.len(), 3);
}

#[test]
fn test_add_event_triggers_flash() {
    let mut app = test_app();
    app.add_event(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff::default(),
    });

    assert!(app.is_flashing());
}

#[test]
fn test_flash_expires() {
    let mut app = test_app();
    app.flash_until = Some(Instant::now() - Duration::from_secs(1));
    assert!(!app.is_flashing());
}

#[test]
fn test_scroll_bounds() {
    let mut app = test_app();
    app.diff_scroll_up(100); // Should not go negative
    assert_eq!(app.diff_scroll_offset, 0);
}

#[test]
fn test_get_current_event_empty() {
    let app = test_app();
    assert!(app.get_current_event().is_none());
}

#[test]
fn test_get_first_changed_line_empty() {
    let app = test_app();
    assert!(app.get_first_changed_line().is_none());
}

#[test]
fn test_theme_selector_wraps() {
    let mut app = test_app();
    app.selected_theme_index = 0;
    app.theme_selector_up();
    assert_eq!(
        app.selected_theme_index,
        gwatch::ui::theme::Theme::available_themes().len() - 1
    );
}

#[test]
fn test_select_invalid_theme_index() {
    let mut app = test_app();
    let old_theme = app.theme.name.clone();
    app.select_theme(999);
    assert_eq!(app.theme.name, old_theme); // Unchanged
}

#[test]
fn test_open_and_close_overlay() {
    let mut app = test_app();
    app.open_help();
    assert_eq!(app.state, AppState::HelpPanel);
    app.close_overlay();
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_clear_history() {
    let mut app = test_app();
    app.events.push_front(DisplayedEvent {
        file_path: PathBuf::from("/test/file.rs"),
        relative_path: "file.rs".to_string(),
        timestamp: chrono::Utc::now(),
        diff: FileDiff::default(),
    });
    app.clear_history();
    assert!(app.events.is_empty());
}
