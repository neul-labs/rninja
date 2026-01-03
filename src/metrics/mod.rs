//! Build metrics collection and export
//!
//! Provides metrics tracking for builds with Prometheus-compatible export.

pub mod prometheus;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Build metrics with atomic counters for thread-safe updates
#[derive(Debug, Default)]
pub struct BuildMetrics {
    // Target counters
    pub targets_total: AtomicUsize,
    pub targets_built: AtomicUsize,
    pub targets_skipped: AtomicUsize,
    pub targets_failed: AtomicUsize,

    // Cache counters
    pub cache_hits: AtomicUsize,
    pub cache_misses: AtomicUsize,
    pub cache_errors: AtomicUsize,

    // Remote cache counters
    pub remote_hits: AtomicUsize,
    pub remote_misses: AtomicUsize,
    pub remote_errors: AtomicUsize,

    // Timing (in microseconds)
    pub build_time_us: AtomicU64,
    pub exec_time_us: AtomicU64,
    pub cache_lookup_time_us: AtomicU64,
    pub cache_store_time_us: AtomicU64,

    // Queue depth tracking
    pub max_queue_depth: AtomicUsize,
}

impl BuildMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_target_built(&self, exec_time: Duration) {
        self.targets_built.fetch_add(1, Ordering::Relaxed);
        self.exec_time_us
            .fetch_add(exec_time.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn record_target_skipped(&self) {
        self.targets_skipped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_target_failed(&self) {
        self.targets_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self, lookup_time: Duration) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
        self.cache_lookup_time_us
            .fetch_add(lookup_time.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self, lookup_time: Duration) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        self.cache_lookup_time_us
            .fetch_add(lookup_time.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn record_cache_store(&self, store_time: Duration) {
        self.cache_store_time_us
            .fetch_add(store_time.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn record_queue_depth(&self, depth: usize) {
        let current_max = self.max_queue_depth.load(Ordering::Relaxed);
        if depth > current_max {
            self.max_queue_depth.store(depth, Ordering::Relaxed);
        }
    }

    pub fn set_total_targets(&self, count: usize) {
        self.targets_total.store(count, Ordering::Relaxed);
    }

    pub fn set_build_time(&self, duration: Duration) {
        self.build_time_us
            .store(duration.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get a snapshot of the metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            targets_total: self.targets_total.load(Ordering::Relaxed),
            targets_built: self.targets_built.load(Ordering::Relaxed),
            targets_skipped: self.targets_skipped.load(Ordering::Relaxed),
            targets_failed: self.targets_failed.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            cache_errors: self.cache_errors.load(Ordering::Relaxed),
            remote_hits: self.remote_hits.load(Ordering::Relaxed),
            remote_misses: self.remote_misses.load(Ordering::Relaxed),
            remote_errors: self.remote_errors.load(Ordering::Relaxed),
            build_time_us: self.build_time_us.load(Ordering::Relaxed),
            exec_time_us: self.exec_time_us.load(Ordering::Relaxed),
            cache_lookup_time_us: self.cache_lookup_time_us.load(Ordering::Relaxed),
            cache_store_time_us: self.cache_store_time_us.load(Ordering::Relaxed),
            max_queue_depth: self.max_queue_depth.load(Ordering::Relaxed),
        }
    }
}

/// Immutable snapshot of metrics for reporting
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub targets_total: usize,
    pub targets_built: usize,
    pub targets_skipped: usize,
    pub targets_failed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_errors: usize,
    pub remote_hits: usize,
    pub remote_misses: usize,
    pub remote_errors: usize,
    pub build_time_us: u64,
    pub exec_time_us: u64,
    pub cache_lookup_time_us: u64,
    pub cache_store_time_us: u64,
    pub max_queue_depth: usize,
}

impl MetricsSnapshot {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::json!({
            "targets": {
                "total": self.targets_total,
                "built": self.targets_built,
                "skipped": self.targets_skipped,
                "failed": self.targets_failed
            },
            "cache": {
                "hits": self.cache_hits,
                "misses": self.cache_misses,
                "errors": self.cache_errors,
                "hit_rate": self.cache_hit_rate()
            },
            "remote": {
                "hits": self.remote_hits,
                "misses": self.remote_misses,
                "errors": self.remote_errors
            },
            "timing_us": {
                "build": self.build_time_us,
                "exec": self.exec_time_us,
                "cache_lookup": self.cache_lookup_time_us,
                "cache_store": self.cache_store_time_us
            },
            "max_queue_depth": self.max_queue_depth
        })
        .to_string()
    }
}
