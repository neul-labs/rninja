//! Chrome tracing format export for build visualization
//!
//! Generates JSON that can be loaded in:
//! - chrome://tracing
//! - https://ui.perfetto.dev
//! - Speedscope

use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// A trace event in Chrome tracing format
#[derive(Serialize, Clone)]
struct TraceEvent {
    name: String,
    #[serde(rename = "cat")]
    category: String,
    #[serde(rename = "ph")]
    phase: String,
    #[serde(rename = "ts")]
    timestamp_us: u64,
    #[serde(rename = "dur", skip_serializing_if = "Option::is_none")]
    duration_us: Option<u64>,
    pid: u32,
    tid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<HashMap<String, String>>,
}

/// Build trace collector
pub struct BuildTrace {
    events: Mutex<Vec<TraceEvent>>,
    start_time: Instant,
    start_timestamp: u64,
    next_tid: AtomicU64,
    enabled: bool,
}

impl BuildTrace {
    /// Create a new build trace
    pub fn new(enabled: bool) -> Self {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        Self {
            events: Mutex::new(Vec::new()),
            start_time: Instant::now(),
            start_timestamp,
            next_tid: AtomicU64::new(1),
            enabled,
        }
    }

    /// Get a thread ID for a worker
    pub fn allocate_tid(&self) -> u32 {
        self.next_tid.fetch_add(1, Ordering::SeqCst) as u32
    }

    /// Get the current timestamp in microseconds since trace start
    pub fn timestamp(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    /// Record the start of a build target
    pub fn begin_target(&self, target: &str, command: Option<&str>, tid: u32) -> u64 {
        if !self.enabled {
            return 0;
        }

        let ts = self.start_time.elapsed().as_micros() as u64;

        let mut args = HashMap::new();
        if let Some(cmd) = command {
            args.insert("command".to_string(), cmd.to_string());
        }

        let event = TraceEvent {
            name: target.to_string(),
            category: "build".to_string(),
            phase: "B".to_string(), // Begin
            timestamp_us: ts,
            duration_us: None,
            pid: 1,
            tid,
            args: if args.is_empty() { None } else { Some(args) },
        };

        self.events.lock().push(event);
        ts
    }

    /// Record the end of a build target
    pub fn end_target(&self, target: &str, tid: u32, cache_hit: bool) {
        if !self.enabled {
            return;
        }

        let ts = self.start_time.elapsed().as_micros() as u64;

        let mut args = HashMap::new();
        if cache_hit {
            args.insert("cache".to_string(), "hit".to_string());
        }

        let event = TraceEvent {
            name: target.to_string(),
            category: "build".to_string(),
            phase: "E".to_string(), // End
            timestamp_us: ts,
            duration_us: None,
            pid: 1,
            tid,
            args: if args.is_empty() { None } else { Some(args) },
        };

        self.events.lock().push(event);
    }

    /// Record a complete event (X phase)
    pub fn complete_target(
        &self,
        target: &str,
        start_us: u64,
        duration_us: u64,
        tid: u32,
        command: Option<&str>,
        cache_hit: bool,
    ) {
        if !self.enabled {
            return;
        }

        let mut args = HashMap::new();
        if let Some(cmd) = command {
            args.insert("command".to_string(), cmd.to_string());
        }
        if cache_hit {
            args.insert("cache".to_string(), "hit".to_string());
        }

        let event = TraceEvent {
            name: target.to_string(),
            category: "build".to_string(),
            phase: "X".to_string(), // Complete
            timestamp_us: start_us,
            duration_us: Some(duration_us),
            pid: 1,
            tid,
            args: if args.is_empty() { None } else { Some(args) },
        };

        self.events.lock().push(event);
    }

    /// Record a metadata event
    pub fn add_metadata(&self, name: &str, value: &str) {
        if !self.enabled {
            return;
        }

        let mut args = HashMap::new();
        args.insert("name".to_string(), value.to_string());

        let event = TraceEvent {
            name: name.to_string(),
            category: "__metadata".to_string(),
            phase: "M".to_string(),
            timestamp_us: 0,
            duration_us: None,
            pid: 1,
            tid: 0,
            args: Some(args),
        };

        self.events.lock().push(event);
    }

    /// Record an instant event (e.g., cache miss)
    pub fn instant(&self, name: &str, category: &str, tid: u32) {
        if !self.enabled {
            return;
        }

        let ts = self.start_time.elapsed().as_micros() as u64;

        let event = TraceEvent {
            name: name.to_string(),
            category: category.to_string(),
            phase: "i".to_string(), // Instant
            timestamp_us: ts,
            duration_us: None,
            pid: 1,
            tid,
            args: None,
        };

        self.events.lock().push(event);
    }

    /// Write trace to a file
    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let events = self.events.lock();

        // Add process name metadata
        let mut all_events = vec![TraceEvent {
            name: "process_name".to_string(),
            category: "__metadata".to_string(),
            phase: "M".to_string(),
            timestamp_us: 0,
            duration_us: None,
            pid: 1,
            tid: 0,
            args: Some({
                let mut m = HashMap::new();
                m.insert("name".to_string(), "rninja".to_string());
                m
            }),
        }];

        all_events.extend(events.iter().cloned());

        let trace = serde_json::json!({
            "traceEvents": all_events,
            "displayTimeUnit": "ms",
            "metadata": {
                "generator": "rninja",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string_pretty(&trace)?.as_bytes())?;

        Ok(())
    }

    /// Get total build duration in microseconds
    pub fn total_duration_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    /// Get the critical path (longest chain of dependent builds)
    pub fn critical_path(&self) -> Vec<String> {
        // For now, return the events sorted by duration (longest first)
        let events = self.events.lock();
        let mut complete_events: Vec<_> = events
            .iter()
            .filter(|e| e.phase == "X" && e.duration_us.is_some())
            .collect();

        complete_events.sort_by(|a, b| {
            b.duration_us.unwrap_or(0).cmp(&a.duration_us.unwrap_or(0))
        });

        complete_events
            .iter()
            .take(10)
            .map(|e| e.name.clone())
            .collect()
    }

    /// Get slow targets (above threshold)
    pub fn slow_targets(&self, threshold_ms: u64) -> Vec<(String, u64)> {
        let threshold_us = threshold_ms * 1000;
        let events = self.events.lock();

        let mut slow: Vec<_> = events
            .iter()
            .filter(|e| e.phase == "X" && e.duration_us.unwrap_or(0) > threshold_us)
            .map(|e| (e.name.clone(), e.duration_us.unwrap_or(0) / 1000))
            .collect();

        slow.sort_by(|a, b| b.1.cmp(&a.1));
        slow
    }

    /// Get build statistics
    pub fn stats(&self) -> TraceStats {
        let events = self.events.lock();
        let complete_events: Vec<_> = events
            .iter()
            .filter(|e| e.phase == "X")
            .collect();

        let total_targets = complete_events.len();
        let total_time_us: u64 = complete_events
            .iter()
            .filter_map(|e| e.duration_us)
            .sum();

        let cache_hits = complete_events
            .iter()
            .filter(|e| {
                e.args.as_ref()
                    .map(|a| a.get("cache").map(|v| v == "hit").unwrap_or(false))
                    .unwrap_or(false)
            })
            .count();

        let avg_time_us = if total_targets > 0 {
            total_time_us / total_targets as u64
        } else {
            0
        };

        let max_time_us = complete_events
            .iter()
            .filter_map(|e| e.duration_us)
            .max()
            .unwrap_or(0);

        TraceStats {
            total_targets,
            cache_hits,
            total_time_us,
            avg_time_us,
            max_time_us,
            wall_time_us: self.total_duration_us(),
        }
    }
}

/// Build trace statistics
#[derive(Debug)]
pub struct TraceStats {
    pub total_targets: usize,
    pub cache_hits: usize,
    pub total_time_us: u64,
    pub avg_time_us: u64,
    pub max_time_us: u64,
    pub wall_time_us: u64,
}

impl TraceStats {
    /// Calculate parallelism (total CPU time / wall time)
    pub fn parallelism(&self) -> f64 {
        if self.wall_time_us == 0 {
            0.0
        } else {
            self.total_time_us as f64 / self.wall_time_us as f64
        }
    }

    /// Format as human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Build statistics:\n\
             Targets built: {}\n\
             Cache hits: {}\n\
             Wall time: {:.2}s\n\
             Total CPU time: {:.2}s\n\
             Parallelism: {:.2}x\n\
             Avg target time: {}ms\n\
             Max target time: {}ms",
            self.total_targets,
            self.cache_hits,
            self.wall_time_us as f64 / 1_000_000.0,
            self.total_time_us as f64 / 1_000_000.0,
            self.parallelism(),
            self.avg_time_us / 1000,
            self.max_time_us / 1000,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_collection() {
        let trace = BuildTrace::new(true);
        let tid = trace.allocate_tid();

        trace.complete_target("foo.o", 0, 1000, tid, Some("gcc -c foo.c"), false);
        trace.complete_target("bar.o", 1000, 2000, tid, Some("gcc -c bar.c"), true);

        let stats = trace.stats();
        assert_eq!(stats.total_targets, 2);
        assert_eq!(stats.cache_hits, 1);
    }

    #[test]
    fn test_slow_targets() {
        let trace = BuildTrace::new(true);
        let tid = trace.allocate_tid();

        trace.complete_target("fast.o", 0, 100_000, tid, None, false); // 100ms
        trace.complete_target("slow.o", 100_000, 5_000_000, tid, None, false); // 5s

        let slow = trace.slow_targets(1000); // > 1s
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].0, "slow.o");
    }
}
