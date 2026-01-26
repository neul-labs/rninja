---
title: Hashing Strategy
description: Content hashing in rninja
tags:
  - architecture
  - cache
---

# Hashing Strategy

How rninja computes cache keys.

## BLAKE3

rninja uses BLAKE3 for all hashing:

- **Fast**: ~6 GB/s on modern CPUs
- **Secure**: Cryptographic strength
- **Parallel**: Uses all cores

## Cache Key Computation

```
key = BLAKE3(
    rule_name +
    command_line +
    input_contents +
    environment_vars
)
```

### What's Included

| Input | Example |
|-------|---------|
| Rule | `cc`, `link` |
| Command | `gcc -O2 -c ...` |
| Source content | Hash of input files |
| Headers | Via depfile |
| Environment | `CC`, `CFLAGS` |

### What's NOT Included

- File paths (only content)
- Timestamps
- Machine-specific info

## Why BLAKE3

| Algorithm | Speed | Security |
|-----------|-------|----------|
| MD5 | Medium | Broken |
| SHA-256 | Slow | Good |
| BLAKE3 | Fast | Excellent |

## Determinism

Same inputs always produce same key:

- Reproducible builds
- No false cache hits
- Correct invalidation
