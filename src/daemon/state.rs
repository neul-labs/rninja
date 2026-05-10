//! Daemon shared state management
//!
//! Manages cached manifests, graphs, and build logs across multiple builds.

use crate::buildlog::BuildLog;
use crate::graph::Graph;
use crate::parser::Manifest;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// Shared state for the daemon
pub struct DaemonState {
    /// Cached manifests and graphs per build directory
    manifests: RwLock<HashMap<PathBuf, CachedManifest>>,

    /// Daemon start time
    start_time: Instant,

    /// Configuration
    config: DaemonConfig,
}

/// Configuration for the daemon
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Maximum number of cached manifests
    pub max_cached_manifests: usize,

    /// How long to keep manifests before revalidating (seconds)
    pub manifest_ttl_secs: u64,

    /// Maximum concurrent builds
    pub max_concurrent_builds: usize,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            max_cached_manifests: 100,
            manifest_ttl_secs: 300, // 5 minutes
            max_concurrent_builds: 4,
        }
    }
}

/// Cached manifest with associated state
pub struct CachedManifest {
    /// The parsed manifest
    pub manifest: Arc<Manifest>,

    /// The dependency graph
    pub graph: Arc<Graph>,

    /// Build log for this directory
    pub build_log: Arc<RwLock<BuildLog>>,

    /// Hash of the build.ninja file for invalidation
    pub fingerprint: u64,

    /// When this was last validated
    pub last_validated: Instant,

    /// Files included by this manifest (for watching)
    pub included_files: Vec<PathBuf>,
}

impl DaemonState {
    /// Create a new daemon state
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            manifests: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
            config,
        }
    }

    /// Get daemon uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get the number of cached manifests
    pub fn cached_manifest_count(&self) -> usize {
        self.manifests.read().len()
    }

    /// Get or parse a manifest for a build directory
    pub fn get_or_parse_manifest(
        &self,
        build_dir: &PathBuf,
        build_file: &str,
    ) -> anyhow::Result<Arc<CachedManifest>> {
        let build_path = build_dir.join(build_file);

        // Compute fingerprint first (before acquiring locks)
        let current_fingerprint = compute_fingerprint(&build_path)?;

        // Use write lock to avoid TOCTOU race: we need to check and potentially
        // update the cache atomically
        let mut manifests = self.manifests.write();

        if let Some(cached) = manifests.get(build_dir) {
            // Re-check fingerprint under write lock to avoid race
            if cached.fingerprint == current_fingerprint {
                debug!("Using cached manifest for {}", build_dir.display());
                return Ok(Arc::new(CachedManifest {
                    manifest: cached.manifest.clone(),
                    graph: cached.graph.clone(),
                    build_log: cached.build_log.clone(),
                    fingerprint: cached.fingerprint,
                    last_validated: Instant::now(),
                    included_files: cached.included_files.clone(),
                }));
            }
            debug!("Manifest changed for {}, reparsing", build_dir.display());
        }

        // Parse the manifest
        info!("Parsing manifest: {}", build_path.display());
        let manifest = crate::parser::parse_file(&build_path)?;
        let graph = Graph::from_manifest(&manifest)?;
        let fingerprint = compute_fingerprint(&build_path)?;

        // Collect included files
        let mut included_files = vec![build_path.clone()];
        for include in &manifest.includes {
            included_files.push(build_dir.join(include));
        }

        // Open build log
        let build_log = BuildLog::open(build_dir);

        let cached = CachedManifest {
            manifest: Arc::new(manifest),
            graph: Arc::new(graph),
            build_log: Arc::new(RwLock::new(build_log)),
            fingerprint,
            last_validated: Instant::now(),
            included_files,
        };

        // Evict old entries if at capacity
        if manifests.len() >= self.config.max_cached_manifests {
            // Remove least recently validated
            if let Some(oldest) = manifests
                .iter()
                .min_by_key(|(_, v)| v.last_validated)
                .map(|(k, _)| k.clone())
            {
                manifests.remove(&oldest);
            }
        }

        manifests.insert(
            build_dir.clone(),
            CachedManifest {
                manifest: cached.manifest.clone(),
                graph: cached.graph.clone(),
                build_log: cached.build_log.clone(),
                fingerprint: cached.fingerprint,
                last_validated: cached.last_validated,
                included_files: cached.included_files.clone(),
            },
        );

        Ok(Arc::new(cached))
    }

    /// Invalidate cache for a build directory
    pub fn invalidate(&self, build_dir: &PathBuf) {
        let mut manifests = self.manifests.write();
        if manifests.remove(build_dir).is_some() {
            info!("Invalidated cache for {}", build_dir.display());
        }
    }

    /// Invalidate all caches
    pub fn invalidate_all(&self) {
        let mut manifests = self.manifests.write();
        let count = manifests.len();
        manifests.clear();
        info!("Invalidated {} cached manifests", count);
    }

    /// Get all watched paths for file watching
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        let manifests = self.manifests.read();
        manifests
            .values()
            .flat_map(|m| m.included_files.iter().cloned())
            .collect()
    }

    /// Find which build directory a changed file belongs to
    pub fn find_build_dir_for_file(&self, path: &PathBuf) -> Option<PathBuf> {
        let manifests = self.manifests.read();
        for (build_dir, cached) in manifests.iter() {
            if cached.included_files.contains(path) {
                return Some(build_dir.clone());
            }
        }
        None
    }
}

/// Compute a fingerprint for a manifest file
fn compute_fingerprint(path: &PathBuf) -> anyhow::Result<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let metadata = std::fs::metadata(path)?;
    let mtime = metadata
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let size = metadata.len();

    let mut hasher = DefaultHasher::new();
    mtime.hash(&mut hasher);
    size.hash(&mut hasher);
    path.hash(&mut hasher);

    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_state_creation() {
        let state = DaemonState::new(DaemonConfig::default());
        assert_eq!(state.cached_manifest_count(), 0);
        assert!(state.uptime_secs() < 1);
    }

    #[test]
    fn test_invalidate() {
        let state = DaemonState::new(DaemonConfig::default());
        let path = PathBuf::from("/tmp/test");
        state.invalidate(&path); // Should not panic
    }
}
