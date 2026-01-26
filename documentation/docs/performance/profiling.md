---
title: Profiling Builds
description: Analyzing build performance
tags:
  - performance
  - profiling
---

# Profiling Builds

Analyze build performance to identify bottlenecks.

## Chrome Trace

### Generate Trace

```bash
rninja --trace build_trace.json
```

### View Trace

1. Open Chrome
2. Navigate to `chrome://tracing`
3. Load `build_trace.json`

### What to Look For

- **Long bars**: Slow build steps
- **Gaps**: Scheduling inefficiency
- **Serialization**: Sequential dependencies

## Build Statistics

```bash
rninja -d stats
```

Shows:

- Total build time
- Commands executed
- Cache statistics

## Explain Mode

```bash
rninja -d explain
```

Shows why targets are rebuilding.

## Cache Analysis

```bash
rninja -t cache-stats
```

Low hit rate? Check:

- Undeclared dependencies
- Non-deterministic builds
- Environment differences

## Example Analysis

```bash
# 1. Generate trace
rninja --trace trace.json

# 2. Check stats
rninja -d stats

# 3. Check cache
rninja -t cache-stats

# 4. Investigate rebuilds
rninja -d explain
```

## Common Issues

| Issue | Symptom | Solution |
|-------|---------|----------|
| Sequential builds | Gaps in trace | Add parallelism |
| Low cache hits | Rebuilding cached | Check deps |
| Slow no-op | > 100ms startup | Use daemon |
