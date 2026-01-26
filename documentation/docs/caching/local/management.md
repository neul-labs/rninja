---
title: Local Cache Management
description: Managing and maintaining the local cache
tags:
  - caching
  - local
  - maintenance
---

# Local Cache Management

Tools and practices for managing the local build cache.

## Cache Tools

### View Statistics

```bash
rninja -t cache-stats
```

Output:

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

### Run Garbage Collection

```bash
rninja -t cache-gc
```

Removes:

- Entries older than `max_age`
- Oldest entries when exceeding `max_size`
- Orphaned blob files
- Corrupt entries

### Check Health

```bash
rninja -t cache-health
```

Verifies:

- Database integrity
- Blob storage consistency
- Index correctness

## Maintenance Tasks

### Regular Cleanup

Schedule periodic garbage collection:

```bash
# Weekly cron job
0 0 * * 0 rninja -t cache-gc
```

Or run manually after large builds:

```bash
rninja
rninja -t cache-gc
```

### Disk Space Recovery

When disk is full:

```bash
# Check current usage
rninja -t cache-stats

# Aggressive cleanup (reduce to 1GB)
RNINJA_CACHE_MAX_SIZE=1G rninja -t cache-gc

# Or clear entirely
rm -rf ~/.cache/rninja
```

### Health Monitoring

Check periodically:

```bash
# Quick health check
rninja -t cache-health

# Full verification
rninja -t cache-health
rninja -t cache-gc
rninja -t cache-stats
```

## Cache Reset

### Soft Reset (GC)

Cleans up while preserving valid entries:

```bash
rninja -t cache-gc
```

### Hard Reset (Clear)

Removes entire cache:

```bash
rm -rf ~/.cache/rninja

# Next build repopulates
rninja
```

### Selective Reset

Remove specific entries (not directly supported, use time-based):

```bash
# Set aggressive max_age to remove old entries
RNINJA_CACHE_MAX_AGE=3600 rninja -t cache-gc  # Keep only last hour
```

## Monitoring

### Size Tracking

```bash
# Check cache size
du -sh ~/.cache/rninja

# Monitor growth
watch -n 60 'du -sh ~/.cache/rninja'
```

### Hit Rate Tracking

```bash
# After builds
rninja -t cache-stats | grep "Hit rate"
```

### Automated Monitoring

```bash
#!/bin/bash
# cache_monitor.sh

while true; do
    STATS=$(rninja -t cache-stats 2>/dev/null)
    SIZE=$(echo "$STATS" | grep "Total size" | awk '{print $3, $4}')
    RATE=$(echo "$STATS" | grep "Hit rate" | awk '{print $3}')

    echo "$(date): Size=$SIZE, HitRate=$RATE"
    sleep 300  # Every 5 minutes
done
```

## Backup and Restore

### Backup Cache

```bash
# Tar the cache directory
tar -czf rninja-cache-backup.tar.gz -C ~/.cache rninja
```

### Restore Cache

```bash
# Extract backup
tar -xzf rninja-cache-backup.tar.gz -C ~/.cache
```

### Share Cache

For machines with identical toolchains:

```bash
# On source machine
tar -czf cache.tar.gz -C ~/.cache rninja

# Transfer and extract on target
scp cache.tar.gz target:/tmp/
ssh target 'tar -xzf /tmp/cache.tar.gz -C ~/.cache'
```

!!! warning "Toolchain Compatibility"
    Cache is only valid for identical toolchains. Different compiler versions will cause cache misses (not errors).

## Troubleshooting

### Cache Growing Too Fast

```bash
# Check for large artifacts
du -sh ~/.cache/rninja/blobs/* | sort -h | tail -20

# Set size limit
export RNINJA_CACHE_MAX_SIZE=5G
rninja -t cache-gc
```

### Low Hit Rate

```bash
# Check what's causing misses
rninja -d explain

# Verify cache is working
rninja -t cache-stats
```

### Database Errors

```bash
# Try health check first
rninja -t cache-health

# If errors persist, reset cache
rm -rf ~/.cache/rninja
```

### Slow Cache Access

```bash
# Check disk performance
dd if=/dev/zero of=/tmp/test bs=1M count=100
rm /tmp/test

# Move cache to faster storage
export RNINJA_CACHE_DIR=/ssd/rninja-cache
```

## Best Practices

### Regular Maintenance

- Run `cache-gc` weekly
- Monitor hit rates
- Check health after system issues

### Size Management

- Set appropriate `max_size` for your disk
- Use `max_age` for automatic cleanup
- Monitor cache growth

### Multiple Projects

Cache handles multiple projects automatically:

- Each project has unique cache keys
- No conflicts between projects
- Shared artifacts deduplicated

### CI Integration

For CI, consider:

- Using remote cache instead
- Limiting local cache size
- Clearing cache between major builds

## Automation Scripts

### Weekly Maintenance

```bash
#!/bin/bash
# /etc/cron.weekly/rninja-maintenance

# Garbage collection
rninja -t cache-gc

# Health check
if ! rninja -t cache-health > /dev/null 2>&1; then
    echo "Warning: rninja cache health check failed" | mail -s "rninja cache warning" admin@example.com
fi

# Log stats
rninja -t cache-stats >> /var/log/rninja-cache.log
```

### Disk Space Monitor

```bash
#!/bin/bash
# Check cache size and warn if too large

MAX_SIZE_MB=10240  # 10GB
CACHE_SIZE=$(du -sm ~/.cache/rninja 2>/dev/null | cut -f1)

if [ "${CACHE_SIZE:-0}" -gt "$MAX_SIZE_MB" ]; then
    echo "rninja cache is ${CACHE_SIZE}MB (>${MAX_SIZE_MB}MB)"
    rninja -t cache-gc
fi
```
