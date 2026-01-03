//! File system watcher for manifest invalidation
//!
//! Watches build.ninja files and their includes for changes.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Events from the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// A watched file was modified
    Modified(PathBuf),
    /// A watched file was deleted
    Deleted(PathBuf),
    /// A watched file was created (for new includes)
    Created(PathBuf),
    /// Watcher error
    Error(String),
}

/// File system watcher for build manifests
pub struct FileWatcher {
    /// The underlying watcher
    watcher: Option<RecommendedWatcher>,

    /// Paths currently being watched
    watched_paths: RwLock<HashSet<PathBuf>>,

    /// Channel for receiving events
    event_rx: Option<Receiver<WatchEvent>>,

    /// Sender for events (kept for cloning to watcher callback)
    event_tx: Sender<WatchEvent>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> anyhow::Result<Self> {
        let (event_tx, event_rx) = channel();

        let tx_clone = event_tx.clone();
        let watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        Self::handle_event(&tx_clone, event);
                    }
                    Err(e) => {
                        let _ = tx_clone.send(WatchEvent::Error(e.to_string()));
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        Ok(Self {
            watcher: Some(watcher),
            watched_paths: RwLock::new(HashSet::new()),
            event_rx: Some(event_rx),
            event_tx,
        })
    }

    /// Handle a notify event
    fn handle_event(tx: &Sender<WatchEvent>, event: Event) {
        use notify::EventKind;

        for path in event.paths {
            let watch_event = match event.kind {
                EventKind::Create(_) => WatchEvent::Created(path),
                EventKind::Modify(_) => WatchEvent::Modified(path),
                EventKind::Remove(_) => WatchEvent::Deleted(path),
                _ => continue,
            };

            if let Err(e) = tx.send(watch_event) {
                error!("Failed to send watch event: {}", e);
            }
        }
    }

    /// Watch a path
    pub fn watch(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            // Only watch if not already watching
            if self.watched_paths.read().contains(path) {
                return Ok(());
            }

            debug!("Watching: {}", path.display());
            watcher.watch(path, RecursiveMode::NonRecursive)?;
            self.watched_paths.write().insert(path.clone());
        }
        Ok(())
    }

    /// Unwatch a path
    pub fn unwatch(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            if self.watched_paths.read().contains(path) {
                debug!("Unwatching: {}", path.display());
                watcher.unwatch(path)?;
                self.watched_paths.write().remove(path);
            }
        }
        Ok(())
    }

    /// Watch multiple paths
    pub fn watch_paths(&mut self, paths: &[PathBuf]) -> anyhow::Result<()> {
        for path in paths {
            if let Err(e) = self.watch(path) {
                warn!("Failed to watch {}: {}", path.display(), e);
            }
        }
        Ok(())
    }

    /// Get the number of watched paths
    pub fn watched_count(&self) -> usize {
        self.watched_paths.read().len()
    }

    /// Take the event receiver (can only be called once)
    pub fn take_event_receiver(&mut self) -> Option<Receiver<WatchEvent>> {
        self.event_rx.take()
    }

    /// Try to receive an event (non-blocking)
    pub fn try_recv(&self) -> Option<WatchEvent> {
        // This will return None if the receiver was taken
        None
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            watcher: None,
            watched_paths: RwLock::new(HashSet::new()),
            event_rx: None,
            event_tx: channel().0,
        })
    }
}

/// Watcher event processor that integrates with DaemonState
pub struct WatcherProcessor {
    event_rx: Receiver<WatchEvent>,
}

impl WatcherProcessor {
    /// Create a new watcher processor from a file watcher
    pub fn new(watcher: &mut FileWatcher) -> Option<Self> {
        watcher.take_event_receiver().map(|rx| Self { event_rx: rx })
    }

    /// Process events, returning paths that need invalidation
    pub fn process_events(&self) -> Vec<PathBuf> {
        let mut invalidate = Vec::new();

        // Drain all pending events
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                WatchEvent::Modified(path) | WatchEvent::Deleted(path) => {
                    info!("File changed: {}", path.display());
                    invalidate.push(path);
                }
                WatchEvent::Created(path) => {
                    debug!("File created: {}", path.display());
                    // Created files might be new includes
                    invalidate.push(path);
                }
                WatchEvent::Error(e) => {
                    warn!("Watcher error: {}", e);
                }
            }
        }

        invalidate
    }

    /// Block waiting for an event with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Option<WatchEvent> {
        self.event_rx.recv_timeout(timeout).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_watched_count() {
        let watcher = FileWatcher::new().unwrap();
        assert_eq!(watcher.watched_count(), 0);
    }
}
