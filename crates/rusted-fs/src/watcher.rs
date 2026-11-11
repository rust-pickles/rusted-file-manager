use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatchError {
    #[error("filesystem watcher failed: {0}")]
    Notify(#[from] notify::Error),
    #[error("watcher channel closed")]
    Closed,
}

/// Watches exactly one directory (non-recursively by default) and exposes the
/// coalesced changed paths. Dropping the value stops the OS watcher.
pub struct DirectoryWatcher {
    _watcher: RecommendedWatcher,
    events: Receiver<Result<Event, notify::Error>>,
}

impl DirectoryWatcher {
    pub fn watch(path: &Path) -> Result<Self, WatchError> {
        Self::watch_with(path, RecursiveMode::NonRecursive)
    }

    pub fn watch_recursive(path: &Path) -> Result<Self, WatchError> {
        Self::watch_with(path, RecursiveMode::Recursive)
    }

    fn watch_with(path: &Path, mode: RecursiveMode) -> Result<Self, WatchError> {
        let (sender, receiver) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })?;
        watcher.watch(path, mode)?;
        Ok(Self { _watcher: watcher, events: receiver })
    }

    /// Drains pending events without blocking; the caller coalesces refreshes.
    pub fn try_changed_paths(&self) -> Result<Vec<PathBuf>, WatchError> {
        let mut paths = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            for path in event.map_err(WatchError::Notify)?.paths {
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        Ok(paths)
    }

    pub fn changed(&self) -> bool {
        self.events.try_recv().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn reports_a_real_write() {
        let temp = tempfile::tempdir().unwrap();
        let watcher = DirectoryWatcher::watch(temp.path()).unwrap();
        std::fs::write(temp.path().join("new.txt"), b"data").unwrap();
        let mut seen = Vec::new();
        for _ in 0..40 {
            seen.extend(watcher.try_changed_paths().unwrap());
            if !seen.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(!seen.is_empty(), "no change notification arrived");
        assert!(seen.iter().any(|path| path.ends_with("new.txt")));
    }
}
