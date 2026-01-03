# Rolling Restart Guide

This guide covers zero-downtime procedures for upgrading and restarting rninja components without disrupting active builds.

## Overview

rninja supports rolling restarts for:
- **rninja-daemon**: The local build daemon process
- **rninja-cached**: The remote cache server

Both components handle in-flight requests gracefully during shutdown.

## Daemon Rolling Restart

### Understanding Daemon Lifecycle

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Running   │───►│  Draining   │───►│   Stopped   │
│  (active)   │    │ (finishing) │    │             │
└─────────────┘    └─────────────┘    └─────────────┘
      │                  │
      │ New requests     │ Rejects new requests
      │ accepted         │ Finishes active builds
      ▼                  ▼
```

### Graceful Daemon Restart

The daemon automatically handles graceful shutdown when receiving SIGTERM:

```bash
# Check current daemon status
rninja -t daemon-status

# Graceful restart (waits for active builds)
rninja -t daemon-stop
# Daemon will finish active builds before exiting

# Next rninja command auto-spawns new daemon
rninja -t daemon-status
```

### Forced Restart

For immediate restart (cancels active builds):

```bash
# Force stop (SIGKILL)
pkill -9 -f rninja-daemon

# Or find and kill by socket
SOCKET_PATH=$(rninja -t daemon-status 2>/dev/null | grep socket | awk '{print $2}')
fuser -k "$SOCKET_PATH" 2>/dev/null

# Start fresh
rninja
```

### Upgrade Procedure

```bash
#!/bin/bash
# daemon-upgrade.sh - Zero-downtime daemon upgrade

set -e

NEW_BINARY="$1"
if [[ -z "$NEW_BINARY" ]]; then
    echo "Usage: $0 /path/to/new/rninja-daemon"
    exit 1
fi

# 1. Verify new binary works
"$NEW_BINARY" --version || exit 1

# 2. Signal graceful shutdown
echo "Stopping current daemon..."
rninja -t daemon-stop 2>/dev/null || true

# 3. Wait for daemon to exit (with timeout)
for i in {1..30}; do
    if ! pgrep -f rninja-daemon > /dev/null; then
        break
    fi
    echo "Waiting for daemon to finish ($i/30)..."
    sleep 1
done

# 4. Replace binary
INSTALL_PATH="${INSTALL_PATH:-/usr/local/bin/rninja-daemon}"
sudo cp "$NEW_BINARY" "$INSTALL_PATH"

# 5. New daemon spawns automatically on next build
echo "Upgrade complete. New daemon will start on next build."
```

### Monitoring During Restart

```bash
#!/bin/bash
# monitor-daemon-restart.sh

# Watch daemon status during restart
watch -n 1 'rninja -t daemon-status 2>&1 || echo "Daemon not running"'
```

## Cache Server Rolling Restart

### Single Server Restart

For a single cache server deployment:

```bash
#!/bin/bash
# cache-server-restart.sh

# 1. Check active connections
curl -s http://localhost:9999/metrics | grep rninja_active_connections

# 2. Enable drain mode (stop accepting new connections)
curl -X POST http://localhost:9999/admin/drain

# 3. Wait for connections to finish
while true; do
    CONNS=$(curl -s http://localhost:9999/metrics | grep rninja_active_connections | awk '{print $2}')
    if [[ "$CONNS" == "0" ]]; then
        break
    fi
    echo "Active connections: $CONNS"
    sleep 2
done

# 4. Stop server
systemctl stop rninja-cached

# 5. Start new version
systemctl start rninja-cached

# 6. Verify health
curl -s http://localhost:9999/health
```

### Multi-Server Rolling Restart

For deployments with multiple cache servers behind a load balancer:

```bash
#!/bin/bash
# rolling-restart-cluster.sh

SERVERS=("cache1.internal" "cache2.internal" "cache3.internal")
HEALTH_TIMEOUT=30

for server in "${SERVERS[@]}"; do
    echo "=== Restarting $server ==="

    # 1. Remove from load balancer
    echo "Removing $server from load balancer..."
    # Example: consul, nginx, haproxy, etc.
    # consul services deregister -id="rninja-cache-$server"

    # 2. Wait for in-flight requests
    echo "Draining connections..."
    ssh "$server" 'curl -X POST http://localhost:9999/admin/drain'
    sleep 5

    # 3. Stop service
    echo "Stopping service..."
    ssh "$server" 'systemctl stop rninja-cached'

    # 4. Upgrade if needed
    # ssh "$server" 'apt upgrade rninja-cached'

    # 5. Start service
    echo "Starting service..."
    ssh "$server" 'systemctl start rninja-cached'

    # 6. Wait for health
    echo "Waiting for health check..."
    for i in $(seq 1 $HEALTH_TIMEOUT); do
        if ssh "$server" 'curl -sf http://localhost:9999/health' > /dev/null; then
            echo "$server is healthy"
            break
        fi
        sleep 1
    done

    # 7. Add back to load balancer
    echo "Adding $server back to load balancer..."
    # consul services register ...

    # 8. Wait before next server
    echo "Waiting before next server..."
    sleep 10
done

echo "Rolling restart complete"
```

### Blue-Green Deployment

For zero-downtime upgrades with instant rollback capability:

```bash
#!/bin/bash
# blue-green-deploy.sh

# Assuming:
# - Blue cluster: cache-blue-{1,2,3}.internal (current)
# - Green cluster: cache-green-{1,2,3}.internal (new version)

BLUE_SERVERS=("cache-blue-1" "cache-blue-2" "cache-blue-3")
GREEN_SERVERS=("cache-green-1" "cache-green-2" "cache-green-3")

# 1. Deploy new version to green cluster
echo "Deploying to green cluster..."
for server in "${GREEN_SERVERS[@]}"; do
    ssh "$server" 'systemctl start rninja-cached'
done

# 2. Warm up green cluster cache (optional)
echo "Warming green cluster..."
# Run test builds against green cluster

# 3. Health check green cluster
echo "Checking green cluster health..."
for server in "${GREEN_SERVERS[@]}"; do
    if ! ssh "$server" 'curl -sf http://localhost:9999/health'; then
        echo "ERROR: $server unhealthy, aborting"
        exit 1
    fi
done

# 4. Switch traffic to green
echo "Switching traffic to green cluster..."
# Update DNS, load balancer, or service discovery
# Example: update-dns cache.internal -> green cluster IPs

# 5. Monitor for issues
echo "Monitoring for 5 minutes..."
sleep 300

# 6. Drain blue cluster
echo "Draining blue cluster..."
for server in "${BLUE_SERVERS[@]}"; do
    ssh "$server" 'systemctl stop rninja-cached'
done

echo "Blue-green deployment complete"
echo "Blue cluster stopped, green cluster active"
echo "To rollback: restart blue cluster and switch DNS back"
```

## Client-Side Handling

### Client Retry Configuration

Configure clients to handle server restarts gracefully:

```toml
# ~/.config/rninja/config.toml
[cache.remote]
timeout_ms = 5000
retry_count = 3
retry_delay_ms = 1000
```

### Fallback Behavior

When remote cache is unavailable during restart:

| Cache Mode | Behavior |
|------------|----------|
| `local` | Uses local cache only (no impact) |
| `remote` | Retries, then fails build on timeout |
| `auto` | Retries, then falls back to local cache |

Recommended: Use `auto` mode in production:

```bash
export RNINJA_CACHE_MODE=auto
```

### Build Script Resilience

```bash
#!/bin/bash
# resilient-build.sh

# Set auto mode for cache resilience
export RNINJA_CACHE_MODE=auto

# Build with retry on failure
MAX_RETRIES=3
for i in $(seq 1 $MAX_RETRIES); do
    if rninja "$@"; then
        exit 0
    fi
    echo "Build attempt $i failed, retrying..."
    sleep 2
done

echo "Build failed after $MAX_RETRIES attempts"
exit 1
```

## Kubernetes Deployments

### Pod Disruption Budget

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: rninja-cache-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: rninja-cache
```

### Rolling Update Strategy

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rninja-cache
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      terminationGracePeriodSeconds: 60
      containers:
      - name: rninja-cached
        image: rninja/cached:latest
        lifecycle:
          preStop:
            exec:
              command:
              - /bin/sh
              - -c
              - curl -X POST http://localhost:9999/admin/drain && sleep 30
        readinessProbe:
          httpGet:
            path: /health
            port: 9999
          initialDelaySeconds: 5
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health
            port: 9999
          initialDelaySeconds: 15
          periodSeconds: 10
```

### Helm Values

```yaml
# values.yaml
replicaCount: 3

podDisruptionBudget:
  enabled: true
  minAvailable: 2

updateStrategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1
    maxUnavailable: 0

terminationGracePeriodSeconds: 60

livenessProbe:
  enabled: true
  initialDelaySeconds: 15
  periodSeconds: 10

readinessProbe:
  enabled: true
  initialDelaySeconds: 5
  periodSeconds: 5
```

## Systemd Integration

### Graceful Stop Configuration

```ini
# /etc/systemd/system/rninja-cached.service
[Unit]
Description=rninja Remote Cache Server
After=network.target

[Service]
Type=simple
User=rninja
ExecStart=/usr/local/bin/rninja-cached --cache-dir /var/cache/rninja
ExecStop=/bin/kill -TERM $MAINPID
TimeoutStopSec=60
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Reload vs Restart

```bash
# Reload configuration (no restart needed for some settings)
systemctl reload rninja-cached

# Full restart (graceful)
systemctl restart rninja-cached

# Check status
systemctl status rninja-cached
```

## Monitoring During Restarts

### Key Metrics to Watch

```bash
# During restart, monitor these metrics
watch -n 1 'curl -s http://localhost:9999/metrics | grep -E "rninja_(active|queue|error)"'

# Key metrics:
# - rninja_active_connections: Should drain to 0
# - rninja_request_queue_depth: Should process remaining
# - rninja_errors_total: Should not spike
```

### Alerting

Prometheus alerting rules for restart monitoring:

```yaml
groups:
- name: rninja-cache-restarts
  rules:
  - alert: RninjaCacheRestarting
    expr: changes(process_start_time_seconds{job="rninja-cache"}[5m]) > 0
    for: 0m
    labels:
      severity: info
    annotations:
      summary: "rninja cache server restarted"

  - alert: RninjaCacheUnavailable
    expr: up{job="rninja-cache"} == 0
    for: 1m
    labels:
      severity: warning
    annotations:
      summary: "rninja cache server unavailable"

  - alert: RninjaCacheHighErrorRate
    expr: rate(rninja_errors_total[5m]) > 10
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "High error rate during/after restart"
```

## Troubleshooting

### Daemon Won't Stop

```bash
# Check for stuck processes
ps aux | grep rninja

# Check socket status
lsof /tmp/rninja-*/daemon.sock

# Force cleanup
rm -f /tmp/rninja-*/daemon.sock
pkill -9 -f rninja-daemon
```

### Cache Server Stuck Draining

```bash
# Check active connections
curl http://localhost:9999/metrics | grep active

# Check for long-running requests
curl http://localhost:9999/admin/requests

# Force stop if needed
systemctl kill -s KILL rninja-cached
```

### Clients Failing During Restart

```bash
# Check client timeout settings
grep -r timeout ~/.config/rninja/

# Increase timeouts temporarily
export RNINJA_CACHE_TIMEOUT_MS=30000

# Or disable remote cache temporarily
export RNINJA_CACHE_MODE=local
```

## Best Practices

1. **Always use graceful shutdown** - Let active builds complete
2. **Use `auto` cache mode** - Clients fall back to local cache during outages
3. **Monitor during restarts** - Watch metrics and logs
4. **Test rollback procedures** - Know how to quickly revert
5. **Schedule during low-traffic periods** - Minimize impact
6. **Communicate maintenance windows** - Notify teams of planned restarts
7. **Keep previous version available** - For quick rollback
8. **Validate health before adding to rotation** - Check health endpoint
