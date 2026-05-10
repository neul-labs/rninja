mod depfile;
mod runner;

use crate::cache::{Cache, CacheConfig};
use crate::error::ExecError;
use crate::graph::{Graph, Node};
use crate::output::{JsonEvent, OutputMode};
use crate::trace::BuildTrace;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Collect environment variables that commonly affect build outputs.
///
/// This whitelist covers the most common compiler and linker environment
/// variables. A full implementation may need to be configurable per-project.
fn collect_relevant_env_vars() -> Vec<(&'static str, String)> {
    const KEYS: [&str; 12] = [
        "CC",
        "CXX",
        "LD",
        "AR",
        "RANLIB",
        "STRIP",
        "CFLAGS",
        "CXXFLAGS",
        "LDFLAGS",
        "CPPFLAGS",
        "RUSTFLAGS",
        "PATH",
    ];
    KEYS.iter()
        .filter_map(|&k| std::env::var(k).ok().map(|v| (k, v)))
        .collect()
}

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
    /// Trace output file (None = disabled)
    pub trace_file: Option<PathBuf>,
    /// Tokio runtime handle (None means create one per run)
    pub runtime: Option<tokio::runtime::Handle>,
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
            trace_file: None,
            runtime: None,
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

/// Tracks the current state of a build node
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum NodeState {
    /// Waiting for dependencies
    Pending,
    /// Dependencies satisfied, waiting for semaphore
    Ready,
    /// Actively executing
    Running,
    /// Successfully completed
    Completed,
    /// Command failed
    Failed,
    /// Skipped because a dependency failed or build was cancelled
    Cancelled,
}

/// Error returned when an invalid state transition is attempted
#[derive(Debug)]
struct InvalidTransition {
    from: NodeState,
    to: NodeState,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid node state transition: {:?} -> {:?}",
            self.from, self.to
        )
    }
}

impl NodeState {
    /// Attempt to transition from `self` to `new`, returning `Err` if invalid.
    fn transition(self, new: NodeState) -> Result<NodeState, InvalidTransition> {
        match (self, new) {
            // From Pending
            (NodeState::Pending, NodeState::Ready)
            | (NodeState::Pending, NodeState::Running)
            | (NodeState::Pending, NodeState::Cancelled) => Ok(new),
            // From Ready
            (NodeState::Ready, NodeState::Running) | (NodeState::Ready, NodeState::Cancelled) => {
                Ok(new)
            }
            // From Running
            (NodeState::Running, NodeState::Completed)
            | (NodeState::Running, NodeState::Failed)
            | (NodeState::Running, NodeState::Cancelled) => Ok(new),
            // Same state is idempotent
            (old, new) if old == new => Ok(new),
            // All other combinations are invalid
            (from, to) => Err(InvalidTransition { from, to }),
        }
    }
}

/// Shared state for parallel execution
struct BuildState {
    /// Node states (completed, failed, etc.) - single source of truth
    node_states: Mutex<HashMap<String, NodeState>>,
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
    /// Build trace for chrome://tracing output
    trace: Arc<BuildTrace>,
}

impl BuildState {
    fn new(
        parallelism: usize,
        total: usize,
        pools: &HashMap<String, usize>,
        output_mode: OutputMode,
        trace: Arc<BuildTrace>,
    ) -> Self {
        let mut pool_sems = HashMap::new();
        for (name, depth) in pools {
            pool_sems.insert(name.clone(), Arc::new(Semaphore::new(*depth)));
        }

        Self {
            node_states: Mutex::new(HashMap::new()),
            fail_count: AtomicUsize::new(0),
            progress: AtomicUsize::new(0),
            total,
            pools: pool_sems,
            job_semaphore: Arc::new(Semaphore::new(parallelism)),
            console_semaphore: Arc::new(Semaphore::new(1)),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
            output_mode,
            trace,
        }
    }

    fn apply_transition(&self, path: &str, new: NodeState) {
        let mut states = self.node_states.lock();
        let current = states.get(path).copied().unwrap_or(NodeState::Pending);
        match current.transition(new) {
            Ok(NodeState::Failed) if current != NodeState::Failed => {
                states.insert(path.to_string(), NodeState::Failed);
                self.fail_count.fetch_add(1, Ordering::SeqCst);
            }
            Ok(final_state) => {
                states.insert(path.to_string(), final_state);
            }
            Err(e) => {
                tracing::warn!("State transition error for {}: {}", path, e);
            }
        }
    }

    fn mark_ready(&self, path: &str) {
        self.apply_transition(path, NodeState::Ready);
    }

    fn mark_running(&self, path: &str) {
        self.apply_transition(path, NodeState::Running);
    }

    fn mark_completed(&self, path: &str) {
        self.apply_transition(path, NodeState::Completed);
    }

    fn mark_failed(&self, path: &str) {
        self.apply_transition(path, NodeState::Failed);
    }

    fn mark_cancelled(&self, path: &str) {
        self.apply_transition(path, NodeState::Cancelled);
    }

    fn has_failed_dep(&self, deps: &[String]) -> bool {
        let states = self.node_states.lock();
        deps.iter().any(|d| {
            matches!(
                states.get(d).copied(),
                Some(NodeState::Failed) | Some(NodeState::Cancelled)
            )
        })
    }

    fn deps_ready(&self, deps: &[String], graph: &Graph) -> bool {
        let states = self.node_states.lock();
        deps.iter().all(|d| {
            matches!(
                states.get(d).copied(),
                Some(NodeState::Completed) | Some(NodeState::Ready) | Some(NodeState::Running)
            ) || graph.get_node(d).map(|n| n.is_source).unwrap_or(true)
        })
    }

    fn next_progress(&self) -> usize {
        self.progress.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn get_pool_semaphore(&self, pool: Option<&str>) -> Arc<Semaphore> {
        match pool {
            Some("console") => self.console_semaphore.clone(),
            Some(name) => self
                .pools
                .get(name)
                .cloned()
                .unwrap_or(self.job_semaphore.clone()),
            None => self.job_semaphore.clone(),
        }
    }

    fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::SeqCst);
    }

    fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::SeqCst);
    }

    fn allocate_tid(&self) -> u32 {
        self.trace.allocate_tid()
    }

    #[allow(dead_code)]
    fn trace_begin(&self, target: &str, command: Option<&str>, tid: u32) -> u64 {
        self.trace.begin_target(target, command, tid)
    }

    fn trace_complete(
        &self,
        target: &str,
        start_us: u64,
        duration_us: u64,
        tid: u32,
        command: Option<&str>,
        cache_hit: bool,
    ) {
        self.trace
            .complete_target(target, start_us, duration_us, tid, command, cache_hit);
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
        // Use configured runtime handle if available, otherwise create one
        if let Some(handle) = &self.config.runtime {
            handle.block_on(self.run_async(graph, targets))
        } else {
            // Create a new runtime for this build
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(self.config.parallelism.min(num_cpus::get()))
                .enable_all()
                .build()
                .map_err(ExecError::SpawnError)?;

            rt.block_on(self.run_async(graph, targets))
        }
    }

    /// Handle the result of a completed task
    #[allow(clippy::type_complexity)]
    fn handle_task_result(
        result: Result<
            Result<(String, bool, Option<String>), (String, ExecError)>,
            tokio::task::JoinError,
        >,
        cache: &Option<Cache>,
    ) {
        match result {
            Ok(Ok((path, executed, cache_key))) => {
                // Store in cache if executed (not from cache)
                if executed {
                    if let (Some(c), Some(key)) = (cache, cache_key) {
                        let outputs = vec![Path::new(&path)];
                        if let Err(e) = c.store(&key, &outputs, "") {
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
            }
            .emit();
        }

        // Create build trace (enabled if trace_file is set)
        let trace = Arc::new(BuildTrace::new(self.config.trace_file.is_some()));
        trace.add_metadata("process_name", "rninja");

        // Create shared state
        let state = Arc::new(BuildState::new(
            self.config.parallelism,
            total,
            &pools,
            self.config.output_mode,
            trace.clone(),
        ));

        // Execute nodes respecting dependencies
        let mut handles = Vec::new();
        let mut pending: Vec<&Node> = work.clone();

        while !pending.is_empty() || !handles.is_empty() {
            // Check if we should stop due to failures
            let fail_count = state.fail_count.load(Ordering::SeqCst);
            if self.config.keep_going > 0 && fail_count >= self.config.keep_going {
                // Cancel remaining pending work
                for node in &pending {
                    state.mark_cancelled(&node.path);
                }
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

                // Collect environment variables that affect the build
                let env_vars = collect_relevant_env_vars();
                let env_refs: Vec<(&str, &str)> =
                    env_vars.iter().map(|(k, v)| (*k, v.as_str())).collect();

                // Compute cache key
                let cache_key = if let (Some(cache), Some(cmd)) = (&self.cache, &command) {
                    if !is_phony {
                        let input_paths: Vec<_> =
                            deps.iter().map(|d| Path::new(d.as_str())).collect();
                        match cache.action_key(cmd, &input_paths, &env_refs) {
                            Ok(key) => Some(key),
                            Err(e) => {
                                tracing::warn!("Failed to compute cache key: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Check cache and try restore synchronously before spawning
                let mut restored = false;
                if let (Some(cache), Some(key)) = (&self.cache, &cache_key) {
                    if let Some(entry) = cache.lookup(key) {
                        match cache.restore(&entry) {
                            Ok(true) => {
                                state.record_cache_hit();
                                state.mark_completed(&path);
                                restored = true;
                            }
                            Ok(false) => {
                                debug!("Cache entry found but blob missing for {}", path);
                            }
                            Err(e) => {
                                debug!("Cache restore failed for {}: {}", path, e);
                            }
                        }
                    }
                }

                if restored {
                    // Emit cache hit output and trace inline (no task spawn)
                    let idx = state.next_progress();
                    match state.output_mode {
                        OutputMode::Human => {
                            let desc = description.as_deref().unwrap_or("cached");
                            println!("[{}/{}] {} (cached)", idx, state.total, desc);
                            let _ = std::io::stdout().flush();
                        }
                        OutputMode::Json => {
                            JsonEvent::CacheHit {
                                target: &path,
                                index: idx,
                                total: state.total,
                            }
                            .emit();
                        }
                    }
                    let tid = state.allocate_tid();
                    let trace_start = state.trace.timestamp();
                    state.trace_complete(&path, trace_start, 0, tid, command.as_deref(), true);
                    continue;
                }

                // Not restored from cache; spawn task for execution
                state.mark_ready(&path);

                let handle = tokio::spawn(async move {
                    // Acquire semaphore permit
                    let sem = state.get_pool_semaphore(pool.as_deref());
                    let _permit = match sem.acquire().await {
                        Ok(permit) => permit,
                        Err(e) => {
                            tracing::error!("Failed to acquire semaphore: {}", e);
                            state.mark_failed(&path);
                            return Err((
                                path,
                                ExecError::SpawnError(std::io::Error::other(format!(
                                    "semaphore error: {}",
                                    e
                                ))),
                            ));
                        }
                    };

                    state.mark_running(&path);

                    // Allocate thread ID for tracing
                    let tid = state.allocate_tid();
                    let trace_start = state.trace.timestamp();
                    let exec_start = Instant::now();

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
                        &state,
                    )
                    .await;

                    // Record trace event
                    let duration_us = exec_start.elapsed().as_micros() as u64;
                    state.trace_complete(
                        &path,
                        trace_start,
                        duration_us,
                        tid,
                        command.as_deref(),
                        false,
                    );

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

                Self::handle_task_result(result, &self.cache);
            }
        }

        // Wait for remaining handles
        for handle in handles {
            let result = handle.await;
            Self::handle_task_result(result, &self.cache);
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
                cache_hits: if cache_hits > 0 {
                    Some(cache_hits)
                } else {
                    None
                },
                cache_misses: if cache_misses > 0 {
                    Some(cache_misses)
                } else {
                    None
                },
            }
            .emit();
        }

        // Write trace file if configured
        if let Some(ref trace_path) = self.config.trace_file {
            if let Err(e) = trace.write_to_file(trace_path) {
                eprintln!("Warning: failed to write trace file: {}", e);
            } else {
                info!("Trace written to {}", trace_path.display());
            }
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

/// Execute a single node asynchronously.
///
/// Returns `Ok((path, executed, cache_key))` where:
/// - `executed` is `true` if the command was run, `false` if restored from cache
/// - `cache_key` is the action key if the command was executed
///
/// # Security Warning
///
/// Commands are executed via `sh -c` for shell compatibility with ninja build
/// files. This means shell injection is possible if build rules contain untrusted
/// input. Only use build files from trusted sources. Consider running rninja
/// with `--dry-run` for untrusted projects.
#[allow(clippy::too_many_arguments)]
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
            if let Err(e) = std::io::stdout().flush() {
                debug!("Failed to flush stdout: {}", e);
            }
        }
        OutputMode::Json => {
            JsonEvent::TargetStarted {
                target: path,
                index: idx,
                total,
                command: if config.verbose { Some(command) } else { None },
            }
            .emit();
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

    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };

    // Execute the command
    let output = Command::new(shell)
        .arg(shell_arg)
        .arg(command)
        .output()
        .await?;

    // Print output if any
    if !output.stdout.is_empty() {
        if let Err(e) = std::io::stdout().write_all(&output.stdout) {
            debug!("Failed to write to stdout: {}", e);
        }
    }
    if !output.stderr.is_empty() {
        if let Err(e) = std::io::stderr().write_all(&output.stderr) {
            debug!("Failed to write to stderr: {}", e);
        }
    }

    // Parse depfile if present
    if !depfile.is_empty() {
        if let Ok(deps) = depfile::parse(depfile) {
            debug!("Parsed depfile {}: {:?}", depfile, deps);
        }
    }

    // Clean up response file
    if let Some(rsp) = rspfile {
        if let Err(e) = tokio::fs::remove_file(rsp).await {
            tracing::warn!("Failed to clean up response file {}: {}", rsp, e);
        }
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
            }
            .emit();
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
            }
            .emit();
        }
        Err(ExecError::CommandFailed {
            command: command.to_string(),
            code,
        })
    }
}
