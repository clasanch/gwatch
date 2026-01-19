use gwatch::config::{Config, DiffViewerConfig, DiffViewerType};

#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert_eq!(config.theme.name, "nord");
    assert_eq!(config.watcher.debounce_ms, 50);
    assert_eq!(config.watcher.max_events_buffer, 300);
    assert_eq!(config.display.context_lines, 3);
    assert!(config.display.show_line_numbers);
}

#[test]
fn test_diff_viewer_config_delta_args() {
    let config = DiffViewerConfig::default();
    assert_eq!(config.delta_args, vec!["--side-by-side".to_string()]);
}

#[test]
fn test_diff_viewer_type_difft_alias() {
    assert_eq!(
        DiffViewerType::parse_from_str("difft"),
        DiffViewerType::Difftastic
    );
}

#[test]
fn test_diff_viewer_type_builtin_alias() {
    assert_eq!(
        DiffViewerType::parse_from_str("builtin"),
        DiffViewerType::Internal
    );
}

#[test]
fn test_config_dir_exists() {
    let dir = Config::config_dir();
    assert!(dir.to_string_lossy().contains("gwatch"));
}

#[test]
fn test_config_path_is_json() {
    let path = Config::config_path();
    assert!(path.to_string_lossy().ends_with("config.json"));
}
