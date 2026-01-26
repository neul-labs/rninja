---
title: Optimization Guide
description: Optimizing rninja performance
tags:
  - performance
  - optimization
---

# Optimization Guide

Getting maximum performance from rninja.

## Parallelism

### Use All Cores

```bash
rninja -j0  # Uses all CPU cores
```

### Tune for Memory

Large builds may need limiting:

```bash
rninja -j8  # Limit to 8 jobs
```

## Cache Optimization

### Enable Caching

```bash
export RNINJA_CACHE_ENABLED=1
```

### Use Fast Storage

```bash
export RNINJA_CACHE_DIR=/ssd/rninja-cache
```

### Remote Cache for Teams

```bash
export RNINJA_CACHE_MODE=auto
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache:9999
```

## Build File Optimization

### Declare All Dependencies

Undeclared dependencies cause cache misses.

### Use Depfiles

Let the compiler report dependencies:

```ninja
rule cc
  command = $cc -MD -MF $out.d -c $in -o $out
  depfile = $out.d
```

### Avoid Non-Determinism

Remove timestamps and random values from builds.

## Daemon Mode

Keep daemon running:

```bash
rninja  # Auto-spawns daemon
```

Subsequent builds are faster.

## Profiling

Identify bottlenecks:

```bash
rninja --trace trace.json
# Open in chrome://tracing
```

## Checklist

- [ ] Use all CPU cores (`-j0`)
- [ ] Enable caching
- [ ] Use SSD for cache
- [ ] Declare all dependencies
- [ ] Use depfiles
- [ ] Remove non-determinism
- [ ] Keep daemon running
- [ ] Profile slow builds
