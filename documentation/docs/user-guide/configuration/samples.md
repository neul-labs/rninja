---
title: Sample Configurations
description: Ready-to-use configuration examples
tags:
  - user-guide
  - configuration
  - examples
---

# Sample Configurations

Ready-to-use configuration examples for common scenarios.

## Development Workstation

Basic configuration for local development:

```toml title="~/.config/rninja/config.toml"
# Development workstation configuration

[build]
# Use all CPU cores
jobs = 0
# Stop on first failure
keep_going = 1
# Don't explain by default
explain = false

[cache]
# Enable local caching
enabled = true
mode = "local"
# 5GB cache limit
max_size = 5368709120

[output]
# Quiet output
verbose = false
# Show stats after builds
stats = true
# Auto-detect colors
color = "auto"
```

## CI/CD Pipeline

Configuration for continuous integration:

```toml title="ci-config.toml"
# CI/CD configuration

[build]
# Use all available cores
jobs = 0
# Keep going to find all errors
keep_going = 0
# Don't explain (too verbose)
explain = false

[cache]
# Enable with remote fallback
enabled = true
mode = "auto"

[output]
# Verbose for CI logs
verbose = true
# Show statistics
stats = true
# No colors in CI
color = "never"
```

With environment variables:

```bash
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.ci.internal:9999
export RNINJA_CACHE_TOKEN=$CI_CACHE_TOKEN
```

## Team with Shared Cache

Configuration for teams sharing a remote cache:

```toml title="~/.config/rninja/config.toml"
# Team configuration with remote cache

[build]
jobs = 0
keep_going = 1

[cache]
enabled = true
# Try remote first, fall back to local
mode = "auto"
# Local cache limit
max_size = 2147483648  # 2GB

[output]
stats = true
color = "auto"
```

Team members set:

```bash
# In ~/.bashrc or team setup script
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.team.internal:9999
export RNINJA_CACHE_TOKEN=team-shared-token
```

## Resource-Constrained Machine

For machines with limited CPU or memory:

```toml title="~/.config/rninja/config.toml"
# Low-resource configuration

[build]
# Limit parallelism
jobs = 2
# Stop quickly on failure
keep_going = 1

[cache]
enabled = true
mode = "local"
# Small cache
max_size = 1073741824  # 1GB

[output]
verbose = false
stats = false
color = "auto"
```

## High-Performance Build Server

For dedicated build machines:

```toml title="/etc/rninja/config.toml"
# High-performance build server

[build]
# All cores
jobs = 0
# Build as much as possible
keep_going = 0

[cache]
enabled = true
mode = "auto"
# Large cache
max_size = 53687091200  # 50GB

[output]
verbose = false
stats = true
color = "never"
```

## Debug Configuration

For debugging build issues:

```toml title=".rninjarc"
# Debug configuration (temporary)

[build]
# Sequential for easier debugging
jobs = 1
keep_going = 1
# Always explain
explain = true

[cache]
# Disable caching for debugging
enabled = false

[output]
verbose = true
stats = true
color = "auto"
trace_file = "debug_trace.json"
```

## Project-Specific: Large C++ Project

```toml title=".rninjarc"
# Large C++ project configuration

[build]
# Limit to prevent memory exhaustion
jobs = 8
# Find all errors
keep_going = 5

[cache]
enabled = true
mode = "auto"

[output]
stats = true
```

## Project-Specific: Quick Iteration

For projects needing fast iteration:

```toml title=".rninjarc"
# Fast iteration configuration

[build]
jobs = 0
keep_going = 1

[cache]
enabled = true
mode = "local"

[output]
verbose = false
stats = false
color = "auto"
```

## Environment-Based Configurations

### Shell Script Setup

```bash title="setup-rninja.sh"
#!/bin/bash
# rninja environment setup

# Detect environment
if [ -n "$CI" ]; then
    # CI environment
    export RNINJA_CACHE_MODE=auto
    export RNINJA_CACHE_REMOTE_SERVER=${CI_CACHE_SERVER:-}
    export RNINJA_CACHE_TOKEN=${CI_CACHE_TOKEN:-}
else
    # Local development
    export RNINJA_CACHE_MODE=local
    export RNINJA_CACHE_MAX_SIZE=5G
fi

# Common settings
export RNINJA_CACHE_ENABLED=1
```

### Docker Development

```dockerfile title="Dockerfile"
FROM rust:latest

# Install rninja
RUN cargo install rninja

# Configure for container
ENV RNINJA_CACHE_ENABLED=1
ENV RNINJA_CACHE_MODE=local
ENV RNINJA_CACHE_DIR=/cache/rninja

# Mount cache volume at /cache
VOLUME /cache
```

### GitHub Actions

```yaml title=".github/workflows/build.yml"
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      RNINJA_CACHE_ENABLED: "1"
      RNINJA_CACHE_MODE: "auto"
      RNINJA_CACHE_REMOTE_SERVER: ${{ secrets.CACHE_SERVER }}
      RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}
    steps:
      - uses: actions/checkout@v4
      - name: Install rninja
        run: cargo install rninja
      - name: Build
        run: rninja -C build
```

### GitLab CI

```yaml title=".gitlab-ci.yml"
variables:
  RNINJA_CACHE_ENABLED: "1"
  RNINJA_CACHE_MODE: "auto"
  RNINJA_CACHE_REMOTE_SERVER: $CACHE_SERVER
  RNINJA_CACHE_TOKEN: $CACHE_TOKEN

build:
  script:
    - cargo install rninja
    - rninja -C build
```

## Minimal Configuration

The absolute minimum (everything defaults):

```toml title="~/.config/rninja/config.toml"
# Minimal - just enable caching
[cache]
enabled = true
```

## No Caching

For debugging or special cases:

```toml title=".rninjarc"
# Disable all caching
[cache]
enabled = false
```

## Tips for Configuration

### Start Simple

Begin with defaults and add configuration as needed:

```toml
# Start with just cache settings
[cache]
enabled = true
mode = "local"
```

### Use Environment for Secrets

Never put tokens in config files:

```toml
# Config file
[cache]
mode = "auto"
# Token comes from environment
```

```bash
# Environment
export RNINJA_CACHE_TOKEN=secret
```

### Project vs User Config

- **Project config (`.rninjarc`)**: Settings all developers need
- **User config (`~/.config/rninja/config.toml`)**: Personal preferences
