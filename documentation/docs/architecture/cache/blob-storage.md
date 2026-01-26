---
title: Blob Storage
description: Artifact storage in rninja
tags:
  - architecture
  - cache
---

# Blob Storage

Content-addressed artifact storage.

## Structure

```
blobs/
├── ab/
│   └── abcdef123456...
├── cd/
│   └── cdef789abc...
└── ef/
    └── efgh012def...
```

## Content Addressing

Files named by their BLAKE3 hash:

- Automatic deduplication
- Corruption detection
- Efficient storage

## Storage Format

Uses rkyv for zero-copy serialization:

```rust
struct CacheEntry {
    output_data: Vec<u8>,
    metadata: EntryMetadata,
}
```

## Deduplication

Identical content stored once:

```
file1.o (hash: abc) ─┐
                     ├─→ blob:abc (stored once)
file2.o (hash: abc) ─┘
```

## Cleanup

Garbage collection removes:

- Orphaned blobs
- Old entries
- Entries exceeding size limit
