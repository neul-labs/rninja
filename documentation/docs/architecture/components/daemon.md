---
title: rninja-daemon Architecture
description: Daemon component architecture
tags:
  - architecture
---

# rninja-daemon Architecture

Long-running build daemon.

## Responsibilities

- Cache parsed manifests
- Schedule build execution
- Manage cache access
- Handle multiple clients

## Key Modules

| Module | Purpose |
|--------|---------|
| `daemon/server.rs` | IPC server |
| `daemon/session.rs` | Build sessions |
| `daemon/state.rs` | Cached state |

## Design Decisions

### Single Process

One daemon per machine:

- Shared manifest cache
- Coordinated disk access
- Memory efficiency

### Session Isolation

Each build request:

- Separate session
- Isolated state
- Independent failure handling

### Auto-Spawn

CLI spawns daemon if needed:

- Transparent to user
- No manual management
- Graceful degradation
