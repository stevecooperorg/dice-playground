//! Watch filesystem paths and wait for relevant changes (used by `dice eval --watch`).

use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

/// Receives `notify` events for paths registered via [`Self::watch_path`] or [`Self::setup`].
pub struct FileWatcher {
    watch_root: PathBuf,
    event_receiver: mpsc::UnboundedReceiver<notify::Result<notify::Event>>,
    _watcher: RecommendedWatcher,
}

impl FileWatcher {
    /// Create a watcher; call [`Self::watch_path`] or [`Self::setup`] before waiting.
    pub fn new(watch_root: PathBuf) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = event_sender.send(res) {
                eprintln!("Error sending file event: {e}");
            }
        })
        .context("Failed to create file watcher")?;

        Ok(Self {
            watch_root,
            event_receiver,
            _watcher: watcher,
        })
    }

    /// Watch [`Self::watch_root`] recursively.
    pub fn setup(&mut self) -> Result<()> {
        self._watcher
            .watch(&self.watch_root, RecursiveMode::Recursive)
            .with_context(|| format!("Failed to watch directory: {}", self.watch_root.display()))?;
        Ok(())
    }

    /// Watch a single path. Files use non-recursive mode; directories use recursive when `recursive` is true.
    pub fn watch_path(&mut self, path: &Path, recursive: bool) -> Result<()> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        self._watcher
            .watch(path, mode)
            .with_context(|| format!("Failed to watch path: {}", path.display()))?;
        Ok(())
    }

    /// Block until an event names a path for which `predicate` returns true.
    pub async fn wait_for_change_matching<F>(&mut self, predicate: F) -> Result<()>
    where
        F: Fn(&Path) -> bool,
    {
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                Ok(notify::Event { paths, .. }) => {
                    for path in paths {
                        if predicate(&path) {
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {e}");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn watcher_creation_succeeds() {
        let temp_dir = TempDir::new().context("temp dir").unwrap();
        let watcher = FileWatcher::new(temp_dir.path().to_path_buf());
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn watcher_detects_file_change() {
        let temp_dir = TempDir::new().context("temp dir").unwrap();
        let script = temp_dir.path().join("roll.dice");
        std::fs::write(&script, "output(\"x\", 1d6)")
            .context("write script")
            .unwrap();

        let mut watcher = FileWatcher::new(temp_dir.path().to_path_buf()).unwrap();
        watcher.watch_path(&script, false).unwrap();

        let script_for_task = script.clone();
        let handle = tokio::spawn(async move {
            tokio::time::timeout(
                Duration::from_secs(5),
                watcher.wait_for_change_matching(|p| paths_same_file(p, &script_for_task)),
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        std::fs::write(&script, "output(\"x\", 2d6)")
            .context("rewrite script")
            .unwrap();

        let result = handle.await.context("join watcher task").unwrap();
        assert!(result.is_ok());
    }

    fn paths_same_file(a: &Path, b: &Path) -> bool {
        match (a.canonicalize(), b.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => a == b,
        }
    }
}
