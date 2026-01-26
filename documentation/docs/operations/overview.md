---
title: Operations Overview
description: Operating rninja in production environments
tags:
  - operations
---

# Operations Overview

Guide for operating rninja infrastructure in production.

## Components to Operate

| Component | Purpose | Scaling |
|-----------|---------|---------|
| rninja CLI | Build execution | Per-developer |
| rninja-daemon | Build acceleration | Per-machine |
| rninja-cached | Remote cache server | Per-team/org |

## Key Operational Areas

### Monitoring

- Cache hit rates
- Build performance
- Server health

[See Monitoring Guide](monitoring/metrics.md)

### Maintenance

- Garbage collection
- Cache health checks
- Rolling restarts

[See Maintenance Guide](maintenance/garbage-collection.md)

### Security

- Authentication
- TLS encryption
- Access control

[See Security Guide](security/overview.md)

## Quick Reference

### Health Check

```bash
rninja -t cache-health
rninja -t cache-stats
```

### Garbage Collection

```bash
rninja -t cache-gc
```

### View Logs

```bash
journalctl -u rninja-cached -f
```

## Operational Runbooks

- [Garbage Collection](maintenance/garbage-collection.md)
- [Cache Health](maintenance/cache-health.md)
- [Rolling Restarts](maintenance/rolling-restarts.md)
- [Backup & Recovery](maintenance/backup-recovery.md)
