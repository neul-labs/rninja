---
title: Configuration Files
description: Configuration file format and locations
tags:
  - user-guide
  - configuration
---

# Configuration Files

rninja uses TOML configuration files for persistent settings.

## File Locations

Configuration files are loaded from these locations, in order of precedence:

| Location | Scope | Priority |
|----------|-------|----------|
| `.rninjarc` | Project | Highest |
| `~/.rninjarc` | User | Medium |
| `~/.config/rninja/config.toml` | User (XDG) | Lowest |

The first file found is used. Files are not merged.

## File Format

Configuration files use TOML format:

```toml
# rninja configuration file

[build]
jobs = 0
keep_going = 1
explain = false
default_targets = []

[cache]
enabled = true
mode = "auto"
# directory = "/custom/cache/path"
# max_size = 10737418240

[output]
verbose = false
stats = false
color = "auto"
# trace_file = "build_trace.json"
```

## Build Section

```toml
[build]
```

### `jobs`

Number of parallel jobs. `0` means use all CPU cores.

```toml
jobs = 0      # All cores (default)
jobs = 8      # 8 parallel jobs
jobs = 1      # Sequential
```

### `keep_going`

How many failures to allow before stopping.

```toml
keep_going = 1    # Stop on first failure (default)
keep_going = 0    # Never stop
keep_going = 5    # Stop after 5 failures
```

### `explain`

Whether to explain why targets are being rebuilt.

```toml
explain = false   # Default
explain = true    # Always explain
```

### `default_targets`

Default targets when none specified on command line.

```toml
default_targets = []          # Use manifest defaults
default_targets = ["all"]     # Build 'all' target
default_targets = ["foo", "bar"]  # Build specific targets
```

## Cache Section

```toml
[cache]
```

### `enabled`

Enable or disable the build cache.

```toml
enabled = true    # Default
enabled = false   # Disable caching
```

### `mode`

Cache operation mode.

```toml
mode = "local"    # Local cache only
mode = "remote"   # Remote cache only (fail if unavailable)
mode = "auto"     # Try remote, fall back to local (default)
```

### `directory`

Custom cache directory location.

```toml
# Default: ~/.cache/rninja
directory = "/path/to/custom/cache"
```

### `max_size`

Maximum cache size in bytes.

```toml
# Default: 10GB
max_size = 10737418240

# Examples:
max_size = 5368709120     # 5GB
max_size = 1073741824     # 1GB
```

### `daemon_socket`

Custom daemon socket path.

```toml
# Default: /tmp/rninja-daemon.sock
daemon_socket = "/custom/path/rninja.sock"
```

## Output Section

```toml
[output]
```

### `verbose`

Show all command lines by default.

```toml
verbose = false   # Default
verbose = true    # Always verbose
```

### `stats`

Show build statistics at the end.

```toml
stats = false     # Default
stats = true      # Always show stats
```

### `color`

Color output mode.

```toml
color = "auto"    # Detect terminal (default)
color = "always"  # Force colors
color = "never"   # No colors
```

### `trace_file`

Always write trace to this file.

```toml
# Default: not set (only with --trace flag)
trace_file = "build_trace.json"
```

## Complete Example

```toml
# ~/.config/rninja/config.toml
# rninja configuration for development workstation

[build]
# Use all CPU cores
jobs = 0

# Stop on first failure during development
keep_going = 1

# Don't explain by default (use -d explain when needed)
explain = false

# No default targets (use manifest defaults)
default_targets = []

[cache]
# Enable caching
enabled = true

# Use remote cache if available, fall back to local
mode = "auto"

# Store cache in home directory
# directory = "~/.cache/rninja"

# Limit cache to 5GB
max_size = 5368709120

[output]
# Quiet by default
verbose = false

# Show stats after builds
stats = true

# Auto-detect color support
color = "auto"

# Don't always generate traces
# trace_file = ""
```

## Project-Specific Configuration

Create `.rninjarc` in your project root:

```toml
# .rninjarc
# Project-specific rninja configuration

[build]
# This project needs limited parallelism
jobs = 4

# Build 'test' target by default
default_targets = ["test"]

[cache]
# Project uses remote cache
mode = "auto"

[output]
# Verbose for this project
verbose = true
```

## Generating Configuration

Generate a sample config file:

```bash
# Print sample config
rninja -t config -v

# Save to file
rninja -t config -v > ~/.config/rninja/config.toml
```

## Validating Configuration

If a config file has errors, rninja will use defaults and log a warning.

Test your config:

```bash
# Show which config file is loaded
rninja -t config

# Verbose shows parsed values
rninja -t config -v
```

## Tips

### Use Project Configs Sparingly

Project configs (`.rninjarc`) affect all developers. Use for:

- Project-specific build settings
- Required parallelism limits
- Default targets

Avoid for:

- Personal preferences (use `~/.rninjarc`)
- Machine-specific paths

### Environment Variables Override

Remember: environment variables override config files:

```bash
# config has jobs = 4
export RNINJA_BUILD_JOBS=8
rninja  # Uses 8 jobs
```

### Version Control

Consider adding `.rninjarc` to `.gitignore` if developers need different settings, or commit it if settings should be shared.
