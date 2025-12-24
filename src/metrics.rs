//! Build metrics and statistics collection

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Global build metrics
#[derive(Debug, Default)]
pub struct BuildMetrics {
    /// Total number of targets processed
    pub targets_total: AtomicUsize,
    /// Number of targets built
    pub targets_built: AtomicUsize,
    /// Number of targets skipped (up-to-date)
    pub targets_skipped: AtomicUsize,
    /// Number of cache hits
    pub cache_hits: AtomicUsize,
    /// Number of cache misses
    pub cache_misses: AtomicUsize,
    /// Number of build failures
    pub failures: AtomicUsize,
    /// Total build time in milliseconds
    pub build_time_ms: AtomicU64,
    /// Total command execution time in milliseconds
    pub exec_time_ms: AtomicU64,
    /// Total cache lookup time in milliseconds
    pub cache_lookup_time_ms: AtomicU64,
    /// Total cache store time in milliseconds
    pub cache_store_time_ms: AtomicU64,
}

impl BuildMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_target_built(&self, exec_time: Duration) {
        self.targets_built.fetch_add(1, Ordering::SeqCst);
        self.exec_time_ms.fetch_add(exec_time.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn record_target_skipped(&self) {
        self.targets_skipped.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_cache_hit(&self, lookup_time: Duration) {
        self.cache_hits.fetch_add(1, Ordering::SeqCst);
        self.cache_lookup_time_ms.fetch_add(lookup_time.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn record_cache_miss(&self, lookup_time: Duration) {
        self.cache_misses.fetch_add(1, Ordering::SeqCst);
        self.cache_lookup_time_ms.fetch_add(lookup_time.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn record_cache_store(&self, store_time: Duration) {
        self.cache_store_time_ms.fetch_add(store_time.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::SeqCst);
    }

    pub fn set_total_targets(&self, count: usize) {
        self.targets_total.store(count, Ordering::SeqCst);
    }

    pub fn set_build_time(&self, duration: Duration) {
        self.build_time_ms.store(duration.as_millis() as u64, Ordering::SeqCst);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            targets_total: self.targets_total.load(Ordering::SeqCst),
            targets_built: self.targets_built.load(Ordering::SeqCst),
            targets_skipped: self.targets_skipped.load(Ordering::SeqCst),
            cache_hits: self.cache_hits.load(Ordering::SeqCst),
            cache_misses: self.cache_misses.load(Ordering::SeqCst),
            failures: self.failures.load(Ordering::SeqCst),
            build_time_ms: self.build_time_ms.load(Ordering::SeqCst),
            exec_time_ms: self.exec_time_ms.load(Ordering::SeqCst),
            cache_lookup_time_ms: self.cache_lookup_time_ms.load(Ordering::SeqCst),
            cache_store_time_ms: self.cache_store_time_ms.load(Ordering::SeqCst),
        }
    }
}

/// Immutable snapshot of metrics for reporting
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub targets_total: usize,
    pub targets_built: usize,
    pub targets_skipped: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub failures: usize,
    pub build_time_ms: u64,
    pub exec_time_ms: u64,
    pub cache_lookup_time_ms: u64,
    pub cache_store_time_ms: u64,
}

impl MetricsSnapshot {
    /// Cache hit rate as percentage (0-100)
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }

    /// Format as human-readable summary
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Build completed in {}ms",
            self.build_time_ms
        ));

        lines.push(format!(
            "Targets: {} total, {} built, {} up-to-date, {} failed",
            self.targets_total,
            self.targets_built,
            self.targets_skipped,
            self.failures
        ));

        if self.cache_hits > 0 || self.cache_misses > 0 {
            lines.push(format!(
                "Cache: {} hits, {} misses ({:.1}% hit rate)",
                self.cache_hits,
                self.cache_misses,
                self.cache_hit_rate()
            ));

            lines.push(format!(
                "Cache time: {}ms lookup, {}ms store",
                self.cache_lookup_time_ms,
                self.cache_store_time_ms
            ));
        }

        lines.push(format!(
            "Execution time: {}ms",
            self.exec_time_ms
        ));

        lines.join("\n")
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "targets": {
                "total": self.targets_total,
                "built": self.targets_built,
                "skipped": self.targets_skipped,
                "failed": self.failures
            },
            "cache": {
                "hits": self.cache_hits,
                "misses": self.cache_misses,
                "hit_rate": self.cache_hit_rate()
            },
            "timing_ms": {
                "total": self.build_time_ms,
                "execution": self.exec_time_ms,
                "cache_lookup": self.cache_lookup_time_ms,
                "cache_store": self.cache_store_time_ms
            }
        }).to_string()
    }
}

/// Timer guard for measuring durations
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
