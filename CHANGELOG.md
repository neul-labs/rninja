# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-05-07

### Fixed

- **Cache restore now works**: cache hits actually restore output files from blob storage instead of being a no-op stub.
- **Environment variables in cache keys**: `CC`, `CXX`, `CFLAGS`, `LDFLAGS`, etc. are now included in action keys, preventing stale cache hits after compiler flag changes.
- **State machine enforcement** for `NodeState`, `SessionState`, `ConnectionState`, and `WatcherState` — invalid transitions are now rejected at compile time or logged at runtime.
- **Removed `panic!` in production path** in cache server handler.
- **`BuildLog::save` now resets the dirty flag** after successful writes.
- **Blocking NNG calls moved off the async runtime** via `tokio::task::spawn_blocking`.
- **Admin tools no longer `unwrap` JSON serialization**.
- **`handle_query` deduplication** in daemon server.

### Added

- **Homebrew formula** available via `brew tap neul-labs/tap`.
- **Pre-built binaries** for macOS (Intel & Apple Silicon) and Linux (x86_64 & aarch64).
- **NPM wrapper** for global installation via `npm install -g rninja`.
- **PyPI wrapper** for installation via `pip install rninja`.

## [0.1.0] - 2024-12-01

### Added

- Initial release of rninja
- Full Ninja build file compatibility
- Local build cache with BLAKE3 hashing
- Content-addressed blob storage using sled
- Remote cache support via nng transport
- Daemon mode for faster subsequent builds
- All standard ninja subtools (`clean`, `compdb`, `graph`, `deps`, `query`, etc.)
- Cache management tools (`cache-stats`, `cache-gc`, `cache-health`)
- Configuration via TOML files and environment variables
- Chrome trace output for build profiling
- JSON output mode for scripting and automation
- CMake, Meson, and GN generator compatibility

### Performance

- 23x faster no-op builds compared to ninja
- 2-5x faster warm incremental builds with caching
- 1.3-2x faster cold builds with improved parallelism

[0.1.1]: https://github.com/neul-labs/rninja/releases/tag/v0.1.1
[0.1.0]: https://github.com/neul-labs/rninja/releases/tag/v0.1.0
