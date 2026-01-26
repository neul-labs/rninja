---
title: Data Flow
description: Request lifecycle in rninja
tags:
  - architecture
---

# Data Flow

How requests flow through rninja.

## Build Request Flow

```mermaid
sequenceDiagram
    participant CLI as rninja CLI
    participant D as Daemon
    participant LC as Local Cache
    participant RC as Remote Cache
    participant E as Executor

    CLI->>D: Build request
    D->>D: Parse manifest
    loop Each target
        D->>D: Compute cache key
        D->>LC: Check local cache
        alt Local hit
            LC-->>D: Artifact
        else Local miss
            D->>RC: Check remote cache
            alt Remote hit
                RC-->>D: Artifact
                D->>LC: Store locally
            else Remote miss
                D->>E: Execute command
                E-->>D: Output
                D->>LC: Store locally
                D->>RC: Store remotely
            end
        end
    end
    D-->>CLI: Build complete
```

## Cache Key Computation

```
key = BLAKE3(
    rule_name +
    command +
    input_hashes +
    env_vars
)
```

## Artifact Storage

```
~/.cache/rninja/
├── index/          # Key → blob mapping
└── blobs/
    └── ab/
        └── abcdef...  # Content-addressed blobs
```
