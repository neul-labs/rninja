---
title: Cache Troubleshooting
description: Solving common caching issues
tags:
  - caching
  - troubleshooting
---

# Cache Troubleshooting

Solutions for common caching issues.

## Low Cache Hit Rate

### Symptoms

- Cache stats show low hit rate (<50%)
- Builds not faster than expected
- Same files rebuilding repeatedly

### Diagnosis

```bash
# Check cache stats
rninja -t cache-stats

# Enable explain mode
rninja -d explain
```

### Common Causes

#### Non-Deterministic Inputs

Builds that include timestamps, random values, or machine-specific paths:

```bash
# Check for problematic patterns
rninja -t commands | grep -E '__DATE__|__TIME__|RANDOM'
```

**Solution:** Remove non-deterministic inputs or normalize them.

#### Environment Differences

Different compilers or flags between machines:

```bash
# Compare environments
env | grep -E 'CC|CXX|CFLAGS' | sort
```

**Solution:** Standardize build environments.

#### Missing Dependencies

Implicit dependencies not declared in build files:

```bash
# Check declared dependencies
rninja -t deps target
```

**Solution:** Declare all dependencies properly.

#### Compiler Version Changes

Different compiler versions produce different artifacts.

**Solution:** Use consistent compiler versions across team/CI.

## Cache Not Working

### Symptoms

- Every build is a cache miss
- Cache stats show 0% hit rate
- Cache directory empty or not growing

### Diagnosis

```bash
# Check if caching is enabled
rninja -t cache-stats

# Verify cache directory
ls -la ~/.cache/rninja
```

### Common Causes

#### Caching Disabled

```bash
# Check environment
echo $RNINJA_CACHE_ENABLED

# Check config
rninja -t config -v
```

**Solution:**

```bash
export RNINJA_CACHE_ENABLED=1
```

#### Permission Issues

```bash
# Check directory permissions
ls -la ~/.cache/rninja
```

**Solution:**

```bash
chmod -R u+rw ~/.cache/rninja
```

#### Disk Full

```bash
df -h ~/.cache
```

**Solution:**

```bash
rninja -t cache-gc
# Or clear cache
rm -rf ~/.cache/rninja
```

## Remote Cache Issues

### Cannot Connect

```bash
# Test connectivity
nc -zv cache.internal 9999
```

**Solutions:**

- Check firewall rules
- Verify server is running
- Check network connectivity
- Use `auto` mode for fallback

### Authentication Failures

```bash
# Verify token is set
echo $RNINJA_CACHE_TOKEN
```

**Solutions:**

- Verify token is correct
- Check token hasn't expired
- Contact cache administrator

### Slow Remote Cache

```bash
# Check latency
ping cache.internal

# Check timeout settings
echo $RNINJA_CACHE_CONNECT_TIMEOUT
```

**Solutions:**

```bash
# Increase timeouts
export RNINJA_CACHE_CONNECT_TIMEOUT=10
export RNINJA_CACHE_REQUEST_TIMEOUT=60

# Or use local-first
export RNINJA_CACHE_PULL_POLICY=on_miss
```

## Cache Corruption

### Symptoms

- Build errors mentioning cache
- Inconsistent build results
- Health check failures

### Diagnosis

```bash
rninja -t cache-health
```

### Solution

```bash
# Try garbage collection first
rninja -t cache-gc

# If still broken, clear cache
rm -rf ~/.cache/rninja

# Rebuild
rninja
```

## Build Produces Wrong Results

### Symptoms

- Build succeeds but output is incorrect
- Different results with/without cache
- Stale artifacts being used

### Diagnosis

```bash
# Build without cache
RNINJA_CACHE_ENABLED=0 rninja -t clean && RNINJA_CACHE_ENABLED=0 rninja

# Compare to cached build
rninja -t clean && rninja
```

### Common Causes

#### Undeclared Dependencies

File changes not detected because dependency not declared.

**Solution:** Add missing dependencies to build file.

#### Order-Dependent Builds

Build depends on execution order.

**Solution:** Make build deterministic.

### Emergency Fix

```bash
# Disable caching temporarily
RNINJA_CACHE_ENABLED=0 rninja

# Report issue at github.com/neul-labs/rninja/issues
```

## Cache Too Large

### Symptoms

- Disk space warnings
- Cache using excessive storage

### Diagnosis

```bash
du -sh ~/.cache/rninja
rninja -t cache-stats
```

### Solutions

```bash
# Set size limit
export RNINJA_CACHE_MAX_SIZE=5G

# Run garbage collection
rninja -t cache-gc

# Set age limit
export RNINJA_CACHE_MAX_AGE=604800  # 7 days
```

## Performance Issues

### Cache Slower Than Expected

#### Check Cache Location

```bash
# If on network drive, use local
export RNINJA_CACHE_DIR=/local/fast/disk/rninja
```

#### Check Cache Health

```bash
rninja -t cache-health
rninja -t cache-gc  # Cleanup can improve performance
```

### Builds Slower with Cache

Rare, but possible with:

- Very fast builds (cache overhead > build time)
- Slow disk
- Network cache latency

**Solution:**

```bash
# Disable for small projects
export RNINJA_CACHE_ENABLED=0
```

## Debugging Tools

### Enable Debug Logging

```bash
# Set RUST_LOG for detailed cache logs
RUST_LOG=rninja::cache=debug rninja
```

### Cache Statistics

```bash
# Detailed stats
rninja -t cache-stats

# Watch over time
watch -n 5 'rninja -t cache-stats'
```

### Health Check

```bash
rninja -t cache-health
```

## Getting Help

If issues persist:

1. Collect information:
   ```bash
   rninja --version
   rninja -t cache-stats
   rninja -t cache-health
   env | grep RNINJA
   ```

2. Check GitHub Issues:
   [github.com/neul-labs/rninja/issues](https://github.com/neul-labs/rninja/issues)

3. Open new issue with:
   - rninja version
   - Operating system
   - Steps to reproduce
   - Cache stats output
