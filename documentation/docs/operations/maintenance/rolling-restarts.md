---
title: Rolling Restarts
description: Zero-downtime restarts for rninja-cached
tags:
  - operations
  - maintenance
---

# Rolling Restarts

Restart rninja-cached without disrupting builds.

## Single Server

```bash
# Graceful restart
systemctl restart rninja-cached
```

Clients will retry automatically.

## With Load Balancer

1. Remove server from pool
2. Wait for active requests to complete
3. Restart server
4. Add back to pool

```bash
# Remove from LB
curl -X POST http://lb/servers/cache1/disable

# Wait and restart
sleep 30
systemctl restart rninja-cached

# Re-enable
curl -X POST http://lb/servers/cache1/enable
```

## Client Resilience

Clients handle restarts gracefully:

- Auto-retry on connection failure
- Fall back to local cache (`mode=auto`)
- Reconnect to new server

## Best Practices

1. Use `auto` mode for client resilience
2. Set reasonable timeouts
3. Monitor during restart
4. Restart during low-usage periods
