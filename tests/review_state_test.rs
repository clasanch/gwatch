use gwatch::review_state::ReviewState;
use std::path::PathBuf;

#[test]
fn test_toggle_reviewed() {
    let mut state = ReviewState::default();
    let path = PathBuf::from("/test/file.rs");

    assert!(!state.is_reviewed(&path));
    state.toggle_reviewed(&path);
    assert!(state.is_reviewed(&path));
    state.toggle_reviewed(&path);
    assert!(!state.is_reviewed(&path));
}

#[test]
fn test_load_nonexistent_returns_default() {
    let path = PathBuf::from("/nonexistent/path/state.json");
    let state = ReviewState::load_from(&path).unwrap();
    assert!(state.reviewed_files.is_empty());
}

#[test]
fn test_state_path_contains_gwatch() {
    let path = ReviewState::state_path();
    assert!(path.to_string_lossy().contains("gwatch"));
    assert!(path.to_string_lossy().ends_with("review_state.json"));
}
