---
title: Common Issues
description: Solutions to common rninja problems
tags:
  - troubleshooting
---

# Common Issues

Solutions to frequently encountered problems.

## Build Issues

### "ninja: error: loading 'build.ninja'"

**Cause:** Build file not found.

**Solution:**

```bash
# Check file exists
ls build.ninja

# Specify different file
rninja -f path/to/build.ninja

# Generate build files first
cmake -G Ninja -B build
meson setup build
```

### "ninja: error: unknown target 'X'"

**Cause:** Target doesn't exist in build file.

**Solution:**

```bash
# List available targets
rninja -t targets

# Check target name
rninja -t query targetname
```

### Build runs but produces wrong output

**Cause:** Stale build artifacts or incorrect dependencies.

**Solution:**

```bash
# Clean and rebuild
rninja -t clean
rninja

# Or full clean
rm -rf .ninja_log .ninja_deps
rninja
```

### Build hangs or is very slow

**Cause:** Resource exhaustion or too many parallel jobs.

**Solution:**

```bash
# Limit parallelism
rninja -j4

# Check system load
rninja -l 4.0  # Stop spawning at load 4.0

# Check for stuck processes
ps aux | grep rninja
```

## Cache Issues

### "Cache miss on every build"

**Cause:** Cache keys changing unexpectedly.

**Solution:**

```bash
# Check cache mode
rninja --explain

# Verify cache is enabled
rninja -t cache-stats

# Check environment variables affecting builds
env | grep -E 'CC|CXX|FLAGS'
```

Common causes:
- Environment variables in cache key
- Absolute paths in commands
- Timestamps embedded in output

### "Cache directory growing too large"

**Cause:** No garbage collection configured.

**Solution:**

```bash
# Run garbage collection
rninja -t cache-gc

# Set size limit
echo '[cache]
max_size = "10G"' >> ~/.config/rninja/config.toml

# Clear old entries
rninja -t cache-gc --max-age 7d
```

### "Remote cache connection failed"

**Cause:** Network or authentication issue.

**Solution:**

```bash
# Test connectivity
curl -v $RNINJA_REMOTE_URL/health

# Check token
echo $RNINJA_CACHE_TOKEN

# Try with debug logging
RNINJA_LOG_LEVEL=debug rninja

# Fall back to local cache
rninja --cache=local
```

### "Cache corruption detected"

**Cause:** Interrupted writes or disk issues.

**Solution:**

```bash
# Check cache health
rninja -t cache-health

# Repair if possible
rninja -t cache-repair

# Clear and start fresh
rninja -t cache-clear
```

## Daemon Issues

### "Failed to connect to daemon"

**Cause:** Daemon not running or socket issue.

**Solution:**

```bash
# Check daemon status
rninja -t daemon-status

# Restart daemon
rninja -t daemon-stop
rninja -t daemon-start

# Run without daemon
rninja --no-daemon
```

### "Daemon using too much memory"

**Cause:** Large project or memory leak.

**Solution:**

```bash
# Restart daemon
rninja -t daemon-stop

# Set memory limit
echo '[daemon]
max_memory = "1G"' >> ~/.config/rninja/config.toml

# Use single-shot mode for large builds
rninja --no-daemon
```

### "Multiple daemon instances"

**Cause:** Stale lock files.

**Solution:**

```bash
# Kill all daemons
pkill -f rninja-daemon

# Remove lock files
rm /tmp/rninja-*.lock

# Restart
rninja -t daemon-start
```

## Compatibility Issues

### "Works with ninja but not rninja"

**Cause:** Edge case or rninja bug.

**Solution:**

```bash
# Compare output
ninja -n -v > ninja.out 2>&1
rninja -n -v > rninja.out 2>&1
diff ninja.out rninja.out

# Report the issue with details
# Include: build.ninja, command output, versions
```

### "depfile parsing error"

**Cause:** Non-standard depfile format.

**Solution:**

```bash
# Check depfile content
cat foo.o.d

# Use keepdepfile to preserve for debugging
rninja -d keepdepfile

# Check compiler generating correct format
```

### "response file errors"

**Cause:** Response file issues on Windows or with specific compilers.

**Solution:**

```bash
# Keep response files for debugging
rninja -d keeprsp

# Check response file content
cat foo.o.rsp
```

## Permission Issues

### "Permission denied on socket"

**Cause:** Socket permissions or different user.

**Solution:**

```bash
# Check socket permissions
ls -la /tmp/rninja-*.sock

# Remove stale socket
rm /tmp/rninja-$USER-*.sock

# Set custom socket path
export RNINJA_DAEMON_SOCKET=/tmp/my-rninja.sock
```

### "Cannot write to cache directory"

**Cause:** Permission or disk space issue.

**Solution:**

```bash
# Check permissions
ls -la ~/.cache/rninja

# Check disk space
df -h ~/.cache

# Use different location
export RNINJA_CACHE_DIR=/path/with/space
```

## Performance Issues

### "rninja slower than ninja"

**Cause:** Overhead from features or misconfiguration.

**Solution:**

```bash
# Disable features for comparison
rninja --no-cache --no-daemon

# Check what's taking time
rninja --trace build.trace
# Open in chrome://tracing

# Verify not doing unnecessary work
rninja -d explain
```

### "Cache lookups slow"

**Cause:** Large cache or slow storage.

**Solution:**

```bash
# Compact cache
rninja -t recompact

# Check cache size
rninja -t cache-stats

# Run garbage collection
rninja -t cache-gc

# Use SSD for cache
export RNINJA_CACHE_DIR=/ssd/rninja-cache
```

## Getting Help

If your issue isn't listed:

1. Check [Debug Mode](debug-mode.md) for diagnostic tools
2. Review [Diagnostics](diagnostics.md) for system checks
3. Search [FAQ](faq.md)
4. Open issue at [GitHub](https://github.com/anthropics/rninja/issues)

Include:
- rninja version (`rninja --version`)
- OS and version
- Minimal reproducer
- Full error output
