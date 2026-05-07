---
title: Diagnostics
description: rninja diagnostic commands and checks
tags:
  - troubleshooting
---

# Diagnostics

Diagnostic tools for checking rninja health and configuration.

## System Diagnostics

### Version Information

```bash
rninja --version
```

Output:

```
rninja 0.1.1
  Built with Rust 1.75.0
  Features: cache, daemon, remote
  Git: abc1234
```

### Configuration Check

```bash
rninja --dump-config
```

Shows effective configuration after merging all sources:

```toml
[general]
jobs = 8
verbose = false

[cache]
mode = "auto"
local_dir = "/home/user/.cache/rninja"
max_size = 10737418240

[daemon]
mode = "auto"
socket_path = "/tmp/rninja-user-abc123.sock"
```

### Environment Check

```bash
env | grep -E '^(RNINJA|NINJA|CC|CXX)'
```

Check for variables that might affect builds.

## Build File Diagnostics

### Parse Check

```bash
rninja -n
```

Parses build file without executing. Errors indicate syntax issues.

### Target Query

```bash
# Query specific target
rninja -t query foo.o
```

Output:

```
foo.o:
  input: cc
  outputs:
    foo.o
  inputs:
    foo.cpp
  deps:
    foo.h
    bar.h
```

### List All Targets

```bash
# All targets
rninja -t targets all

# By rule
rninja -t targets rule cc
```

### Dependency Graph

```bash
# Generate graph
rninja -t graph foo.o | dot -Tpng -o deps.png
```

Visual representation of dependencies.

### List Rules

```bash
rninja -t rules
```

## Cache Diagnostics

### Cache Health

```bash
rninja -t cache-health
```

Output:

```
Cache Health Check:
  Index database: OK
  Blob storage: OK
  Integrity: OK (12345/12345 entries valid)
  Disk space: OK (5.2 GB / 10 GB)
  Permissions: OK
```

### Cache Statistics

```bash
rninja -t cache-stats
```

Detailed cache statistics and hit rates.

### Verify Cache Integrity

```bash
rninja -t cache-verify
```

Checks all cache entries for corruption.

### List Cache Entries

```bash
# Recent entries
rninja -t cache-list --limit 10

# Entries for target
rninja -t cache-list --target foo.o
```

## Daemon Diagnostics

### Daemon Status

```bash
rninja -t daemon-status
```

Output:

```
Daemon Status:
  Status: running
  PID: 12345
  Socket: /tmp/rninja-user-abc.sock
  Uptime: 2h 30m
  Memory: 128 MB
  Builds completed: 47
  Currently building: 1
```

### Daemon Ping

```bash
rninja -t daemon-ping
```

Tests daemon connectivity.

### Daemon Metrics

```bash
rninja -t daemon-metrics
```

Prometheus-format metrics from daemon.

## Network Diagnostics

### Remote Cache Check

```bash
# Test connection
rninja -t remote-check
```

Output:

```
Remote Cache Check:
  URL: tcp://cache.example.com:9876
  Connection: OK (5ms latency)
  Authentication: OK
  Server version: 0.1.1
  Server health: OK
```

### Latency Test

```bash
rninja -t remote-latency
```

Output:

```
Remote Cache Latency:
  Connect: 2ms
  GET (miss): 5ms
  PUT (1KB): 8ms
  GET (hit, 1KB): 6ms
  PUT (1MB): 45ms
  GET (hit, 1MB): 38ms
```

## Build Diagnostics

### Dependency Check

```bash
# Check for missing deps
rninja -t deps --check
```

### Compilation Database

```bash
# Generate and verify
rninja -t compdb > compile_commands.json
```

### Clean State

```bash
# Show what would be cleaned
rninja -t clean -n

# Show stale outputs
rninja -t cleandead -n
```

## Log Analysis

### Ninja Log

```bash
# Show log entries
head .ninja_log

# Check log format
rninja -t recompact -n
```

### Build History

```bash
# Recent builds (if tracking enabled)
rninja -t history
```

## Automated Diagnostic Script

Create a diagnostic report:

```bash
#!/bin/bash
# rninja-diagnostic.sh

echo "=== rninja Diagnostics ==="
echo

echo "--- Version ---"
rninja --version
echo

echo "--- Configuration ---"
rninja --dump-config
echo

echo "--- Environment ---"
env | grep -E '^(RNINJA|NINJA)'
echo

echo "--- Cache Health ---"
rninja -t cache-health 2>&1
echo

echo "--- Daemon Status ---"
rninja -t daemon-status 2>&1
echo

echo "--- Build File ---"
rninja -n 2>&1 | tail -5
echo

echo "--- Disk Space ---"
df -h ~/.cache/rninja 2>/dev/null || echo "Cache dir not found"
echo

echo "=== End Diagnostics ==="
```

Run:

```bash
chmod +x rninja-diagnostic.sh
./rninja-diagnostic.sh > diagnostic-report.txt
```

## Health Check Endpoints

For monitoring:

```bash
# Local daemon
curl http://localhost:9878/health

# Remote cache
curl http://cache.example.com:9877/api/v1/health
```

## Exit Codes

Diagnostic commands exit codes:

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | Some checks failed |
| 2 | Invalid arguments |

```bash
rninja -t cache-health
if [ $? -eq 0 ]; then
    echo "Cache is healthy"
else
    echo "Cache needs attention"
fi
```
