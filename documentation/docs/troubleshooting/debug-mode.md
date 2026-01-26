---
title: Debug Mode
description: Using rninja debug features
tags:
  - troubleshooting
---

# Debug Mode

rninja debugging features for troubleshooting builds.

## Debug Flags (-d)

### Stats Mode

Show build statistics:

```bash
rninja -d stats
```

Output:

```
metric                  count
rules                      15
pools                       2
edges                     250
outputs                   248
phonys                     12
build time               5.2s
command time            42.1s
```

### Explain Mode

Show why targets rebuild:

```bash
rninja -d explain
```

Output:

```
ninja explain: output foo.o doesn't exist
ninja explain: foo.cpp is newer than foo.o
ninja explain: command line changed for foo.o
```

This is invaluable for understanding unexpected rebuilds.

### Keep Depfile

Preserve dependency files:

```bash
rninja -d keepdepfile
```

Normally, `.d` files are deleted after parsing. This keeps them for inspection:

```bash
cat foo.o.d
# foo.o: foo.cpp foo.h bar.h
```

### Keep Response Files

Preserve response files:

```bash
rninja -d keeprsp
```

Keeps `.rsp` files for debugging command-line issues:

```bash
cat foo.o.rsp
# -O2 -Wall -I/include foo.cpp
```

## Verbose Mode (-v)

Show full commands:

```bash
rninja -v
```

Output:

```
[1/100] g++ -O2 -Wall -I/usr/include -c foo.cpp -o foo.o
[2/100] g++ -O2 -Wall -I/usr/include -c bar.cpp -o bar.o
```

Without `-v`, only descriptions are shown.

## Dry Run (-n)

Show what would be built without executing:

```bash
rninja -n
```

Combine with verbose:

```bash
rninja -n -v
```

This shows all commands that would run.

## Build Tracing

### Chrome Trace Format

Generate trace for visualization:

```bash
rninja --trace build.trace
```

View in:
- `chrome://tracing` (Chrome browser)
- [Perfetto UI](https://ui.perfetto.dev/)

The trace shows:
- Command execution timeline
- Parallel job utilization
- Critical path
- Cache hits/misses

### Interpreting Traces

```
Timeline:
|--foo.o----|
      |--bar.o----|
           |--baz.o--|
                     |--link--|
```

Look for:
- **Gaps**: Idle time, could increase -j
- **Long tasks**: Bottlenecks to optimize
- **Serial chains**: Critical path

## Logging

### Log Levels

```bash
# Error only
RNINJA_LOG_LEVEL=error rninja

# Warnings
RNINJA_LOG_LEVEL=warn rninja

# Info (default)
RNINJA_LOG_LEVEL=info rninja

# Debug
RNINJA_LOG_LEVEL=debug rninja

# Trace (very verbose)
RNINJA_LOG_LEVEL=trace rninja
```

### Rust Logging

For detailed internal logs:

```bash
RUST_LOG=rninja=debug rninja
RUST_LOG=rninja::cache=trace rninja
```

### Log to File

```bash
RNINJA_LOG_FILE=/tmp/rninja.log rninja
```

## Cache Debugging

### Cache Statistics

```bash
rninja -t cache-stats
```

Output:

```
Cache Statistics:
  Mode: auto (local + remote fallback)
  Local:
    Entries: 12,345
    Size: 5.2 GB / 10 GB
    Hit rate: 85%
  Remote:
    URL: tcp://cache.example.com:9876
    Connected: yes
    Hit rate: 92%
```

### Cache Explain

See why cache misses happen:

```bash
RNINJA_LOG_LEVEL=debug rninja 2>&1 | grep cache
```

Output:

```
[DEBUG] cache miss for foo.o: key abc123 not found
[DEBUG] cache hit for bar.o: key def456
[DEBUG] cache miss for baz.o: input changed
```

### Cache Key Inspection

```bash
# Show what affects cache key
rninja -d explain --cache-debug
```

## Daemon Debugging

### Daemon Status

```bash
rninja -t daemon-status
```

Output:

```
Daemon Status:
  PID: 12345
  Uptime: 2h 15m
  Memory: 125 MB
  Active builds: 1
  Total builds: 47
  Socket: /tmp/rninja-user-abc.sock
```

### Daemon Logs

```bash
# View daemon logs
journalctl -u rninja-daemon

# Or if running in foreground
rninja-daemon --foreground --log-level debug
```

## Network Debugging

### Connection Issues

```bash
# Test remote cache
curl -v tcp://cache.example.com:9876/health

# Debug connection
RNINJA_LOG_LEVEL=debug rninja 2>&1 | grep -i connect
```

### Timeout Issues

```bash
# Increase timeout
export RNINJA_REMOTE_TIMEOUT=60000  # 60 seconds

# Debug timeouts
RNINJA_LOG_LEVEL=debug rninja
```

## Common Debug Workflows

### Why is target rebuilding?

```bash
rninja -d explain target
```

### Why is build slow?

```bash
rninja --trace build.trace
# Open in chrome://tracing
```

### Why cache miss?

```bash
RNINJA_LOG_LEVEL=debug rninja 2>&1 | grep "cache miss"
```

### What commands will run?

```bash
rninja -n -v
```

### Is daemon working?

```bash
rninja -t daemon-status
rninja --no-daemon  # Compare performance
```

## Debug Configuration

Permanent debug settings in config:

```toml
# ~/.config/rninja/config.toml
[general]
verbose = true

[daemon]
log_level = "debug"

[cache]
debug = true
```
