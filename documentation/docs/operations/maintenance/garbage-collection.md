---
title: Garbage Collection
description: Managing cache cleanup
tags:
  - operations
  - maintenance
---

# Garbage Collection

Managing cache storage with garbage collection.

## Running GC

```bash
rninja -t cache-gc
```

## What Gets Cleaned

- Entries older than `max_age`
- Oldest entries when exceeding `max_size`
- Orphaned blob files
- Corrupt entries

## Scheduling GC

### Cron

```bash
# Weekly cleanup
0 0 * * 0 rninja -t cache-gc
```

### Systemd Timer

```ini
# /etc/systemd/system/rninja-gc.timer
[Unit]
Description=rninja GC Timer

[Timer]
OnCalendar=weekly
Persistent=true

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/rninja-gc.service
[Unit]
Description=rninja Garbage Collection

[Service]
Type=oneshot
ExecStart=/usr/local/bin/rninja -t cache-gc
```

## Aggressive Cleanup

```bash
# Reduce to 1GB
RNINJA_CACHE_MAX_SIZE=1G rninja -t cache-gc
```

## Monitoring GC

```bash
# Before
rninja -t cache-stats

# Run GC
rninja -t cache-gc

# After
rninja -t cache-stats
```
