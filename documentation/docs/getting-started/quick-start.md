---
title: Quick Start
description: Get rninja running in 5 minutes
tags:
  - getting-started
  - quick-start
---

# Quick Start

This guide gets you from zero to your first cached build in under 5 minutes.

## Prerequisites

- Rust toolchain (for installation)
- An existing project with a `build.ninja` file (or a build generator like CMake)

## Step 1: Install rninja

```bash
cargo install rninja
```

Verify the installation:

```bash
rninja --version
```

## Step 2: Run Your First Build

Navigate to a project directory containing a `build.ninja` file:

```bash
cd /path/to/your/project
rninja
```

!!! success "That's it!"
    rninja reads your existing `build.ninja` file with no configuration needed.

If your project uses CMake, Meson, or another generator:

=== "CMake"

    ```bash
    # Generate build files
    cmake -G Ninja -B build

    # Build with rninja
    rninja -C build
    ```

=== "Meson"

    ```bash
    # Generate build files
    meson setup build

    # Build with rninja
    rninja -C build
    ```

=== "Existing Ninja project"

    ```bash
    # Just run rninja in place of ninja
    rninja
    ```

## Step 3: Experience Cached Builds

Run the build again without changes:

```bash
rninja
```

Expected output:

```
ninja: no work to do.
```

Now clean and rebuild:

```bash
rninja -t clean
rninja
```

!!! tip "Cache Hit"
    Notice how the rebuild is much faster? rninja restored cached artifacts instead of recompiling everything.

## Step 4: Check Cache Statistics

View your cache status:

```bash
rninja -t cache-stats
```

Example output:

```
Cache Statistics:
  Enabled: true
  Mode: local
  Directory: /home/user/.cache/rninja

  Local Cache:
    Total entries: 42
    Total size: 128.5 MB
    Hit rate: 95.2%
```

## Step 5: Enable Verbose Mode (Optional)

See what rninja is doing:

```bash
# Show all commands
rninja -v

# Explain why targets are rebuilt
rninja -d explain
```

## Common Workflows

### Building Specific Targets

```bash
# Build a specific target
rninja my_target

# Build multiple targets
rninja target1 target2
```

### Parallel Builds

```bash
# Use all CPU cores (default)
rninja -j0

# Limit to 4 parallel jobs
rninja -j4
```

### Different Build Directories

```bash
# Build in a different directory
rninja -C out/Release

# Use a different build file
rninja -f custom.ninja
```

### Dry Run

See what would be built without actually building:

```bash
rninja -n
```

## What's Next?

You now have rninja working with local caching. Here are some next steps:

<div class="grid cards" markdown>

-   :material-school: [__Your First Build Tutorial__](first-build.md)

    Step-by-step tutorial with a sample project

-   :material-swap-horizontal: [__Migration Guide__](migration.md)

    Full guide for teams switching from Ninja

-   :material-cloud-upload: [__Set Up Remote Caching__](../caching/remote/quick-setup.md)

    Share cache across your team

-   :material-cog: [__Configuration__](../user-guide/configuration/overview.md)

    Customize rninja for your workflow

</div>

## Quick Reference

| Task | Command |
|------|---------|
| Build default targets | `rninja` |
| Build specific target | `rninja target_name` |
| Build in directory | `rninja -C path/to/build` |
| Parallel jobs | `rninja -j8` |
| Verbose output | `rninja -v` |
| Dry run | `rninja -n` |
| Clean build outputs | `rninja -t clean` |
| Show cache stats | `rninja -t cache-stats` |
| List all tools | `rninja -t list` |
| Show dependencies | `rninja -t deps target` |
| Generate compile_commands.json | `rninja -t compdb` |
