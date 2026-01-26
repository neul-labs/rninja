---
title: Single-Shot Mode
description: Running rninja without the daemon
tags:
  - daemon
  - single-shot
---

# Single-Shot Mode

Run rninja without using the daemon process.

## Overview

Single-shot mode (`--no-daemon`) runs builds directly without connecting to or spawning a daemon.

```bash
rninja --no-daemon
```

## When to Use

### Containers and Ephemeral Environments

In Docker or short-lived environments:

```dockerfile
# Dockerfile
RUN rninja --no-daemon
```

Daemon provides no benefit when the environment is destroyed after the build.

### Debugging Build Issues

When isolating problems:

```bash
# Remove daemon as a variable
rninja --no-daemon -d explain
```

### CI Single-Job Runners

For CI jobs that run one build and exit:

```yaml
# .github/workflows/build.yml
- run: rninja --no-daemon
```

### Resource-Constrained Systems

When memory for daemon is not available:

```bash
# Embedded system or low-memory VM
rninja --no-daemon
```

### Scripted Builds

For predictable, isolated execution:

```bash
#!/bin/bash
# build.sh - deterministic build script
rninja --no-daemon -j4
```

## Usage

### Basic

```bash
rninja --no-daemon
```

### With Other Options

```bash
# Combine with any other flags
rninja --no-daemon -j8 -v my_target
rninja --no-daemon -C build/release
rninja --no-daemon -t clean
```

### Environment Variable

```bash
# Always use single-shot mode
export RNINJA_NO_DAEMON=1
rninja  # Runs without daemon
```

## Performance Comparison

| Metric | With Daemon | Single-Shot |
|--------|-------------|-------------|
| First build | ~200ms + build | ~200ms + build |
| Subsequent builds | ~20ms + build | ~200ms + build |
| Memory baseline | 50-200 MB | 0 (process exits) |

### When Daemon Helps

- Multiple builds in same session
- Interactive development
- IDE integration

### When Daemon Doesn't Help

- Single builds (CI)
- Isolated environments
- Infrequent builds

## Caching Still Works

Single-shot mode still uses the cache:

```bash
# First build - populates cache
rninja --no-daemon

# Second build - uses cache
rninja --no-daemon
```

The cache is independent of the daemon.

## CI Configuration Examples

### GitHub Actions

```yaml
steps:
  - name: Build
    run: rninja --no-daemon
```

### GitLab CI

```yaml
build:
  script:
    - rninja --no-daemon
```

### Docker Build

```dockerfile
FROM rust:latest

COPY . /app
WORKDIR /app

# No daemon in container
RUN rninja --no-daemon
```

### Makefile Integration

```makefile
.PHONY: build
build:
	rninja --no-daemon
```

## Comparison with Daemon Mode

### Process Model

**Daemon Mode:**
```
[rninja CLI] → [daemon process] → [build execution]
     ↓              ↓
   exits        persists
```

**Single-Shot Mode:**
```
[rninja process] → [build execution]
         ↓
       exits
```

### Resource Usage

**Daemon Mode:**

- Daemon process always running
- Consumes memory for cached state
- Faster subsequent builds

**Single-Shot Mode:**

- No persistent process
- Memory freed after build
- Consistent startup time

## Troubleshooting

### Build Slower in Single-Shot

Expected - daemon provides caching benefits:

```bash
# If startup time matters, use daemon
rninja  # Let daemon spawn
```

### Cache Not Working

Cache should work in both modes:

```bash
# Check cache
rninja --no-daemon -t cache-stats
```

### Want Daemon But Getting Single-Shot

Check for environment variables:

```bash
env | grep RNINJA_NO_DAEMON
unset RNINJA_NO_DAEMON
```

## Best Practices

### Use Single-Shot When:

- Running in containers
- CI single-job runners
- Debugging
- Scripts requiring predictability

### Use Daemon When:

- Interactive development
- Multiple builds per session
- IDE integration
- Fast iteration cycles

### Default Recommendation

Let rninja decide:

```bash
rninja  # Uses daemon if beneficial
```

Specify only when you have a specific need:

```bash
rninja --no-daemon  # When daemon is definitely not needed
```
