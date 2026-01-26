---
title: Cache Health
description: Monitoring and maintaining cache health
tags:
  - operations
  - maintenance
---

# Cache Health

Ensuring cache integrity and performance.

## Health Check

```bash
rninja -t cache-health
```

Output:

```
Cache Health Check:
  Database: OK
  Blob storage: OK
  Index integrity: OK
  Orphaned blobs: 0

Cache is healthy.
```

## Health Issues

### Database Issues

```
Database: ERROR
```

Solution:

```bash
# Try GC first
rninja -t cache-gc

# If still broken, reset
rm -rf ~/.cache/rninja
```

### Orphaned Blobs

```
Orphaned blobs: 15
```

Solution:

```bash
rninja -t cache-gc
```

### Index Integrity

```
Index integrity: ERROR
```

Solution:

```bash
# Reset cache
rm -rf ~/.cache/rninja
rninja  # Rebuild
```

## Automated Health Check

```bash
#!/bin/bash
# /etc/cron.daily/rninja-health

if ! rninja -t cache-health > /dev/null 2>&1; then
    logger "rninja cache health check failed"
    rninja -t cache-gc
fi
```
