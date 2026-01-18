use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::App;
use super::theme::Theme;

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_theme_selector(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(40, 50, f.area());

    f.render_widget(Clear, area);

    let themes = Theme::available_themes();
    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.selected_theme_index {
                Style::default()
                    .fg(theme.text)
                    .bg(theme.border_focused)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_dim)
            };
            ListItem::new(format!("  {name}  ")).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Select Theme ")
            .title_style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_focused))
            .style(Style::default().bg(theme.background)),
    );

    f.render_widget(list, area);
}

pub fn draw_help_panel(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(60, 70, f.area());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Navigation",
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(vec![
            Span::styled(
                "  ↑/↓ or j/k   ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Scroll within current diff",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  p / n        ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Navigate between events (prev/next)",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  ←/→ or h/l   ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Pan diff horizontally", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled(
                "  PgUp/PgDn    ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Fast scroll within diff (10 lines)",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Hunk Navigation",
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(vec![
            Span::styled(
                "  ] / [        ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Jump to next/prev hunk",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  z            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle hunk collapsed", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled(
                "  Z            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Toggle hide context lines",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Actions",
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(vec![
            Span::styled(
                "  Space        ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Pause/Resume live streaming",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Enter        ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Open current file in $EDITOR",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  c            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Clear event history", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled(
                "  d            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Open diff in external viewer (delta/difftastic/pager)",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  t            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Open theme selector", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled(
                "  m            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Cycle diff mode (All/Unstaged/Staged)",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  r            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Toggle reviewed status for current file",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  R            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Clear all reviewed markers",
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  s            ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Open settings editor", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled(
                "  q / Esc      ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Quit gwatch", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press any key to close this panel",
            Style::default().fg(theme.context),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused))
                .style(Style::default().bg(theme.background)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(help, area);
}

pub fn draw_settings_editor(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(80, 85, f.area());

    f.render_widget(Clear, area);

    let state = &app.settings_editor;
    let lines: Vec<&str> = state.content.lines().collect();
    let line_count = lines.len();

    let inner_height = area.height.saturating_sub(4) as usize;
    let visible_start = if state.cursor_line >= inner_height {
        state.cursor_line - inner_height + 1
    } else {
        0
    };

    let mut text_lines: Vec<Line> = Vec::new();
    for (i, line) in lines
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(inner_height)
    {
        let line_num = format!("{:>3} ", i + 1);
        let is_cursor_line = i == state.cursor_line;

        let mut spans = vec![Span::styled(
            line_num,
            Style::default().fg(theme.line_number),
        )];

        if is_cursor_line {
            let before = if state.cursor_col > 0 {
                &line[..state.cursor_col.min(line.len())]
            } else {
                ""
            };
            let cursor_char = if state.cursor_col < line.len() {
                &line[state.cursor_col..state.cursor_col + 1]
            } else {
                " "
            };
            let after = if state.cursor_col + 1 < line.len() {
                &line[state.cursor_col + 1..]
            } else {
                ""
            };

            spans.push(Span::styled(
                before.to_string(),
                Style::default().fg(theme.text),
            ));
            spans.push(Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .fg(theme.background)
                    .bg(theme.text)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                after.to_string(),
                Style::default().fg(theme.text),
            ));
        } else {
            spans.push(Span::styled(
                line.to_string(),
                Style::default().fg(theme.text),
            ));
        }

        text_lines.push(Line::from(spans));
    }

    if line_count == 0 {
        text_lines.push(Line::from(vec![
            Span::styled("  1 ", Style::default().fg(theme.line_number)),
            Span::styled(
                " ",
                Style::default()
                    .fg(theme.background)
                    .bg(theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let footer_text = if let Some(ref err) = state.error_message {
        Line::from(vec![Span::styled(
            format!(" Error: {err} "),
            Style::default()
                .fg(theme.deleted)
                .add_modifier(Modifier::BOLD),
        )])
    } else {
        Line::from(vec![
            Span::styled(
                " [Ctrl+S]",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Save  ", Style::default().fg(theme.text_dim)),
            Span::styled(
                "[Esc]",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cancel  ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("Line {}/{}", state.cursor_line + 1, line_count.max(1)),
                Style::default().fg(theme.context),
            ),
        ])
    };

    text_lines.push(Line::from(""));
    text_lines.push(footer_text);

    let editor = Paragraph::new(text_lines).block(
        Block::default()
            .title(" Settings Editor ")
            .title_style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_focused))
            .style(Style::default().bg(theme.background)),
    );

    f.render_widget(editor, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect_50_50() {
        let parent = Rect::new(0, 0, 100, 100);
        let result = centered_rect(50, 50, parent);

        assert!(result.x >= 20 && result.x <= 30);
        assert!(result.y >= 20 && result.y <= 30);
        assert!(result.width >= 45 && result.width <= 55);
        assert!(result.height >= 45 && result.height <= 55);
    }

    #[test]
    fn test_centered_rect_small() {
        let parent = Rect::new(0, 0, 80, 24);
        let result = centered_rect(40, 50, parent);

        assert!(result.width > 0);
        assert!(result.height > 0);
        assert!(result.x > 0);
        assert!(result.y > 0);
    }
}
