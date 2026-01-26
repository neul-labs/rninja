---
title: GN Integration
description: Using rninja with GN (Generate Ninja) projects
tags:
  - user-guide
  - generators
  - gn
---

# GN Integration

rninja works with GN (Generate Ninja), the meta-build system used by Chromium, Fuchsia, and other large projects.

## Basic Usage

### Generate and Build

```bash
# Generate ninja files
gn gen out/Default

# Build with rninja
rninja -C out/Default
```

### Specify rninja in GN Args

```bash
gn gen out/Default --args='ninja_path="rninja"'
rninja -C out/Default
```

## Build Configurations

### Debug Build

```bash
gn gen out/Debug --args='is_debug=true'
rninja -C out/Debug
```

### Release Build

```bash
gn gen out/Release --args='is_debug=false is_official_build=true'
rninja -C out/Release
```

### Component Build

```bash
gn gen out/Component --args='is_component_build=true'
rninja -C out/Component
```

## Common Workflows

### Initial Setup

```bash
# Fetch dependencies (Chromium example)
gclient sync

# Generate build files
gn gen out/Default

# Build
rninja -C out/Default
```

### Iterative Development

```bash
# After code changes
rninja -C out/Default

# After BUILD.gn changes
gn gen out/Default
rninja -C out/Default
```

### Clean Build

```bash
# Clean outputs
rninja -C out/Default -t clean

# Full clean
rm -rf out/Default
gn gen out/Default
rninja -C out/Default
```

## Using rninja as Default

### Symlink Method

```bash
# Replace ninja with rninja
sudo ln -sf $(which rninja) /usr/local/bin/ninja

# GN will now use rninja automatically
gn gen out/Default
ninja -C out/Default  # Actually uses rninja
```

### GN Args Method

In your `args.gn`:

```
ninja_path = "/path/to/rninja"
```

Or on command line:

```bash
gn gen out/Default --args='ninja_path="/home/user/.cargo/bin/rninja"'
```

## Large Project Considerations

### Chromium

```bash
# Generate for component build (faster)
gn gen out/Default --args='is_component_build=true symbol_level=1'

# Build Chrome
rninja -C out/Default chrome

# Build specific component
rninja -C out/Default content_shell
```

### Fuchsia

```bash
# Set up build
fx set core.x64

# Build with rninja
rninja -C out/default
# Or
fx build
```

### V8

```bash
# Generate
tools/dev/v8gen.py x64.release

# Build
rninja -C out/x64.release d8
```

## Advanced Usage

### Parallel Builds

```bash
# Use all cores
rninja -C out/Default -j0

# Limit for memory-constrained machines
rninja -C out/Default -j4
```

### Building Specific Targets

```bash
# Build one target
rninja -C out/Default my_target

# Build multiple targets
rninja -C out/Default target1 target2

# Build all tests
rninja -C out/Default all_tests
```

### Verbose Output

```bash
# Show all commands
rninja -C out/Default -v

# Explain rebuilds
rninja -C out/Default -d explain
```

### Incremental Builds

rninja's caching is especially beneficial for large GN projects:

```bash
# First build caches artifacts
rninja -C out/Default

# Switch branches
git checkout feature-branch

# Rebuild uses cache
rninja -C out/Default  # Much faster with cache hits
```

## GN Introspection

### List All Targets

```bash
gn ls out/Default
```

### Describe Target

```bash
gn desc out/Default //path/to:target
```

### Show Dependencies

```bash
gn desc out/Default //path/to:target deps --tree
```

### Check Build Files

```bash
gn check out/Default
```

## IDE Integration

### VS Code (Chromium)

Use the official VS Code configuration:

```bash
# Generate VS Code files
gn gen out/Default --ide=vs2022
```

### CLion

GN can generate CLion project files:

```bash
gn gen out/Default --ide=clion
```

## Compilation Database

Generate for IDE support:

```bash
# GN method
gn gen out/Default --export-compile-commands

# rninja method
rninja -C out/Default -t compdb cc cxx > compile_commands.json
```

## Troubleshooting

### GN Uses Original Ninja

Check which ninja GN is using:

```bash
gn args out/Default --list=ninja_path
```

Set explicitly:

```bash
gn gen out/Default --args='ninja_path="rninja"'
```

### Build Errors After GN Changes

```bash
# Regenerate
gn gen out/Default

# If still failing, clean and regenerate
rm -rf out/Default
gn gen out/Default
rninja -C out/Default
```

### Memory Issues on Large Projects

For projects like Chromium:

```bash
# Limit parallelism
rninja -C out/Default -j4

# Use component build
gn gen out/Default --args='is_component_build=true'
```

## Performance Tips

### Component Builds

For faster iteration:

```gn
# args.gn
is_component_build = true
symbol_level = 1
```

### Remote Caching

For team builds:

```bash
export RNINJA_CACHE_MODE=auto
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=team-token

rninja -C out/Default
```

### Monitor Build Performance

```bash
# Generate trace
rninja -C out/Default --trace build_trace.json

# View in chrome://tracing
```

### Cache Statistics

```bash
rninja -C out/Default -t cache-stats
```

## Tips

### Use Output Directories

Keep multiple configurations:

```bash
out/
  Debug/
  Release/
  Component/
  ASAN/
```

### GN Args File

Store common args in `args.gn`:

```gn
# out/Default/args.gn
is_debug = true
is_component_build = true
symbol_level = 1
enable_nacl = false
```

### Build Aliases

```bash
# In ~/.bashrc
alias cbuild='rninja -C out/Default'
alias cbuild-r='rninja -C out/Release'
```
