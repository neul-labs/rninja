---
title: CI/CD Best Practices
description: Optimizing rninja for CI/CD pipelines
tags:
  - ci-cd
  - best-practices
---

# CI/CD Best Practices

Optimize your CI/CD pipelines with rninja.

## Cache Strategy

### Use Remote Cache

For maximum benefit across CI runs:

```bash
export RNINJA_CACHE_MODE=auto
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache:9999
```

### Push Policy by Branch

```bash
if [ "$BRANCH" = "main" ]; then
    export RNINJA_CACHE_PUSH_POLICY=always
else
    export RNINJA_CACHE_PUSH_POLICY=never
fi
```

### Cache Key Design

Include relevant factors:

```yaml
key: build-${{ hashFiles('**/CMakeLists.txt', '**/Cargo.lock') }}
```

## Performance Tips

### Maximize Parallelism

```bash
rninja -j0  # Use all cores
```

### Keep Going on Errors

Find all errors in one run:

```bash
rninja -k0  # Don't stop on failures
```

### Use Single-Shot in Ephemeral Runners

```bash
rninja --no-daemon  # No daemon overhead
```

## Security

### Protect Tokens

Use CI secret management:

```yaml
env:
  RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}
```

### Limit Push Access

Only main branch populates cache:

```yaml
- if: github.ref == 'refs/heads/main'
  run: export RNINJA_CACHE_PUSH_POLICY=always
```

## Monitoring

### Log Cache Stats

```yaml
- run: rninja
- run: rninja -t cache-stats
```

### Generate Build Traces

```yaml
- run: rninja --trace trace.json
- uses: actions/upload-artifact@v3
  with:
    path: trace.json
```

## Common Patterns

### Build Matrix

Test multiple configurations:

```yaml
strategy:
  matrix:
    build_type: [Debug, Release]
    os: [ubuntu-latest, macos-latest]
```

### Artifact Caching

Combine CI cache with rninja cache:

```yaml
- uses: actions/cache@v3
  with:
    path: ~/.cache/rninja
    key: rninja-${{ runner.os }}
```

### Conditional Builds

Skip builds when no source changes:

```yaml
- uses: dorny/paths-filter@v2
  id: changes
  with:
    filters: |
      src:
        - 'src/**'
- if: steps.changes.outputs.src == 'true'
  run: rninja
```

## Checklist

- [ ] Install rninja in CI image
- [ ] Configure cache (local or remote)
- [ ] Set appropriate parallelism
- [ ] Add cache statistics logging
- [ ] Secure tokens with secrets
- [ ] Test cache hit rates
- [ ] Monitor build times
