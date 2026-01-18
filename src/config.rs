use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub editor: EditorConfig,
    pub watcher: WatcherConfig,
    pub display: DisplayConfig,
    pub keybindings: KeybindingConfig,
    pub diff_viewer: DiffViewerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    #[serde(default)]
    pub custom: Option<CustomColors>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffViewerConfig {
    pub viewer: DiffViewerType,
    pub pager: Option<String>,
    pub delta_args: Vec<String>,
    pub difftastic_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiffViewerType {
    #[default]
    Auto, // Auto-detect available viewer
    Delta,      // Use delta
    Difftastic, // Use difftastic
    Internal,   // Use built-in TUI viewer only
}

impl DiffViewerType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "delta" => Self::Delta,
            "difftastic" | "difft" => Self::Difftastic,
            "internal" | "builtin" => Self::Internal,
            _ => Self::Auto,
        }
    }
}

impl Default for DiffViewerConfig {
    fn default() -> Self {
        Self {
            viewer: DiffViewerType::Auto,
            pager: None,
            delta_args: vec!["--side-by-side".to_string()],
            difftastic_args: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub added_line: String,
    pub deleted_line: String,
    pub context_line: String,
    pub line_number: String,
    pub border: String,
    pub text: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub debounce_ms: u64,
    pub max_events_buffer: usize,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub context_lines: usize,
    pub truncate_long_lines: bool,
    pub max_line_length: usize,
    pub show_line_numbers: bool,
    pub show_file_path: bool,
    pub use_nerd_font_icons: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub pause_resume: String,
    pub scroll_up: String,
    pub scroll_down: String,
    pub open_editor: String,
    pub theme_selector: String,
    pub settings: String,
    pub clear_history: String,
    pub quit: String,
    pub help: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                name: "nord".to_string(),
                custom: None,
            },
            editor: EditorConfig {
                command: std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string()),
                args: vec!["+{line}".to_string(), "{file}".to_string()],
            },
            watcher: WatcherConfig {
                debounce_ms: 50,
                max_events_buffer: 300,
                ignore_patterns: vec![
                    "node_modules".to_string(),
                    "dist".to_string(),
                    "build".to_string(),
                    "*.log".to_string(),
                    "target".to_string(),
                ],
            },
            display: DisplayConfig {
                context_lines: 3,
                truncate_long_lines: false,
                max_line_length: 120,
                show_line_numbers: true,
                show_file_path: true,
                use_nerd_font_icons: true,
            },
            keybindings: KeybindingConfig {
                pause_resume: "space".to_string(),
                scroll_up: "up".to_string(),
                scroll_down: "down".to_string(),
                open_editor: "enter".to_string(),
                theme_selector: "t".to_string(),
                settings: "s".to_string(),
                clear_history: "c".to_string(),
                quit: "q".to_string(),
                help: "?".to_string(),
            },
            diff_viewer: DiffViewerConfig::default(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gwatch")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            match serde_json::from_str(&content) {
                Ok(config) => Ok(config),
                Err(e) => {
                    tracing::warn!("Config JSON invalid, using defaults. Error: {}", e);
                    Ok(Self::default())
                }
            }
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir)?;

        let config_path = Self::config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_viewer_config_default() {
        let config = Config::default();
        assert_eq!(config.diff_viewer.viewer, DiffViewerType::Auto);
        assert!(config.diff_viewer.pager.is_none());
    }

    #[test]
    fn test_diff_viewer_type_from_str() {
        assert_eq!(DiffViewerType::from_str("delta"), DiffViewerType::Delta);
        assert_eq!(
            DiffViewerType::from_str("difftastic"),
            DiffViewerType::Difftastic
        );
        assert_eq!(
            DiffViewerType::from_str("internal"),
            DiffViewerType::Internal
        );
        assert_eq!(DiffViewerType::from_str("auto"), DiffViewerType::Auto);
        assert_eq!(DiffViewerType::from_str("unknown"), DiffViewerType::Auto);
    }
}
