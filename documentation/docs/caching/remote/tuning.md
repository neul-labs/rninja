---
title: Remote Cache Performance Tuning
description: Optimizing remote cache performance
tags:
  - caching
  - remote
  - performance
---

# Remote Cache Performance Tuning

Optimize your remote cache for maximum performance.

## Client-Side Tuning

### Timeout Configuration

Adjust timeouts based on network conditions:

```bash
# Fast, reliable network
export RNINJA_CACHE_CONNECT_TIMEOUT=2
export RNINJA_CACHE_REQUEST_TIMEOUT=15

# Slower or less reliable network
export RNINJA_CACHE_CONNECT_TIMEOUT=10
export RNINJA_CACHE_REQUEST_TIMEOUT=60

# High-latency WAN
export RNINJA_CACHE_CONNECT_TIMEOUT=15
export RNINJA_CACHE_REQUEST_TIMEOUT=120
```

### Concurrency Settings

```bash
# Default (good for most cases)
export RNINJA_CACHE_MAX_CONCURRENT=4

# High-bandwidth network
export RNINJA_CACHE_MAX_CONCURRENT=8

# Limited bandwidth or shared connection
export RNINJA_CACHE_MAX_CONCURRENT=2
```

### Pull Policy Optimization

```bash
# Network is faster than disk: always check remote
export RNINJA_CACHE_PULL_POLICY=always

# Disk is faster: check local first
export RNINJA_CACHE_PULL_POLICY=on_miss
```

## Server-Side Tuning

### Storage Configuration

#### Use Fast Storage

```bash
# NVMe SSD recommended
--storage /nvme/rninja-cache

# Avoid HDD for large caches
# Avoid network storage (NFS, CIFS)
```

#### Filesystem Tuning

```bash
# XFS with optimal settings
mkfs.xfs -n size=64k /dev/nvme0n1p1
mount -o noatime,nodiratime /dev/nvme0n1p1 /var/lib/rninja-cache
```

### Memory Configuration

The server uses memory for:

- Database caching
- Connection handling
- Request buffering

Recommendations:

| Cache Size | Minimum RAM | Recommended RAM |
|------------|-------------|-----------------|
| < 50 GB | 2 GB | 4 GB |
| 50-200 GB | 4 GB | 8 GB |
| > 200 GB | 8 GB | 16 GB |

### Size Limits

```bash
# Set appropriate max size
--max-size 100G

# Consider:
# - Available disk space
# - Number of projects
# - Artifact sizes
# - Retention needs
```

## Network Optimization

### Server Placement

```
Best:
[Clients] <--LAN--> [Cache Server]

Good:
[Clients] <--Datacenter Network--> [Cache Server]

Avoid:
[Clients] <--Internet--> [Cache Server]
```

### Bandwidth Planning

```
Required bandwidth = (Concurrent builds) × (Avg transfer size) / (Acceptable latency)

Example:
- 10 concurrent builds
- 5 MB average artifact
- 1 second acceptable latency
- Bandwidth = 10 × 5 MB / 1s = 50 MB/s = 400 Mbps
```

### Load Balancing

For high-traffic deployments:

```nginx
upstream rninja_cache {
    least_conn;
    server cache1.internal:9999;
    server cache2.internal:9999;
}
```

!!! note "Cache Consistency"
    Load balancing requires cache synchronization (not yet implemented). For now, use single server or sticky sessions.

## Monitoring Performance

### Key Metrics

| Metric | Good | Warning | Action |
|--------|------|---------|--------|
| Hit rate | > 80% | < 50% | Check cache size/retention |
| Latency (lookup) | < 50ms | > 200ms | Check network/storage |
| Latency (store) | < 100ms | > 500ms | Check storage I/O |

### Collecting Metrics

```bash
# Client-side stats
rninja -t cache-stats

# Server logs
journalctl -u rninja-cached | grep -E 'latency|throughput'
```

### Performance Testing

```bash
# Measure cache lookup time
time rninja -t cache-stats

# Measure build with caching
time rninja

# Compare with cache disabled
time RNINJA_CACHE_ENABLED=0 rninja
```

## Common Performance Issues

### High Cache Miss Rate

**Symptoms:** Cache stats show low hit rate

**Causes:**

- Cache too small
- High artifact churn
- Non-deterministic builds

**Solutions:**

```bash
# Increase cache size
--max-size 200G

# Increase retention
# (Remove max_age or increase it)

# Fix non-deterministic builds
rninja -d explain  # Check what's changing
```

### Slow Lookups

**Symptoms:** Builds slower with remote cache

**Causes:**

- Network latency
- Server overload
- Slow storage

**Solutions:**

```bash
# Client: Use on_miss policy
export RNINJA_CACHE_PULL_POLICY=on_miss

# Server: Use faster storage
--storage /nvme/cache

# Network: Place server closer to clients
```

### Slow Stores

**Symptoms:** Build time increases when caching new artifacts

**Causes:**

- Network bandwidth
- Slow storage writes
- Large artifacts

**Solutions:**

```bash
# Increase concurrency for better throughput
export RNINJA_CACHE_MAX_CONCURRENT=8

# Use async push (if available)
export RNINJA_CACHE_PUSH_POLICY=on_success

# Server: Use SSD storage
```

### Connection Timeouts

**Symptoms:** Frequent fallbacks to local cache

**Causes:**

- Network issues
- Server overloaded
- Timeout too short

**Solutions:**

```bash
# Increase timeouts
export RNINJA_CACHE_CONNECT_TIMEOUT=10
export RNINJA_CACHE_REQUEST_TIMEOUT=60

# Check server health
# Add more server resources
```

## Configuration Profiles

### Low-Latency Profile (LAN)

```bash
# Client
export RNINJA_CACHE_CONNECT_TIMEOUT=2
export RNINJA_CACHE_REQUEST_TIMEOUT=15
export RNINJA_CACHE_MAX_CONCURRENT=8
export RNINJA_CACHE_PULL_POLICY=always
```

### High-Throughput Profile (CI)

```bash
# Client
export RNINJA_CACHE_CONNECT_TIMEOUT=5
export RNINJA_CACHE_REQUEST_TIMEOUT=60
export RNINJA_CACHE_MAX_CONCURRENT=16
export RNINJA_CACHE_PUSH_POLICY=always
```

### Resilient Profile (Unreliable Network)

```bash
# Client
export RNINJA_CACHE_MODE=auto
export RNINJA_CACHE_CONNECT_TIMEOUT=3
export RNINJA_CACHE_REQUEST_TIMEOUT=30
export RNINJA_CACHE_PULL_POLICY=on_miss
export RNINJA_CACHE_MAX_CONCURRENT=2
```

## Best Practices

1. **Start with defaults** - Tune only when needed
2. **Measure before optimizing** - Use cache stats
3. **Tune client and server together** - Both affect performance
4. **Monitor continuously** - Performance can degrade over time
5. **Plan for growth** - Size cache for future needs
