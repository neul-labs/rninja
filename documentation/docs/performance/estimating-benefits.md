---
title: Estimating Benefits
description: Estimate rninja benefits for your project
tags:
  - performance
---

# Estimating Benefits

Estimate the performance improvements for your project.

## Quick Estimation

| Project Type | Expected Speedup |
|--------------|------------------|
| Small (< 100 files) | 1.5-3x |
| Medium (100-1000 files) | 2-5x |
| Large (> 1000 files) | 3-10x |
| Monorepo with sharing | 5-20x |

## Factors

### Positive Factors

- Large number of targets
- Frequent incremental builds
- Team sharing (remote cache)
- Branch switching
- CI/CD pipelines

### Neutral Factors

- Already fast builds (< 10s)
- Single developer
- Always-changing inputs

## Calculation

```
Time Saved = (Builds/day) × (Avg build time) × (1 - 1/Speedup)

Example:
- 20 builds/day
- 5 minute average
- 3x speedup
- Saved: 20 × 5 × (1 - 1/3) = 67 minutes/day
```

## Test Your Project

```bash
# Current (ninja)
time ninja

# Clean rebuild
ninja -t clean
time ninja

# With rninja
time rninja

# Clean + cached
rninja -t clean
time rninja

# Check cache hits
rninja -t cache-stats
```

## ROI Estimation

| Metric | Value |
|--------|-------|
| Developer hourly cost | $X |
| Time saved per day | Y minutes |
| Developers | N |
| Annual savings | X × Y/60 × N × 250 |
