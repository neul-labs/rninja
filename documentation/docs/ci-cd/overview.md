---
title: CI/CD Integration Overview
description: Using rninja in continuous integration and deployment pipelines
tags:
  - ci-cd
  - overview
---

# CI/CD Integration Overview

rninja is a drop-in replacement for Ninja, so it slots into any existing pipeline
that already calls `ninja`. The integration story is mostly about two things:

1. Making sure the `rninja` binary is on `PATH` (or symlinked as `ninja`).
2. Pointing the cache at something durable across CI runs.

## Why rninja in CI?

Per the [README](https://github.com/neul-labs/rninja), CI with shared cache typically
sees a **2x to 5x** speedup. Cache hits skip compilation entirely; cache misses
still benefit from rninja's modern scheduler.

| Workflow             | Typical speedup |
|----------------------|-----------------|
| Cold CI build        | 1.3x to 2x      |
| Warm CI (shared cache) | 2x to 5x      |
| No-op verify         | Sub-millisecond |

## Installation in CI

The same install paths from the README work in CI:

=== "Cargo"

    ```bash
    cargo install rninja
    ```

=== "From release binary"

    Download a prebuilt binary from the
    [Releases page](https://github.com/neul-labs/rninja/releases)
    and place it on `PATH`.

=== "Symlink as ninja"

    ```bash
    ln -s "$(which rninja)" /usr/local/bin/ninja
    ```

## Cache strategy

Two options, depending on how your CI is set up:

- **Local cache + CI cache action.** Persist `~/.cache/rninja` (or the directory
  set via `RNINJA_CACHE_DIR`) between runs using your CI provider's cache
  primitive. Simple, no extra infrastructure.
- **Remote cache server.** Run `rninja-cached` somewhere reachable by all runners
  and point `RNINJA_REMOTE_URL` at it. Best for teams and large fleets.

See [Caching: Remote Cache](../caching/remote/architecture.md) for the remote
setup, and the per-provider guides for concrete examples:

- [GitHub Actions](github-actions.md)
- [GitLab CI](gitlab-ci.md)
- [Jenkins](jenkins.md)
- [Docker](docker.md)
- [Kubernetes](kubernetes.md)

## Relevant environment variables

These are the knobs you'll typically set in CI, sourced from
[`README.md`](https://github.com/neul-labs/rninja/blob/main/README.md):

| Variable               | Purpose                                  |
|------------------------|------------------------------------------|
| `RNINJA_CACHE_DIR`     | Cache directory location                 |
| `RNINJA_CACHE_ENABLED` | Toggle caching (`0` or `1`)              |
| `RNINJA_REMOTE_URL`    | Remote cache server URL                  |
| `RNINJA_CACHE_MODE`    | `local`, `remote`, or `both`             |

For CI it usually makes sense to disable the daemon (`--no-daemon`) since
runners are short-lived and there's nothing to keep warm between jobs.

## Next steps

- [Best Practices](best-practices.md)
- [GitHub Actions guide](github-actions.md)
- [GitLab CI guide](gitlab-ci.md)
