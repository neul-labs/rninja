---
title: Migrating from Ninja
description: Guide for switching from Ninja to rninja
tags:
  - getting-started
  - migration
---

# Migrating from Ninja

This guide helps teams migrate from Ninja to rninja with minimal disruption.

## Migration Overview

rninja is designed as a **drop-in replacement** for Ninja. In most cases, migration is as simple as:

```bash
# Instead of
ninja

# Use
rninja
```

No changes to your `build.ninja` files, build generators, or CI scripts are required.

## Compatibility Checklist

rninja is compatible with:

| Feature | Status |
|---------|--------|
| `.ninja` file format | :material-check: Full support |
| `.ninja_log` format | :material-check: Full support |
| `.ninja_deps` format | :material-check: Full support |
| All CLI flags | :material-check: Full support |
| All subtools | :material-check: Full support |
| CMake generator | :material-check: Tested |
| Meson generator | :material-check: Tested |
| GN generator | :material-check: Tested |

## Step-by-Step Migration

### Step 1: Install rninja

```bash
cargo install rninja
```

Verify installation:

```bash
rninja --version
```

### Step 2: Test Locally

Before changing anything, test rninja on your project:

```bash
# Clean to ensure fresh build
ninja -t clean

# Build with rninja
rninja

# Run tests to verify outputs
```

Compare the outputs to ensure everything builds correctly.

### Step 3: Create an Alias (Optional)

For gradual migration, create an alias:

```bash
# In ~/.bashrc or ~/.zshrc
alias ninja='rninja'
```

This lets you use `ninja` commands while actually using rninja.

### Step 4: Replace Ninja Binary (Full Migration)

For complete migration, replace the ninja binary:

```bash
# Create symlink
sudo ln -sf $(which rninja) /usr/local/bin/ninja

# Or rename the original
sudo mv /usr/bin/ninja /usr/bin/ninja.orig
sudo ln -s $(which rninja) /usr/bin/ninja
```

### Step 5: Update CI Configuration

Update your CI scripts to use rninja:

=== "GitHub Actions"

    ```yaml title=".github/workflows/build.yml"
    - name: Install rninja
      run: cargo install rninja

    - name: Build
      run: rninja -C build
    ```

=== "GitLab CI"

    ```yaml title=".gitlab-ci.yml"
    build:
      script:
        - cargo install rninja
        - rninja -C build
    ```

=== "Jenkins"

    ```groovy title="Jenkinsfile"
    stage('Build') {
        sh 'cargo install rninja'
        sh 'rninja -C build'
    }
    ```

See [CI/CD Integration](../ci-cd/overview.md) for detailed guides.

## Handling Differences

### Caching Behavior

rninja caches build outputs by default. This is almost always beneficial, but you can disable it if needed:

```bash
# Disable caching for a single build
RNINJA_CACHE_ENABLED=0 rninja

# Or in config file
# ~/.config/rninja/config.toml
[cache]
enabled = false
```

### Daemon Mode

rninja uses a daemon for faster subsequent builds. This is transparent but can be disabled:

```bash
# Run without daemon
rninja --no-daemon
```

### Additional Output

rninja may show additional output (cache statistics, timing). Use standard flags to control this:

```bash
# Quiet mode (same as ninja)
rninja 2>/dev/null
```

## Migration for Teams

### Gradual Rollout

1. **Start with developers**: Have a few developers test rninja locally
2. **Add to CI**: Update one CI pipeline to use rninja
3. **Monitor**: Watch for any issues or unexpected behavior
4. **Expand**: Roll out to more developers and pipelines
5. **Complete**: Replace ninja system-wide

### Communication Template

Share with your team:

> We're switching from Ninja to rninja for faster builds. rninja is a drop-in replacement with:
>
> - 23x faster no-op builds
> - Built-in caching for 2-5x faster incremental builds
> - Optional remote cache sharing
>
> Migration is simple: replace `ninja` with `rninja`. All flags and build files work identically.

### Rollback Plan

If issues occur, rolling back is simple:

```bash
# Remove symlink
sudo rm /usr/local/bin/ninja

# Reinstall original ninja
# Ubuntu/Debian
sudo apt install ninja-build

# Or just use ninja directly
ninja  # original binary
```

## Common Migration Issues

### Issue: Build Outputs Differ

rninja should produce identical outputs, but if you notice differences:

1. Check for non-deterministic build steps (timestamps, random values)
2. Ensure all inputs are properly declared in the build file
3. Report issues at [github.com/neul-labs/rninja/issues](https://github.com/neul-labs/rninja/issues)

### Issue: Cache Takes Too Much Space

The default cache can grow large. Configure limits:

```toml title="~/.config/rninja/config.toml"
[cache]
max_size = 5368709120  # 5GB
```

Or run garbage collection:

```bash
rninja -t cache-gc
```

### Issue: Daemon Won't Start

If the daemon fails to start:

```bash
# Run without daemon
rninja --no-daemon

# Check daemon logs
rninja-daemon --foreground

# Use custom socket path
rninja --daemon-socket /tmp/my-rninja.sock
```

### Issue: CI Builds Are Slower

First builds in CI may be slower because the cache is cold. Solutions:

1. **Persist cache**: Save `~/.cache/rninja` between CI runs
2. **Use remote cache**: Set up shared cache server
3. **Warm cache**: Pre-populate cache with common builds

## Verifying Migration

After migration, verify everything works:

```bash
# 1. Clean build
rninja -t clean
rninja

# 2. Incremental build
touch some_file.c
rninja

# 3. No-op build
rninja

# 4. Run tests
./run_tests.sh

# 5. Check cache
rninja -t cache-stats
```

## Getting Help

If you encounter issues:

1. Check [Troubleshooting](../troubleshooting/common-issues.md)
2. Search [GitHub Issues](https://github.com/neul-labs/rninja/issues)
3. Open a new issue with:
   - rninja version (`rninja --version`)
   - Operating system
   - Build generator (CMake, Meson, etc.)
   - Steps to reproduce

## Next Steps

<div class="grid cards" markdown>

-   :material-cog: [__Configuration__](../user-guide/configuration/overview.md)

    Customize rninja for your workflow

-   :material-cloud-upload: [__Remote Cache__](../caching/remote/quick-setup.md)

    Share cache across your team

-   :material-cloud: [__CI/CD Integration__](../ci-cd/overview.md)

    Optimize CI builds with rninja

</div>
