mod depfile;
mod runner;

use crate::cache::{Cache, CacheConfig, CacheEntry};
use crate::error::ExecError;
use crate::graph::{Graph, Node};
use crate::output::{JsonEvent, OutputMode};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Executor configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of parallel jobs
    pub parallelism: usize,
    /// Dry run mode (don't execute commands)
    pub dry_run: bool,
    /// Verbose mode (print all commands)
    pub verbose: bool,
    /// Keep going after failures (0 = infinite)
    pub keep_going: usize,
    /// Explain why targets are rebuilt
    pub explain: bool,
    /// Show stats at end
    pub stats: bool,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Output mode (human or JSON)
    pub output_mode: OutputMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parallelism: num_cpus::get(),
            dry_run: false,
            verbose: false,
            keep_going: 1,
            explain: false,
            stats: false,
            cache_config: CacheConfig::from_env(),
            output_mode: OutputMode::Human,
        }
    }
}

/// Build statistics
#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub started: usize,
    pub finished: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub total_time: Duration,
}

impl Stats {
    pub fn print(&self, output_mode: OutputMode) {
        match output_mode {
            OutputMode::Human => {
                eprintln!();
                eprintln!("build statistics:");
                eprintln!("    started: {} edges", self.started);
                eprintln!("   finished: {} edges", self.finished);
                eprintln!("     failed: {} edges", self.failed);
                eprintln!("    skipped: {} edges", self.skipped);
                if self.cache_hits > 0 || self.cache_misses > 0 {
                    let total = self.cache_hits + self.cache_misses;
                    let hit_rate = if total > 0 {
                        100.0 * self.cache_hits as f64 / total as f64
                    } else {
                        0.0
                    };
                    eprintln!("  cache hit: {} ({:.1}%)", self.cache_hits, hit_rate);
                    eprintln!(" cache miss: {}", self.cache_misses);
                }
                eprintln!("       time: {:.3}s", self.total_time.as_secs_f64());
            }
            OutputMode::Json => {
                // JSON stats are included in BuildFinished event
            }
        }
    }
}

/// Shared state for parallel execution
struct BuildState {
    /// Nodes that have completed successfully
    completed: Mutex<HashSet<String>>,
    /// Nodes that have failed
    failed_nodes: Mutex<HashSet<String>>,
    /// Number of failures
    fail_count: AtomicUsize,
    /// Counter for progress display
    progress: AtomicUsize,
    /// Total number of nodes to build
    total: usize,
    /// Pool semaphores for limiting parallelism
    pools: HashMap<String, Arc<Semaphore>>,
    /// Default semaphore for job limiting
    job_semaphore: Arc<Semaphore>,
    /// Console pool (depth=1 by default)
    console_semaphore: Arc<Semaphore>,
    /// Cache hits counter
    cache_hits: AtomicUsize,
    /// Cache misses counter
    cache_misses: AtomicUsize,
    /// Output mode for progress reporting
    output_mode: OutputMode,
}

impl BuildState {
    fn new(parallelism: usize, total: usize, pools: &HashMap<String, usize>, output_mode: OutputMode) -> Self {
        let mut pool_sems = HashMap::new();
        for (name, depth) in pools {
            pool_sems.insert(name.clone(), Arc::new(Semaphore::new(*depth)));
        }

        Self {
            completed: Mutex::new(HashSet::new()),
            failed_nodes: Mutex::new(HashSet::new()),
            fail_count: AtomicUsize::new(0),
            progress: AtomicUsize::new(0),
            total,
            pools: pool_sems,
            job_semaphore: Arc::new(Semaphore::new(parallelism)),
            console_semaphore: Arc::new(Semaphore::new(1)),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
            output_mode,
        }
    }

    fn mark_completed(&self, path: &str) {
        self.completed.lock().insert(path.to_string());
    }

    fn mark_failed(&self, path: &str) {
        self.failed_nodes.lock().insert(path.to_string());
        self.fail_count.fetch_add(1, Ordering::SeqCst);
    }

    fn has_failed_dep(&self, deps: &[String]) -> bool {
        let failed = self.failed_nodes.lock();
        deps.iter().any(|d| failed.contains(d))
    }

    fn deps_ready(&self, deps: &[String], graph: &Graph) -> bool {
        let completed = self.completed.lock();
        deps.iter().all(|d| {
            completed.contains(d) || graph.get_node(d).map(|n| n.is_source).unwrap_or(true)
        })
    }

    fn next_progress(&self) -> usize {
        self.progress.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn get_pool_semaphore(&self, pool: Option<&str>) -> Arc<Semaphore> {
        match pool {
            Some("console") => self.console_semaphore.clone(),
            Some(name) => self.pools.get(name).cloned().unwrap_or(self.job_semaphore.clone()),
            None => self.job_semaphore.clone(),
        }
    }

    fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::SeqCst);
    }

    fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::SeqCst);
    }
}

/// The executor runs build commands respecting the dependency graph
pub struct Executor {
    config: Config,
    cache: Option<Cache>,
}

impl Executor {
    pub fn new(config: Config) -> Self {
        // Try to open the cache
        let cache = if config.cache_config.enabled {
            match Cache::open(config.cache_config.clone()) {
                Ok(c) => {
                    info!("Cache enabled at {}", c.cache_dir().display());
                    Some(c)
                }
                Err(e) => {
                    eprintln!("Warning: failed to open cache: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self { config, cache }
    }

    /// Run the build for given targets
    pub fn run(&self, graph: &Graph, targets: &[&str]) -> Result<Stats, ExecError> {
        // Use tokio runtime for async execution
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.parallelism.min(num_cpus::get()))
            .enable_all()
            .build()
            .map_err(|e| ExecError::SpawnError(e.into()))?;

        rt.block_on(self.run_async(graph, targets))
    }

    async fn run_async(&self, graph: &Graph, targets: &[&str]) -> Result<Stats, ExecError> {
        let start = Instant::now();

        // Get all nodes in topological order
        let topo_order = graph
            .topo_order(targets)
            .map_err(|_| ExecError::SubcommandFailed)?;

        // Filter to nodes that need rebuilding
        let mut needs_build: HashSet<String> = HashSet::new();

        // Mark nodes that need building (reverse order to propagate dirty state)
        for node in topo_order.iter().rev() {
            let needs_it =
                node.needs_rebuild() || node.deps.iter().any(|d| needs_build.contains(d));

            if needs_it && !node.is_source {
                needs_build.insert(node.path.clone());
                if self.config.explain && node.needs_rebuild() {
                    self.explain_rebuild(node);
                }
            }
        }

        // Collect nodes to build in order
        let work: Vec<&Node> = topo_order
            .iter()
            .filter(|n| needs_build.contains(&n.path))
            .copied()
            .collect();

        let total = work.len();
        if total == 0 {
            match self.config.output_mode {
                OutputMode::Human => println!("ninja: no work to do."),
                OutputMode::Json => JsonEvent::NoWorkToDo.emit(),
            }
            return Ok(Stats {
                total_time: start.elapsed(),
                ..Default::default()
            });
        }

        // Collect pool depths from graph
        let pools = graph.pool_depths();

        // Emit build started event for JSON mode
        if self.config.output_mode == OutputMode::Json {
            JsonEvent::BuildStarted {
                total_targets: total,
                parallelism: self.config.parallelism,
            }.emit();
        }

        // Create shared state
        let state = Arc::new(BuildState::new(self.config.parallelism, total, &pools, self.config.output_mode));

        // Execute nodes respecting dependencies
        let mut handles = Vec::new();
        let mut pending: Vec<&Node> = work.clone();

        while !pending.is_empty() || !handles.is_empty() {
            // Check if we should stop due to failures
            let fail_count = state.fail_count.load(Ordering::SeqCst);
            if self.config.keep_going > 0 && fail_count >= self.config.keep_going {
                // Cancel pending work
                break;
            }

            // Find nodes that are ready to execute
            let mut ready = Vec::new();
            let mut still_pending = Vec::new();

            for node in pending {
                if state.has_failed_dep(&node.deps) {
                    // Skip nodes with failed dependencies
                    state.mark_failed(&node.path);
                    continue;
                }

                if state.deps_ready(&node.deps, graph) {
                    ready.push(node);
                } else {
                    still_pending.push(node);
                }
            }

            pending = still_pending;
            let spawned_any = !ready.is_empty();

            // Spawn tasks for ready nodes
            for node in ready {
                let state = state.clone();
                let config = self.config.clone();
                let path = node.path.clone();
                let command = node.command.clone();
                let description = node.description.clone();
                let is_phony = node.is_phony;
                let depfile = node.depfile.clone();
                let rspfile = node.rspfile.clone();
                let rspfile_content = node.rspfile_content.clone();
                let pool = node.pool.clone();
                let deps = node.deps.clone();

                // Get cache reference if available
                let cache_key = if let (Some(cache), Some(cmd)) = (&self.cache, &command) {
                    if !is_phony {
                        let input_paths: Vec<_> = deps.iter().map(|d| Path::new(d.as_str())).collect();
                        cache.action_key(cmd, &input_paths, &[]).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Check cache before spawning
                let cached_entry = if let (Some(cache), Some(key)) = (&self.cache, &cache_key) {
                    cache.lookup(key)
                } else {
                    None
                };

                let handle = tokio::spawn(async move {
                    // Acquire semaphore permit
                    let sem = state.get_pool_semaphore(pool.as_deref());
                    let _permit = sem.acquire().await.unwrap();

                    let result = execute_node_async(
                        &path,
                        command.as_deref(),
                        description.as_deref(),
                        is_phony,
                        &depfile,
                        rspfile.as_deref(),
                        rspfile_content.as_deref(),
                        state.next_progress(),
                        state.total,
                        &config,
                        cached_entry.as_ref(),
                        &state,
                    )
                    .await;

                    match result {
                        Ok(executed) => {
                            state.mark_completed(&path);
                            Ok((path, executed, cache_key))
                        }
                        Err(e) => {
                            state.mark_failed(&path);
                            Err((path, e))
                        }
                    }
                });

                handles.push(handle);
            }

            // If nothing is ready and we have pending handles, wait for some to complete
            if pending.is_empty() && handles.is_empty() {
                break;
            }

            if !handles.is_empty() && (pending.is_empty() || !spawned_any) {
                // Wait for at least one task to complete
                let (result, _idx, remaining) = futures::future::select_all(handles).await;
                handles = remaining;

                match result {
                    Ok(Ok((path, executed, cache_key))) => {
                        // Store in cache if executed (not from cache)
                        if executed {
                            if let (Some(cache), Some(key)) = (&self.cache, cache_key) {
                                let outputs = vec![Path::new(&path)];
                                if let Err(e) = cache.store(&key, &outputs, "") {
                                    debug!("Failed to cache {}: {}", path, e);
                                }
                            }
                        }
                    }
                    Ok(Err((path, e))) => {
                        eprintln!("FAILED: {}", path);
                        if let ExecError::CommandFailed { command, code } = &e {
                            eprintln!("Command: {}", command);
                            eprintln!("Exit code: {}", code);
                        }
                    }
                    Err(e) => {
                        eprintln!("Task error: {}", e);
                    }
                }
            }
        }

        // Wait for remaining handles
        for handle in handles {
            match handle.await {
                Ok(Ok((path, executed, cache_key))) => {
                    if executed {
                        if let (Some(cache), Some(key)) = (&self.cache, cache_key) {
                            let outputs = vec![Path::new(&path)];
                            if let Err(e) = cache.store(&key, &outputs, "") {
                                debug!("Failed to cache {}: {}", path, e);
                            }
                        }
                    }
                }
                Ok(Err((path, e))) => {
                    eprintln!("FAILED: {}", path);
                    if let ExecError::CommandFailed { command, code } = &e {
                        eprintln!("Command: {}", command);
                        eprintln!("Exit code: {}", code);
                    }
                }
                Err(e) => {
                    eprintln!("Task error: {}", e);
                }
            }
        }

        let fail_count = state.fail_count.load(Ordering::SeqCst);
        let finish_count = state.progress.load(Ordering::SeqCst);
        let cache_hits = state.cache_hits.load(Ordering::SeqCst);
        let cache_misses = state.cache_misses.load(Ordering::SeqCst);

        let stats = Stats {
            started: finish_count,
            finished: finish_count.saturating_sub(fail_count),
            failed: fail_count,
            skipped: total.saturating_sub(finish_count),
            cache_hits,
            cache_misses,
            total_time: start.elapsed(),
        };

        if self.config.stats {
            stats.print(self.config.output_mode);
        }

        // Emit build finished event for JSON mode
        if self.config.output_mode == OutputMode::Json {
            JsonEvent::BuildFinished {
                success: fail_count == 0,
                targets_built: finish_count,
                targets_total: total,
                duration_ms: stats.total_time.as_millis() as u64,
                cache_hits: if cache_hits > 0 { Some(cache_hits) } else { None },
                cache_misses: if cache_misses > 0 { Some(cache_misses) } else { None },
            }.emit();
        }

        if fail_count > 0 {
            Err(ExecError::BuildFailed(fail_count))
        } else {
            Ok(stats)
        }
    }

    fn explain_rebuild(&self, node: &Node) {
        if !std::path::Path::new(&node.path).exists() {
            println!("ninja explain: {} is missing", node.path);
            return;
        }

        let output_mtime = std::path::Path::new(&node.path)
            .metadata()
            .and_then(|m| m.modified())
            .ok();

        for dep in &node.deps {
            let dep_mtime = std::path::Path::new(dep)
                .metadata()
                .and_then(|m| m.modified())
                .ok();

            if let (Some(out_t), Some(dep_t)) = (output_mtime, dep_mtime) {
                if dep_t > out_t {
                    println!("ninja explain: {} is newer than {}", dep, node.path);
                    return;
                }
            }
        }
    }
}

/// Execute a single node asynchronously
/// Returns Ok(true) if command was executed, Ok(false) if restored from cache
async fn execute_node_async(
    path: &str,
    command: Option<&str>,
    description: Option<&str>,
    is_phony: bool,
    depfile: &str,
    rspfile: Option<&str>,
    rspfile_content: Option<&str>,
    idx: usize,
    total: usize,
    config: &Config,
    cached: Option<&CacheEntry>,
    state: &BuildState,
) -> Result<bool, ExecError> {
    // Skip phony targets
    if is_phony {
        return Ok(false);
    }

    let command = match command {
        Some(cmd) => cmd,
        None => return Ok(false),
    };

    // Check if we have a cached result
    if let Some(_entry) = cached {
        // Restore from cache
        match state.output_mode {
            OutputMode::Human => {
                let desc = description.unwrap_or("cached");
                println!("[{}/{}] {} (cached)", idx, total, desc);
                std::io::stdout().flush().ok();
            }
            OutputMode::Json => {
                JsonEvent::CacheHit {
                    target: path,
                    index: idx,
                    total,
                }.emit();
            }
        }

        // For now, we trust the cache - the file should already exist from the blob restore
        // In a full implementation, we'd restore the blobs here
        state.record_cache_hit();
        return Ok(false);
    }

    state.record_cache_miss();

    // Print status
    match state.output_mode {
        OutputMode::Human => {
            let desc = description.unwrap_or(command);
            if config.verbose {
                println!("[{}/{}] {}", idx, total, command);
            } else {
                println!("[{}/{}] {}", idx, total, desc);
            }
            std::io::stdout().flush().ok();
        }
        OutputMode::Json => {
            JsonEvent::TargetStarted {
                target: path,
                index: idx,
                total,
                command: if config.verbose { Some(command) } else { None },
            }.emit();
        }
    }

    // Dry run - don't actually execute
    if config.dry_run {
        return Ok(false);
    }

    // Write response file if needed
    if let (Some(rsp), Some(content)) = (rspfile, rspfile_content) {
        tokio::fs::write(rsp, content).await?;
    }

    // Execute the command
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .await?;

    // Print output if any
    if !output.stdout.is_empty() {
        std::io::stdout().write_all(&output.stdout).ok();
    }
    if !output.stderr.is_empty() {
        std::io::stderr().write_all(&output.stderr).ok();
    }

    // Parse depfile if present
    if !depfile.is_empty() {
        if let Ok(deps) = depfile::parse(depfile) {
            debug!("Parsed depfile {}: {:?}", depfile, deps);
        }
    }

    // Clean up response file
    if let Some(rsp) = rspfile {
        let _ = tokio::fs::remove_file(rsp).await;
    }

    if output.status.success() {
        // Emit success event for JSON mode
        if state.output_mode == OutputMode::Json {
            JsonEvent::TargetFinished {
                target: path,
                index: idx,
                total,
                success: true,
                error: None,
            }.emit();
        }
        Ok(true) // Command was executed
    } else {
        let code = output.status.code().unwrap_or(-1);
        // Emit failure event for JSON mode
        if state.output_mode == OutputMode::Json {
            let error_msg = format!("exit code {}", code);
            JsonEvent::TargetFinished {
                target: path,
                index: idx,
                total,
                success: false,
                error: Some(&error_msg),
            }.emit();
        }
        Err(ExecError::CommandFailed {
            command: command.to_string(),
            code,
        })
    }
}
