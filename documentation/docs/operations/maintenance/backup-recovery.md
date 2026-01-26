---
title: Backup & Recovery
description: Backing up and restoring rninja cache
tags:
  - operations
  - maintenance
---

# Backup & Recovery

Protecting cache data.

## Backup

### Local Cache

```bash
tar -czf rninja-backup.tar.gz -C ~/.cache rninja
```

### Server Cache

```bash
# Stop server for consistent backup
systemctl stop rninja-cached
tar -czf cache-backup.tar.gz -C /var/lib rninja-cache
systemctl start rninja-cached
```

### Automated Backup

```bash
#!/bin/bash
# /etc/cron.daily/rninja-backup

BACKUP_DIR=/backups/rninja
DATE=$(date +%Y%m%d)

tar -czf $BACKUP_DIR/cache-$DATE.tar.gz -C /var/lib rninja-cache

# Keep 7 days
find $BACKUP_DIR -name "cache-*.tar.gz" -mtime +7 -delete
```

## Recovery

### Restore Local Cache

```bash
rm -rf ~/.cache/rninja
tar -xzf rninja-backup.tar.gz -C ~/.cache
```

### Restore Server Cache

```bash
systemctl stop rninja-cached
rm -rf /var/lib/rninja-cache
tar -xzf cache-backup.tar.gz -C /var/lib
systemctl start rninja-cached
```

## Disaster Recovery

If cache is lost:

1. Start with empty cache
2. First builds populate cache
3. Subsequent builds get hits

Cache is a performance optimization, not critical data.
