---
title: GitHub Actions
description: Running rninja inside GitHub Actions workflows
tags:
  - ci-cd
  - github-actions
---

# GitHub Actions

rninja drops into a GitHub Actions workflow anywhere `ninja` would be used.
This page shows the two common patterns: **local cache via `actions/cache`**
and **remote cache via `rninja-cached`**.

## Local cache pattern

The simplest setup: install `rninja`, hand `actions/cache` the rninja cache
directory, run the build.

```yaml title=".github/workflows/build.yml"
name: build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install rninja
        run: cargo install rninja

      - name: Restore rninja cache
        uses: actions/cache@v4
        with:
          path: ~/.cache/rninja
          key: rninja-${{ runner.os }}-${{ hashFiles('**/build.ninja', '**/CMakeLists.txt') }}
          restore-keys: |
            rninja-${{ runner.os }}-

      - name: Configure
        run: cmake -G Ninja -B build

      - name: Build
        env:
          RNINJA_CACHE_ENABLED: "1"
          RNINJA_CACHE_MODE: local
        run: rninja -C build --no-daemon
    
```

`--no-daemon` is the right default in CI: runners are short-lived, so there's
no warm daemon to amortize startup against.

## Remote cache pattern

If you have a `rninja-cached` server reachable from the runners, point
`RNINJA_REMOTE_URL` at it. This is taken from the deployment guide in the
[root docs](https://github.com/neul-labs/rninja/blob/main/docs/remote-cache-deployment.md):

```yaml title=".github/workflows/build.yml"
jobs:
  build:
    runs-on: ubuntu-latest
    services:
      cache:
        image: rninja/cached:latest
        ports:
          - 9999:9999
    env:
      RNINJA_REMOTE_URL: tcp://cache:9999
      RNINJA_CACHE_MODE: remote
    steps:
      - uses: actions/checkout@v4
      - run: cargo install rninja
      - run: cmake -G Ninja -B build
      - run: rninja -C build --no-daemon
```

For a long-lived shared cache (recommended for teams), run `rninja-cached`
on your own infrastructure and use its public URL instead of an inline
service container.

## Symlinking as `ninja`

If the rest of your workflow already calls `ninja`, alias it so you don't
need to touch every step:

```yaml
- name: Use rninja as ninja
  run: ln -sf "$(which rninja)" /usr/local/bin/ninja
```

## Matrix builds

Each matrix leg gets its own cache key. Include the matrix dimensions in the
key so different toolchains don't trample each other:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    compiler: [gcc, clang]
steps:
  - uses: actions/cache@v4
    with:
      path: ~/.cache/rninja
      key: rninja-${{ matrix.os }}-${{ matrix.compiler }}-${{ hashFiles('**/build.ninja') }}
```

## Reporting cache effectiveness

Run with `-d stats` (a built-in debug mode, see
[`src/cli.rs`](https://github.com/neul-labs/rninja/blob/main/src/cli.rs))
to surface cache hit/miss counts in the job log:

```yaml
- run: rninja -C build --no-daemon -d stats
```

## See also

- [Best Practices](best-practices.md) for cache-key strategy and pitfalls.
- [GitLab CI](gitlab-ci.md) for the equivalent setup on GitLab.
- [Caching > Cache Modes](../caching/cache-modes.md) for `local` / `remote` / `both`.
