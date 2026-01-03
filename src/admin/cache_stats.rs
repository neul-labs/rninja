//! Cache statistics tool

use crate::cache::{CacheConfig, CacheEntry};
use crate::error::ExecError;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Cache statistics report
#[derive(Debug, Serialize)]
pub struct CacheStatsReport {
    /// Cache directory path
    pub cache_dir: String,
    /// Number of index entries
    pub index_entries: u64,
    /// Size of index database on disk
    pub index_size_bytes: u64,
    /// Number of blobs
    pub blob_count: u64,
    /// Total size of blobs
    pub blob_size_bytes: u64,
    /// Age distribution of entries
    pub age_distribution: AgeDistribution,
}

/// Age distribution of cache entries
#[derive(Debug, Default, Serialize)]
pub struct AgeDistribution {
    pub under_1h: EntryBucket,
    pub under_1d: EntryBucket,
    pub under_1w: EntryBucket,
    pub over_1w: EntryBucket,
}

/// A bucket of entries
#[derive(Debug, Default, Serialize)]
pub struct EntryBucket {
    pub count: u64,
    pub size_bytes: u64,
}

/// Run the cache-stats tool
pub fn run_cache_stats(verbose: bool, json: bool) -> Result<(), ExecError> {
    let config = CacheConfig::from_env();

    if !config.cache_dir.exists() {
        if json {
            println!(r#"{{"error": "cache directory does not exist"}}"#);
        } else {
            eprintln!("Cache directory does not exist: {}", config.cache_dir.display());
        }
        return Ok(());
    }

    // Open sled database
    let db_path = config.cache_dir.join("index");
    let db = sled::open(&db_path).map_err(|e| {
        ExecError::SpawnError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to open cache db: {}", e),
        ))
    })?;

    // Collect statistics
    let mut report = CacheStatsReport {
        cache_dir: config.cache_dir.display().to_string(),
        index_entries: db.len() as u64,
        index_size_bytes: db.size_on_disk().unwrap_or(0),
        blob_count: 0,
        blob_size_bytes: 0,
        age_distribution: AgeDistribution::default(),
    };

    // Count blobs
    let blobs_dir = config.cache_dir.join("blobs");
    if blobs_dir.exists() {
        let (count, size) = count_blobs(&blobs_dir);
        report.blob_count = count;
        report.blob_size_bytes = size;
    }

    // Analyze age distribution
    let now = SystemTime::now();
    for item in db.iter() {
        if let Ok((_, value)) = item {
            if let Ok(entry) = CacheEntry::deserialize(&value) {
                let age = now.duration_since(entry.created).unwrap_or_default();
                let size = value.len() as u64;

                if age < Duration::from_secs(3600) {
                    report.age_distribution.under_1h.count += 1;
                    report.age_distribution.under_1h.size_bytes += size;
                } else if age < Duration::from_secs(86400) {
                    report.age_distribution.under_1d.count += 1;
                    report.age_distribution.under_1d.size_bytes += size;
                } else if age < Duration::from_secs(604800) {
                    report.age_distribution.under_1w.count += 1;
                    report.age_distribution.under_1w.size_bytes += size;
                } else {
                    report.age_distribution.over_1w.count += 1;
                    report.age_distribution.over_1w.size_bytes += size;
                }
            }
        }
    }

    // Output
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print_human_readable(&report);
    }

    Ok(())
}

fn count_blobs(dir: &std::path::Path) -> (u64, u64) {
    let mut count = 0u64;
    let mut size = 0u64;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let (c, s) = count_blobs(&path);
                count += c;
                size += s;
            } else if path.is_file() {
                count += 1;
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }

    (count, size)
}

fn print_human_readable(report: &CacheStatsReport) {
    println!("Cache Statistics:");
    println!("  Location:      {}", report.cache_dir);
    println!("  Index entries: {}", report.index_entries);
    println!("  Index size:    {}", format_bytes(report.index_size_bytes));
    println!("  Blob count:    {}", report.blob_count);
    println!("  Blob size:     {}", format_bytes(report.blob_size_bytes));
    println!();
    println!("  Age distribution:");
    println!(
        "    < 1 hour:  {} entries ({})",
        report.age_distribution.under_1h.count,
        format_bytes(report.age_distribution.under_1h.size_bytes)
    );
    println!(
        "    < 1 day:   {} entries ({})",
        report.age_distribution.under_1d.count,
        format_bytes(report.age_distribution.under_1d.size_bytes)
    );
    println!(
        "    < 1 week:  {} entries ({})",
        report.age_distribution.under_1w.count,
        format_bytes(report.age_distribution.under_1w.size_bytes)
    );
    println!(
        "    > 1 week:  {} entries ({})",
        report.age_distribution.over_1w.count,
        format_bytes(report.age_distribution.over_1w.size_bytes)
    );
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
