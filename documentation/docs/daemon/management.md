---
title: Daemon Management
description: Managing the rninja daemon process
tags:
  - daemon
  - management
---

# Daemon Management

Tools and techniques for managing the rninja daemon.

## Starting the Daemon

### Auto-Start (Recommended)

Daemon starts automatically on first build:

```bash
rninja  # Daemon spawns if needed
```

### Manual Start

```bash
# Background
rninja-daemon &

# Foreground (for debugging)
rninja-daemon --foreground

# With options
rninja-daemon --socket /tmp/custom.sock --max-builds 4
```

## Stopping the Daemon

### Graceful Stop

```bash
# Send SIGTERM
pkill -f rninja-daemon

# Or find PID and kill
pgrep -f rninja-daemon
kill <pid>
```

### Force Stop

```bash
pkill -9 -f rninja-daemon
```

### Clean Up Socket

After stopping, the socket file may remain:

```bash
rm /tmp/rninja-daemon.sock
```

## Checking Status

### Is Daemon Running?

```bash
pgrep -f rninja-daemon
# Returns PID if running, nothing if not
```

### Detailed Status

```bash
ps aux | grep rninja-daemon
```

### Socket Exists?

```bash
ls -la /tmp/rninja-daemon.sock
```

## Daemon Options

### `--socket PATH`

Custom socket location:

```bash
rninja-daemon --socket /var/run/rninja/daemon.sock
```

### `--max-builds N`

Maximum concurrent builds:

```bash
rninja-daemon --max-builds 4
```

### `--max-cached N`

Maximum cached repositories:

```bash
rninja-daemon --max-cached 20
```

### `--foreground`

Don't daemonize (useful for debugging):

```bash
rninja-daemon --foreground
```

### `--verbose`

Enable verbose logging:

```bash
rninja-daemon --foreground --verbose
```

## Restarting

### Graceful Restart

```bash
pkill -f rninja-daemon
sleep 1
rninja  # Auto-spawns new daemon
```

### Rolling Restart (Production)

```bash
# Start new daemon on different socket
rninja-daemon --socket /tmp/rninja-new.sock &

# Switch clients to new socket
# Then stop old daemon
pkill -f "rninja-daemon.*old.sock"
```

## Systemd Integration

### Create Service File

```ini title="/etc/systemd/system/rninja-daemon.service"
[Unit]
Description=rninja Build Daemon
After=network.target

[Service]
Type=simple
User=%i
ExecStart=/usr/local/bin/rninja-daemon --foreground
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Service Commands

```bash
# Enable on boot
sudo systemctl enable rninja-daemon

# Start
sudo systemctl start rninja-daemon

# Stop
sudo systemctl stop rninja-daemon

# Status
sudo systemctl status rninja-daemon

# Logs
journalctl -u rninja-daemon
```

## Per-User Service

### Systemd User Service

```ini title="~/.config/systemd/user/rninja-daemon.service"
[Unit]
Description=rninja Build Daemon

[Service]
Type=simple
ExecStart=/usr/local/bin/rninja-daemon --foreground
Restart=on-failure

[Install]
WantedBy=default.target
```

```bash
# Enable for user
systemctl --user enable rninja-daemon
systemctl --user start rninja-daemon
```

## Monitoring

### Process Monitoring

```bash
# Watch daemon resource usage
top -p $(pgrep -f rninja-daemon)

# Or with htop
htop -p $(pgrep -f rninja-daemon)
```

### Log Monitoring

```bash
# Follow daemon logs (if using systemd)
journalctl -u rninja-daemon -f

# Or daemon's stderr if foreground
rninja-daemon --foreground 2>&1 | tee daemon.log
```

### Health Check Script

```bash
#!/bin/bash
# check-daemon.sh

if pgrep -f rninja-daemon > /dev/null; then
    echo "Daemon is running"
    exit 0
else
    echo "Daemon is NOT running"
    exit 1
fi
```

## Troubleshooting

### Daemon Won't Start

```bash
# Check if socket exists from dead daemon
rm /tmp/rninja-daemon.sock

# Check for port conflicts
lsof /tmp/rninja-daemon.sock

# Run foreground to see errors
rninja-daemon --foreground
```

### Connection Refused

```bash
# Daemon not running or wrong socket
pgrep -fa rninja-daemon

# Check socket exists
ls -la /tmp/rninja-daemon.sock

# Restart daemon
pkill -f rninja-daemon
rninja
```

### High Memory Usage

```bash
# Check memory
ps aux | grep rninja-daemon

# Restart to clear caches
pkill -f rninja-daemon

# Or limit cached repos
rninja-daemon --max-cached 5
```

### Builds Stalling

```bash
# Check daemon is responsive
rninja --no-daemon  # Bypass daemon

# If works, restart daemon
pkill -f rninja-daemon
rninja
```

## Best Practices

1. **Let auto-spawn work** for most cases
2. **Use systemd** for persistent systems
3. **Monitor resources** on build servers
4. **Restart periodically** if memory grows
5. **Use custom sockets** for isolation
