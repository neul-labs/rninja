---
title: Environment Variables
description: All rninja environment variables
tags:
  - user-guide
  - configuration
  - environment
---

# Environment Variables

rninja can be configured through environment variables. These override config file settings but are overridden by command-line arguments.

## Cache Variables

### `RNINJA_CACHE_ENABLED`

Enable or disable the build cache.

```bash
RNINJA_CACHE_ENABLED=1    # Enable (default)
RNINJA_CACHE_ENABLED=0    # Disable
```

### `RNINJA_CACHE_DIR`

Set the cache directory location.

```bash
RNINJA_CACHE_DIR=/path/to/cache
```

Default: `~/.cache/rninja` (or `$XDG_CACHE_HOME/rninja`)

### `RNINJA_CACHE_MODE`

Set the cache operation mode.

```bash
RNINJA_CACHE_MODE=local   # Local cache only
RNINJA_CACHE_MODE=remote  # Remote cache only
RNINJA_CACHE_MODE=auto    # Remote with local fallback (default)
```

### `RNINJA_CACHE_MAX_AGE`

Maximum age for cache entries in seconds.

```bash
RNINJA_CACHE_MAX_AGE=86400    # 24 hours
RNINJA_CACHE_MAX_AGE=604800   # 7 days
```

Default: No expiry

### `RNINJA_CACHE_MAX_SIZE`

Maximum cache size. Supports suffixes: K, M, G.

```bash
RNINJA_CACHE_MAX_SIZE=10G     # 10 gigabytes
RNINJA_CACHE_MAX_SIZE=500M    # 500 megabytes
RNINJA_CACHE_MAX_SIZE=1073741824  # bytes
```

Default: 10GB

## Remote Cache Variables

### `RNINJA_CACHE_REMOTE_SERVER`

Remote cache server address.

```bash
RNINJA_CACHE_REMOTE_SERVER=tcp://cache.example.com:9999
```

### `RNINJA_CACHE_TOKEN`

Authentication token for remote cache.

```bash
RNINJA_CACHE_TOKEN=your-secret-token
```

!!! warning "Security"
    Don't commit tokens to version control. Use CI secrets or secure environment management.

### `RNINJA_CACHE_PUSH_POLICY`

When to push artifacts to remote cache.

```bash
RNINJA_CACHE_PUSH_POLICY=never       # Never push
RNINJA_CACHE_PUSH_POLICY=on_success  # Push successful builds (default)
RNINJA_CACHE_PUSH_POLICY=always      # Always push
```

### `RNINJA_CACHE_PULL_POLICY`

When to pull artifacts from remote cache.

```bash
RNINJA_CACHE_PULL_POLICY=always   # Always try remote first (default)
RNINJA_CACHE_PULL_POLICY=on_miss  # Only pull on local miss
RNINJA_CACHE_PULL_POLICY=never    # Never pull (push-only mode)
```

### `RNINJA_CACHE_CONNECT_TIMEOUT`

Connection timeout for remote cache in seconds.

```bash
RNINJA_CACHE_CONNECT_TIMEOUT=5    # 5 seconds (default)
RNINJA_CACHE_CONNECT_TIMEOUT=10   # 10 seconds
```

### `RNINJA_CACHE_REQUEST_TIMEOUT`

Request timeout for remote cache operations in seconds.

```bash
RNINJA_CACHE_REQUEST_TIMEOUT=30   # 30 seconds (default)
RNINJA_CACHE_REQUEST_TIMEOUT=60   # 60 seconds
```

### `RNINJA_CACHE_MAX_CONCURRENT`

Maximum concurrent remote cache operations.

```bash
RNINJA_CACHE_MAX_CONCURRENT=4     # Default
RNINJA_CACHE_MAX_CONCURRENT=8     # Higher for fast networks
```

## Remote Cache Server Variables

These configure `rninja-cached` server:

### `RNINJA_SERVER_LISTEN`

Address for the server to listen on.

```bash
RNINJA_SERVER_LISTEN=tcp://0.0.0.0:9999
```

### `RNINJA_SERVER_STORAGE`

Storage directory for cache server.

```bash
RNINJA_SERVER_STORAGE=/var/lib/rninja-cache
```

### `RNINJA_SERVER_MAX_SIZE`

Maximum storage size for server.

```bash
RNINJA_SERVER_MAX_SIZE=100G
```

### `RNINJA_SERVER_TOKENS`

Comma-separated list of valid authentication tokens.

```bash
RNINJA_SERVER_TOKENS=token1,token2,token3
```

### `RNINJA_SERVER_ENTRY_TTL`

Time-to-live for cache entries in seconds.

```bash
RNINJA_SERVER_ENTRY_TTL=604800    # 7 days
```

## Standard Environment Variables

### `XDG_CACHE_HOME`

If set, rninja uses `$XDG_CACHE_HOME/rninja` as the default cache directory.

```bash
XDG_CACHE_HOME=~/.cache  # Default on most systems
```

### `HOME`

Used to locate `~/.rninjarc` and default cache directory.

## Usage Examples

### Development Setup

```bash
# ~/.bashrc or ~/.zshrc

# Enable caching with local-only mode
export RNINJA_CACHE_ENABLED=1
export RNINJA_CACHE_MODE=local

# Limit cache size
export RNINJA_CACHE_MAX_SIZE=5G
```

### CI Pipeline

```bash
# In CI configuration

export RNINJA_CACHE_ENABLED=1
export RNINJA_CACHE_MODE=auto
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=$CI_CACHE_TOKEN  # From CI secrets
```

### Disable Caching Temporarily

```bash
# For debugging or testing
RNINJA_CACHE_ENABLED=0 rninja
```

### Team Cache Server

```bash
# Server setup
export RNINJA_SERVER_LISTEN=tcp://0.0.0.0:9999
export RNINJA_SERVER_STORAGE=/var/lib/rninja-cache
export RNINJA_SERVER_MAX_SIZE=500G
export RNINJA_SERVER_TOKENS=team-token-123

rninja-cached
```

```bash
# Client setup
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.team.internal:9999
export RNINJA_CACHE_TOKEN=team-token-123

rninja
```

## Variable Reference Table

| Variable | Description | Default |
|----------|-------------|---------|
| `RNINJA_CACHE_ENABLED` | Enable caching | `1` |
| `RNINJA_CACHE_DIR` | Cache directory | `~/.cache/rninja` |
| `RNINJA_CACHE_MODE` | Cache mode | `local` |
| `RNINJA_CACHE_MAX_AGE` | Max entry age (seconds) | None |
| `RNINJA_CACHE_MAX_SIZE` | Max cache size | `10G` |
| `RNINJA_CACHE_REMOTE_SERVER` | Remote server URL | None |
| `RNINJA_CACHE_TOKEN` | Auth token | None |
| `RNINJA_CACHE_PUSH_POLICY` | Push policy | `on_success` |
| `RNINJA_CACHE_PULL_POLICY` | Pull policy | `always` |
| `RNINJA_CACHE_CONNECT_TIMEOUT` | Connect timeout (sec) | `5` |
| `RNINJA_CACHE_REQUEST_TIMEOUT` | Request timeout (sec) | `30` |
| `RNINJA_CACHE_MAX_CONCURRENT` | Max concurrent ops | `4` |

## Tips

### Use .env Files

For project-specific settings, create a `.env` file:

```bash
# .env (add to .gitignore)
RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
```

Load with:

```bash
source .env
rninja
```

### CI Secret Management

Never hardcode tokens. Use your CI system's secret management:

=== "GitHub Actions"

    ```yaml
    env:
      RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}
    ```

=== "GitLab CI"

    ```yaml
    variables:
      RNINJA_CACHE_TOKEN: $CACHE_TOKEN
    ```

### Debugging Configuration

Check which variables are set:

```bash
env | grep RNINJA
```
