use gwatch::config::{DiffViewerConfig, DiffViewerType};
use gwatch::diff_viewer::{get_viewer_display_name, is_command_available, resolve_viewer};

#[test]
fn test_resolve_viewer_explicit_delta() {
    let config = DiffViewerConfig {
        viewer: DiffViewerType::Delta,
        ..Default::default()
    };
    let resolved = resolve_viewer(&config);
    if is_command_available("delta") {
        assert_eq!(resolved, DiffViewerType::Delta);
    } else {
        assert_eq!(resolved, DiffViewerType::Internal);
    }
}

#[test]
fn test_resolve_viewer_explicit_internal() {
    let config = DiffViewerConfig {
        viewer: DiffViewerType::Internal,
        ..Default::default()
    };
    assert_eq!(resolve_viewer(&config), DiffViewerType::Internal);
}

#[test]
fn test_get_viewer_display_name() {
    assert_eq!(get_viewer_display_name(&DiffViewerType::Delta), "delta");
    assert_eq!(
        get_viewer_display_name(&DiffViewerType::Difftastic),
        "difftastic"
    );
    assert_eq!(
        get_viewer_display_name(&DiffViewerType::Internal),
        "Internal"
    );
    assert_eq!(get_viewer_display_name(&DiffViewerType::Auto), "Auto");
}
