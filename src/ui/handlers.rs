use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::process::Command;

use crate::config::DiffViewerType;
use crate::diff_viewer::resolve_viewer;

use super::app::{App, AppState};

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.state {
        AppState::ThemeSelector => handle_theme_selector_keys(app, key),
        AppState::HelpPanel => handle_help_panel_keys(app, key),
        AppState::SettingsEditor => handle_settings_editor_keys(app, key),
        _ => handle_main_keys(app, key),
    }
}

fn handle_main_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char(' ') => {
            app.toggle_pause();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let max = app.get_current_diff_line_count();
            app.diff_scroll_up(1);
            let _ = max;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = app.get_current_diff_line_count();
            app.diff_scroll_down(1, max);
        }
        KeyCode::PageUp => {
            app.diff_scroll_up(10);
        }
        KeyCode::PageDown => {
            let max = app.get_current_diff_line_count();
            app.diff_scroll_down(10, max);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.diff_scroll_left();
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.diff_scroll_right();
        }
        KeyCode::Char('p') => {
            app.scroll_up();
        }
        KeyCode::Char('n') => {
            app.scroll_down();
        }
        KeyCode::Char(']') => {
            app.next_hunk();
        }
        KeyCode::Char('[') => {
            app.prev_hunk();
        }
        KeyCode::Char('z') => {
            app.toggle_current_hunk_collapsed();
        }
        KeyCode::Char('Z') => {
            app.toggle_context_collapsed();
        }
        KeyCode::Char('c') => {
            app.clear_history();
        }
        KeyCode::Char('t') => {
            app.open_theme_selector();
        }
        KeyCode::Char('m') => {
            app.cycle_diff_mode();
        }
        KeyCode::Char('r') => {
            app.toggle_current_reviewed();
        }
        KeyCode::Char('R') => {
            app.clear_all_reviewed();
        }
        KeyCode::Char('d') => {
            open_in_diff_viewer(app)?;
        }
        KeyCode::Char('?') => {
            app.open_help();
        }
        KeyCode::Char('s') => {
            app.open_settings_editor();
        }
        KeyCode::Enter => {
            open_in_editor(app)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_theme_selector_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('t') => {
            app.close_overlay();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.theme_selector_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.theme_selector_down();
        }
        KeyCode::Enter => {
            app.select_theme(app.selected_theme_index);
            app.close_overlay();
        }
        _ => {}
    }
    Ok(())
}

fn handle_help_panel_keys(app: &mut App, _key: KeyEvent) -> Result<()> {
    app.close_overlay();
    Ok(())
}

fn handle_settings_editor_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    let state = &mut app.settings_editor;
    let lines: Vec<&str> = state.content.lines().collect();
    let line_count = lines.len().max(1);

    match key.code {
        KeyCode::Esc => {
            app.close_overlay();
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.save_settings() {
                app.close_overlay();
            }
        }
        KeyCode::Up => {
            if state.cursor_line > 0 {
                state.cursor_line -= 1;
                let new_line_len = lines.get(state.cursor_line).map(|l| l.len()).unwrap_or(0);
                state.cursor_col = state.cursor_col.min(new_line_len);
            }
        }
        KeyCode::Down => {
            if state.cursor_line < line_count.saturating_sub(1) {
                state.cursor_line += 1;
                let new_line_len = lines.get(state.cursor_line).map(|l| l.len()).unwrap_or(0);
                state.cursor_col = state.cursor_col.min(new_line_len);
            }
        }
        KeyCode::Left => {
            if state.cursor_col > 0 {
                state.cursor_col -= 1;
            } else if state.cursor_line > 0 {
                state.cursor_line -= 1;
                state.cursor_col = lines.get(state.cursor_line).map(|l| l.len()).unwrap_or(0);
            }
        }
        KeyCode::Right => {
            let current_line_len = lines.get(state.cursor_line).map(|l| l.len()).unwrap_or(0);
            if state.cursor_col < current_line_len {
                state.cursor_col += 1;
            } else if state.cursor_line < line_count.saturating_sub(1) {
                state.cursor_line += 1;
                state.cursor_col = 0;
            }
        }
        KeyCode::Home => {
            state.cursor_col = 0;
        }
        KeyCode::End => {
            state.cursor_col = lines.get(state.cursor_line).map(|l| l.len()).unwrap_or(0);
        }
        KeyCode::Enter => {
            let pos = get_cursor_position(&state.content, state.cursor_line, state.cursor_col);
            state.content.insert(pos, '\n');
            state.cursor_line += 1;
            state.cursor_col = 0;
            state.error_message = None;
        }
        KeyCode::Backspace => {
            if state.cursor_col > 0 {
                let pos = get_cursor_position(&state.content, state.cursor_line, state.cursor_col);
                if pos > 0 {
                    state.content.remove(pos - 1);
                    state.cursor_col -= 1;
                }
            } else if state.cursor_line > 0 {
                let prev_line_len = lines
                    .get(state.cursor_line - 1)
                    .map(|l| l.len())
                    .unwrap_or(0);
                let pos = get_cursor_position(&state.content, state.cursor_line, 0);
                if pos > 0 {
                    state.content.remove(pos - 1);
                    state.cursor_line -= 1;
                    state.cursor_col = prev_line_len;
                }
            }
            state.error_message = None;
        }
        KeyCode::Delete => {
            let pos = get_cursor_position(&state.content, state.cursor_line, state.cursor_col);
            if pos < state.content.len() {
                state.content.remove(pos);
            }
            state.error_message = None;
        }
        KeyCode::Char(c) => {
            let pos = get_cursor_position(&state.content, state.cursor_line, state.cursor_col);
            state.content.insert(pos, c);
            state.cursor_col += 1;
            state.error_message = None;
        }
        KeyCode::Tab => {
            let pos = get_cursor_position(&state.content, state.cursor_line, state.cursor_col);
            state.content.insert_str(pos, "  ");
            state.cursor_col += 2;
            state.error_message = None;
        }
        _ => {}
    }

    let new_lines: Vec<&str> = app.settings_editor.content.lines().collect();
    let new_line_count = new_lines.len();
    if app.settings_editor.cursor_line >= new_line_count && new_line_count > 0 {
        app.settings_editor.cursor_line = new_line_count - 1;
    }

    Ok(())
}

pub fn get_cursor_position(content: &str, line: usize, col: usize) -> usize {
    let mut pos = 0;
    for (i, l) in content.lines().enumerate() {
        if i == line {
            return pos + col.min(l.len());
        }
        pos += l.len() + 1;
    }
    content.len()
}

fn open_in_editor(app: &App) -> Result<()> {
    let event = match app.get_current_event() {
        Some(e) => e,
        None => return Ok(()),
    };

    let file_path = &event.file_path;
    let line = app.get_first_changed_line().unwrap_or(1);

    let editor_cmd = &app.config.editor.command;
    let args: Vec<String> = app
        .config
        .editor
        .args
        .iter()
        .map(|arg| {
            arg.replace("{line}", &line.to_string())
                .replace("{file}", &file_path.to_string_lossy())
        })
        .collect();

    tracing::info!("Opening editor: {} {:?}", editor_cmd, args);

    Command::new(editor_cmd).args(&args).status().ok();

    Ok(())
}

fn open_in_diff_viewer(app: &App) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let event = match app.get_current_event() {
        Some(e) => e,
        None => return Ok(()),
    };

    let viewer = resolve_viewer(&app.config.diff_viewer);
    let file_path = &event.file_path;

    tracing::info!("Opening diff with {:?} for {:?}", viewer, file_path);

    match viewer {
        DiffViewerType::Delta => {
            let git_diff = Command::new("git")
                .args(["diff", "HEAD", "--"])
                .arg(file_path)
                .output()?;

            let mut delta = Command::new("delta")
                .args(&app.config.diff_viewer.delta_args)
                .stdin(Stdio::piped())
                .spawn()?;

            if let Some(ref mut stdin) = delta.stdin {
                stdin.write_all(&git_diff.stdout)?;
            }
            delta.wait()?;
        }
        DiffViewerType::Difftastic => {
            Command::new("difft")
                .args(&app.config.diff_viewer.difftastic_args)
                .arg(file_path)
                .status()?;
        }
        DiffViewerType::Internal | DiffViewerType::Auto => {
            let pager =
                app.config.diff_viewer.pager.clone().unwrap_or_else(|| {
                    std::env::var("PAGER").unwrap_or_else(|_| "less".to_string())
                });

            let git_diff = Command::new("git")
                .args(["diff", "HEAD", "--color=always", "--"])
                .arg(file_path)
                .output()?;

            let mut pager_cmd = Command::new(&pager)
                .arg("-R")
                .stdin(Stdio::piped())
                .spawn()?;

            if let Some(ref mut stdin) = pager_cmd.stdin {
                stdin.write_all(&git_diff.stdout)?;
            }
            pager_cmd.wait()?;
        }
    }

    Ok(())
}
