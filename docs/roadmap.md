# Roadmap to Full Functionality

rninja's path to a fully featured, trustworthy drop-in replacement spans multiple phases. Each phase adds capabilities while preserving strict compatibility with Ninja. Cross-cutting tracks for benchmarks and compatibility testing ensure every milestone is measurable and safe to adopt.

## Phase 0 – Foundations & Daemon Skeleton ✅ COMPLETE

- ✅ Port Ninja's parser and execution semantics verbatim, targeting identical behavior without caching enabled.
- ✅ Add sled+rkyv scaffolding so later phases can toggle caching without invasive refactors.
- ✅ Deliver initial observability: structured logs, `rninja --stats`, and basic tracing hooks.
- ✅ Establish a core unit-test suite covering parser correctness and core executor behavior.
- ✅ Publish initial documentation: README updates, architecture overview.
- ✅ CLI/daemon split with auto-spawn, concurrent builds, NNG IPC, and `--no-daemon` fallback.

## Phase 1 – Local Cache MVP ✅ COMPLETE

- ✅ Enable sled-based cache indexing and blob storage for local-only caching with deterministic hashing.
- ✅ Validate depfile, restat, and pool semantics when cache hits/skips occur.
- ✅ Provide configuration surface (`RNINJA_CACHE_DIR`, `RNINJA_CACHE_ENABLED`) plus config files.
- ✅ Publish the first benchmark runs comparing local incremental builds vs. stock Ninja (see BENCHMARK.md).
- ✅ Expand unit tests to cover cache hashing, sled durability, and cache eviction logic.
- ✅ Extend documentation with cache configuration instructions.

## Phase 2 – Remote Cache & async-nng Transport ✅ COMPLETE

- ✅ Integrate async-nng transport between daemon instances or dedicated cache servers.
- ✅ Support remote push/pull policies, authentication (token-based), and backpressure handling.
- ✅ Remote cache client with retry logic, semaphore-based concurrency limiting.
- ✅ Cache server binary (`rninja-cached`) with NNG REQ/REP protocol.
- ✅ CacheMode enum (Local, Remote, Auto) with graceful fallback on remote failures.
- ✅ MessagePack serialization for wire protocol.
- ✅ Multi-repo concurrency benchmarks (`benchmarks/run_benchmark.sh`).
- ✅ Network failure recovery integration tests (`tests/integration/network_test.sh`).
- ✅ Remote cache deployment documentation (`docs/remote-cache-deployment.md`).

## Phase 3 – Observability, Tooling, & Hardening ✅ COMPLETE

- ✅ Ship dashboard-ready metrics (cache hit rate, queue depth, remote latency) with Prometheus export format.
- ✅ Add admin tooling: `rninja -t cache-stats`, `rninja -t cache-gc`, `rninja -t cache-health`.
- ✅ Cache schema versioning and migration framework (`src/cache/schema.rs`).
- ✅ Build tracing integration with Chrome tracing format output (`--trace FILE`).
- ✅ 100% compatibility suite pass rate (15/15 tests).
- ✅ Rolling restart documentation (`docs/rolling-restart.md`).
- ✅ Full operations manual and API references (`docs/operations-manual.md`).

## Benchmarking Milestones ✅ COMPLETE

1. **Baseline Capture** ✅
   - `benchmarks/run_benchmark.sh` captures Ninja vs rninja performance
   - `benchmarks/generate_project.sh` creates small/medium/large test projects
   - Results stored in `benchmarks/results.json` with hardware specs

2. **Local Cache Benchmarks** ✅
   - `benchmarks/cache_benchmark.sh` measures cold/warm cache performance
   - Metrics: wall-clock time, cache hit/miss counts, speedup ratios

3. **Remote Cache Benchmarks** ✅
   - `benchmarks/remote_cache_benchmark.sh` tests remote cache latency
   - Tests concurrent clients and cache server performance

4. **CI/Automation**
   - ✅ Integrate benchmarks into CI

## Compatibility Test Milestones ✅ COMPLETE

1. **Ninja Regression Suite** ✅
   - `scripts/compat_test.sh` - 15 basic compatibility tests (100% pass)
   - `scripts/fuzzy_compat_test.sh` - 55 mutation tests (100% pass)

2. **Generator Coverage** ✅
   - `tests/generators/cmake_test.sh` - CMake-generated builds
   - `tests/generators/meson_test.sh` - Meson-generated builds
   - `tests/generators/gn_test.sh` - GN-generated builds (requires gn)

3. **Stress & Concurrency** ✅
   - `tests/integration/stress_test.sh` - 5 stress tests (100% pass)
   - Multi-repo concurrent builds, high/low parallelism, keep-going

4. **Serialization Tests** ✅
   - `tests/integration/serialization_test.sh` - 6 tests (100% pass)
   - Build log read/write, restat, recompact, clean tools

5. **Network Recovery Tests** ✅
   - `tests/integration/network_test.sh` - 4 tests (100% pass)
   - Cache disabled, local cache, invalid socket fallback

## Distribution Milestones ✅ COMPLETE

- ✅ Pre-built binaries for macOS (x86_64, aarch64) and Linux (x86_64, aarch64)
- ✅ Homebrew formula published to `neul-labs/homebrew-tap`
- ✅ NPM wrapper package for `npm install -g rninja-cli`
- ✅ PyPI wrapper package for `pip install rninja-cli`

## Deliverables

- `benchmarks/` harness and published dataset across roadmap phases.
- `tests/compat/` suite run in CI, with badges/logs highlighting status.
- Comprehensive unit/integration tests per phase, tracked under `tests/unit` and `tests/integration`.
- Phase-specific documentation updates committed alongside features.
- Documentation of benchmarking methodology, compatibility coverage, and phase readiness checklists.

Tracking these milestones keeps the drop-in promise credible while demonstrating rninja’s performance claims and charting the path to full functionality.
