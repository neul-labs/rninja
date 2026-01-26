---
title: Cache Tools
description: Subtools for managing the build cache
tags:
  - user-guide
  - subtools
  - cache
---

# Cache Tools

Tools for managing and inspecting the rninja build cache.

## cache-stats

Show cache statistics.

### Usage

```bash
rninja -t cache-stats
```

### Output

```
Cache Statistics:
  Enabled: true
  Mode: local
  Directory: /home/user/.cache/rninja

  Local Cache:
    Total entries: 1,234
    Total size: 456.7 MB
    Hit rate: 87.3%

  Session:
    Hits: 45
    Misses: 12
    Stores: 12
```

### Fields Explained

| Field | Description |
|-------|-------------|
| Enabled | Whether caching is active |
| Mode | Current mode (local/remote/auto) |
| Directory | Cache storage location |
| Total entries | Number of cached artifacts |
| Total size | Disk space used |
| Hit rate | Percentage of cache hits |
| Hits | Cache hits this session |
| Misses | Cache misses this session |
| Stores | New artifacts stored |

### Examples

```bash
# Quick status check
rninja -t cache-stats

# Monitor cache growth
watch -n 60 'rninja -t cache-stats'
```

## cache-gc

Run garbage collection on the cache.

### Usage

```bash
# Run GC with default settings
rninja -t cache-gc

# Aggressive cleanup
RNINJA_CACHE_MAX_SIZE=1G rninja -t cache-gc
```

### Description

Removes old cache entries to:

- Keep cache size under `max_size`
- Remove entries older than `max_age`
- Clean up orphaned blob files

### Examples

```bash
# Regular cleanup
rninja -t cache-gc

# Reclaim space when disk is full
RNINJA_CACHE_MAX_SIZE=500M rninja -t cache-gc

# Check size before and after
rninja -t cache-stats
rninja -t cache-gc
rninja -t cache-stats
```

### When to Run

- Disk space is low
- Cache has grown large
- After major project changes
- As part of regular maintenance

## cache-health

Check cache integrity.

### Usage

```bash
rninja -t cache-health
```

### Output

```
Cache Health Check:
  Database: OK
  Blob storage: OK
  Index integrity: OK
  Orphaned blobs: 0

Cache is healthy.
```

Or if issues found:

```
Cache Health Check:
  Database: OK
  Blob storage: WARNING - 3 orphaned blobs
  Index integrity: OK

Run 'rninja -t cache-gc' to clean up.
```

### Examples

```bash
# Regular health check
rninja -t cache-health

# Check and fix
rninja -t cache-health
rninja -t cache-gc
rninja -t cache-health
```

### When to Run

- After unexpected shutdowns
- If builds behave unexpectedly
- Before important builds
- As part of CI setup

## config

Show and generate configuration.

### Usage

```bash
# Show config file locations
rninja -t config

# Generate sample config
rninja -t config -v
```

### Examples

```bash
# See where config is loaded from
rninja -t config

# Generate new config file
rninja -t config -v > ~/.config/rninja/config.toml

# Show effective configuration
rninja -t config -v
```

## Maintenance Workflows

### Daily Maintenance

```bash
# Quick status check
rninja -t cache-stats
```

### Weekly Maintenance

```bash
# Health check
rninja -t cache-health

# Clean up old entries
rninja -t cache-gc

# Verify
rninja -t cache-stats
```

### Disk Space Recovery

```bash
# Check current usage
rninja -t cache-stats

# Aggressive cleanup
RNINJA_CACHE_MAX_SIZE=1G rninja -t cache-gc

# Or clear everything
rm -rf ~/.cache/rninja
```

### Troubleshooting Cache Issues

```bash
# Check health
rninja -t cache-health

# If issues found
rninja -t cache-gc

# If still broken, reset cache
rm -rf ~/.cache/rninja
rninja  # Rebuilds and repopulates cache
```

## Monitoring Cache Performance

### Track Hit Rate Over Time

```bash
# Log cache stats
rninja -t cache-stats >> ~/cache_stats.log
```

### Alert on Low Hit Rate

```bash
#!/bin/bash
# check_cache.sh
HIT_RATE=$(rninja -t cache-stats | grep "Hit rate" | awk '{print $3}' | tr -d '%')
if (( $(echo "$HIT_RATE < 50" | bc -l) )); then
    echo "Warning: Cache hit rate is low ($HIT_RATE%)"
fi
```

## Cache Location

Default locations:

| OS | Default Path |
|----|--------------|
| Linux | `~/.cache/rninja` |
| macOS | `~/Library/Caches/rninja` |
| Windows | `%LOCALAPPDATA%\rninja\cache` |

Override with:

```bash
export RNINJA_CACHE_DIR=/custom/path
```

## Cache Contents

The cache directory contains:

```
~/.cache/rninja/
├── index/          # sled database (metadata)
└── blobs/          # Content-addressed artifacts
```

!!! warning "Don't Modify Manually"
    Don't manually edit cache contents. Use the cache tools instead.

## Tips

### Regular GC

Add to cron or scheduled tasks:

```bash
# Weekly cleanup
0 0 * * 0 rninja -t cache-gc
```

### Monitor in CI

```yaml
# .github/workflows/build.yml
- name: Cache stats
  run: rninja -t cache-stats

- name: Build
  run: rninja

- name: Cache stats after build
  run: rninja -t cache-stats
```

### Size Limits

Set appropriate limits for your environment:

```bash
# Development machine
export RNINJA_CACHE_MAX_SIZE=10G

# CI runner
export RNINJA_CACHE_MAX_SIZE=2G

# Shared server
export RNINJA_CACHE_MAX_SIZE=50G
```
