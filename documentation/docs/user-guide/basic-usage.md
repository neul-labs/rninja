---
title: Basic Usage
description: Common usage patterns for rninja
tags:
  - user-guide
  - usage
---

# Basic Usage

This guide covers common rninja usage patterns for everyday development.

## Running Builds

### Default Build

Build all default targets:

```bash
rninja
```

### Build Specific Targets

```bash
# Build a single target
rninja my_target

# Build multiple targets
rninja target1 target2 target3
```

### Build in Different Directory

```bash
# Change to directory first
rninja -C out/Release

# Equivalent to:
cd out/Release && rninja
```

### Use Different Build File

```bash
# Default is build.ninja
rninja -f custom.ninja
```

## Controlling Parallelism

### Set Job Count

```bash
# Use all CPU cores (default when -j0 or no -j)
rninja -j0

# Use 8 parallel jobs
rninja -j8

# Use single job (sequential)
rninja -j1
```

### Respect System Load

Stop spawning new jobs if system load is high:

```bash
# Don't start new jobs if load average > 4.0
rninja -l 4.0
```

## Handling Build Failures

### Stop on First Failure (Default)

```bash
rninja -k1
```

### Continue After Failures

```bash
# Keep going after 5 failures
rninja -k5

# Keep going indefinitely
rninja -k0
```

## Verbose and Debug Output

### Show Commands

```bash
# Show all commands being executed
rninja -v
```

### Explain Rebuilds

```bash
# Show why targets are being rebuilt
rninja -d explain
```

### Show Statistics

```bash
# Show build statistics at the end
rninja -d stats
```

## Dry Run

See what would be built without building:

```bash
rninja -n
```

Example output:

```
[1/3] CC main.o
[2/3] CC greet.o
[3/3] LINK hello
```

## Working with the Cache

### Check Cache Status

```bash
rninja -t cache-stats
```

### Disable Caching Temporarily

```bash
RNINJA_CACHE_ENABLED=0 rninja
```

### Clear Cache

```bash
rninja -t cache-gc
```

## Using Subtools

### List Available Tools

```bash
rninja -t list
```

### Common Subtools

```bash
# Clean build outputs
rninja -t clean

# Show dependencies
rninja -t deps target_name

# Show inputs/outputs for a path
rninja -t query path/to/file

# Generate compilation database
rninja -t compdb > compile_commands.json

# Generate dependency graph
rninja -t graph | dot -Tpng > deps.png
```

See [Subtools Overview](subtools/overview.md) for complete documentation.

## Build Tracing

Generate Chrome trace for performance analysis:

```bash
rninja --trace build_trace.json
```

Open `chrome://tracing` in Chrome and load the file.

## JSON Output

Get machine-readable output for scripting:

```bash
rninja --json
```

## Environment Variables

Common environment variables:

```bash
# Set cache directory
export RNINJA_CACHE_DIR=/path/to/cache

# Disable caching
export RNINJA_CACHE_ENABLED=0

# Set cache mode
export RNINJA_CACHE_MODE=local  # or remote, auto
```

See [Environment Variables](configuration/environment-variables.md) for complete list.

## Examples by Workflow

### Development Workflow

```bash
# Edit code...

# Incremental build
rninja

# Run tests
./run_tests.sh

# Check what changed
rninja -d explain
```

### Clean Build

```bash
# Remove all outputs
rninja -t clean

# Full rebuild (uses cache)
rninja
```

### Release Build

```bash
# Build in release directory
rninja -C out/Release -j0
```

### CI Build

```bash
# Verbose build with all cores
rninja -v -j0

# Or with JSON output
rninja --json
```

### Debugging Build Issues

```bash
# See why things rebuild
rninja -d explain

# Verbose output
rninja -v

# Check dependencies
rninja -t deps problematic_target

# Check target inputs/outputs
rninja -t query problematic_target
```

## Tips and Best Practices

### Use Appropriate Parallelism

- **Development**: Default (`-j0` uses all cores) usually works well
- **CI**: May need limiting if running multiple builds
- **Shared machines**: Use `-l` to respect system load

### Leverage Caching

- Don't disable caching unless debugging
- Run `rninja -t cache-stats` periodically
- Consider remote caching for team builds

### Investigate Slow Builds

1. Generate trace: `rninja --trace trace.json`
2. Open in `chrome://tracing`
3. Look for sequential bottlenecks

### Keep Build Files Clean

- Declare all dependencies
- Use depfile for auto-detected dependencies
- Run `rninja -t cleandead` to remove stale outputs

## Quick Reference

| Task | Command |
|------|---------|
| Build all | `rninja` |
| Build target | `rninja target` |
| Build in dir | `rninja -C dir` |
| Parallel jobs | `rninja -j8` |
| Verbose | `rninja -v` |
| Explain | `rninja -d explain` |
| Dry run | `rninja -n` |
| Clean | `rninja -t clean` |
| Cache stats | `rninja -t cache-stats` |

## Next Steps

<div class="grid cards" markdown>

-   :material-console: [__CLI Reference__](cli-reference.md)

    Complete command-line reference

-   :material-cog: [__Configuration__](configuration/overview.md)

    Customize rninja behavior

-   :material-tools: [__Subtools__](subtools/overview.md)

    All available subtools

</div>
