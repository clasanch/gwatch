use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub added: Color,
    pub deleted: Color,
    pub context: Color,
    pub line_number: Color,
    pub border: Color,
    pub border_focused: Color,
    pub text: Color,
    pub text_dim: Color,
    pub background: Color,
    pub header_bg: Color,
    pub footer_bg: Color,
    pub status_paused: Color,
    pub status_running: Color,
}

impl Theme {
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "catppuccin-mocha" | "catppuccin_mocha" => Self::catppuccin_mocha(),
            "catppuccin-frappe" | "catppuccin_frappe" => Self::catppuccin_frappe(),
            "dracula" | "dracula-modified" => Self::dracula_modified(),
            "monochrome" => Self::monochrome(),
            _ => Self::nord(),
        }
    }

    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "nord",
            "catppuccin-mocha",
            "catppuccin-frappe",
            "dracula",
            "monochrome",
        ]
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            added: Color::Rgb(163, 190, 140),          // #a3be8c
            deleted: Color::Rgb(191, 97, 106),         // #bf616a
            context: Color::Rgb(76, 86, 106),          // #4c566a
            line_number: Color::Rgb(94, 129, 172),     // #5e81ac
            border: Color::Rgb(59, 66, 82),            // #3b4252
            border_focused: Color::Rgb(129, 161, 193), // #81a1c1
            text: Color::Rgb(236, 239, 244),           // #eceff4
            text_dim: Color::Rgb(216, 222, 233),       // #d8dee9
            background: Color::Rgb(46, 52, 64),        // #2e3440
            header_bg: Color::Rgb(59, 66, 82),         // #3b4252
            footer_bg: Color::Rgb(59, 66, 82),         // #3b4252
            status_paused: Color::Rgb(235, 203, 139),  // #ebcb8b
            status_running: Color::Rgb(163, 190, 140), // #a3be8c
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            added: Color::Rgb(166, 218, 149),       // #a6da95 green
            deleted: Color::Rgb(237, 135, 150),     // #ed8796 red
            context: Color::Rgb(110, 115, 141),     // #6e738d overlay0
            line_number: Color::Rgb(138, 173, 244), // #8aadf4 blue
            border: Color::Rgb(73, 77, 100),        // #494d64 surface1
            border_focused: Color::Rgb(125, 196, 228), // #7dc4e4 sapphire
            text: Color::Rgb(202, 211, 245),        // #cad3f5 text
            text_dim: Color::Rgb(165, 173, 206),    // #a5adce subtext0
            background: Color::Rgb(36, 39, 58),     // #24273a base
            header_bg: Color::Rgb(54, 58, 79),      // #363a4f surface0
            footer_bg: Color::Rgb(54, 58, 79),      // #363a4f surface0
            status_paused: Color::Rgb(238, 212, 159), // #eed49f yellow
            status_running: Color::Rgb(166, 218, 149), // #a6da95 green
        }
    }

    pub fn catppuccin_frappe() -> Self {
        Self {
            name: "Catppuccin Frappé".to_string(),
            added: Color::Rgb(166, 209, 137),       // #a6d189 green
            deleted: Color::Rgb(231, 130, 132),     // #e78284 red
            context: Color::Rgb(115, 121, 148),     // #737994 overlay0
            line_number: Color::Rgb(140, 170, 238), // #8caaee blue
            border: Color::Rgb(81, 87, 109),        // #51576d surface1
            border_focused: Color::Rgb(133, 193, 220), // #85c1dc sapphire
            text: Color::Rgb(198, 208, 245),        // #c6d0f5 text
            text_dim: Color::Rgb(165, 173, 203),    // #a5adcb subtext0
            background: Color::Rgb(48, 52, 70),     // #303446 base
            header_bg: Color::Rgb(65, 69, 89),      // #414559 surface0
            footer_bg: Color::Rgb(65, 69, 89),      // #414559 surface0
            status_paused: Color::Rgb(229, 200, 144), // #e5c890 yellow
            status_running: Color::Rgb(166, 209, 137), // #a6d189 green
        }
    }

    pub fn dracula_modified() -> Self {
        Self {
            name: "Dracula".to_string(),
            added: Color::Rgb(80, 250, 123),        // #50fa7b green
            deleted: Color::Rgb(255, 85, 85),       // #ff5555 red
            context: Color::Rgb(98, 114, 164),      // #6272a4 comment
            line_number: Color::Rgb(139, 233, 253), // #8be9fd cyan
            border: Color::Rgb(68, 71, 90),         // #44475a current line
            border_focused: Color::Rgb(255, 121, 198), // #ff79c6 pink
            text: Color::Rgb(248, 248, 242),        // #f8f8f2 foreground
            text_dim: Color::Rgb(98, 114, 164),     // #6272a4 comment
            background: Color::Rgb(40, 42, 54),     // #282a36 background
            header_bg: Color::Rgb(68, 71, 90),      // #44475a current line
            footer_bg: Color::Rgb(68, 71, 90),      // #44475a current line
            status_paused: Color::Rgb(241, 250, 140), // #f1fa8c yellow
            status_running: Color::Rgb(80, 250, 123), // #50fa7b green
        }
    }

    pub fn monochrome() -> Self {
        Self {
            name: "Monochrome".to_string(),
            added: Color::Green,
            deleted: Color::Red,
            context: Color::DarkGray,
            line_number: Color::Cyan,
            border: Color::DarkGray,
            border_focused: Color::White,
            text: Color::White,
            text_dim: Color::Gray,
            background: Color::Black,
            header_bg: Color::DarkGray,
            footer_bg: Color::DarkGray,
            status_paused: Color::Yellow,
            status_running: Color::Green,
        }
    }
}
