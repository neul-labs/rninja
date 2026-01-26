---
title: Local Cache Configuration
description: Configuring the local build cache
tags:
  - caching
  - local
  - configuration
---

# Local Cache Configuration

Configure the local cache for your environment.

## Basic Configuration

### Enable/Disable

```bash
# Enable (default)
export RNINJA_CACHE_ENABLED=1

# Disable
export RNINJA_CACHE_ENABLED=0
```

### Set Cache Directory

```bash
# Default: ~/.cache/rninja
export RNINJA_CACHE_DIR=/path/to/cache
```

### Set Size Limit

```bash
# Default: 10GB
export RNINJA_CACHE_MAX_SIZE=5G
```

## Configuration File

```toml title="~/.config/rninja/config.toml"
[cache]
enabled = true
mode = "local"
directory = "/path/to/cache"
max_size = 5368709120  # 5GB in bytes
```

## Size Configuration

### Formats

```bash
# Bytes
export RNINJA_CACHE_MAX_SIZE=5368709120

# With suffix
export RNINJA_CACHE_MAX_SIZE=5G      # Gigabytes
export RNINJA_CACHE_MAX_SIZE=500M    # Megabytes
export RNINJA_CACHE_MAX_SIZE=1024K   # Kilobytes
```

### Recommended Sizes

| Environment | Recommended Size |
|-------------|------------------|
| Personal laptop | 5-10 GB |
| Development workstation | 10-20 GB |
| CI runner | 2-5 GB |
| Build server | 50-100 GB |

## Age Configuration

### Maximum Age

```bash
# Remove entries older than 7 days
export RNINJA_CACHE_MAX_AGE=604800
```

### Common Values

| Duration | Seconds |
|----------|---------|
| 1 day | 86400 |
| 7 days | 604800 |
| 30 days | 2592000 |
| 90 days | 7776000 |

## Directory Configuration

### Default Locations

| OS | Default Path |
|----|--------------|
| Linux | `~/.cache/rninja` |
| macOS | `~/Library/Caches/rninja` |
| Windows | `%LOCALAPPDATA%\rninja\cache` |

### XDG Support

On Linux, respects `XDG_CACHE_HOME`:

```bash
export XDG_CACHE_HOME=~/.cache  # Default
# Cache will be at ~/.cache/rninja
```

### Custom Location

```bash
# SSD for faster access
export RNINJA_CACHE_DIR=/ssd/rninja-cache

# Shared location (read-only machines)
export RNINJA_CACHE_DIR=/shared/cache/rninja

# Temporary (cleared on reboot)
export RNINJA_CACHE_DIR=/tmp/rninja-cache
```

## Environment-Specific Configurations

### Development Machine

```toml
[cache]
enabled = true
mode = "local"
max_size = 10737418240  # 10GB
# No max_age - keep entries indefinitely
```

### CI Runner

```toml
[cache]
enabled = true
mode = "local"
max_size = 2147483648  # 2GB (limited disk)
# Note: Consider remote cache for CI
```

### Disk-Constrained Machine

```toml
[cache]
enabled = true
mode = "local"
directory = "/path/to/larger/disk/rninja"
max_size = 1073741824  # 1GB
```

## Performance Tuning

### Fast Storage

Put cache on fastest available storage:

```bash
# NVMe SSD
export RNINJA_CACHE_DIR=/nvme/rninja-cache

# RAM disk (volatile but fast)
export RNINJA_CACHE_DIR=/dev/shm/rninja-cache
```

### Network Storage Warning

Avoid network-mounted storage for local cache:

```bash
# BAD: NFS/CIFS mount
export RNINJA_CACHE_DIR=/mnt/nfs/cache  # Slow!

# GOOD: Local disk
export RNINJA_CACHE_DIR=/home/user/.cache/rninja
```

## Complete Configuration Example

```toml title="~/.config/rninja/config.toml"
# Local development configuration

[cache]
# Enable caching
enabled = true

# Local-only mode
mode = "local"

# Custom directory on SSD
directory = "/ssd/rninja-cache"

# 10GB size limit
max_size = 10737418240

# Keep entries for 30 days
# max_age = 2592000  # Uncomment to enable
```

Environment variables:

```bash title="~/.bashrc"
# Optional overrides
export RNINJA_CACHE_ENABLED=1
export RNINJA_CACHE_MODE=local
```

## Verifying Configuration

### Check Current Settings

```bash
# Show configuration
rninja -t config -v

# Check cache stats
rninja -t cache-stats
```

### Test Configuration

```bash
# Build something
rninja

# Verify cache is working
rninja -t cache-stats
# Should show entries and size > 0
```

## Troubleshooting Configuration

### Cache Not Using Custom Directory

```bash
# Verify environment variable
echo $RNINJA_CACHE_DIR

# Check config file is loaded
rninja -t config
```

### Size Limit Not Applied

Size limits are enforced during GC:

```bash
# Run GC to apply size limit
rninja -t cache-gc
```

### Permission Denied

```bash
# Check directory permissions
ls -la $(dirname $RNINJA_CACHE_DIR)

# Create directory with correct permissions
mkdir -p $RNINJA_CACHE_DIR
chmod 755 $RNINJA_CACHE_DIR
```
