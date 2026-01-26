---
title: Metrics Reference
description: rninja metrics for monitoring
tags:
  - operations
  - monitoring
---

# Metrics Reference

Key metrics for monitoring rninja infrastructure.

## Cache Metrics

### Client-Side

```bash
rninja -t cache-stats
```

| Metric | Description |
|--------|-------------|
| Total entries | Number of cached artifacts |
| Total size | Disk space used |
| Hit rate | Cache hit percentage |
| Session hits | Hits in current session |
| Session misses | Misses in current session |

### Server-Side

| Metric | Description |
|--------|-------------|
| Requests/sec | Incoming request rate |
| Cache size | Total storage used |
| Active connections | Current client count |

## Performance Metrics

### Build Timing

```bash
rninja --trace trace.json
```

Provides:

- Total build time
- Per-target timing
- Parallelism efficiency

### Latency

| Operation | Good | Warning |
|-----------|------|---------|
| Cache lookup | < 50ms | > 200ms |
| Cache store | < 100ms | > 500ms |
| No-op build | < 50ms | > 200ms |

## Health Metrics

```bash
rninja -t cache-health
```

| Check | Status |
|-------|--------|
| Database | OK/Error |
| Blob storage | OK/Warning |
| Index integrity | OK/Error |

## Monitoring Integration

### Prometheus (Future)

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'rninja'
    static_configs:
      - targets: ['cache:9998']
```

### Custom Script

```bash
#!/bin/bash
# monitor.sh
STATS=$(rninja -t cache-stats 2>/dev/null)
echo "rninja_cache_size $(echo "$STATS" | grep 'Total size' | awk '{print $3}')"
echo "rninja_hit_rate $(echo "$STATS" | grep 'Hit rate' | awk '{print $3}' | tr -d '%')"
```
