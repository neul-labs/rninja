---
title: CLI Options Reference
description: Complete rninja command-line options
tags:
  - reference
  - cli
---

# CLI Options Reference

Complete reference for all rninja command-line options.

## Synopsis

```bash
rninja [OPTIONS] [TARGETS...]
```

## Core Options

### Target Selection

| Option | Description |
|--------|-------------|
| `TARGETS` | Targets to build (default: all) |
| `-t TOOL` | Run a subtool |

### Build File

| Option | Default | Description |
|--------|---------|-------------|
| `-f FILE` | `build.ninja` | Specify build file |
| `-C DIR` | `.` | Change to directory before building |

### Parallelism

| Option | Default | Description |
|--------|---------|-------------|
| `-j N` | CPU count | Parallel jobs |
| `-l N` | (none) | Load average limit |

### Error Handling

| Option | Default | Description |
|--------|---------|-------------|
| `-k N` | `1` | Keep going until N failures (0 = infinite) |

### Output Control

| Option | Description |
|--------|-------------|
| `-v` | Verbose output (show commands) |
| `-d MODE` | Debug mode (stats, explain, keepdepfile, keeprsp) |
| `-w FLAG` | Warning control |
| `-q` | Quiet mode (no status output) |

### Execution Mode

| Option | Description |
|--------|-------------|
| `-n` | Dry run (don't execute commands) |

## rninja Extensions

### Caching

| Option | Default | Description |
|--------|---------|-------------|
| `--cache` | auto | Enable caching (auto/local/remote/off) |
| `--no-cache` | - | Disable caching |
| `--cache-read-only` | false | Read from cache but don't write |
| `--cache-write-only` | false | Write to cache but don't read |

### Daemon

| Option | Default | Description |
|--------|---------|-------------|
| `--daemon` | auto | Use build daemon (auto/on/off) |
| `--no-daemon` | - | Force single-shot mode |

### Output Format

| Option | Description |
|--------|-------------|
| `--json` | JSON output format |
| `--trace FILE` | Write Chrome trace to FILE |

### Diagnostics

| Option | Description |
|--------|-------------|
| `--version` | Show version |
| `--help` | Show help |
| `--explain` | Explain why targets rebuild |

## Subtools (-t)

### Build Management

```bash
rninja -t clean [TARGETS]     # Remove build outputs
rninja -t cleandead           # Remove stale outputs
rninja -t restat [FILES]      # Update file timestamps in log
rninja -t recompact           # Compact .ninja_log
```

### Queries

```bash
rninja -t query TARGET        # Show target info
rninja -t deps [TARGET]       # Show dependencies
rninja -t targets [RULE]      # List targets
rninja -t commands [TARGET]   # Show build commands
```

### Export

```bash
rninja -t compdb [RULES]      # Generate compile_commands.json
rninja -t graph [TARGET]      # Generate GraphViz dot
```

### Cache (rninja extension)

```bash
rninja -t cache-stats         # Show cache statistics
rninja -t cache-gc            # Run garbage collection
rninja -t cache-clear         # Clear entire cache
rninja -t cache-health        # Check cache health
```

### Daemon (rninja extension)

```bash
rninja -t daemon-status       # Show daemon status
rninja -t daemon-stop         # Stop daemon
rninja -t daemon-start        # Start daemon
```

## Debug Modes (-d)

| Mode | Description |
|------|-------------|
| `stats` | Print build statistics |
| `explain` | Explain what caused rebuilds |
| `keepdepfile` | Don't delete .d files |
| `keeprsp` | Don't delete response files |

```bash
# Example: Show why targets rebuild
rninja -d explain

# Example: Show build statistics
rninja -d stats
```

## Warning Flags (-w)

| Flag | Description |
|------|-------------|
| `dupbuild=warn` | Warn on duplicate rules |
| `dupbuild=err` | Error on duplicate rules |
| `phonycycle=warn` | Warn on phony cycles |
| `phonycycle=err` | Error on phony cycles |

```bash
# Treat duplicate builds as errors
rninja -w dupbuild=err
```

## Option Combinations

### Maximum Performance

```bash
rninja -j$(nproc) --cache=auto
```

### CI/CD Build

```bash
rninja -j4 -k0 --cache=remote --json
```

### Debugging

```bash
rninja -v -d explain --no-cache
```

### Clean Room Build

```bash
rninja --no-cache --no-daemon
```

## Environment Overrides

Many options have environment variable equivalents:

| Option | Environment Variable |
|--------|---------------------|
| `-j N` | `RNINJA_JOBS` |
| `--cache` | `RNINJA_CACHE_MODE` |
| `--daemon` | `RNINJA_DAEMON_MODE` |

See [Environment Variables](environment-variables.md) for the complete list.

## Ninja Compatibility

rninja accepts all standard Ninja options:

```bash
# These work identically in ninja and rninja
-C DIR          # Change directory
-f FILE         # Build file
-j N            # Jobs
-k N            # Keep going
-l N            # Load limit
-n              # Dry run
-v              # Verbose
-d MODE         # Debug
-t TOOL         # Subtool
-w FLAG         # Warning
```

Extended options (`--cache`, `--daemon`, `--json`, `--trace`) are rninja-specific but don't affect compatibility.
