---
title: Alerting
description: Setting up alerts for rninja
tags:
  - operations
  - monitoring
  - alerting
---

# Alerting

Configure alerts for rninja infrastructure.

## Alert Conditions

| Condition | Severity | Action |
|-----------|----------|--------|
| Cache hit rate < 50% | Warning | Investigate cache misses |
| Cache server down | Critical | Restart server |
| Disk > 90% full | Warning | Run GC |
| Build timeout | Error | Check build logs |

## Simple Monitoring Script

```bash
#!/bin/bash
# /etc/cron.hourly/rninja-alerts

# Check cache health
if ! rninja -t cache-health > /dev/null 2>&1; then
    echo "rninja cache unhealthy" | mail -s "Alert: rninja" ops@example.com
fi

# Check disk usage
USAGE=$(df ~/.cache/rninja | tail -1 | awk '{print $5}' | tr -d '%')
if [ "$USAGE" -gt 90 ]; then
    echo "rninja cache disk usage: ${USAGE}%" | mail -s "Warning: rninja" ops@example.com
    rninja -t cache-gc
fi
```

## Prometheus Alertmanager

```yaml
# alertmanager.yml
route:
  receiver: 'ops-team'

receivers:
  - name: 'ops-team'
    email_configs:
      - to: 'ops@example.com'
```

## PagerDuty Integration

For critical alerts:

```yaml
receivers:
  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: '<key>'
```
