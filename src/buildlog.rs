//! Build log for tracking output mtimes and enabling fast no-op detection
//!
//! This implements ninja's .ninja_log format for compatibility.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tracing::warn;

/// Entry in the build log
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Start time of the build (ms since epoch, for ninja compat)
    pub start_time_ms: u64,
    /// End time of the build
    pub end_time_ms: u64,
    /// Mtime of the output after build (ns since epoch)
    pub mtime_ns: u64,
    /// Output path
    pub output: String,
    /// Hash of the command (for restat)
    pub command_hash: u64,
}

/// Build log for fast up-to-date checking
pub struct BuildLog {
    path: PathBuf,
    entries: HashMap<String, LogEntry>,
    dirty: AtomicBool,
}

impl BuildLog {
    /// Open or create a build log
    pub fn open(dir: &Path) -> Self {
        let path = dir.join(".ninja_log");
        let mut log = Self {
            path,
            entries: HashMap::new(),
            dirty: AtomicBool::new(false),
        };
        log.load();
        log
    }

    /// Load entries from disk
    fn load(&mut self) {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Check version header
        if let Some(Ok(header)) = lines.next() {
            if !header.starts_with("# ninja log v") {
                return; // Incompatible version
            }
        }

        for line in lines {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            // Format: start_time end_time mtime_ns output command_hash
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 5 {
                if let (Ok(start), Ok(end), Ok(mtime), Ok(hash)) = (
                    parts[0].parse::<u64>(),
                    parts[1].parse::<u64>(),
                    parts[2].parse::<u64>(),
                    parts[4].parse::<u64>(),
                ) {
                    let output = parts[3].to_string();
                    self.entries.insert(
                        output.clone(),
                        LogEntry {
                            start_time_ms: start,
                            end_time_ms: end,
                            mtime_ns: mtime,
                            output,
                            command_hash: hash,
                        },
                    );
                }
            }
        }
    }

    /// Record a build
    pub fn record(&mut self, output: &str, command_hash: u64, start_ms: u64, end_ms: u64) {
        let mtime_ns = get_mtime_ns(Path::new(output)).unwrap_or(0);

        self.entries.insert(
            output.to_string(),
            LogEntry {
                start_time_ms: start_ms,
                end_time_ms: end_ms,
                mtime_ns,
                output: output.to_string(),
                command_hash,
            },
        );
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Get recorded mtime for an output
    pub fn get_mtime(&self, output: &str) -> Option<u64> {
        self.entries.get(output).map(|e| e.mtime_ns)
    }

    /// Get command hash for an output
    pub fn get_command_hash(&self, output: &str) -> Option<u64> {
        self.entries.get(output).map(|e| e.command_hash)
    }

    /// Save log to disk
    pub fn save(&self) -> std::io::Result<()> {
        if !self.dirty.load(Ordering::Relaxed) && self.path.exists() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;

        writeln!(file, "# ninja log v5")?;

        for entry in self.entries.values() {
            writeln!(
                file,
                "{}\t{}\t{}\t{}\t{}",
                entry.start_time_ms,
                entry.end_time_ms,
                entry.mtime_ns,
                entry.output,
                entry.command_hash
            )?;
        }

        self.dirty.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Get all output names in the log
    pub fn entries(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Check if output needs rebuild based on log
    pub fn needs_rebuild(&self, output: &str, command_hash: u64) -> bool {
        match self.entries.get(output) {
            Some(entry) => {
                // Command changed?
                if entry.command_hash != command_hash {
                    return true;
                }

                // Check if file mtime matches log
                let current_mtime = get_mtime_ns(Path::new(output)).unwrap_or(0);
                if current_mtime == 0 {
                    return true; // File doesn't exist
                }

                // If mtime changed, needs rebuild
                current_mtime != entry.mtime_ns
            }
            None => true, // Not in log, needs build
        }
    }
}

impl Drop for BuildLog {
    fn drop(&mut self) {
        let _ = self.save();
    }
}

/// Mtime cache for avoiding repeated stat() calls
pub struct MtimeCache {
    cache: HashMap<PathBuf, Option<u64>>,
}

impl MtimeCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(1024),
        }
    }

    /// Get mtime in nanoseconds, cached
    pub fn get(&mut self, path: &Path) -> Option<u64> {
        if let Some(&cached) = self.cache.get(path) {
            return cached;
        }

        let mtime = get_mtime_ns(path);
        self.cache.insert(path.to_path_buf(), mtime);
        mtime
    }

    /// Invalidate cache entry (after build)
    pub fn invalidate(&mut self, path: &Path) {
        self.cache.remove(path);
    }

    /// Pre-populate cache with multiple paths (sequential)
    pub fn prefetch(&mut self, paths: &[&Path]) {
        for path in paths {
            if !self.cache.contains_key(*path) {
                let mtime = get_mtime_ns(path);
                self.cache.insert(path.to_path_buf(), mtime);
            }
        }
    }

    /// Pre-populate cache with multiple paths in parallel using rayon
    pub fn prefetch_parallel(&mut self, paths: &[PathBuf]) {
        use rayon::prelude::*;

        // Filter paths we haven't cached yet
        let uncached: Vec<&PathBuf> = paths
            .iter()
            .filter(|p| !self.cache.contains_key(*p))
            .collect();

        if uncached.is_empty() {
            return;
        }

        // Stat files in parallel
        let results: Vec<(PathBuf, Option<u64>)> = uncached
            .par_iter()
            .map(|p| ((*p).clone(), get_mtime_ns(p)))
            .collect();

        // Insert results into cache
        for (path, mtime) in results {
            self.cache.insert(path, mtime);
        }
    }

    /// Insert a known mtime (used for batch operations)
    pub fn insert(&mut self, path: PathBuf, mtime: Option<u64>) {
        self.cache.insert(path, mtime);
    }
}

impl Default for MtimeCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Get file mtime in nanoseconds since epoch
pub fn get_mtime_ns(path: &Path) -> Option<u64> {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to get metadata for {}: {}", path.display(), e);
            return None;
        }
    };
    let mtime = match metadata.modified() {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to get mtime for {}: {}", path.display(), e);
            return None;
        }
    };
    let duration = match mtime.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d,
        Err(e) => {
            warn!("Failed to compute duration for {}: {}", path.display(), e);
            return None;
        }
    };
    Some(duration.as_nanos() as u64)
}

/// Hash a command string for log storage
pub fn hash_command(cmd: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    cmd.hash(&mut hasher);
    hasher.finish()
}

/// Fast check if all targets are up-to-date (no tokio needed)
/// Uses parallel stat() for large builds, sequential for small ones.
pub fn quick_uptodate_check(
    graph: &crate::graph::Graph,
    targets: &[&str],
    log: &BuildLog,
    mtime_cache: &mut MtimeCache,
) -> bool {
    // Get all nodes we'd need to build
    let order = match graph.topo_order(targets) {
        Ok(o) => o,
        Err(_) => return false,
    };

    // Collect nodes to check (non-source, non-phony)
    let mut nodes_to_check: Vec<(&str, &crate::graph::Node)> = Vec::with_capacity(order.len());
    let mut total_paths = 0;

    for node in &order {
        if !node.is_source && !node.is_phony {
            nodes_to_check.push((node.path.as_str(), node));
            total_paths += 1 + node.deps.len();
        }
    }

    // Use parallel stat() for large builds (> 2000 paths), sequential otherwise
    if total_paths > 2000 {
        quick_uptodate_check_parallel(nodes_to_check, log)
    } else {
        quick_uptodate_check_sequential(nodes_to_check, log, mtime_cache)
    }
}

/// Sequential up-to-date check for smaller builds
#[inline]
fn quick_uptodate_check_sequential(
    nodes_to_check: Vec<(&str, &crate::graph::Node)>,
    log: &BuildLog,
    mtime_cache: &mut MtimeCache,
) -> bool {
    for (output, node) in nodes_to_check {
        // Check output exists
        let output_path = Path::new(output);
        let output_mtime = match mtime_cache.get(output_path) {
            Some(m) => m,
            None => return false, // Output doesn't exist
        };

        // Check command hash hasn't changed (uses pre-computed hash)
        if node.command.is_some() {
            if let Some(logged_hash) = log.get_command_hash(output) {
                if logged_hash != node.command_hash {
                    return false; // Command changed
                }
            }
        }

        // Check all inputs are older than output
        for dep in &node.deps {
            let dep_path = Path::new(dep.as_str());
            if let Some(dep_mtime) = mtime_cache.get(dep_path) {
                if dep_mtime > output_mtime {
                    return false; // Input is newer than output
                }
            }
        }
    }

    true
}

/// Parallel up-to-date check for large builds
fn quick_uptodate_check_parallel(
    nodes_to_check: Vec<(&str, &crate::graph::Node)>,
    log: &BuildLog,
) -> bool {
    use rayon::prelude::*;

    // Collect all unique paths we need to stat
    let mut all_paths: Vec<&str> = Vec::with_capacity(nodes_to_check.len() * 3);
    for (output, node) in &nodes_to_check {
        all_paths.push(*output);
        for dep in &node.deps {
            all_paths.push(dep.as_str());
        }
    }

    // Parallel stat() all paths at once
    let mtimes: HashMap<&str, Option<u64>> = all_paths
        .par_iter()
        .map(|&path| (path, get_mtime_ns(Path::new(path))))
        .collect();

    // Now check each node using cached mtimes
    for (output, node) in nodes_to_check {
        // Check output exists
        let output_mtime = match mtimes.get(output) {
            Some(Some(m)) => *m,
            _ => return false,
        };

        // Check command hash hasn't changed
        if node.command.is_some() {
            if let Some(logged_hash) = log.get_command_hash(output) {
                if logged_hash != node.command_hash {
                    return false;
                }
            }
        }

        // Check all inputs are older than output
        for dep in &node.deps {
            if let Some(Some(dep_mtime)) = mtimes.get(dep.as_str()) {
                if *dep_mtime > output_mtime {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_mtime_cache() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, "content").unwrap();

        let mut cache = MtimeCache::new();

        let mtime1 = cache.get(&file);
        assert!(mtime1.is_some());

        let mtime2 = cache.get(&file);
        assert_eq!(mtime1, mtime2);
    }

    #[test]
    fn test_build_log_roundtrip() {
        let dir = tempdir().unwrap();

        {
            let mut log = BuildLog::open(dir.path());
            log.record("output.o", 12345, 1000, 2000);
            log.save().unwrap();
        }

        {
            let log = BuildLog::open(dir.path());
            assert_eq!(log.get_command_hash("output.o"), Some(12345));
        }
    }

    #[test]
    fn test_hash_command() {
        let h1 = hash_command("gcc -c foo.c -o foo.o");
        let h2 = hash_command("gcc -c foo.c -o foo.o");
        let h3 = hash_command("gcc -c bar.c -o bar.o");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
