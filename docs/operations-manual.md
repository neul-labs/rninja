# rninja Operations Manual

Comprehensive reference for operating rninja in development and production environments.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Installation](#installation)
3. [Configuration Reference](#configuration-reference)
4. [CLI Reference](#cli-reference)
5. [Daemon Operations](#daemon-operations)
6. [Cache Operations](#cache-operations)
7. [Monitoring](#monitoring)
8. [Troubleshooting](#troubleshooting)
9. [Performance Tuning](#performance-tuning)
10. [Security](#security)

---

## Architecture Overview

### Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                         rninja Ecosystem                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐     ┌──────────────────┐     ┌─────────────────┐ │
│  │   rninja     │     │  rninja-daemon   │     │  rninja-cached  │ │
│  │   (CLI)      │◄───►│  (local daemon)  │◄───►│  (cache server) │ │
│  └──────────────┘     └──────────────────┘     └─────────────────┘ │
│         │                      │                        │           │
│         │                      ▼                        ▼           │
│         │              ┌──────────────┐         ┌─────────────┐    │
│         │              │ Local Cache  │         │ Blob Store  │    │
│         │              │   (sled)     │         │  (disk/NFS) │    │
│         │              └──────────────┘         └─────────────┘    │
│         │                                                          │
│         ▼                                                          │
│  ┌──────────────┐                                                  │
│  │ build.ninja  │                                                  │
│  │   (input)    │                                                  │
│  └──────────────┘                                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Descriptions

| Component | Purpose | Process Model |
|-----------|---------|---------------|
| `rninja` | CLI client, parses args, connects to daemon | Short-lived |
| `rninja-daemon` | Build execution, caching, file watching | Long-running |
| `rninja-cached` | Remote cache server | Long-running |

### Data Flow

1. User runs `rninja [targets]`
2. CLI connects to daemon (auto-spawns if needed)
3. Daemon checks local cache for hits
4. On miss, optionally queries remote cache
5. Builds uncached targets
6. Stores results in local (and optionally remote) cache
7. Returns exit code to CLI

---

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/example/rninja.git
cd rninja

# Build release binaries
cargo build --release

# Install
sudo cp target/release/rninja /usr/local/bin/
sudo cp target/release/rninja-daemon /usr/local/bin/
sudo cp target/release/rninja-cached /usr/local/bin/
```

### Package Managers

```bash
# Cargo (Rust)
cargo install rninja

# Homebrew (macOS)
brew install rninja

# APT (Debian/Ubuntu)
sudo apt install rninja

# DNF (Fedora/RHEL)
sudo dnf install rninja
```

### Verify Installation

```bash
# Check version
rninja --version

# Run self-test
rninja -t list  # List available tools

# Verify Ninja compatibility
echo 'rule touch
  command = touch $out
build test.txt: touch
' > build.ninja
rninja
ls test.txt  # Should exist
rm build.ninja test.txt
```

---

## Configuration Reference

### Configuration File Locations

| Platform | User Config | System Config |
|----------|-------------|---------------|
| Linux | `~/.config/rninja/config.toml` | `/etc/rninja/config.toml` |
| macOS | `~/Library/Application Support/rninja/config.toml` | `/etc/rninja/config.toml` |
| Windows | `%APPDATA%\rninja\config.toml` | `C:\ProgramData\rninja\config.toml` |

### Complete Configuration Reference

```toml
# ~/.config/rninja/config.toml

#
# General Settings
#
[general]
# Default parallelism (0 = auto-detect CPU cores)
parallelism = 0

# Verbose output by default
verbose = false

# Keep going on errors
keep_going = false

#
# Daemon Settings
#
[daemon]
# Enable daemon mode (auto-spawn)
enabled = true

# Custom socket path (default: platform-specific)
# socket = "/tmp/rninja-daemon.sock"

# Auto-shutdown after idle (seconds, 0 = never)
idle_timeout = 3600

# Maximum concurrent builds
max_concurrent_builds = 4

#
# Local Cache Settings
#
[cache]
# Enable local caching
enabled = true

# Cache directory (default: platform-specific)
# dir = "/home/user/.cache/rninja"

# Maximum cache size in bytes (default: 10GB)
max_size = 10737418240

# Garbage collection settings
[cache.gc]
enabled = true
interval_secs = 3600
max_age_days = 30

#
# Remote Cache Settings
#
[cache.remote]
# Enable remote cache
enabled = false

# Cache mode: "local", "remote", "auto"
mode = "auto"

# Socket connection (for local server)
# socket = "/run/rninja/cache.sock"

# TCP connection (for remote server)
# host = "cache.example.com"
# port = 9999

# Authentication token
# auth_token = "your-secret-token"

# Timeouts and retries
timeout_ms = 5000
retry_count = 3
retry_delay_ms = 1000

#
# Logging Settings
#
[logging]
# Log level: "error", "warn", "info", "debug", "trace"
level = "info"

# Log format: "text", "json"
format = "text"

# Log file (optional)
# file = "/var/log/rninja/rninja.log"

#
# Tracing Settings
#
[tracing]
# Enable build tracing
enabled = false

# Trace output file
# file = "build_trace.json"

# Format: "chrome" (Chrome tracing format)
format = "chrome"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_CACHE_ENABLED` | Enable/disable cache (0/1) | `1` |
| `RNINJA_CACHE_DIR` | Local cache directory | Platform-specific |
| `RNINJA_CACHE_MODE` | Cache mode (local/remote/auto) | `local` |
| `RNINJA_CACHE_SOCKET` | Remote cache socket path | None |
| `RNINJA_CACHE_HOST` | Remote cache hostname | None |
| `RNINJA_CACHE_PORT` | Remote cache port | `9999` |
| `RNINJA_AUTH_TOKEN` | Authentication token | None |
| `RNINJA_DAEMON_SOCKET` | Daemon socket path | Platform-specific |
| `RNINJA_LOG_LEVEL` | Log level | `info` |
| `RUST_LOG` | Detailed logging control | None |

### Configuration Precedence

1. Command-line arguments (highest priority)
2. Environment variables
3. User config file
4. System config file
5. Built-in defaults (lowest priority)

---

## CLI Reference

### Basic Usage

```bash
rninja [OPTIONS] [TARGETS...]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--version` | `-v` | Print version |
| `--help` | `-h` | Print help |
| `--verbose` | | Verbose output |
| `--quiet` | | Suppress output |
| `--jobs N` | `-j N` | Parallel jobs (0=auto) |
| `--keep-going` | `-k` | Continue on errors |
| `--dry-run` | `-n` | Don't execute commands |
| `--explain` | | Explain why targets rebuild |
| `--tool NAME` | `-t NAME` | Run subtool |
| `--no-daemon` | | Single-shot mode |
| `--daemon-socket PATH` | | Custom daemon socket |
| `-f FILE` | | Use FILE as build file |
| `-C DIR` | | Change to DIR before building |

### Subtools (`-t`)

| Tool | Description |
|------|-------------|
| `list` | List all available tools |
| `targets` | List all targets |
| `rules` | List all rules |
| `commands` | List all build commands |
| `deps` | Show target dependencies |
| `graph` | Output graphviz dependency graph |
| `query` | Query the build graph |
| `clean` | Remove built files |
| `cleandead` | Remove stale outputs |
| `restat` | Update build log timestamps |
| `recompact` | Compact the build log |
| `browse` | Start a web browser for browsing |
| `compdb` | Output compilation database |
| `cache-stats` | Show cache statistics |
| `cache-gc` | Run cache garbage collection |
| `cache-health` | Check cache server health |
| `daemon-status` | Show daemon status |
| `daemon-stop` | Stop the daemon |

### Examples

```bash
# Build default target
rninja

# Build specific targets
rninja foo.o bar.o

# Parallel build with 8 jobs
rninja -j 8

# Dry run (show commands without executing)
rninja -n

# Continue building on errors
rninja -k

# Verbose output with explanations
rninja --verbose --explain

# Use different build file
rninja -f other.ninja

# Build in different directory
rninja -C /path/to/project

# Single-shot mode (no daemon)
rninja --no-daemon

# Query dependencies
rninja -t deps foo.o

# Generate compilation database
rninja -t compdb > compile_commands.json

# Show cache statistics
rninja -t cache-stats
```

---

## Daemon Operations

### Daemon Lifecycle

The daemon is automatically managed by the CLI:

```bash
# First build auto-spawns daemon
rninja

# Subsequent builds reuse daemon
rninja  # Much faster (cached manifest)

# Check daemon status
rninja -t daemon-status

# Stop daemon
rninja -t daemon-stop
```

### Manual Daemon Control

```bash
# Start daemon explicitly
rninja-daemon --socket /tmp/rninja.sock &

# Or with custom cache directory
rninja-daemon \
    --socket /tmp/rninja.sock \
    --cache-dir /var/cache/rninja

# Connect to specific daemon
rninja --daemon-socket /tmp/rninja.sock
```

### Daemon Status Output

```
Daemon Status:
  Version: 0.1.0
  Socket: /tmp/rninja-1000/daemon.sock
  Uptime: 2h 34m 12s
  Active builds: 1
  Cached manifests: 3
  Cache stats:
    Local hits: 1,234
    Local misses: 567
    Remote hits: 890
    Hit rate: 78.5%
```

### Socket Locations

| Platform | Default Socket |
|----------|----------------|
| Linux | `/tmp/rninja-{uid}/daemon.sock` |
| macOS | `/tmp/rninja-{uid}/daemon.sock` |
| Windows | `\\.\pipe\rninja-{username}` |

---

## Cache Operations

### Cache Modes

| Mode | Description |
|------|-------------|
| `local` | Only use local disk cache |
| `remote` | Only use remote cache server |
| `auto` | Try remote, fall back to local |

### Cache Statistics

```bash
rninja -t cache-stats
```

Output:
```
Cache Statistics:
  Mode: auto
  Local cache:
    Directory: /home/user/.cache/rninja
    Size: 2.3 GB / 10 GB (23%)
    Entries: 12,345
    Hits: 8,234
    Misses: 4,111
    Hit rate: 66.7%
  Remote cache:
    Server: cache.internal:9999
    Status: connected
    Hits: 2,456
    Misses: 1,234
    Avg latency: 12ms
  Combined hit rate: 78.5%
```

### Garbage Collection

```bash
# Manual GC
rninja -t cache-gc

# GC with custom max age
rninja -t cache-gc --max-age 7  # Delete entries older than 7 days

# Aggressive GC (reduce to 50% capacity)
rninja -t cache-gc --target-size 50%
```

### Cache Health Check

```bash
rninja -t cache-health
```

Output:
```
Cache Health:
  Local: OK
    - Directory writable: yes
    - Disk space: 50 GB available
    - Database: healthy
  Remote: OK
    - Connection: established
    - Latency: 8ms
    - Server version: 0.1.0
```

### Cache Invalidation

```bash
# Clear specific target from cache
rninja -t cache-clear foo.o

# Clear all cache
rm -rf ~/.cache/rninja/*

# Invalidate remote cache entry
curl -X DELETE "http://cache:9999/v1/entries/$(rninja -t hash foo.o)"
```

---

## Monitoring

### Prometheus Metrics

The daemon and cache server expose Prometheus metrics:

```bash
# Daemon metrics (when enabled)
curl http://localhost:9998/metrics

# Cache server metrics
curl http://localhost:9999/metrics
```

### Key Metrics

| Metric | Description |
|--------|-------------|
| `rninja_builds_total` | Total builds executed |
| `rninja_build_duration_seconds` | Build duration histogram |
| `rninja_targets_built_total` | Total targets built |
| `rninja_cache_hits_total` | Cache hits |
| `rninja_cache_misses_total` | Cache misses |
| `rninja_cache_size_bytes` | Current cache size |
| `rninja_active_connections` | Active client connections |
| `rninja_request_duration_seconds` | Request latency histogram |

### Grafana Dashboard

Import the provided dashboard from `monitoring/grafana-dashboard.json`:

```json
{
  "dashboard": {
    "title": "rninja Build Metrics",
    "panels": [
      {
        "title": "Build Duration",
        "type": "graph",
        "targets": [
          {"expr": "histogram_quantile(0.95, rate(rninja_build_duration_seconds_bucket[5m]))"}
        ]
      },
      {
        "title": "Cache Hit Rate",
        "type": "gauge",
        "targets": [
          {"expr": "rate(rninja_cache_hits_total[5m]) / (rate(rninja_cache_hits_total[5m]) + rate(rninja_cache_misses_total[5m]))"}
        ]
      }
    ]
  }
}
```

### Structured Logging

Enable JSON logging for log aggregation:

```toml
[logging]
format = "json"
```

Output:
```json
{"timestamp":"2024-01-15T10:30:45Z","level":"info","message":"Build started","target":"foo.o","session":"abc123"}
{"timestamp":"2024-01-15T10:30:46Z","level":"info","message":"Cache hit","target":"bar.o","source":"local"}
```

### Build Tracing

Generate Chrome tracing format for detailed analysis:

```bash
rninja --trace build_trace.json

# Open in Chrome: chrome://tracing
```

---

## Troubleshooting

### Common Issues

#### Build Fails with "No such file or directory"

```bash
# Check if build.ninja exists
ls -la build.ninja

# Check current directory
pwd

# Use -C to specify directory
rninja -C /path/to/project
```

#### Daemon Connection Failed

```bash
# Check daemon status
rninja -t daemon-status

# Check socket permissions
ls -la /tmp/rninja-*/

# Restart daemon
rninja -t daemon-stop
rninja  # Auto-spawns new daemon

# Use single-shot mode as fallback
rninja --no-daemon
```

#### Cache Not Working

```bash
# Verify cache is enabled
echo $RNINJA_CACHE_ENABLED  # Should be 1 or unset

# Check cache health
rninja -t cache-health

# Check cache directory permissions
ls -la ~/.cache/rninja/

# Enable debug logging
RUST_LOG=debug rninja
```

#### Remote Cache Connection Refused

```bash
# Test connectivity
nc -zv cache.internal 9999

# Check authentication
curl -H "Authorization: Bearer $RNINJA_AUTH_TOKEN" http://cache.internal:9999/health

# Fall back to local cache
export RNINJA_CACHE_MODE=local
```

#### Build Hangs

```bash
# Check for blocked commands
ps aux | grep rninja

# Check daemon activity
rninja -t daemon-status

# Kill and restart
pkill -f rninja
rninja --no-daemon  # Test without daemon
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug rninja --verbose

# Trace specific components
RUST_LOG=rninja::cache=trace,rninja::executor=debug rninja

# Log to file
RUST_LOG=debug rninja 2>&1 | tee build.log
```

### Diagnostic Commands

```bash
# Full diagnostic report
rninja -t diagnostics

# Check Ninja compatibility
rninja -t ninja-compat

# Verify manifest parsing
rninja -t parse-check

# Dump internal state
rninja -t debug-state
```

---

## Performance Tuning

### Parallelism

```bash
# Auto-detect (default)
rninja -j 0

# Match CPU cores
rninja -j $(nproc)

# Oversubscribe for I/O-bound builds
rninja -j $(($(nproc) * 2))

# Limit for memory-constrained systems
rninja -j 4
```

### Cache Optimization

```toml
[cache]
# Increase cache size for large projects
max_size = 53687091200  # 50GB

# More aggressive GC
[cache.gc]
interval_secs = 1800  # Every 30 minutes
max_age_days = 14     # Keep 2 weeks
```

### Remote Cache Tuning

```toml
[cache.remote]
# Increase timeout for slow networks
timeout_ms = 10000

# More retries for unreliable connections
retry_count = 5

# Use auto mode for resilience
mode = "auto"
```

### Daemon Tuning

```toml
[daemon]
# More concurrent builds for CI servers
max_concurrent_builds = 8

# Longer idle timeout for busy systems
idle_timeout = 7200  # 2 hours
```

### Storage Recommendations

| Scenario | Storage Type | Notes |
|----------|--------------|-------|
| Development laptop | SSD | Best performance |
| CI server | NVMe SSD | Handle high throughput |
| Shared cache server | SSD RAID | Balance capacity and speed |
| Large team | NFS on SSD | Shared, but higher latency |

---

## Security

### Authentication

Always use authentication in production:

```bash
# Generate secure token
TOKEN=$(openssl rand -hex 32)

# Server
rninja-cached --auth-token "$TOKEN"

# Client
export RNINJA_AUTH_TOKEN="$TOKEN"
```

### Network Security

For remote cache over untrusted networks, use TLS via reverse proxy:

```nginx
# nginx.conf
server {
    listen 443 ssl;
    server_name cache.example.com;

    ssl_certificate /etc/ssl/certs/cache.crt;
    ssl_certificate_key /etc/ssl/private/cache.key;

    location / {
        proxy_pass http://localhost:9999;
        proxy_set_header Host $host;
    }
}
```

### Filesystem Permissions

```bash
# Daemon socket
chmod 700 /tmp/rninja-$UID/
chmod 600 /tmp/rninja-$UID/daemon.sock

# Cache directory
chmod 700 ~/.cache/rninja/

# Cache server directory
sudo chown rninja:rninja /var/cache/rninja
sudo chmod 750 /var/cache/rninja
```

### Systemd Hardening

```ini
[Service]
# Run as unprivileged user
User=rninja
Group=rninja

# Filesystem restrictions
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/cache/rninja /run/rninja

# Network restrictions (local only)
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6

# Capabilities
CapabilityBoundingSet=
AmbientCapabilities=
```

### Secrets Management

Never commit tokens to version control:

```bash
# Use environment variables
export RNINJA_AUTH_TOKEN="$(cat /run/secrets/rninja-token)"

# Or use a secrets manager
export RNINJA_AUTH_TOKEN="$(vault kv get -field=token secret/rninja)"
```

---

## API Reference

### Cache Server REST API

#### Health Check
```
GET /health
Response: 200 OK
```

#### Metrics
```
GET /metrics
Response: Prometheus text format
```

#### Get Cache Entry
```
GET /v1/entries/{hash}
Headers:
  Authorization: Bearer {token}
Response: 200 OK with blob data, or 404 Not Found
```

#### Put Cache Entry
```
PUT /v1/entries/{hash}
Headers:
  Authorization: Bearer {token}
  Content-Type: application/octet-stream
Body: blob data
Response: 201 Created
```

#### Delete Cache Entry
```
DELETE /v1/entries/{hash}
Headers:
  Authorization: Bearer {token}
Response: 204 No Content
```

#### Admin: Drain Mode
```
POST /admin/drain
Response: 200 OK
```

#### Admin: Statistics
```
GET /admin/stats
Response: JSON statistics
```

### IPC Protocol (NNG)

The daemon uses MessagePack-encoded messages over NNG REQ/REP sockets. See `src/daemon/protocol.rs` for message definitions.

---

## Appendix

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Build failed |
| 2 | Invalid arguments |
| 3 | Build file not found |
| 4 | Parse error |
| 5 | Daemon connection failed |
| 6 | Cache error |

### File Formats

#### Build Log (`.ninja_log`)
```
# ninja log v5
0       100     1234567890      foo.o   abc123def456
100     200     1234567891      bar.o   def456abc789
```

#### Deps Log (`.ninja_deps`)
Binary format tracking implicit dependencies.

### Compatibility Notes

rninja aims for 100% compatibility with Ninja. Known differences:
- Additional command-line options (`--no-daemon`, etc.)
- Additional tools (`cache-*`, `daemon-*`)
- Different internal file formats (cache database)

Ninja-generated build files work without modification.
