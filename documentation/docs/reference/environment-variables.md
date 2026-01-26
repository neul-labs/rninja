---
title: Environment Variables
description: rninja environment variable reference
tags:
  - reference
  - configuration
---

# Environment Variables

Complete reference for rninja environment variables.

## Core Variables

### Build Control

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_JOBS` | Number of parallel jobs | CPU count |
| `RNINJA_KEEP_GOING` | Continue after failures (count) | `1` |
| `RNINJA_VERBOSE` | Verbose output (`0`/`1`) | `0` |
| `RNINJA_BUILD_DIR` | Build directory | `.` |

### Cache Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_CACHE_MODE` | Cache mode (`auto`/`local`/`remote`/`off`) | `auto` |
| `RNINJA_CACHE_DIR` | Local cache directory | `~/.cache/rninja` |
| `RNINJA_CACHE_MAX_SIZE` | Maximum cache size | `10G` |
| `RNINJA_CACHE_READ` | Enable cache reads (`0`/`1`) | `1` |
| `RNINJA_CACHE_WRITE` | Enable cache writes (`0`/`1`) | `1` |

### Remote Cache

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_REMOTE_URL` | Remote cache server URL | (none) |
| `RNINJA_CACHE_TOKEN` | Authentication token | (none) |
| `RNINJA_REMOTE_TIMEOUT` | Request timeout (ms) | `30000` |
| `RNINJA_REMOTE_RETRY` | Enable retries (`0`/`1`) | `1` |

### Daemon

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_DAEMON_MODE` | Daemon mode (`auto`/`on`/`off`) | `auto` |
| `RNINJA_DAEMON_SOCKET` | Socket path | (auto) |
| `RNINJA_DAEMON_IDLE_TIMEOUT` | Idle timeout (seconds) | `300` |

### Logging

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_LOG_LEVEL` | Log level (`error`/`warn`/`info`/`debug`/`trace`) | `info` |
| `RNINJA_LOG_FILE` | Log file path | (stderr) |
| `RUST_LOG` | Rust logging filter | (none) |

### Config Files

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_CONFIG` | Path to config file | (auto-detect) |
| `RNINJA_NO_CONFIG` | Disable config files (`1`) | `0` |

## Server Variables

For rninja-cached server:

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_SERVER_BIND` | Bind address | `0.0.0.0:9876` |
| `RNINJA_SERVER_WORKERS` | Worker threads | CPU count |
| `RNINJA_STORAGE_PATH` | Storage directory | `/var/cache/rninja` |
| `RNINJA_STORAGE_MAX_SIZE` | Maximum storage | `100G` |
| `RNINJA_AUTH_MODE` | Auth mode (`none`/`token`) | `token` |
| `RNINJA_AUTH_TOKENS` | Comma-separated tokens | (none) |

## Ninja Compatibility

rninja respects standard Ninja variables:

| Variable | Description |
|----------|-------------|
| `NINJA_STATUS` | Status line format string |
| `CLICOLOR_FORCE` | Force colored output |
| `NO_COLOR` | Disable colored output |

## CI/CD Variables

Automatic CI detection:

| Variable | Effect |
|----------|--------|
| `CI=true` | Enables CI mode (quieter output) |
| `GITHUB_ACTIONS=true` | GitHub Actions integration |
| `GITLAB_CI=true` | GitLab CI integration |
| `JENKINS_URL` | Jenkins integration |

## Usage Examples

### Basic Configuration

```bash
# Set parallel jobs
export RNINJA_JOBS=8

# Enable verbose output
export RNINJA_VERBOSE=1

# Set cache directory
export RNINJA_CACHE_DIR=/fast-ssd/cache
```

### Remote Cache Setup

```bash
# Configure remote cache
export RNINJA_CACHE_MODE=remote
export RNINJA_REMOTE_URL=tcp://cache.example.com:9876
export RNINJA_CACHE_TOKEN=your-secret-token
```

### CI/CD Configuration

```bash
# CI build settings
export RNINJA_CACHE_MODE=remote
export RNINJA_DAEMON_MODE=off
export RNINJA_REMOTE_URL="${CACHE_SERVER}"
export RNINJA_CACHE_TOKEN="${CACHE_TOKEN}"
export RNINJA_JOBS=4
```

### Debugging

```bash
# Enable debug logging
export RNINJA_LOG_LEVEL=debug
export RUST_LOG=rninja=debug

# Disable cache for clean build
export RNINJA_CACHE_MODE=off
```

### Shell Configuration

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# rninja defaults
export RNINJA_CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/rninja"
export RNINJA_CACHE_MAX_SIZE=20G

# Remote cache (if available)
if [[ -n "${RNINJA_REMOTE_URL}" ]]; then
    export RNINJA_CACHE_MODE=auto
fi
```

## Priority

Environment variables override config files but are overridden by command-line arguments:

```
Command line > Environment > Config files
```

Example:

```bash
# Environment sets 8 jobs
export RNINJA_JOBS=8

# Command line wins
rninja -j16  # Uses 16 jobs
```

## Inspecting Configuration

```bash
# Show effective configuration
rninja --dump-config

# Show specific variable effect
RNINJA_VERBOSE=1 rninja -n
```

## Unsetting Variables

```bash
# Unset to use defaults
unset RNINJA_CACHE_MODE

# Or set to empty
export RNINJA_CACHE_TOKEN=
```

## Security Notes

!!! warning "Token Security"
    Never commit cache tokens to version control. Use:

    - CI/CD secrets
    - Environment files (not committed)
    - Secret managers

```bash
# Good: Use CI secrets
export RNINJA_CACHE_TOKEN="${{ secrets.CACHE_TOKEN }}"

# Bad: Hardcoded token
export RNINJA_CACHE_TOKEN=abc123  # Don't do this!
```
