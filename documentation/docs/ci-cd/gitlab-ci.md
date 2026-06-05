---
title: GitLab CI
description: Running rninja inside GitLab CI/CD pipelines
tags:
  - ci-cd
  - gitlab
---

# GitLab CI

rninja drops into a GitLab CI pipeline anywhere `ninja` would be used. The two
common patterns are **per-job cache** and **shared `rninja-cached` service**.

## Per-job cache pattern

GitLab's [`cache:`](https://docs.gitlab.com/ee/ci/caching/) keyword persists
the rninja cache directory across runs:

```yaml title=".gitlab-ci.yml"
build:
  image: rust:1.75
  variables:
    RNINJA_CACHE_ENABLED: "1"
    RNINJA_CACHE_MODE: local
    RNINJA_CACHE_DIR: "$CI_PROJECT_DIR/.rninja-cache"
  cache:
    key:
      files:
        - build.ninja
        - CMakeLists.txt
    paths:
      - .rninja-cache/
  before_script:
    - cargo install rninja
  script:
    - cmake -G Ninja -B build
    - rninja -C build --no-daemon
```

Pointing `RNINJA_CACHE_DIR` inside `$CI_PROJECT_DIR` is what lets GitLab's
cache keyword see it - the default `~/.cache/rninja` lives outside the
project tree.

`--no-daemon` is the right default in CI: runners are short-lived, so
there's no warm daemon to amortize startup against.

## Shared cache service

Adapted from the root deployment guide
([`docs/remote-cache-deployment.md`](https://github.com/neul-labs/rninja/blob/main/docs/remote-cache-deployment.md)):

```yaml title=".gitlab-ci.yml"
build:
  image: rust:1.75
  services:
    - name: rninja/cached:latest
      alias: cache
  variables:
    RNINJA_CACHE_HOST: cache
    RNINJA_CACHE_PORT: "9999"
    RNINJA_REMOTE_URL: tcp://cache:9999
    RNINJA_CACHE_MODE: remote
  before_script:
    - cargo install rninja
  script:
    - cmake -G Ninja -B build
    - rninja -C build --no-daemon
```

For a long-lived shared cache (recommended for teams), deploy `rninja-cached`
on your own infrastructure and point `RNINJA_REMOTE_URL` at the public address
instead of an inline service.

## Symlinking as `ninja`

If the rest of your pipeline already calls `ninja`, alias it so you don't
have to touch every step:

```yaml
before_script:
  - cargo install rninja
  - ln -sf "$(which rninja)" /usr/local/bin/ninja
```

## Parallel jobs

GitLab's [`parallel:`](https://docs.gitlab.com/ee/ci/yaml/#parallel) maps
nicely onto rninja's per-job cache - each instance gets the same cache key,
so they share artifacts.

```yaml
build:
  parallel: 4
  cache:
    key: rninja-shared
    paths:
      - .rninja-cache/
  script:
    - rninja -C build --no-daemon
```

## Stats and verbosity

Use `-d stats` (a built-in debug mode, see
[`src/cli.rs`](https://github.com/neul-labs/rninja/blob/main/src/cli.rs))
to surface cache hit/miss counts in the job log, or `-v` for full command
lines.

```yaml
script:
  - rninja -C build --no-daemon -d stats
```

## See also

- [Best Practices](best-practices.md)
- [GitHub Actions](github-actions.md) for the same setup on GitHub.
- [Caching > Cache Modes](../caching/cache-modes.md) for `local` / `remote` / `both`.
