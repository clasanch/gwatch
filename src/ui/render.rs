use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::types::DiffKind;

use super::app::{App, AppState};
use super::diff_view::{build_side_by_side_lines, truncate_with_offset};
use super::layout::{draw_event_header, draw_footer, draw_header};
use super::overlays::{draw_help_panel, draw_settings_editor, draw_theme_selector};
use super::theme::Theme;

pub fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    match app.state {
        AppState::ThemeSelector => draw_theme_selector(f, app),
        AppState::HelpPanel => draw_help_panel(f, app),
        AppState::SettingsEditor => draw_settings_editor(f, app),
        _ => {}
    }
}

fn draw_main_content(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.background));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.events.is_empty() {
        draw_empty_state(f, theme, inner);
        return;
    }

    let event = match app.events.get(app.scroll_offset) {
        Some(e) => e,
        None => return,
    };

    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 2,
    };

    let diff_area = Rect {
        x: inner.x,
        y: inner.y + 2,
        width: inner.width,
        height: inner.height.saturating_sub(2),
    };

    draw_event_header(f, event, theme, header_area, app);
    draw_diff_content(f, event, theme, diff_area, app);
}

fn draw_empty_state(f: &mut Frame, theme: &Theme, area: Rect) {
    let empty_msg = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Waiting for file changes...",
            Style::default().fg(theme.text_dim),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Save a file in this repository to see diffs here.",
            Style::default().fg(theme.context),
        )),
    ])
    .style(Style::default().bg(theme.background));
    f.render_widget(empty_msg, area);
}

fn draw_diff_content(
    f: &mut Frame,
    event: &crate::types::DisplayedEvent,
    theme: &Theme,
    area: Rect,
    app: &App,
) {
    if event.diff.is_binary {
        let msg = Paragraph::new(Line::from(Span::styled(
            "  Binary file, diff not shown",
            Style::default().fg(theme.text_dim),
        )))
        .style(Style::default().bg(theme.background));
        f.render_widget(msg, area);
        return;
    }

    if event.diff.hunks.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "  No changes detected",
            Style::default().fg(theme.text_dim),
        )))
        .style(Style::default().bg(theme.background));
        f.render_widget(msg, area);
        return;
    }

    let mut display_lines: Vec<Line> = Vec::new();
    let is_flashing = app.is_flashing();

    for (hunk_idx, hunk) in event.diff.hunks.iter().enumerate() {
        let is_focused = hunk_idx == app.hunk_state.focused_hunk;
        let is_collapsed = app.hunk_state.is_collapsed(hunk_idx);

        // Hunk header
        let header_style = if is_focused {
            Style::default()
                .fg(theme.text)
                .bg(theme.border)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(theme.context)
        };

        let collapse_indicator = if is_collapsed { "▶" } else { "▼" };
        let hunk_header = format!(
            " {} Hunk {}/{}: @@ -{},{} +{},{} @@ ",
            collapse_indicator,
            hunk_idx + 1,
            event.diff.hunks.len(),
            hunk.old_start,
            hunk.old_count,
            hunk.new_start,
            hunk.new_count,
        );

        display_lines.push(Line::from(Span::styled(hunk_header, header_style)));

        if is_collapsed {
            let added = hunk
                .lines
                .iter()
                .filter(|l| l.kind == DiffKind::Added)
                .count();
            let deleted = hunk
                .lines
                .iter()
                .filter(|l| l.kind == DiffKind::Deleted)
                .count();
            let summary = format!("    +{added} -{deleted} lines (press z to expand)");
            display_lines.push(Line::from(Span::styled(
                summary,
                Style::default().fg(theme.text_dim),
            )));
        } else {
            let hunk_lines = build_hunk_lines(hunk, app, theme, is_flashing, area.width);
            display_lines.extend(hunk_lines);
        }
    }

    let visible_height = area.height as usize;
    let scroll_offset = app
        .diff_scroll_offset
        .min(display_lines.len().saturating_sub(1));
    let visible_lines: Vec<Line> = display_lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();

    let p = Paragraph::new(visible_lines).style(Style::default().bg(theme.background));
    f.render_widget(p, area);
}

fn build_hunk_lines<'a>(
    hunk: &crate::types::DiffHunk,
    app: &App,
    theme: &'a Theme,
    is_flashing: bool,
    width: u16,
) -> Vec<Line<'a>> {
    let collapse_context = app.hunk_state.collapse_context;
    let filtered_lines: Vec<_> = hunk
        .lines
        .iter()
        .filter(|l| !collapse_context || l.kind != DiffKind::Context)
        .cloned()
        .collect();

    let side_by_side = build_side_by_side_lines(&filtered_lines);

    let total_fixed: u16 = 11;
    let available_for_content = width.saturating_sub(total_fixed);
    let content_width = (available_for_content / 2) as usize;
    let h_offset = app.diff_horizontal_offset;

    let mut lines = Vec::new();
    for sbs in side_by_side {
        let left_num_str = sbs
            .left_num
            .map(|n| format!("{n:>4}"))
            .unwrap_or_else(|| "  · ".to_string());

        let left_content = truncate_with_offset(&sbs.left_content, h_offset, content_width);
        let is_left_change = matches!(sbs.left_kind, Some(DiffKind::Deleted));
        let flash_left = is_flashing && is_left_change;

        let (left_num_style, left_content_style, left_prefix) =
            get_line_styles(sbs.left_kind.as_ref(), flash_left, theme);

        let right_num_str = sbs
            .right_num
            .map(|n| format!("{n:>4}"))
            .unwrap_or_else(|| "  · ".to_string());

        let right_content = truncate_with_offset(&sbs.right_content, h_offset, content_width);
        let is_right_change = matches!(sbs.right_kind, Some(DiffKind::Added));
        let flash_right = is_flashing && is_right_change;

        let (right_num_style, right_content_style, right_prefix) =
            get_line_styles(sbs.right_kind.as_ref(), flash_right, theme);

        let left_display = format!("{left_content:content_width$}");
        let right_display = format!("{right_content:content_width$}");

        lines.push(Line::from(vec![
            Span::styled(
                left_num_str,
                left_num_style.add_modifier(ratatui::style::Modifier::DIM),
            ),
            Span::styled(
                left_prefix,
                left_content_style.add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled(left_display, left_content_style),
            Span::styled("│", Style::default().fg(theme.border)),
            Span::styled(
                right_num_str,
                right_num_style.add_modifier(ratatui::style::Modifier::DIM),
            ),
            Span::styled(
                right_prefix,
                right_content_style.add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled(right_display, right_content_style),
        ]));
    }

    lines
}

fn get_line_styles(
    kind: Option<&DiffKind>,
    is_flashing: bool,
    theme: &Theme,
) -> (Style, Style, &'static str) {
    match kind {
        Some(DiffKind::Deleted) => {
            let bg = if is_flashing {
                Some(theme.deleted)
            } else {
                None
            };
            let fg = if is_flashing {
                theme.background
            } else {
                theme.deleted
            };
            let mut style = Style::default().fg(fg);
            if let Some(bg_color) = bg {
                style = style.bg(bg_color);
            }
            (style, style, "-")
        }
        Some(DiffKind::Added) => {
            let bg = if is_flashing { Some(theme.added) } else { None };
            let fg = if is_flashing {
                theme.background
            } else {
                theme.added
            };
            let mut style = Style::default().fg(fg);
            if let Some(bg_color) = bg {
                style = style.bg(bg_color);
            }
            (style, style, "+")
        }
        Some(DiffKind::Context) => (
            Style::default().fg(theme.line_number),
            Style::default().fg(theme.context),
            " ",
        ),
        None => (
            Style::default().fg(theme.text_dim),
            Style::default().fg(theme.text_dim),
            " ",
        ),
    }
}
