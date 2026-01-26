---
title: Remote Cache Quick Setup
description: Get remote caching working in minutes
tags:
  - caching
  - remote
  - quickstart
---

# Remote Cache Quick Setup

Get team-wide cache sharing working in minutes.

## Prerequisites

- rninja installed on all machines
- Network connectivity between machines
- One machine to run the cache server

## Step 1: Start the Cache Server

On a machine accessible by all clients:

```bash
rninja-cached \
    --listen tcp://0.0.0.0:9999 \
    --storage /var/lib/rninja-cache \
    --tokens "your-secret-token"
```

The server will start and display:

```
rninja-cached listening on tcp://0.0.0.0:9999
Storage: /var/lib/rninja-cache
```

## Step 2: Configure Clients

On each developer machine and CI runner:

```bash
export RNINJA_CACHE_REMOTE_SERVER=tcp://your-server:9999
export RNINJA_CACHE_TOKEN=your-secret-token
export RNINJA_CACHE_MODE=auto
```

Add to shell profile for persistence:

```bash
# ~/.bashrc or ~/.zshrc
export RNINJA_CACHE_REMOTE_SERVER=tcp://your-server:9999
export RNINJA_CACHE_TOKEN=your-secret-token
export RNINJA_CACHE_MODE=auto
```

## Step 3: Test Connection

Verify the connection works:

```bash
# Run a build
rninja

# Check cache stats
rninja -t cache-stats
```

You should see remote cache statistics in the output.

## Step 4: Verify Team Sharing

### On Machine A:

```bash
# Clean and build
rninja -t clean
rninja
```

### On Machine B:

```bash
# Clean and build - should get cache hits
rninja -t clean
rninja

# Check stats
rninja -t cache-stats
# Should show remote cache hits
```

## Quick Server Options

### Development Testing

```bash
# Foreground with verbose logging
rninja-cached --listen tcp://0.0.0.0:9999 --storage /tmp/cache -v
```

### Team Server

```bash
# Background with larger storage
rninja-cached \
    --listen tcp://0.0.0.0:9999 \
    --storage /var/lib/rninja-cache \
    --max-size 100G \
    --tokens "team-token" &
```

### Systemd Service

```bash
# Install service (see Deployment guide)
sudo systemctl start rninja-cached
sudo systemctl enable rninja-cached
```

## Environment Summary

### Server Environment

| Variable | Description | Example |
|----------|-------------|---------|
| `RNINJA_SERVER_LISTEN` | Listen address | `tcp://0.0.0.0:9999` |
| `RNINJA_SERVER_STORAGE` | Storage path | `/var/lib/rninja-cache` |
| `RNINJA_SERVER_TOKENS` | Valid tokens | `token1,token2` |
| `RNINJA_SERVER_MAX_SIZE` | Max storage | `100G` |

### Client Environment

| Variable | Description | Example |
|----------|-------------|---------|
| `RNINJA_CACHE_REMOTE_SERVER` | Server URL | `tcp://cache:9999` |
| `RNINJA_CACHE_TOKEN` | Auth token | `your-token` |
| `RNINJA_CACHE_MODE` | Cache mode | `auto` |

## Troubleshooting Quick Start

### Connection Refused

```bash
# Check server is running
nc -zv your-server 9999

# Check firewall
sudo ufw allow 9999/tcp
```

### Authentication Failed

```bash
# Verify token matches server
echo $RNINJA_CACHE_TOKEN

# Check server tokens
# On server: check --tokens argument
```

### No Cache Hits

```bash
# Verify mode is set
echo $RNINJA_CACHE_MODE  # Should be 'auto' or 'remote'

# Check connectivity
rninja -t cache-stats
```

## What's Next?

<div class="grid cards" markdown>

-   :material-server: [__Full Deployment__](deployment.md)

    Production deployment guide

-   :material-shield-key: [__Authentication__](authentication.md)

    Set up proper access control

-   :material-tune: [__Performance Tuning__](tuning.md)

    Optimize cache performance

</div>
