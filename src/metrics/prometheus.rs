//! Prometheus metrics export
//!
//! Exports metrics in Prometheus text exposition format.

use super::MetricsSnapshot;

/// Export metrics in Prometheus text format
pub fn export_prometheus(snapshot: &MetricsSnapshot) -> String {
    let mut output = String::new();

    // Target metrics
    output.push_str("# HELP rninja_targets_total Total number of targets\n");
    output.push_str("# TYPE rninja_targets_total gauge\n");
    output.push_str(&format!("rninja_targets_total {}\n", snapshot.targets_total));

    output.push_str("# HELP rninja_targets_built Number of targets built\n");
    output.push_str("# TYPE rninja_targets_built counter\n");
    output.push_str(&format!(
        "rninja_targets_built{{status=\"success\"}} {}\n",
        snapshot.targets_built
    ));
    output.push_str(&format!(
        "rninja_targets_built{{status=\"skipped\"}} {}\n",
        snapshot.targets_skipped
    ));
    output.push_str(&format!(
        "rninja_targets_built{{status=\"failed\"}} {}\n",
        snapshot.targets_failed
    ));

    // Cache metrics
    output.push_str("# HELP rninja_cache_operations_total Cache operations\n");
    output.push_str("# TYPE rninja_cache_operations_total counter\n");
    output.push_str(&format!(
        "rninja_cache_operations_total{{result=\"hit\"}} {}\n",
        snapshot.cache_hits
    ));
    output.push_str(&format!(
        "rninja_cache_operations_total{{result=\"miss\"}} {}\n",
        snapshot.cache_misses
    ));
    output.push_str(&format!(
        "rninja_cache_operations_total{{result=\"error\"}} {}\n",
        snapshot.cache_errors
    ));

    output.push_str("# HELP rninja_cache_hit_rate Cache hit rate (0-1)\n");
    output.push_str("# TYPE rninja_cache_hit_rate gauge\n");
    output.push_str(&format!(
        "rninja_cache_hit_rate {:.4}\n",
        snapshot.cache_hit_rate()
    ));

    // Remote cache metrics
    output.push_str("# HELP rninja_remote_cache_operations_total Remote cache operations\n");
    output.push_str("# TYPE rninja_remote_cache_operations_total counter\n");
    output.push_str(&format!(
        "rninja_remote_cache_operations_total{{result=\"hit\"}} {}\n",
        snapshot.remote_hits
    ));
    output.push_str(&format!(
        "rninja_remote_cache_operations_total{{result=\"miss\"}} {}\n",
        snapshot.remote_misses
    ));
    output.push_str(&format!(
        "rninja_remote_cache_operations_total{{result=\"error\"}} {}\n",
        snapshot.remote_errors
    ));

    // Timing metrics
    output.push_str("# HELP rninja_build_duration_seconds Total build duration\n");
    output.push_str("# TYPE rninja_build_duration_seconds gauge\n");
    output.push_str(&format!(
        "rninja_build_duration_seconds {:.6}\n",
        snapshot.build_time_us as f64 / 1_000_000.0
    ));

    output.push_str("# HELP rninja_exec_duration_seconds Total command execution time\n");
    output.push_str("# TYPE rninja_exec_duration_seconds counter\n");
    output.push_str(&format!(
        "rninja_exec_duration_seconds {:.6}\n",
        snapshot.exec_time_us as f64 / 1_000_000.0
    ));

    output.push_str("# HELP rninja_cache_lookup_duration_seconds Cache lookup time\n");
    output.push_str("# TYPE rninja_cache_lookup_duration_seconds counter\n");
    output.push_str(&format!(
        "rninja_cache_lookup_duration_seconds {:.6}\n",
        snapshot.cache_lookup_time_us as f64 / 1_000_000.0
    ));

    // Queue depth
    output.push_str("# HELP rninja_max_queue_depth Maximum queue depth during build\n");
    output.push_str("# TYPE rninja_max_queue_depth gauge\n");
    output.push_str(&format!(
        "rninja_max_queue_depth {}\n",
        snapshot.max_queue_depth
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_export() {
        let snapshot = MetricsSnapshot {
            targets_total: 100,
            targets_built: 50,
            targets_skipped: 45,
            targets_failed: 5,
            cache_hits: 40,
            cache_misses: 10,
            cache_errors: 0,
            remote_hits: 5,
            remote_misses: 35,
            remote_errors: 0,
            build_time_us: 5_000_000,
            exec_time_us: 4_000_000,
            cache_lookup_time_us: 100_000,
            cache_store_time_us: 200_000,
            max_queue_depth: 8,
        };

        let output = export_prometheus(&snapshot);

        assert!(output.contains("rninja_targets_total 100"));
        assert!(output.contains("rninja_cache_operations_total{result=\"hit\"} 40"));
        assert!(output.contains("rninja_cache_hit_rate 0.8"));
        assert!(output.contains("rninja_build_duration_seconds 5.0"));
    }
}
