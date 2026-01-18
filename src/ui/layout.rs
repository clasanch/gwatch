use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::config::DiffViewerType;
use crate::diff_viewer::{get_viewer_display_name, resolve_viewer};
use crate::types::DisplayedEvent;

use super::app::App;
use super::theme::Theme;

pub fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let status = if app.is_paused() {
        Span::styled(
            " [PAUSED] ",
            Style::default()
                .fg(theme.status_paused)
                .add_modifier(Modifier::BOLD),
        )
    } else if app.is_flashing() {
        Span::styled(
            " [NEW CHANGE] ",
            Style::default()
                .fg(theme.background)
                .bg(theme.added)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [WATCHING] ",
            Style::default()
                .fg(theme.status_running)
                .add_modifier(Modifier::BOLD),
        )
    };

    let title = Line::from(vec![
        Span::styled(
            " gwatch",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        status,
        Span::styled(
            format!("─ {} ", app.theme.name),
            Style::default().fg(theme.text_dim),
        ),
    ]);

    let header = Paragraph::new(title).style(Style::default().bg(theme.header_bg));
    f.render_widget(header, area);
}

pub fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let mut spans = vec![
        Span::styled(
            " [↑↓]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Diff  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[N/P]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Event  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[←→]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Pan  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[Space]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Pause  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[Enter]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Edit  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[?]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Help  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[q]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit", Style::default().fg(theme.text_dim)),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            format!("Mode: {}", app.diff_mode.label()),
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(" [m]", Style::default().fg(theme.context)),
    ];

    let reviewed_count = app.review_state.reviewed_count();
    if reviewed_count > 0 {
        spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        spans.push(Span::styled(
            format!("Reviewed: {reviewed_count}"),
            Style::default().fg(theme.added),
        ));
    }

    let hunk_count = app.get_current_hunk_count();
    if hunk_count > 1 {
        spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        spans.push(Span::styled(
            format!("Hunk {}/{}", app.hunk_state.focused_hunk + 1, hunk_count),
            Style::default().fg(theme.text_dim),
        ));
        spans.push(Span::styled(" []/z", Style::default().fg(theme.context)));
    }

    let viewer = resolve_viewer(&app.config.diff_viewer);
    if !matches!(viewer, DiffViewerType::Internal) {
        spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        spans.push(Span::styled(
            format!("Viewer: {}", get_viewer_display_name(&viewer)),
            Style::default().fg(theme.text_dim),
        ));
        spans.push(Span::styled(" [d]", Style::default().fg(theme.context)));
    }

    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.footer_bg));
    f.render_widget(footer, area);
}

pub fn draw_event_header(
    f: &mut Frame,
    event: &DisplayedEvent,
    theme: &Theme,
    area: Rect,
    app: &App,
) {
    let time_str = event.timestamp.format("%H:%M:%S").to_string();
    let stats = format!(
        "+{} / -{} lines",
        event.diff.stats.added_count, event.diff.stats.deleted_count
    );

    let file_indicator = if event.diff.is_new_file {
        " (new file)".to_string()
    } else if event.diff.is_deleted {
        " (deleted)".to_string()
    } else if event.diff.is_binary {
        " [binary]".to_string()
    } else if event.diff.is_truncated {
        if let Some(ref reason) = event.diff.truncation_reason {
            format!(" [{reason}]")
        } else {
            " [truncated]".to_string()
        }
    } else {
        String::new()
    };

    let indicator_style = if event.diff.is_truncated {
        Style::default().fg(theme.deleted)
    } else {
        Style::default().fg(theme.text_dim)
    };

    let event_index_info = format!(" [{}/{}]", app.scroll_offset + 1, app.events.len());

    let mut spans = vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            event.relative_path.clone(),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
    ];

    if app.review_state.is_reviewed(&event.file_path) {
        spans.push(Span::styled(
            " ✓ Reviewed",
            Style::default()
                .fg(theme.added)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.extend(vec![
        Span::styled(file_indicator, indicator_style),
        Span::styled(
            format!(" @ {time_str} "),
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(stats, Style::default().fg(theme.context)),
        Span::styled(event_index_info, Style::default().fg(theme.text_dim)),
    ]);

    let header = Line::from(spans);

    let p =
        Paragraph::new(vec![header, Line::from("")]).style(Style::default().bg(theme.background));
    f.render_widget(p, area);
}
