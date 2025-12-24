# Roadmap to Full Functionality

rninja's path to a fully featured, trustworthy drop-in replacement spans multiple phases. Each phase adds capabilities while preserving strict compatibility with Ninja. Cross-cutting tracks for benchmarks and compatibility testing ensure every milestone is measurable and safe to adopt.

## Phase 0 – Foundations & Daemon Skeleton ✅ COMPLETE

- ✅ Port Ninja's parser and execution semantics verbatim, targeting identical behavior without caching enabled.
- ✅ Add sled+rkyv scaffolding so later phases can toggle caching without invasive refactors.
- ✅ Deliver initial observability: structured logs, `rninja --stats`, and basic tracing hooks.
- ✅ Establish a core unit-test suite covering parser correctness and core executor behavior.
- ✅ Publish initial documentation: README updates, architecture overview.
- ⏳ CLI/daemon split (future - currently runs as single process)

## Phase 1 – Local Cache MVP ✅ COMPLETE

- ✅ Enable sled-based cache indexing and blob storage for local-only caching with deterministic hashing.
- ✅ Validate depfile, restat, and pool semantics when cache hits/skips occur.
- ✅ Provide configuration surface (`RNINJA_CACHE_DIR`, `RNINJA_CACHE_ENABLED`) plus config files.
- ✅ Publish the first benchmark runs comparing local incremental builds vs. stock Ninja (see BENCHMARK.md).
- ✅ Expand unit tests to cover cache hashing, sled durability, and cache eviction logic.
- ✅ Extend documentation with cache configuration instructions.

## Phase 2 – Remote Cache & async-nng Transport

- Integrate async-nng transport between daemon instances or dedicated cache servers.
- Support remote push/pull policies, authentication, and backpressure handling.
- Demonstrate multi-repo concurrency where single daemon juggles builds hitting shared caches.
- Expand benchmarks to include cold build speedups and cross-machine reuse metrics.
- Add integration tests simulating multi-machine cache sharing and network failure recovery.
- Document remote cache deployment steps, security considerations, and tuning guidance.

## Phase 3 – Observability, Tooling, & Hardening

- Ship dashboard-ready metrics (cache hit rate, queue depth, remote latency) and log scrapers for CI.
- Add admin tooling for cache inspection, GC, and sled/rkyv health checks.
- Finalize upgrade story for async-nng/sled/rkyv (schema migrations, rolling restarts).
- Declare GA criteria: 100% compatibility suite pass rate, benchmark targets met, docs complete.
- Build regression tests for admin tooling and upgrade flows, ensuring migrations are reversible.
- Complete full documentation set (operations manual, benchmarking guide, API references).

## Benchmarking Milestones

1. **Baseline Capture**  
   - Record current Ninja performance on representative repos (small, medium, large) using `ninja -d stats` and wall-clock measurements.  
   - Store raw data plus hardware specs in `benchmarks/baseline.json`.

2. **Local Cache Benchmarks**  
   - Build automation to run repeated incremental builds with rninja (daemon) vs Ninja.  
   - Metrics: wall-clock time, CPU utilization, cache hit/miss counts, scheduler queue depth.

3. **Remote Cache Benchmarks**  
   - Stand up async-nng cache nodes and measure cold/warm remote fetch/push latency, throughput under concurrent repo builds, and resilience to network hiccups.

4. **CI/Automation**  
   - Integrate benchmarks into CI (nightly or weekly) so regressions trigger alerts.  
   - Publish dashboard summaries and highlight 2×–5× targets.

## Compatibility Test Milestones

1. **Ninja Regression Suite**  
   - Mirror upstream Ninja tests (depfile handling, rsp files, pools, restat) and run them against the rninja daemon.  
   - Block releases until parity is confirmed.

2. **Generator Coverage**  
   - Create sample projects from CMake, Meson, GN, and Bazel-to-Ninja exporters.  
   - Ensure rninja handles their unique constructs (phony rules, custom pools, response files).

3. **Stress & Concurrency**  
   - Multi-repo scenarios where concurrent CLI invocations hit the same daemon, validating isolation and cache correctness.

4. **Upgrade/Regression Matrix**  
   - Automated suites covering sled/rkyv/async-nng version upgrades, ensuring serialization formats remain compatible.

## Deliverables

- `benchmarks/` harness and published dataset across roadmap phases.  
- `tests/compat/` suite run in CI, with badges/logs highlighting status.  
- Comprehensive unit/integration tests per phase, tracked under `tests/unit` and `tests/integration`.  
- Phase-specific documentation updates committed alongside features.
- Documentation of benchmarking methodology, compatibility coverage, and phase readiness checklists.

Tracking these milestones keeps the drop-in promise credible while demonstrating rninja’s performance claims and charting the path to full functionality.
