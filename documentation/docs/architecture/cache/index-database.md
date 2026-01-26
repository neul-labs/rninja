---
title: Index Database
description: Cache index database
tags:
  - architecture
  - cache
---

# Index Database

Cache key to blob mapping.

## Technology

Uses sled embedded database:

- ACID transactions
- Crash-safe
- Fast lookups
- Pure Rust

## Schema

```
Key: cache_key (32 bytes, BLAKE3)
Value: {
    blob_hash: [32 bytes],
    created: timestamp,
    size: u64,
    access_count: u32,
}
```

## Operations

| Operation | Complexity |
|-----------|------------|
| Lookup | O(log n) |
| Insert | O(log n) |
| Delete | O(log n) |

## Durability

- Write-ahead logging
- Atomic transactions
- Recovery on startup

## Maintenance

```bash
# Compact database
rninja -t recompact

# Check health
rninja -t cache-health
```
