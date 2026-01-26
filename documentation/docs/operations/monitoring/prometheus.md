---
title: Prometheus Setup
description: Monitoring rninja with Prometheus
tags:
  - operations
  - monitoring
  - prometheus
---

# Prometheus Setup

Monitor rninja with Prometheus (future feature, current workarounds).

## Custom Exporter Script

```bash
#!/bin/bash
# /usr/local/bin/rninja-exporter.sh

while true; do
    STATS=$(rninja -t cache-stats 2>/dev/null)

    cat << EOF > /var/lib/prometheus/rninja.prom
# HELP rninja_cache_entries Total cache entries
# TYPE rninja_cache_entries gauge
rninja_cache_entries $(echo "$STATS" | grep -oP 'Total entries: \K\d+' || echo 0)

# HELP rninja_cache_size_bytes Cache size in bytes
# TYPE rninja_cache_size_bytes gauge
rninja_cache_size_bytes $(du -sb ~/.cache/rninja 2>/dev/null | cut -f1 || echo 0)
EOF

    sleep 60
done
```

## Node Exporter Textfile

Configure node_exporter with textfile collector:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']
```

## Alerting Rules

```yaml
# alerts.yml
groups:
  - name: rninja
    rules:
      - alert: RninjaCacheLow
        expr: rninja_cache_hit_rate < 50
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Low cache hit rate"
```
