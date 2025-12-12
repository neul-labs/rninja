# Market & Adoption Landscape

rninja addresses teams that already rely on Ninja-based build pipelines and want faster iteration without rewriting their build definitions. This document outlines the primary user segments and adoption levers.

## Core Personas

- **Large open-source maintainers**: Projects like browsers, compilers, and OS components that already output Ninja files (via GN, CMake, or Meson) and face multi-minute builds.
- **Enterprise C/C++ organizations**: Internal monorepos, embedded firmware groups, or cross-platform SDK teams where CI latency directly impacts developer productivity.
- **Game and simulation studios**: Heavy C++/GPU workloads with expensive linking and asset generation; these teams already use distributed build farms and value faster iteration loops.
- **High-performance and scientific computing teams**: Need deterministic, reproducible builds for regulated or research environments, making caching plus predictable scheduling attractive.

## Adoption Scope

- Tens of thousands of existing repositories already emit Ninja files; rninja simply replaces the executor binary and adds cache configuration, minimizing friction.
- Remote cache support multiplies value in multi-developer and CI contexts because every new machine can reuse the work of previous builds with zero DSL changes.
- Because rninja is language-agnostic (it executes arbitrary Ninja rules), it extends beyond C/C++ to Rust, Swift, Objective-C, CUDA, or any toolchain that integrates with Ninja today.

## Go-to-Market Motions

1. **Drop-in trials**: Provide static binaries that teams can swap into their CI systems alongside existing Ninja runs for side-by-side benchmarking.
2. **Developer advocacy**: Publish case studies demonstrating 2×–5× incremental build wins on recognizable OSS projects to build trust.
3. **Enterprise support offerings**: Offer SLAs, cache server hosting, and onboarding assistance for companies wanting managed infrastructure.
4. **Community contributions**: Encourage generator projects (CMake, Meson, GN) to document rninja compatibility, lowering awareness barriers.

## Technology Proof Points

- async-nng demonstrates that the remote cache transport relies on a well-tested nanomsg successor, giving ops teams confidence in latency and throughput characteristics.
- sled’s reputation as a crash-consistent embedded database reinforces reliability of local caching even under abrupt CI cancellations.
- ryv’s content-addressed store model highlights deduplicated artifact storage, an attractive efficiency story for teams with expensive builds and large binary outputs.

## Success Signals

- Reduction in CI compute minutes and developer wait times after enabling rninja.
- Growth in remote cache adoption, indicating cross-machine sharing is solving pain points.
- Positive feedback from early adopters regarding compatibility with depfiles, generator rules, and custom toolchains.

Understanding these segments and signals guides prioritization for features, integrations, and documentation as rninja matures.
