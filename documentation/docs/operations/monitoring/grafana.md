---
title: Grafana Dashboards
description: Visualizing rninja metrics in Grafana
tags:
  - operations
  - monitoring
  - grafana
---

# Grafana Dashboards

Visualize rninja metrics with Grafana.

## Sample Dashboard

```json
{
  "title": "rninja Cache",
  "panels": [
    {
      "title": "Cache Hit Rate",
      "type": "gauge",
      "targets": [
        {"expr": "rninja_cache_hit_rate"}
      ]
    },
    {
      "title": "Cache Size",
      "type": "graph",
      "targets": [
        {"expr": "rninja_cache_size_bytes"}
      ]
    }
  ]
}
```

## Key Panels

1. **Cache Hit Rate** - Gauge showing current hit percentage
2. **Cache Size Over Time** - Graph of storage growth
3. **Build Duration** - Average build times
4. **Active Connections** - Server connection count

## Thresholds

| Metric | Green | Yellow | Red |
|--------|-------|--------|-----|
| Hit Rate | > 80% | 50-80% | < 50% |
| Build Time | < 1m | 1-5m | > 5m |
| Cache Size | < 80% | 80-95% | > 95% |
