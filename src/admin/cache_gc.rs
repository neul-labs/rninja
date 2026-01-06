//! Cache garbage collection tool

use crate::cache::{CacheConfig, CacheEntry};
use crate::error::ExecError;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Garbage collection options
#[derive(Debug, Clone, Default)]
pub struct GcOptions {
    /// Dry run (don't actually delete)
    pub dry_run: bool,
    /// Maximum age in days (None = no age limit)
    pub max_age_days: Option<u64>,
    /// Maximum size in bytes (None = no size limit)
    pub max_size_bytes: Option<u64>,
    /// Remove orphaned blobs (not referenced by any entry)
    pub remove_orphans: bool,
}

/// Garbage collection report
#[derive(Debug, Default, Serialize)]
pub struct GcReport {
    /// Number of expired entries removed
    pub expired_entries: u64,
    /// Bytes from expired entries
    pub expired_bytes: u64,
    /// Number of orphaned blobs removed
    pub orphan_blobs: u64,
    /// Bytes from orphaned blobs
    pub orphan_bytes: u64,
    /// Number of entries evicted for size
    pub evicted_entries: u64,
    /// Bytes evicted for size
    pub evicted_bytes: u64,
    /// Whether this was a dry run
    pub dry_run: bool,
}

/// Run cache garbage collection
pub fn run_cache_gc(options: GcOptions, verbose: bool) -> Result<GcReport, ExecError> {
    let config = CacheConfig::from_env();

    if !config.cache_dir.exists() {
        eprintln!("Cache directory does not exist: {}", config.cache_dir.display());
        return Ok(GcReport::default());
    }

    let db_path = config.cache_dir.join("index");
    let db = sled::open(&db_path).map_err(|e| {
        ExecError::SpawnError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to open cache db: {}", e),
        ))
    })?;

    let mut report = GcReport {
        dry_run: options.dry_run,
        ..Default::default()
    };

    let now = SystemTime::now();

    // Collect all referenced blob hashes
    let mut referenced_hashes: HashSet<String> = HashSet::new();
    let mut entries_to_remove: Vec<Vec<u8>> = Vec::new();

    // Phase 1: Find expired entries
    if let Some(max_age_days) = options.max_age_days {
        let max_age = Duration::from_secs(max_age_days * 86400);

        for item in db.iter() {
            if let Ok((key, value)) = item {
                match CacheEntry::deserialize(&value) {
                    Ok(entry) => {
                        let age = now.duration_since(entry.created).unwrap_or_default();

                        if age > max_age {
                            report.expired_entries += 1;
                            report.expired_bytes += value.len() as u64;
                            entries_to_remove.push(key.to_vec());
                            if verbose {
                                debug!(
                                    "Expired entry: {} (age: {} days)",
                                    String::from_utf8_lossy(&key),
                                    age.as_secs() / 86400
                                );
                            }
                        } else {
                            // Track referenced hashes
                            for (_, hash) in &entry.outputs {
                                referenced_hashes.insert(hash.clone());
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cache entry: {}", e);
                    }
                }
            }
        }
    } else {
        // Just collect all referenced hashes
        for item in db.iter() {
            if let Ok((_, value)) = item {
                match CacheEntry::deserialize(&value) {
                    Ok(entry) => {
                        for (_, hash) in &entry.outputs {
                            referenced_hashes.insert(hash.clone());
                        }
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cache entry: {}", e);
                    }
                }
            }
        }
    }

    // Remove expired entries
    if !options.dry_run {
        for key in &entries_to_remove {
            if let Err(e) = db.remove(key) {
                eprintln!("Failed to remove entry: {}", e);
            }
        }
    }

    // Phase 2: Find orphaned blobs
    if options.remove_orphans {
        let blobs_dir = config.cache_dir.join("blobs");
        if blobs_dir.exists() {
            let orphans = find_orphan_blobs(&blobs_dir, &referenced_hashes);

            for (path, size) in &orphans {
                report.orphan_blobs += 1;
                report.orphan_bytes += size;

                if !options.dry_run {
                    if let Err(e) = std::fs::remove_file(path) {
                        eprintln!("Failed to remove blob {:?}: {}", path, e);
                    } else if verbose {
                        debug!("Removed orphan blob: {:?}", path);
                    }
                }
            }
        }
    }

    // Phase 3: LRU eviction if over max size
    if let Some(max_size) = options.max_size_bytes {
        let current_size = calculate_cache_size(&config.cache_dir);
        if current_size > max_size {
            let to_free = current_size - max_size;
            let evicted = evict_lru(&db, to_free, options.dry_run)?;
            report.evicted_entries = evicted.0;
            report.evicted_bytes = evicted.1;
        }
    }

    // Flush database
    if !options.dry_run {
        let _ = db.flush();
    }

    // Print summary
    let action = if options.dry_run { "Would remove" } else { "Removed" };
    info!(
        "GC complete: {} {} expired entries ({} bytes), {} orphan blobs ({} bytes), {} evicted ({} bytes)",
        action,
        report.expired_entries,
        report.expired_bytes,
        report.orphan_blobs,
        report.orphan_bytes,
        report.evicted_entries,
        report.evicted_bytes
    );

    Ok(report)
}

fn find_orphan_blobs(dir: &std::path::Path, referenced: &HashSet<String>) -> Vec<(PathBuf, u64)> {
    let mut orphans = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                orphans.extend(find_orphan_blobs(&path, referenced));
            } else if path.is_file() {
                if let Some(hash) = path.file_name().and_then(|n| n.to_str()) {
                    if !referenced.contains(hash) {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        orphans.push((path, size));
                    }
                }
            }
        }
    }

    orphans
}

fn calculate_cache_size(cache_dir: &std::path::Path) -> u64 {
    let mut size = 0u64;

    let db_path = cache_dir.join("index");
    if db_path.exists() {
        if let Ok(db) = sled::open(&db_path) {
            size += db.size_on_disk().unwrap_or(0);
        }
    }

    let blobs_dir = cache_dir.join("blobs");
    if blobs_dir.exists() {
        size += dir_size(&blobs_dir);
    }

    size
}

fn dir_size(dir: &std::path::Path) -> u64 {
    let mut size = 0u64;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path);
            } else if path.is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }

    size
}

fn evict_lru(db: &sled::Db, bytes_to_free: u64, dry_run: bool) -> Result<(u64, u64), ExecError> {
    // Collect entries with their creation time
    let mut entries: Vec<(Vec<u8>, SystemTime, u64)> = Vec::new();

    for item in db.iter() {
        if let Ok((key, value)) = item {
            match CacheEntry::deserialize(&value) {
                Ok(entry) => {
                    entries.push((key.to_vec(), entry.created, value.len() as u64));
                }
                Err(e) => {
                    warn!("Failed to deserialize cache entry during eviction: {}", e);
                }
            }
        }
    }

    // Sort by creation time (oldest first)
    entries.sort_by_key(|(_, created, _)| *created);

    let mut freed = 0u64;
    let mut count = 0u64;

    for (key, _, size) in entries {
        if freed >= bytes_to_free {
            break;
        }

        if !dry_run {
            if let Err(e) = db.remove(&key) {
                eprintln!("Failed to evict entry: {}", e);
                continue;
            }
        }

        freed += size;
        count += 1;
    }

    Ok((count, freed))
}
