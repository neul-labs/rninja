# Drop-in Replacement Guide

rninja’s primary design goal is to replace the stock Ninja executable without requiring changes to existing generators or build descriptions. Use this guide to understand CLI compatibility, configuration knobs, and validation steps.

## CLI Compatibility

- **Command-line parity**: rninja accepts the same core flags as Ninja (`-C`, `-f`, `-j`, `-k`, `-l`, `-n`, `-t`, `-v`, `-d`). Unsupported or experimental flags should fail fast with clear messaging.
- **Environment variables**: Standard Ninja env vars (e.g., `NINJA_STATUS`) remain intact. rninja adds cache-specific variables prefixed with `RNINJA_` to avoid collisions.
- **Exit codes**: Success/failure codes mirror Ninja so scripts and CI pipelines do not need adjustments.
- **Phony targets and generator edges**: Behavior matches Ninja semantics, including depfile parsing and `restat` handling, ensuring incremental rebuild detection stays correct.
- **Simultaneous invocations**: A single rninja daemon accepts requests from multiple CLI commands across different repos or build directories; cache access is namespaced and synchronized so no manual locking is required.

## Daemon Interaction Model

- The `rninja` CLI is a thin client that forwards commands to the background daemon (starting it automatically if not running).
- Each repo request includes its working directory, environment, and desired targets; the daemon executes them in parallel while isolating state.
- Logs and stdout/stderr are streamed back to the invoking CLI so existing scripts still capture output as before.

## Configuration Layers

1. **Zero-config mode**: By default rninja stores cache metadata and blobs under `$XDG_CACHE_HOME/rninja` (or platform equivalent) and performs only local caching.
2. **Config file**: Optional `rninja.toml` describes cache directories, remote endpoints, and tuning parameters (parallelism caps, sandboxing).
3. **Environment overrides**: `RNINJA_CACHE_DIR`, `RNINJA_REMOTE_URL`, `RNINJA_CACHE_MODE`, and `RNINJA_SCHED_THREADS` provide quick experimentation without editing files.

## Migration Checklist

1. Install rninja alongside Ninja (`ninja.orig`) on developer workstations and CI agents.
2. Run `rninja daemon status` (starts the daemon if needed) and confirm a single instance is serving the machine.
3. Run `rninja -v` to verify version and build metadata; confirm async-nng, sled, and ryv features are compiled in.
4. Execute `rninja -C out/Default -n all` (dry run) to ensure parsing of `build.ninja` succeeds.
5. Perform side-by-side builds: `time ninja.orig target` vs `time rninja target`, comparing logs for unexpected differences.
6. Enable local cache and confirm repeated builds rehydrate objects (check `rninja --stats` output).
7. Configure remote cache and validate that another machine can fetch artifacts without running compilers.

If any discrepancy appears (missing depfile, command re-run unexpectedly), capture logs with `-d explain` and file an issue—semantic alignment is critical for trust.

## Troubleshooting Tips

- **Cache misses everywhere**: ensure commands are deterministic; non-deterministic outputs (timestamps, random seeds) invalidate cache entries.
- **Remote cache latency**: verify async-nng endpoints are reachable and `RNINJA_REMOTE_CONCURRENCY` is tuned to match available bandwidth.
- **Scheduler differences**: if behavior diverges (e.g., target order), use `-d keepdepfile` and `-d explain` to confirm dependencies; rninja should still respect Ninja’s ordering constraints even if internal scheduling changes.

Following this checklist keeps rninja a safe drop-in replacement while unlocking caching and scheduling improvements.
