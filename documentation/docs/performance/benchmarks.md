---
title: Benchmarks
description: Detailed performance benchmarks
tags:
  - performance
  - benchmarks
---

# Benchmarks

Detailed performance comparisons.

## Test Environment

- **OS**: Linux 6.x
- **CPU**: Multi-core system
- **Test**: 100 C source files
- **Toolchain**: GCC

## Build Times

### Full Build (Cold)

| Tool | Time | Notes |
|------|------|-------|
| ninja | 2.63s | Average of 5 runs |
| rninja | 1.79s | First cold run |
| rninja (cached) | 0.15s | With cache |

### No-op Build

| Tool | Time | Notes |
|------|------|-------|
| ninja | 0.23s | After warmup |
| rninja | 0.01s | Fast-path |

**Improvement: 23x faster**

### Incremental Build

After modifying one file:

| Tool | Time |
|------|------|
| ninja | 0.45s |
| rninja | 0.12s |

## Speedup by Scenario

| Scenario | Speedup |
|----------|---------|
| No-op builds | 10-23x |
| Cached rebuilds | 10-20x |
| Warm incremental | 2-5x |
| Cold builds | 1.3-2x |
| CI with remote cache | 2-5x |

## Running Benchmarks

```bash
# Generate benchmark project
python3 scripts/gen_bench.py 100

# Benchmark ninja
rm -f *.o .ninja_log
time ninja

# Benchmark rninja
rm -f *.o .ninja_log
time rninja
```

## Benchmark Tips

1. Run multiple iterations
2. Use hyperfine for accurate timing
3. Test with warm and cold caches
4. Profile with `--trace`
