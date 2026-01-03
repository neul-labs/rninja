# Remote Cache Deployment Guide

This guide covers deploying rninja's remote cache server (`rninja-cached`) for shared caching across teams and CI systems.

## Overview

The remote cache allows multiple developers and CI runners to share build artifacts, dramatically reducing build times for teams working on the same codebase.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Developer A │     │ Developer B │     │   CI Runner │
│   rninja    │     │   rninja    │     │   rninja    │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │rninja-cached│
                    │   server    │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Blob Storage│
                    │  (disk/NFS) │
                    └─────────────┘
```

## Quick Start

### Single Server Setup

1. **Build the cache server:**
   ```bash
   cargo build --release
   ```

2. **Start the server:**
   ```bash
   ./target/release/rninja-cached \
       --cache-dir /var/cache/rninja \
       --socket /tmp/rninja-cache.sock
   ```

3. **Configure clients:**
   ```bash
   export RNINJA_CACHE_MODE=remote
   export RNINJA_CACHE_SOCKET=/tmp/rninja-cache.sock
   ```

4. **Run builds:**
   ```bash
   rninja  # Will use remote cache automatically
   ```

## Server Configuration

### Command Line Options

```
rninja-cached [OPTIONS]

Options:
  --cache-dir <PATH>     Directory for cache storage (required)
  --socket <PATH>        Unix socket path for IPC
  --port <PORT>          TCP port for network access (alternative to socket)
  --max-size <BYTES>     Maximum cache size (default: 10GB)
  --gc-interval <SECS>   Garbage collection interval (default: 3600)
  --auth-token <TOKEN>   Required authentication token
  --verbose              Enable verbose logging
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_CACHE_DIR` | Cache storage directory | Required |
| `RNINJA_CACHE_MAX_SIZE` | Maximum cache size in bytes | 10GB |
| `RNINJA_AUTH_TOKEN` | Authentication token | None |

### Configuration File

Create `/etc/rninja/cached.toml`:

```toml
[server]
cache_dir = "/var/cache/rninja"
socket = "/run/rninja/cache.sock"
max_size = 10737418240  # 10GB

[auth]
enabled = true
token = "your-secret-token"

[gc]
enabled = true
interval_secs = 3600
max_age_days = 30

[limits]
max_connections = 100
max_blob_size = 104857600  # 100MB
rate_limit_rps = 1000
```

## Client Configuration

### Environment Variables

```bash
# Enable remote cache
export RNINJA_CACHE_MODE=remote      # or "auto" for fallback

# Socket connection (local server)
export RNINJA_CACHE_SOCKET=/run/rninja/cache.sock

# Or TCP connection (remote server)
export RNINJA_CACHE_HOST=cache.example.com
export RNINJA_CACHE_PORT=9999

# Authentication (if required)
export RNINJA_AUTH_TOKEN=your-secret-token
```

### Configuration File

Add to `~/.config/rninja/config.toml`:

```toml
[cache]
enabled = true
mode = "auto"  # "local", "remote", or "auto"

[cache.remote]
socket = "/run/rninja/cache.sock"
# Or for TCP:
# host = "cache.example.com"
# port = 9999
auth_token = "your-secret-token"
timeout_ms = 5000
retry_count = 3
```

## Deployment Scenarios

### Scenario 1: Local Team Server

For a small team sharing a development server:

```bash
# On the server
sudo mkdir -p /var/cache/rninja /run/rninja
sudo chown $USER:$USER /var/cache/rninja /run/rninja

rninja-cached \
    --cache-dir /var/cache/rninja \
    --socket /run/rninja/cache.sock &

# On developer machines (same server)
export RNINJA_CACHE_SOCKET=/run/rninja/cache.sock
```

### Scenario 2: Dedicated Cache Server

For larger teams with a dedicated cache server:

```bash
# On cache server (cache.internal)
rninja-cached \
    --cache-dir /data/rninja-cache \
    --port 9999 \
    --auth-token "$CACHE_TOKEN" \
    --max-size 107374182400  # 100GB

# On developer machines
export RNINJA_CACHE_HOST=cache.internal
export RNINJA_CACHE_PORT=9999
export RNINJA_AUTH_TOKEN="$CACHE_TOKEN"
```

### Scenario 3: CI Integration

For CI systems (GitHub Actions, GitLab CI, Jenkins):

**GitHub Actions:**
```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    services:
      cache:
        image: rninja/cached:latest
        ports:
          - 9999:9999
    env:
      RNINJA_CACHE_HOST: localhost
      RNINJA_CACHE_PORT: 9999
    steps:
      - uses: actions/checkout@v4
      - run: rninja
```

**GitLab CI:**
```yaml
build:
  services:
    - name: rninja/cached:latest
      alias: cache
  variables:
    RNINJA_CACHE_HOST: cache
    RNINJA_CACHE_PORT: 9999
  script:
    - rninja
```

## Systemd Service

Create `/etc/systemd/system/rninja-cached.service`:

```ini
[Unit]
Description=rninja Remote Cache Server
After=network.target

[Service]
Type=simple
User=rninja
Group=rninja
ExecStart=/usr/local/bin/rninja-cached \
    --cache-dir /var/cache/rninja \
    --socket /run/rninja/cache.sock
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/cache/rninja /run/rninja

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable rninja-cached
sudo systemctl start rninja-cached
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin rninja-cached

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/rninja-cached /usr/local/bin/
EXPOSE 9999
VOLUME /cache
CMD ["rninja-cached", "--cache-dir", "/cache", "--port", "9999"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  rninja-cache:
    build: .
    ports:
      - "9999:9999"
    volumes:
      - cache-data:/cache
    environment:
      - RNINJA_AUTH_TOKEN=${CACHE_TOKEN}
    restart: unless-stopped

volumes:
  cache-data:
```

## Monitoring

### Health Check

```bash
# Check if server is running
curl -s http://localhost:9999/health

# Or via rninja
rninja -t cache-health
```

### Metrics

The server exposes Prometheus metrics at `/metrics`:

```
# Cache hit rate
rninja_cache_hits_total
rninja_cache_misses_total

# Storage
rninja_cache_size_bytes
rninja_cache_entries_total

# Performance
rninja_cache_get_duration_seconds
rninja_cache_put_duration_seconds
```

### Logging

Enable verbose logging for debugging:

```bash
RUST_LOG=debug rninja-cached --verbose ...
```

## Maintenance

### Garbage Collection

The server automatically runs GC based on `--gc-interval`. Manual GC:

```bash
rninja -t cache-gc
```

### Cache Statistics

```bash
rninja -t cache-stats
```

Output:
```
Cache Statistics:
  Total entries: 12,345
  Total size: 5.2 GB
  Hit rate: 78.5%
  Local hits: 8,234
  Remote hits: 4,111
```

### Clearing Cache

```bash
# Clear all cache entries older than 7 days
rninja -t cache-gc --max-age 7

# Clear entire cache
rm -rf /var/cache/rninja/*
```

## Troubleshooting

### Connection Refused

```
Error: failed to connect to cache server
```

**Solutions:**
1. Check server is running: `systemctl status rninja-cached`
2. Check socket/port permissions
3. Verify firewall rules for TCP connections

### Authentication Failed

```
Error: authentication failed
```

**Solutions:**
1. Verify `RNINJA_AUTH_TOKEN` matches server token
2. Check token is set in both client and server

### Cache Miss Despite Existing Entry

**Possible causes:**
1. Different compiler version
2. Different build flags
3. Source file content changed

**Debug:**
```bash
RUST_LOG=debug rninja --verbose
```

### High Latency

**Solutions:**
1. Move cache server closer to clients
2. Increase cache server resources
3. Use local cache as L1 (`RNINJA_CACHE_MODE=auto`)

## Security Considerations

1. **Authentication**: Always use `--auth-token` in production
2. **Network**: Use TLS for remote connections (via reverse proxy)
3. **Isolation**: Run server with minimal privileges
4. **Storage**: Encrypt cache storage for sensitive projects

## Performance Tuning

### Server Side

```toml
[limits]
max_connections = 200      # Increase for more concurrent clients
max_blob_size = 209715200  # 200MB for large artifacts
rate_limit_rps = 2000      # Higher for fast networks
```

### Client Side

```toml
[cache.remote]
timeout_ms = 10000    # Increase for slow networks
retry_count = 5       # More retries for unreliable connections
```

### Storage

- Use SSD for cache storage
- Consider NVMe for high-throughput scenarios
- NFS works but may have higher latency
