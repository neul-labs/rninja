---
title: FAQ
description: Frequently asked questions about rninja
tags:
  - troubleshooting
---

# FAQ

Frequently asked questions about rninja.

## General

### What is rninja?

rninja is a high-performance, drop-in replacement for Ninja build system, written in Rust. It adds caching (local and remote), a persistent daemon for faster incremental builds, and maintains full compatibility with Ninja.

### Is rninja compatible with my project?

If your project works with Ninja, it works with rninja. rninja reads standard `build.ninja` files and supports all Ninja command-line options.

### How much faster is rninja?

Typical improvements:
- **Incremental builds**: 2-5x faster (daemon + cache)
- **CI/CD builds**: 50-90% time reduction (remote cache)
- **Clean builds**: ~Same as Ninja (no cache hits)

See [Benchmarks](../performance/benchmarks.md) for detailed numbers.

### Is rninja production-ready?

Yes. rninja is used in production by teams building large C/C++ projects. It's designed for reliability with safe Rust implementation.

## Installation

### How do I install rninja?

```bash
# Cargo
cargo install rninja

# Homebrew (macOS)
brew install rninja

# From source
git clone https://github.com/anthropics/rninja
cd rninja && cargo build --release
```

See [Installation](../getting-started/installation.md) for all methods.

### Can I use rninja alongside Ninja?

Yes. Install rninja to a different path:

```bash
# Use as 'rninja'
cargo install rninja

# Or replace 'ninja'
sudo ln -sf $(which rninja) /usr/local/bin/ninja
```

### What platforms are supported?

- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)
- FreeBSD

## Caching

### How does caching work?

rninja computes a hash (cache key) from:
- Build rule and command
- Input file contents
- Relevant environment variables

If an output with that key exists in cache, it's restored instead of rebuilding.

### What's the difference between local and remote cache?

- **Local cache**: On your machine, benefits single user
- **Remote cache**: Shared server, benefits entire team

### Should I use remote cache?

Yes, if you have:
- Multiple developers working on same project
- CI/CD pipelines
- Multiple machines

### How do I set up remote cache?

1. Deploy rninja-cached server
2. Configure clients:

```bash
export RNINJA_REMOTE_URL=tcp://cache.example.com:9876
export RNINJA_CACHE_TOKEN=your-token
```

See [Remote Cache Setup](../caching/remote/quick-setup.md).

### Why am I getting cache misses?

Common causes:
- Different compiler versions
- Different environment variables
- Absolute paths in commands
- Non-deterministic builds

Debug with:
```bash
rninja -d explain
RNINJA_LOG_LEVEL=debug rninja
```

## Daemon

### What is the daemon?

A persistent background process that:
- Keeps build graph in memory
- Provides instant incremental builds
- Shares work across terminals

### Do I need the daemon?

No, but it helps. Without daemon:
- Each build parses build.ninja (slow for large projects)
- No shared state between terminals

### How do I control the daemon?

```bash
rninja -t daemon-status   # Check status
rninja -t daemon-stop     # Stop daemon
rninja --no-daemon        # Single-shot mode
```

### Does the daemon use lots of memory?

Typically 50-200 MB. Set limits if needed:

```toml
[daemon]
max_memory = "500M"
```

## Compatibility

### Does rninja work with CMake?

Yes. CMake generates standard Ninja files:

```bash
cmake -G Ninja -B build
rninja -C build  # or cd build && rninja
```

### Does rninja work with Meson?

Yes. Meson uses Ninja by default:

```bash
meson setup build
rninja -C build
```

### Does rninja work with GN?

Yes:

```bash
gn gen out/Default
rninja -C out/Default
```

### Are there any Ninja features not supported?

All Ninja features are supported:
- Pools (including console pool)
- Phony targets
- Depfiles (GCC and MSVC formats)
- Dynamic dependencies (dyndep)
- Response files

## Troubleshooting

### Build fails with rninja but works with Ninja

This shouldn't happen. Please report it:

1. Try: `rninja --no-cache --no-daemon`
2. Compare: `ninja -n -v` vs `rninja -n -v`
3. Open issue with details

### Cache is corrupted

```bash
# Check health
rninja -t cache-health

# Repair or clear
rninja -t cache-repair
rninja -t cache-clear
```

### Daemon won't start

```bash
# Check for stale sockets
ls /tmp/rninja-*.sock

# Remove and restart
rm /tmp/rninja-*.sock
rninja -t daemon-start
```

### Build is slower than expected

```bash
# Profile build
rninja --trace build.trace
# Open in chrome://tracing

# Check cache hit rate
rninja -t cache-stats
```

## Configuration

### Where do I put configuration?

In order of priority:
1. `.rninja/config.toml` (project)
2. `~/.config/rninja/config.toml` (user)
3. `/etc/rninja/config.toml` (system)

### Can I disable caching for specific rules?

Yes:

```toml
[cache]
exclude_rules = ["phony", "install_*", "test_*"]
```

### Can I use environment variables?

Yes, all options have env var equivalents:

```bash
export RNINJA_JOBS=8
export RNINJA_CACHE_MODE=local
export RNINJA_DAEMON_MODE=off
```

## Security

### Is the cache secure?

- Local cache: Protected by filesystem permissions
- Remote cache: Supports token auth and TLS

### Can cache be poisoned?

Cache keys are cryptographic hashes. Poisoning would require:
- Breaking BLAKE3 (cryptographically secure)
- Having write access to cache

### Should I use authentication?

Yes, for remote cache in production:

```toml
[auth]
mode = "token"
```

## Contributing

### How do I contribute?

See [Contributing Guide](../contributing/guide.md):

1. Fork repository
2. Create branch
3. Make changes
4. Run tests
5. Submit PR

### Where do I report bugs?

GitHub Issues: https://github.com/anthropics/rninja/issues

Include:
- rninja version
- OS and version
- Steps to reproduce
- Expected vs actual behavior

### How do I request features?

Open a GitHub Issue with:
- Use case description
- Proposed solution (optional)
- Willingness to contribute
