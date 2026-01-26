---
title: rninja-cached Architecture
description: Cache server architecture
tags:
  - architecture
---

# rninja-cached Architecture

Remote cache server component.

## Responsibilities

- Store build artifacts
- Serve cache requests
- Handle authentication
- Manage storage limits

## Key Modules

| Module | Purpose |
|--------|---------|
| `server/mod.rs` | Server lifecycle |
| `server/handler.rs` | Request handling |
| `server/auth.rs` | Authentication |

## Protocol

Uses NNG (nanomsg next-gen):

- Request/Reply pattern
- TCP transport
- MessagePack serialization

## Storage

Content-addressed blob storage:

```
/var/lib/rninja-cache/
├── index/      # sled database
└── blobs/      # Artifact files
```

## Scaling

Single server handles:

- ~1000 requests/second
- ~100 MB/second storage
- Hundreds of clients
