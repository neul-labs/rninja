# rninja

A fast, Rust-based drop-in replacement for [Ninja](https://ninja-build.org/) with built-in caching and improved scheduling.

rninja layers advanced scheduling, local caching, and optional remote cache synchronization on top of familiar Ninja semantics. It targets large, multi-language codebases that already rely on generators such as CMake, GN, or Meson, and want dramatic cuts in incremental build times without rewriting their build descriptions.

## Quick Start

```bash
# Build from source
cargo build --release

# Use as drop-in replacement for ninja
./target/release/rninja

# Or install system-wide
cargo install --path .
```

## Why rninja

- **Drop-in familiarity**: Works with existing `.ninja` files, so teams only swap the executor binary while keeping their current generators and workflows.
- **Modern scheduling**: Uses a Rust concurrency runtime to keep CPU, disk, and network resources saturated on large builds, minimizing idle time compared to classic `ninja -jN`.
- **Content-addressed caching**: Hashes inputs, compiler options, and produced artifacts to skip repeated work during local builds and CI runs.
- **Remote cache awareness**: Shares cached objects between developer laptops and CI agents, giving teams "first build fast" behavior once caches are warm.
- **Safety-first engineering**: Rust eliminates common concurrency bugs in the executor; deterministic semantics mirror Ninja to maintain trust.

## Expected Impact

| Scenario | Description | Estimated Speedup* |
| --- | --- | --- |
| Warm incremental builds | Developers rebuilding after small changes with high cache hit rates | 2× – 5× |
| Shared CI cache | CI pipeline reusing cached objects from prior runs or other machines | 2× – 5× |
| Cold builds on modern hardware | Full rebuilds that still benefit from improved scheduling and parallel IO | 1.3× – 2× |

\*Actual gains depend on project size, cache hit rate, and hardware.

## Where rninja Fits Best

- Large C/C++ or mixed-language monorepos that generate Ninja files and suffer from multi-minute incremental builds.
- Organizations running many CI builds per day that want to eliminate redundant compilation across nodes.
- Embedded, graphics, or simulation projects with expensive code generation steps that benefit from artifact reuse.
- Game studios and performance-sensitive software teams that already rely on Ninja but need better scaling across cores.

## Adoption Considerations

- Cache efficacy is the biggest multiplier: projects with frequent repeated builds, shared CI caches, or multiple developers compiling similar targets will see the largest wins.
- Compatibility matters: rninja must faithfully implement Ninja features (depfiles, restat, generator rules) to maintain confidence; early adopters should plan validation runs side-by-side with standard Ninja.
- Installation should be frictionless: a single static binary plus configuration for cache stores (local path or remote endpoint) keeps onboarding straightforward.
- Concurrent invocations across repos are supported through a single rninja daemon: CLI commands submit work to the daemon, which orchestrates multiple repositories simultaneously while keeping per-repo state isolated.

## Drop-in Usage

1. **Generate `build.ninja` as usual** with GN, CMake, Meson, or any other generator. No DSL changes or rule rewrites are required.
2. **Install the rninja binary** on developer machines or CI images (rename to `ninja` if desired). CLI flags mirror Ninja (`-C`, `-j`, `-d`, etc.), so scripts keep working.
3. **Point rninja at cache locations** using config files or env vars like `RNINJA_CACHE_DIR`, `RNINJA_REMOTE_URL`, and `RNINJA_CACHE_MODE=local|remote|both`. Defaults keep caching local-only if no remote is configured.
4. **Run builds exactly the same way**: `rninja -C out/Release chrome` or `ninja` symlinked to rninja. Depfiles, restat logic, rsp files, and generator edges behave identically, so teams can A/B test safely.

During evaluation, keep standard Ninja available (e.g., `ninja.orig`) to compare timings or fall back instantly if needed. The long-term goal is zero behavior drift, so any mismatches should be filed as bugs.

## Available Tools

rninja includes all standard ninja subtools plus additional utilities:

```
rninja -t list
rninja subtools:
    clean      remove built files
    cleandead  clean built files no longer produced by manifest
    commands   list all commands required to rebuild given targets
    compdb     dump JSON compilation database to stdout
    config     show config file locations and generate sample config
    deps       show dependencies stored in the deps log
    graph      output graphviz dot file for targets
    inputs     list all inputs required to rebuild given targets
    path       find dependency path between two targets
    query      show inputs/outputs for a path
    recompact  recompact ninja-internal data structures
    restat     restat all outputs in the build log
    rules      list all rules
    targets    list targets by their rule or depth in the DAG
```

## Configuration

rninja supports configuration files in TOML format. Configuration is loaded from (in order of precedence):

1. `.rninjarc` (project local)
2. `~/.rninjarc` (user home)
3. `~/.config/rninja/config.toml` (XDG style)

Generate a sample config with:
```bash
rninja -t config -v
```

### Environment Variables

- `RNINJA_CACHE_DIR` - Cache directory location
- `RNINJA_CACHE_ENABLED` - Enable/disable caching (0 or 1)
- `RUST_LOG` - Logging level (e.g., `debug`, `info`)

## Performance

See [BENCHMARK.md](BENCHMARK.md) for detailed performance comparisons with ninja.

Key highlights:
- **30% faster** cold builds due to improved scheduling
- **Sub-millisecond** no-op detection via optimized build log
- **Automatic caching** of build artifacts for faster rebuilds

## Technology Choices

- **async-nng** powers the remote cache transport layer, giving rninja a high-throughput, async messaging fabric for publishing and fetching artifacts over NNG sockets.
- **sled** acts as the embedded metadata/index database for the local content-addressed cache, enabling crash-safe tracking of digests, inputs, and compiler flags.
- **rkyv** stores the actual artifact payloads as a content-addressed object store, providing deduplicated blobs that sled and async-nng reference for local and remote reuse.

## Roadmap Highlights

- **Phased delivery**: Phase 0 builds the daemon + compatibility layer, Phase 1 enables local caching, Phase 2 adds async-nng powered remote caches, and Phase 3 focuses on observability/hardening—each phase ships specific unit tests and documentation updates.
- **Benchmark harness**: automated comparisons between rninja and stock Ninja (local and remote cache modes) keep performance goals measurable throughout the phases.
- **Compatibility & test suites**: releases are gated by regression suites plus targeted unit/integration tests for daemon, cache, and remote transport behavior, guarding the drop-in guarantee.

## Additional Documentation

- `docs/performance.md` — deeper dive into speed expectations, what influences cache hit rates, and how to reason about cold vs incremental builds.
- `docs/market.md` — outlines core user personas, project types, and adoption channels for rninja.
- `docs/sensitivities.md` — summarizes the key risks, engineering sensitivities, and success criteria for the project.
- `docs/dropin.md` — explains CLI compatibility, configuration knobs, and validation checklists for swapping rninja in.
- `docs/architecture.md` — describes the executor, cache, and transport components and how they preserve Ninja semantics.
- `docs/roadmap.md` — details planned benchmarking and compatibility milestones.

## Status

rninja is functional and can be used as a drop-in replacement for ninja. The core features are implemented:

- Full ninja manifest parsing
- Parallel build execution with tokio
- Local content-addressed caching with sled
- Build log compatibility (.ninja_log format)
- All standard subtools

Contributions, questions, or architectural discussions are welcome via issues and discussions.
