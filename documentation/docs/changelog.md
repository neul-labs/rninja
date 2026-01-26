---
title: Changelog
description: Version history and release notes
---

# Changelog

All notable changes to rninja are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

---

## Unreleased

### Planned

- Homebrew formula
- Pre-built binaries for major platforms
- Metrics export for Prometheus
- Grafana dashboard templates

[0.1.0]: https://github.com/neul-labs/rninja/releases/tag/v0.1.0
