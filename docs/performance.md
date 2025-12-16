# Performance Expectations

This document explains how rninja improves build performance compared to baseline Ninja executions and how to reason about the benefits during the project’s first two phases (executor + cache + optional remote cache).

## Baseline Characteristics

- Ninja is already optimized for low overhead: it parses `build.ninja` files once, then executes dependency edges with minimal bookkeeping.
- The planner/executor separation (a generator produces `.ninja` files; Ninja executes them) keeps everyday builds fast, but provides little built-in caching or scheduling intelligence beyond `-j`.

Because rninja is a drop-in executor, all improvements must build on this already efficient baseline.

## Where rninja Adds Speed

1. **Aggressive parallel scheduling**  
   rninja keeps CPUs, disk, and network busy by using a smarter queue that accounts for resource contention. This closes gaps where plain Ninja leaves cores idle because it schedules conservatively or bottlenecks on single-threaded bookkeeping.

2. **Content-addressed caching**  
   Every command captures its inputs (source, headers, compiler options) and outputs (object files, generated assets). If the exact command is requested again, rninja rehydrates artifacts instead of spawning the toolchain.

3. **Remote cache sharing**  
   Build artifacts are optionally pushed to a remote store so other machines—developer laptops or CI nodes—can reuse them immediately, turning “first build” on a new machine into a mostly cached build.

### Implementation Ingredients

- `sled` keeps the cache index and dependency fingerprints durable with crash-safe transactions, so the executor can make instant reuse decisions even after abrupt stops.
- `rkyv` stores blob payloads (objects, archives, generated assets) in a deduplicated content-addressed form that sled and the scheduler can reference cheaply.
- `async-nng` streams cache entries between peers or cache servers in parallel, ensuring remote fetch/push latency stays low enough to not bottleneck execution.

## Estimated Speedups

| Scenario | Conditions | Expected Impact |
| --- | --- | --- |
| Warm incremental developer build | Small source edits, no header churn; cache hit rate high | 2× – 5× faster wall clock time |
| CI pipeline with warmed remote cache | Multiple CI jobs or commits compiling similar targets | 2× – 5× fewer compute minutes |
| Cold full build on many cores | Entire codebase recompiled; little cache reuse but improved scheduling | 1.3× – 2× faster |

Estimates assume large C/C++ style repos (hundreds to thousands of translation units) where compile/link dominates cost.

## Factors That Influence Gains

- **Cache hit rate**: Touching widely included headers will invalidate many targets; conversely, focused edits produce near-perfect reuse.
- **Project size**: Small projects finish quickly even under vanilla Ninja, so rninja’s scheduling overheads matter less.
- **IO and network throughput**: Fast SSDs and low-latency cache servers maximize the benefit of remote cache transfers.
- **Rule determinism**: Commands with nondeterministic outputs (timestamps, random seeds) reduce cache usability; enforcing determinism pays off.

## Estimating Benefits for Your Repo

1. Measure baseline incremental and full build times with `ninja -d stats` to capture edge counts and critical path lengths.
2. Estimate cache reuse by inspecting your VCS history: how often do builds touch header files or change compiler flags?
3. Apply the scenario table above to project-specific metrics; e.g., if 70% of daily builds are incremental, multiply their duration by a conservative 0.4–0.5× to gauge rninja savings.
4. Add CI sharing effects by counting how many redundant builds run per day—remote cache can eliminate most after the first job warms the cache.

Tracking these metrics before deployment gives a measurable baseline to validate rninja’s impact once prototypes are available.
