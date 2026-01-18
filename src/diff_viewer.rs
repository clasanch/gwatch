use std::process::Command;

use crate::config::{DiffViewerConfig, DiffViewerType};

pub fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn detect_available_viewer() -> DiffViewerType {
    if is_command_available("delta") {
        return DiffViewerType::Delta;
    }
    if is_command_available("difft") {
        return DiffViewerType::Difftastic;
    }
    DiffViewerType::Internal
}

pub fn resolve_viewer(config: &DiffViewerConfig) -> DiffViewerType {
    match config.viewer {
        DiffViewerType::Auto => detect_available_viewer(),
        DiffViewerType::Delta => {
            if is_command_available("delta") {
                DiffViewerType::Delta
            } else {
                tracing::warn!("delta not found, falling back to internal viewer");
                DiffViewerType::Internal
            }
        }
        DiffViewerType::Difftastic => {
            if is_command_available("difft") {
                DiffViewerType::Difftastic
            } else {
                tracing::warn!("difftastic not found, falling back to internal viewer");
                DiffViewerType::Internal
            }
        }
        DiffViewerType::Internal => DiffViewerType::Internal,
    }
}

pub fn get_viewer_display_name(viewer: &DiffViewerType) -> &'static str {
    match viewer {
        DiffViewerType::Auto => "Auto",
        DiffViewerType::Delta => "delta",
        DiffViewerType::Difftastic => "difftastic",
        DiffViewerType::Internal => "Internal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_viewer_internal_fallback() {
        // When no viewer is found, should fall back to internal
        let result = detect_available_viewer();
        // Result should be Some variant (either detected or internal)
        assert!(matches!(
            result,
            DiffViewerType::Delta | DiffViewerType::Difftastic | DiffViewerType::Internal
        ));
    }

    #[test]
    fn test_is_command_available() {
        // 'echo' should be available on all systems
        assert!(is_command_available("echo"));
        // Random non-existent command should not be available
        assert!(!is_command_available("nonexistent_command_xyz123"));
    }

    #[test]
    fn test_resolve_viewer() {
        let config = DiffViewerConfig::default();
        let resolved = resolve_viewer(&config);
        // Should resolve to something valid
        assert!(matches!(
            resolved,
            DiffViewerType::Delta | DiffViewerType::Difftastic | DiffViewerType::Internal
        ));
    }
}
