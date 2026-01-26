---
title: Configuration Reference
description: Complete rninja configuration options
tags:
  - reference
  - configuration
---

# Configuration Reference

Complete reference for all rninja configuration options.

## Configuration Files

rninja reads configuration from (in order of priority):

1. Command-line arguments (highest)
2. Environment variables
3. Project config: `.rninja/config.toml`
4. User config: `~/.config/rninja/config.toml`
5. System config: `/etc/rninja/config.toml` (lowest)

## TOML Configuration

### General Settings

```toml
[general]
# Default number of parallel jobs (0 = auto-detect CPUs)
jobs = 0

# Default targets when none specified
default_targets = []

# Verbose output
verbose = false

# Keep going on failures (0 = stop on first failure)
keep_going = 1
```

### Cache Settings

```toml
[cache]
# Cache mode: "auto", "local", "remote", "off"
mode = "auto"

# Enable cache reads
read = true

# Enable cache writes
write = true

# Local cache directory
local_dir = "~/.cache/rninja"

# Maximum local cache size (bytes, supports K/M/G suffixes)
max_size = "10G"

# Time-to-live for cache entries (seconds, 0 = forever)
ttl = 0

# Rules to exclude from caching (glob patterns)
exclude_rules = []

# Compress cached artifacts
compress = true

# Compression level (1-9)
compress_level = 6
```

### Remote Cache Settings

```toml
[cache.remote]
# Remote cache server URL
url = ""

# Authentication token
token = ""

# Connection timeout (milliseconds)
connect_timeout = 5000

# Request timeout (milliseconds)
request_timeout = 30000

# Retry failed requests
retry = true

# Maximum retries
max_retries = 3

# Retry delay (milliseconds)
retry_delay = 1000

# Fall back to local on remote failure
fallback_local = true
```

### Daemon Settings

```toml
[daemon]
# Daemon mode: "auto", "on", "off"
mode = "auto"

# Socket path (default: auto-generated)
socket_path = ""

# Idle timeout before daemon exits (seconds)
idle_timeout = 300

# Maximum memory usage (bytes, 0 = unlimited)
max_memory = 0

# Log level: "error", "warn", "info", "debug", "trace"
log_level = "info"
```

### Build Settings

```toml
[build]
# Enable smart scheduling
smart_schedule = true

# Track header dependencies
track_headers = true

# Use response files for long commands
use_rsp_files = true

# Response file threshold (bytes)
rsp_threshold = 8192
```

### Telemetry Settings

```toml
[telemetry]
# Enable build metrics
enabled = true

# Metrics output format: "json", "prometheus"
format = "json"

# Metrics output path
output = ""
```

## Complete Example

```toml
# ~/.config/rninja/config.toml

[general]
jobs = 0
verbose = false
keep_going = 1

[cache]
mode = "auto"
read = true
write = true
local_dir = "~/.cache/rninja"
max_size = "20G"
ttl = 604800  # 1 week
compress = true
compress_level = 6
exclude_rules = ["phony", "install_*"]

[cache.remote]
url = "tcp://cache.example.com:9876"
token = "${RNINJA_CACHE_TOKEN}"
connect_timeout = 5000
request_timeout = 30000
retry = true
max_retries = 3
fallback_local = true

[daemon]
mode = "auto"
idle_timeout = 300
log_level = "info"

[build]
smart_schedule = true
track_headers = true

[telemetry]
enabled = false
```

## Project Configuration

Project-specific settings in `.rninja/config.toml`:

```toml
# .rninja/config.toml (in project root)

[cache]
# Exclude test-related rules from caching
exclude_rules = ["test_*", "*_test"]

[cache.remote]
# Project uses shared cache server
url = "tcp://cache.internal:9876"
token = "${PROJECT_CACHE_TOKEN}"
```

## Server Configuration

For rninja-cached server:

```toml
# /etc/rninja/cached.toml

[server]
# Bind address
bind = "0.0.0.0:9876"

# Maximum connections
max_connections = 1000

# Worker threads (0 = auto)
workers = 0

[storage]
# Storage backend: "filesystem", "s3"
backend = "filesystem"

# Storage path
path = "/var/cache/rninja"

# Maximum storage size
max_size = "100G"

[auth]
# Authentication mode: "none", "token", "mtls"
mode = "token"

# Valid tokens (or path to token file)
tokens = ["/etc/rninja/tokens.txt"]

[tls]
# Enable TLS
enabled = false

# Certificate file
cert = "/etc/rninja/server.crt"

# Key file
key = "/etc/rninja/server.key"

# CA for client verification (mTLS)
ca = ""
```

## Value Types

### Size Values

Size values support suffixes:

| Suffix | Meaning |
|--------|---------|
| `K` | Kilobytes (1024) |
| `M` | Megabytes (1024²) |
| `G` | Gigabytes (1024³) |
| `T` | Terabytes (1024⁴) |

```toml
max_size = "10G"      # 10 gigabytes
max_size = 10737418240  # Same in bytes
```

### Duration Values

Duration values in seconds (or with suffix):

| Suffix | Meaning |
|--------|---------|
| `s` | Seconds |
| `m` | Minutes |
| `h` | Hours |
| `d` | Days |

```toml
ttl = 86400    # 1 day in seconds
ttl = "1d"     # Same with suffix
```

### Environment Variables

Use `${VAR}` for environment variable substitution:

```toml
[cache.remote]
token = "${RNINJA_CACHE_TOKEN}"
url = "${CACHE_URL:-tcp://localhost:9876}"  # With default
```

## Validation

Check configuration validity:

```bash
# Validate config and show effective settings
rninja --dump-config

# Check specific config file
rninja --config-check ~/.config/rninja/config.toml
```

## Precedence

When the same option is set in multiple places:

1. **Command line** wins over everything
2. **Environment variable** wins over config files
3. **Project config** wins over user config
4. **User config** wins over system config

Example:

```bash
# Config file has jobs = 4
# Environment has RNINJA_JOBS=8
# Command line has -j16

# Result: 16 jobs (command line wins)
```
