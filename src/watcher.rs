use anyhow::Result;
use ignore::gitignore::Gitignore;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::config::WatcherConfig;
use crate::types::FileChangeEvent;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(
        repo_root: PathBuf,
        config: &WatcherConfig,
        tx: mpsc::UnboundedSender<FileChangeEvent>,
    ) -> Result<Self> {
        let debounce_duration = Duration::from_millis(config.debounce_ms);
        let last_events: Arc<RwLock<HashMap<PathBuf, Instant>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let gitignore = load_gitignore(&repo_root);
        let git_dir = repo_root.join(".git");

        let extra_ignores: Vec<glob::Pattern> = config
            .ignore_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        let repo_root_clone = repo_root.clone();
        let last_events_clone = last_events.clone();
        let handle = Handle::current();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                let tx = tx.clone();
                let git_dir = git_dir.clone();
                let repo_root_clone = repo_root_clone.clone();
                let gitignore = gitignore.clone();
                let extra_ignores = extra_ignores.clone();
                let last_events_clone = last_events_clone.clone();
                let handle = handle.clone();

                let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    let event = match res {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::warn!("notify error: {}", e);
                            return;
                        }
                    };

                    if !matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    ) {
                        return;
                    }

                    for path in event.paths {
                        if !path.exists() || !path.is_file() {
                            continue;
                        }

                        if path.starts_with(&git_dir) {
                            continue;
                        }

                        let relative = match path.strip_prefix(&repo_root_clone) {
                            Ok(r) => r.to_path_buf(),
                            Err(_) => path.clone(),
                        };

                        if let Some(ref gi) = gitignore {
                            if gi.matched_path_or_any_parents(&relative, false).is_ignore() {
                                continue;
                            }
                        }

                        let relative_str = relative.to_string_lossy();
                        if extra_ignores.iter().any(|p| p.matches(&relative_str)) {
                            continue;
                        }

                        let tx = tx.clone();
                        let path = path.clone();
                        let last_events = last_events_clone.clone();
                        let debounce = debounce_duration;
                        let handle = handle.clone();

                        handle.spawn(async move {
                            {
                                let mut map = last_events.write().await;
                                let now = Instant::now();

                                if let Some(last) = map.get(&path) {
                                    if now.duration_since(*last) < debounce {
                                        map.insert(path.clone(), now);
                                        return;
                                    }
                                }
                                map.insert(path.clone(), now);
                            }

                            tokio::time::sleep(debounce).await;

                            if path.exists() {
                                let _ = tx.send(FileChangeEvent {
                                    path,
                                    timestamp: SystemTime::now(),
                                });
                            }
                        });
                    }
                }));

                if result.is_err() {
                    tracing::error!("panic inside notify callback - recovered");
                }
            },
            Config::default(),
        )?;

        watcher.watch(&repo_root, RecursiveMode::Recursive)?;

        Ok(Self { _watcher: watcher })
    }
}

fn load_gitignore(repo_root: &Path) -> Option<Gitignore> {
    let gitignore_path = repo_root.join(".gitignore");
    if gitignore_path.exists() {
        let (gi, _) = Gitignore::new(&gitignore_path);
        Some(gi)
    } else {
        None
    }
}
