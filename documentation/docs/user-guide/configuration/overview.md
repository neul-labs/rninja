---
title: Configuration Overview
description: How to configure rninja
tags:
  - user-guide
  - configuration
---

# Configuration Overview

rninja is designed to work out of the box with sensible defaults, but offers extensive configuration options for customization.

## Configuration Methods

rninja can be configured through three methods, in order of precedence (highest to lowest):

1. **Command-line arguments** - Highest priority, per-invocation
2. **Environment variables** - Per-session or CI configuration
3. **Configuration files** - Persistent project or user settings

## Quick Configuration

### Generate Sample Config

Generate a sample configuration file:

```bash
rninja -t config -v
```

This outputs a complete sample configuration you can customize.

### Show Config Paths

See where rninja looks for configuration:

```bash
rninja -t config
```

## Configuration Sections

### Build Configuration

Control build behavior:

```toml
[build]
jobs = 0              # Parallel jobs (0 = CPU count)
keep_going = 1        # Failures before stopping
explain = false       # Show rebuild reasons
default_targets = []  # Default targets to build
```

### Cache Configuration

Configure the build cache:

```toml
[cache]
enabled = true        # Enable caching
mode = "auto"         # local, remote, or auto
directory = ""        # Cache directory (default: ~/.cache/rninja)
max_size = 10737418240  # Max size in bytes (10GB)
```

### Output Configuration

Control output behavior:

```toml
[output]
verbose = false       # Verbose output
stats = false         # Show statistics
color = "auto"        # auto, always, or never
trace_file = ""       # Trace output file
```

## Common Configurations

### Development Machine

```toml
[build]
jobs = 0
explain = false

[cache]
enabled = true
mode = "local"

[output]
verbose = false
color = "auto"
```

### CI/CD Pipeline

```toml
[build]
jobs = 0
keep_going = 0  # Build as much as possible

[cache]
enabled = true
mode = "auto"  # Use remote if available

[output]
verbose = true
stats = true
color = "never"
```

### Team with Remote Cache

```toml
[cache]
enabled = true
mode = "auto"
# Remote settings from environment variables
```

With environment:

```bash
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=your-secret-token
```

## Configuration Precedence

When the same option is set in multiple places:

```
CLI flags > Environment variables > Config files
```

Example:

```bash
# config.toml has jobs = 4
# Environment has RNINJA_JOBS=8

rninja -j2  # Uses 2 (CLI wins)
```

## Viewing Effective Configuration

To see what configuration rninja is using:

```bash
# Show config file locations
rninja -t config

# Verbose shows loaded config
rninja -t config -v
```

## Next Steps

<div class="grid cards" markdown>

-   :material-file-cog: [__Config Files__](config-files.md)

    Configuration file format and locations

-   :material-variable: [__Environment Variables__](environment-variables.md)

    All supported environment variables

-   :material-content-copy: [__Sample Configurations__](samples.md)

    Ready-to-use configuration examples

</div>
