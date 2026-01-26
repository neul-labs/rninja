---
title: Performance Overview
description: rninja performance characteristics
tags:
  - performance
---

# Performance Overview

Understanding rninja's performance advantages.

## Performance Comparison

| Scenario | Ninja | rninja | Improvement |
|----------|-------|--------|-------------|
| No-op build | 0.23s | 0.01s | 23x |
| Cold build (100 files) | 2.63s | 1.79s | 1.5x |
| Cached rebuild | 2.63s | 0.15s | 17.5x |
| Warm incremental | - | 2-5x | faster |

## Why rninja is Faster

### Content-Addressed Caching

- Skip rebuilds when inputs unchanged
- Cache survives clean builds
- Branch switching reuses artifacts

### Fast No-op Detection

- Optimized buildlog with pre-computed hashes
- MtimeCache avoids repeated stat() calls
- Parallel file checking with rayon

### Modern Scheduler

- Tokio async runtime
- Pool-aware scheduling
- Better CPU utilization

## Key Metrics

| Metric | Target | Good |
|--------|--------|------|
| No-op build | < 50ms | < 100ms |
| Cache hit rate | > 80% | > 60% |
| Cache lookup | < 50ms | < 100ms |

## Next Steps

- [Benchmarks](benchmarks.md) - Detailed performance data
- [Optimization](optimization.md) - Tuning guide
- [Profiling](profiling.md) - Build analysis
