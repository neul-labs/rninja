//! Styled terminal output for build progress

use console::{Style, Term};
use serde::Serialize;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Output mode for progress reporting
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputMode {
    /// Human-readable output with colors and formatting
    Human,
    /// Machine-readable JSON output (one JSON object per line)
    Json,
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::Human
    }
}

/// JSON event types for machine consumption
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonEvent<'a> {
    /// Build has started
    BuildStarted {
        total_targets: usize,
        parallelism: usize,
    },
    /// A target has started building
    TargetStarted {
        target: &'a str,
        index: usize,
        total: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<&'a str>,
    },
    /// A target was restored from cache
    CacheHit {
        target: &'a str,
        index: usize,
        total: usize,
    },
    /// A target has finished successfully
    TargetFinished {
        target: &'a str,
        index: usize,
        total: usize,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<&'a str>,
    },
    /// Build has finished
    BuildFinished {
        success: bool,
        targets_built: usize,
        targets_total: usize,
        duration_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_hits: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_misses: Option<usize>,
    },
    /// No work needed
    NoWorkToDo,
    /// An error occurred
    Error {
        message: &'a str,
    },
}

impl<'a> JsonEvent<'a> {
    /// Emit the event as a JSON line to stdout
    pub fn emit(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{}", json);
            let _ = stdout.flush();
        }
    }
}

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
    output_mode: OutputMode,
    parallelism: usize,
}

impl ProgressReporter {
    pub fn new(total: usize, verbose: bool) -> Self {
        Self::with_mode(total, verbose, OutputMode::Human, 1)
    }

    pub fn with_mode(total: usize, verbose: bool, output_mode: OutputMode, parallelism: usize) -> Self {
        let is_tty = atty::is(atty::Stream::Stdout);
        Self {
            term: Term::stderr(),
            styles: Styles::default(),
            total,
            built: Arc::new(AtomicUsize::new(0)),
            is_tty,
            verbose,
            enabled: Arc::new(AtomicBool::new(true)),
            output_mode,
            parallelism,
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
            output_mode: self.output_mode,
        }
    }

    /// Report start of build
    pub fn start(&self) {
        match self.output_mode {
            OutputMode::Human => {
                if self.is_tty {
                    let _ = self.term.write_line("");
                }
            }
            OutputMode::Json => {
                JsonEvent::BuildStarted {
                    total_targets: self.total,
                    parallelism: self.parallelism,
                }.emit();
            }
        }
    }

    /// Report build completed
    pub fn finish(&self, success: bool, duration_ms: u64) {
        self.finish_with_cache(success, duration_ms, None, None)
    }

    /// Report build completed with cache statistics
    pub fn finish_with_cache(&self, success: bool, duration_ms: u64, cache_hits: Option<usize>, cache_misses: Option<usize>) {
        let built = self.built.load(Ordering::SeqCst);

        match self.output_mode {
            OutputMode::Human => {
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
            OutputMode::Json => {
                if built == 0 && success {
                    JsonEvent::NoWorkToDo.emit();
                } else {
                    JsonEvent::BuildFinished {
                        success,
                        targets_built: built,
                        targets_total: self.total,
                        duration_ms,
                        cache_hits,
                        cache_misses,
                    }.emit();
                }
            }
        }
    }

    /// Disable progress output (for tests)
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Get the output mode
    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
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
    output_mode: OutputMode,
}

impl ProgressHandle {
    /// Report a target being built
    pub fn building(&self, target: &str, command: Option<&str>) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        let current = self.built.fetch_add(1, Ordering::SeqCst) + 1;

        match self.output_mode {
            OutputMode::Human => {
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
            OutputMode::Json => {
                JsonEvent::TargetStarted {
                    target,
                    index: current,
                    total: self.total,
                    command: if self.verbose { command } else { None },
                }.emit();
            }
        }
    }

    /// Report a target finished building
    pub fn finished(&self, target: &str, success: bool, error: Option<&str>) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        let current = self.built.load(Ordering::SeqCst);

        match self.output_mode {
            OutputMode::Human => {
                // Human mode doesn't show finished events, only errors
                if !success {
                    if let Some(msg) = error {
                        eprintln!(
                            "{} {}: {}",
                            self.styles.error.apply_to("FAILED"),
                            target,
                            msg
                        );
                    }
                }
            }
            OutputMode::Json => {
                JsonEvent::TargetFinished {
                    target,
                    index: current,
                    total: self.total,
                    success,
                    error,
                }.emit();
            }
        }
    }

    /// Report a cache hit
    pub fn cache_hit(&self, target: &str) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        let current = self.built.load(Ordering::SeqCst);

        match self.output_mode {
            OutputMode::Human => {
                let progress = format!("[{}/{}]", current, self.total);
                println!(
                    "{} {} {}",
                    self.styles.progress.apply_to(&progress),
                    self.styles.cache_hit.apply_to("[CACHE]"),
                    self.styles.target.apply_to(target)
                );
            }
            OutputMode::Json => {
                JsonEvent::CacheHit {
                    target,
                    index: current,
                    total: self.total,
                }.emit();
            }
        }
    }

    /// Report an error
    pub fn error(&self, target: &str, message: &str) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        match self.output_mode {
            OutputMode::Human => {
                eprintln!(
                    "{} {}: {}",
                    self.styles.error.apply_to("FAILED"),
                    target,
                    message
                );
            }
            OutputMode::Json => {
                JsonEvent::Error { message }.emit();
            }
        }
    }

    /// Increment built count without printing
    pub fn increment(&self) {
        self.built.fetch_add(1, Ordering::SeqCst);
    }

    /// Get current count
    pub fn current(&self) -> usize {
        self.built.load(Ordering::SeqCst)
    }

    /// Get the output mode
    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
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
