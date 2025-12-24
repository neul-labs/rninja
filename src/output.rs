//! Styled terminal output for build progress

use console::{Style, Term};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Terminal style configuration
#[derive(Clone)]
pub struct Styles {
    pub progress: Style,
    pub success: Style,
    pub error: Style,
    pub cache_hit: Style,
    pub target: Style,
    pub command: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            progress: Style::new().cyan().bold(),
            success: Style::new().green().bold(),
            error: Style::new().red().bold(),
            cache_hit: Style::new().magenta(),
            target: Style::new().white(),
            command: Style::new().dim(),
        }
    }
}

/// Build progress reporter
pub struct ProgressReporter {
    term: Term,
    styles: Styles,
    total: usize,
    built: Arc<AtomicUsize>,
    is_tty: bool,
    verbose: bool,
    enabled: Arc<AtomicBool>,
}

impl ProgressReporter {
    pub fn new(total: usize, verbose: bool) -> Self {
        let is_tty = atty::is(atty::Stream::Stdout);
        Self {
            term: Term::stderr(),
            styles: Styles::default(),
            total,
            built: Arc::new(AtomicUsize::new(0)),
            is_tty,
            verbose,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Get a handle for multi-threaded updates
    pub fn handle(&self) -> ProgressHandle {
        ProgressHandle {
            styles: Styles::default(),
            total: self.total,
            built: self.built.clone(),
            is_tty: self.is_tty,
            verbose: self.verbose,
            enabled: self.enabled.clone(),
        }
    }

    /// Report start of build
    pub fn start(&self) {
        if self.is_tty {
            let _ = self.term.write_line("");
        }
    }

    /// Report build completed
    pub fn finish(&self, success: bool, duration_ms: u64) {
        let built = self.built.load(Ordering::SeqCst);

        if success {
            if built == 0 {
                println!("ninja: no work to do.");
            } else {
                let style = &self.styles.success;
                println!(
                    "{} Built {} target(s) in {:.2}s",
                    style.apply_to("✓"),
                    built,
                    duration_ms as f64 / 1000.0
                );
            }
        } else {
            let style = &self.styles.error;
            println!(
                "{} Build failed",
                style.apply_to("✗")
            );
        }
    }

    /// Disable progress output (for tests)
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }
}

/// Thread-safe handle for progress updates
#[derive(Clone)]
pub struct ProgressHandle {
    styles: Styles,
    total: usize,
    built: Arc<AtomicUsize>,
    is_tty: bool,
    verbose: bool,
    enabled: Arc<AtomicBool>,
}

impl ProgressHandle {
    /// Report a target being built
    pub fn building(&self, target: &str, command: Option<&str>) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        let current = self.built.fetch_add(1, Ordering::SeqCst) + 1;

        if self.verbose {
            if let Some(cmd) = command {
                println!("{}", cmd);
            }
        } else {
            let progress = format!("[{}/{}]", current, self.total);
            println!(
                "{} {}",
                self.styles.progress.apply_to(&progress),
                self.styles.target.apply_to(target)
            );
        }
    }

    /// Report a cache hit
    pub fn cache_hit(&self, target: &str) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        let current = self.built.load(Ordering::SeqCst);
        let progress = format!("[{}/{}]", current, self.total);

        println!(
            "{} {} {}",
            self.styles.progress.apply_to(&progress),
            self.styles.cache_hit.apply_to("[CACHE]"),
            self.styles.target.apply_to(target)
        );
    }

    /// Report an error
    pub fn error(&self, target: &str, message: &str) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        eprintln!(
            "{} {}: {}",
            self.styles.error.apply_to("FAILED"),
            target,
            message
        );
    }

    /// Increment built count without printing
    pub fn increment(&self) {
        self.built.fetch_add(1, Ordering::SeqCst);
    }

    /// Get current count
    pub fn current(&self) -> usize {
        self.built.load(Ordering::SeqCst)
    }
}

/// Format duration in human-readable form
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        format!("{}m{}s", mins, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(65000), "1m5s");
    }
}
