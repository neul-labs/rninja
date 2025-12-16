# Architecture Overview

rninja is organized as a set of focused components that together replicate Ninja’s execution semantics while layering in caching and remote distribution powered by async-nng, sled, and rkyv.

## Component Breakdown

- **Command Interface / CLI Client**  
  Parses user flags/environment variables, then forwards requests to the resident rninja daemon. It mirrors Ninja’s UX while also exposing daemon controls (`rninja daemon status`, `rninja daemon stop`).

- **Daemon Supervisor**  
  A long-lived process that maintains cache handles, thread pools, and repository sessions. It multiplexes work from every CLI invocation on the machine, ensuring a single scheduler arbitrates resources.

- **Scheduler & Executor**  
  Lives inside the daemon, implementing dependency graph traversal, ready queue, and resource accounting. Uses async runtimes to keep CPUs saturated while respecting edge ordering and restat semantics.

- **Cache Index (sled)**  
  Stores records keyed by content digests: command line, inputs, environment, and metadata (timestamps, deps). sled’s crash-safe tree ensures rninja can resume after failures without corrupting the cache.

- **Blob Store (rkyv)**  
  Holds actual build artifacts (object files, archives, generated assets) as deduplicated blobs referenced by the sled index. rkyv’s content-addressing means identical outputs are stored once regardless of origin.

- **Remote Transport (async-nng)**  
  Streams cache entries between machines using Nanomsg Next Gen sockets. async-nng provides async publish/subscribe and request/reply patterns, letting rninja push/fetch blobs without blocking the scheduler.

- **Compatibility Layer**  
  Validates depfiles, rsp files, pools, and generator behaviors to guarantee parity with Ninja. Includes test fixtures mirroring Ninja’s regression suite.

## Drop-in Guarantees

1. **Identical graph interpretation**: `build.ninja` files are parsed with the same rules, and unknown constructs fail the same way as stock Ninja.
2. **Deterministic outputs**: rninja avoids reordering that would break rule expectations; caching is strictly additive.
3. **Transparent fallbacks**: If caching or remote transport encounters errors, rninja can bypass those layers and execute commands directly to keep builds unblocked.

## Data Flow Summary

```
CLI request -> Daemon -> Scheduler -> Command execution
                               |-> Cache lookup (sled index + rkyv blobs)
                               |-> Remote fetch/push via async-nng
```

When a command is ready:

1. Scheduler asks the cache index whether an equivalent result exists (hash computed from inputs/toolchain args).
2. On hit, blob store provides the artifact, optionally fetched remotely through async-nng.
3. On miss, rninja executes the action, writes outputs, updates the sled index, stores blobs in rkyv, and propagates them via async-nng if remote caching is enabled.

This architecture keeps rninja faithful to Ninja’s simplicity while leveraging modern Rust crates to extend performance and distribution capabilities.
