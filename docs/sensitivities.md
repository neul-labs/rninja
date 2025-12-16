# Sensitivities & Risks

Adoption of rninja hinges on getting several engineering details right. This document enumerates the most important sensitivities so they can be tracked during design and implementation.

## Technical Sensitivities

- **Scheduling quality**: Subpar prioritization or resource modeling negates performance claims. Instrument queue depth, critical path execution, and per-core utilization to validate improvements over stock Ninja.
- **Cache correctness**: Content hashing must cover sources, headers, toolchain flags, and environment variables. Any omission risks stale or incorrect outputs, eroding trust quickly.
- **Remote cache stability**: Network retries, authentication, and eviction policies must be robust; CI systems will not tolerate flaky cache servers.
- **Exact Ninja semantics**: rninja must honor depfiles, `restat`, generator edges, and phony targets. Behavioral drift leads to subtle build breakages and blocks migration.
- **Dependency alignment with async-nng/sled/rkyv**: Each crate introduces expectations around threading, durability, and data formats. Upgrades or configuration mistakes can cascade into performance regressions if not validated.

## Product Sensitivities

- **Ease of onboarding**: Installation friction should be near zero—ideally a single binary plus optional cache config. If teams must modify build generators, adoption will stall.
- **Determinism & reproducibility**: Build systems are safety-critical infrastructure. Early versions should prioritize extensive validation suites and reproducibility checks before advertising aggressive performance claims.
- **Observability**: Developers need clear logs and metrics (cache hits, misses, remote fetch timings) to trust the tool. Lack of transparency makes troubleshooting difficult.

## External Factors

- **Cache hit rate variance**: Teams with frequent sweeping changes (e.g., header churn) will see limited gains. Educating users on best practices (e.g., isolating unstable headers) increases success odds.
- **Hardware diversity**: Mixed developer environments (Linux/macOS/Windows) require consistent hashing and artifact compatibility; otherwise caches fragment.
- **Security posture**: Remote caches introduce authentication and artifact-signing requirements, especially inside enterprises with strict compliance norms.
- **Third-party crate evolution**: async-nng, sled, and rkyv each evolve at their own cadence. Pin versions, monitor upstream advisories, and plan migration stories to avoid sudden breaking changes or CVEs.

By tracking these sensitivities alongside performance KPIs, rninja can stay aligned with user expectations and de-risk the product roadmap.
