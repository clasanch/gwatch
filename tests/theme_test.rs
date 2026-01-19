use gwatch::ui::theme::Theme;

#[test]
fn test_by_name_nord() {
    let theme = Theme::by_name("nord");
    assert_eq!(theme.name, "Nord");
}

#[test]
fn test_by_name_catppuccin_mocha() {
    let theme = Theme::by_name("catppuccin-mocha");
    assert_eq!(theme.name, "Catppuccin Mocha");
}

#[test]
fn test_by_name_catppuccin_mocha_underscore() {
    let theme = Theme::by_name("catppuccin_mocha");
    assert_eq!(theme.name, "Catppuccin Mocha");
}

#[test]
fn test_by_name_catppuccin_frappe() {
    let theme = Theme::by_name("catppuccin-frappe");
    assert_eq!(theme.name, "Catppuccin Frappé");
}

#[test]
fn test_by_name_dracula() {
    let theme = Theme::by_name("dracula");
    assert_eq!(theme.name, "Dracula");
}

#[test]
fn test_by_name_monochrome() {
    let theme = Theme::by_name("monochrome");
    assert_eq!(theme.name, "Monochrome");
}

#[test]
fn test_by_name_unknown_defaults_to_nord() {
    let theme = Theme::by_name("unknown-theme");
    assert_eq!(theme.name, "Nord");
}

#[test]
fn test_by_name_case_insensitive() {
    let theme = Theme::by_name("NORD");
    assert_eq!(theme.name, "Nord");
}

#[test]
fn test_available_themes() {
    let themes = Theme::available_themes();
    assert!(themes.contains(&"nord"));
    assert!(themes.contains(&"catppuccin-mocha"));
    assert!(themes.contains(&"dracula"));
    assert!(themes.contains(&"monochrome"));
    assert_eq!(themes.len(), 5);
}

#[test]
fn test_theme_colors_are_set() {
    // Verify each theme has distinct colors
    let nord = Theme::nord();
    let dracula = Theme::dracula_modified();

    assert_ne!(format!("{:?}", nord.added), format!("{:?}", dracula.added));
}
