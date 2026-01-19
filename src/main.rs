use anyhow::Result;
use chrono::Utc;
use crossterm::{
    event::{
        self, Event, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::prelude::*;
use std::io::{self, stdout};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use gwatch::cli::Args;
use gwatch::config::Config;
use gwatch::git_engine::GitEngine;
use gwatch::types::{DiffMode, DisplayedEvent, FileChangeEvent};
use gwatch::ui::{draw_ui, handle_key_event, App};
use gwatch::watcher::FileWatcher;

fn setup_logging(_config: &Config, verbose: u8) -> Result<()> {
    let log_dir = Config::config_dir();
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::NEVER, log_dir, "gwatch.log");

    let level = match verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
        .init();

    Ok(())
}

fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        let msg = format!("gwatch panic: {info}");
        eprintln!("{msg}");
        if let Some(loc) = info.location() {
            let loc_msg = format!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
            eprintln!("{loc_msg}");
            tracing::error!("{} {}", msg, loc_msg);
        } else {
            tracing::error!("{}", msg);
        }
        // Also write to a crash file for debugging
        let crash_path = dirs::config_dir()
            .map(|p| p.join("gwatch").join("crash.log"))
            .unwrap_or_else(|| std::path::PathBuf::from("gwatch_crash.log"));
        let _ = std::fs::write(&crash_path, format!("{msg}\n{info:?}"));
        default(info);
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let _ = execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    );

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = stdout();
    let _ = execute!(stdout, PopKeyboardEnhancementFlags);
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    install_panic_hook();
    let args = Args::parse_args();
    let config = Config::load()?;
    setup_logging(&config, args.verbose)?;

    let current_dir = if args.path == "." {
        std::env::current_dir()?
    } else {
        std::path::PathBuf::from(&args.path)
    };

    if !current_dir.exists() {
        anyhow::bail!("Path does not exist: {}", args.path);
    }

    let git_engine = GitEngine::new(&current_dir)?;
    let repo_root = git_engine.repo_root().to_path_buf();

    tracing::info!("Starting gwatch in repository: {:?}", repo_root);

    let (tx, mut rx) = mpsc::unbounded_channel::<FileChangeEvent>();
    let (config_tx, mut config_rx) = mpsc::unbounded_channel::<()>();

    let _watcher = FileWatcher::new(repo_root.clone(), &config.watcher, tx)?;
    let _config_watcher = setup_config_watcher(config_tx);

    let review_state = gwatch::review_state::ReviewState::load();
    let mut terminal = setup_terminal()?;
    let mut app = App::new(config, repo_root.clone(), review_state);

    let result = run_app(
        &mut terminal,
        &mut app,
        &mut rx,
        &mut config_rx,
        &git_engine,
    )
    .await;

    restore_terminal()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

fn setup_config_watcher(tx: mpsc::UnboundedSender<()>) -> Option<RecommendedWatcher> {
    let config_path = Config::config_path();
    let config_dir = Config::config_dir();

    if !config_dir.exists() {
        return None;
    }

    let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
            if event.paths.iter().any(|p| p == &config_path) {
                match event.kind {
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            }
        }
    });

    match watcher {
        Ok(mut w) => {
            if let Err(e) = w.watch(&config_dir, RecursiveMode::NonRecursive) {
                tracing::warn!("Failed to watch config directory: {}", e);
                return None;
            }
            tracing::info!("Watching config file: {:?}", Config::config_path());
            Some(w)
        }
        Err(e) => {
            tracing::warn!("Failed to create config watcher: {}", e);
            None
        }
    }
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<FileChangeEvent>,
    config_rx: &mut mpsc::UnboundedReceiver<()>,
    git_engine: &GitEngine,
) -> Result<()> {
    loop {
        if let Err(e) = terminal.draw(|f| draw_ui(f, app)) {
            tracing::error!("Draw error: {}", e);
            return Err(e.into());
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                let mut last_key: Option<crossterm::event::KeyEvent> = None;
                while event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            last_key = Some(key);
                        }
                    }
                }
                if let Some(key) = last_key {
                    handle_key_event(app, key)?;
                    if app.should_quit {
                        return Ok(());
                    }
                }
            }
            Some(file_event) = rx.recv() => {
                if !app.is_paused() || app.events.is_empty() {
                    let diff_result = match app.diff_mode {
                        DiffMode::All => git_engine.compute_diff(&file_event.path),
                        DiffMode::Staged => git_engine.compute_staged_diff(&file_event.path),
                        DiffMode::Unstaged => git_engine.compute_unstaged_diff(&file_event.path),
                    };

                    match diff_result {
                        Ok(diff) => {
                            if diff.stats.added_count > 0 || diff.stats.deleted_count > 0 || diff.is_new_file || diff.is_truncated {
                                let displayed = DisplayedEvent {
                                    file_path: file_event.path.clone(),
                                    relative_path: git_engine.relative_path(&file_event.path),
                                    timestamp: Utc::now(),
                                    diff,
                                };
                                app.add_event(displayed);
                                tracing::debug!("Processed change ({:?}): {:?}", app.diff_mode, file_event.path);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to compute diff for {:?}: {}", file_event.path, e);
                        }
                    }
                }
            }
            Some(_) = config_rx.recv() => {
                tracing::info!("Config file changed, reloading...");
                app.reload_config();
            }
        }
    }
}
