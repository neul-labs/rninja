---
title: Subtools Overview
description: Overview of all rninja subtools
tags:
  - user-guide
  - subtools
---

# Subtools Overview

rninja provides subtools for build management, inspection, and cache operations. Access them with the `-t` flag.

## Listing Available Tools

```bash
rninja -t list
```

Output:

```
rninja subtools:
    clean       remove built files
    cleandead   clean built files no longer produced by manifest
    commands    list all commands required to rebuild given targets
    compdb      dump JSON compilation database to stdout
    config      show config file locations and generate sample config
    deps        show dependencies stored in the deps log
    graph       output graphviz dot file for targets
    inputs      list all inputs required to rebuild given targets
    path        find dependency path between two targets
    query       show inputs/outputs for a path
    recompact   recompact ninja-internal data structures
    restat      restat all outputs in the build log
    rules       list all rules
    targets     list targets by their rule or depth in the DAG
    cache-stats show cache statistics
    cache-gc    run cache garbage collection
    cache-health check cache integrity
```

## Tool Categories

### Build Management Tools

Tools for managing build outputs and state:

| Tool | Description |
|------|-------------|
| `clean` | Remove built files |
| `cleandead` | Remove stale outputs no longer in manifest |
| `restat` | Update file timestamps in build log |
| `recompact` | Optimize internal data structures |

[Build Tools Documentation](build-tools.md)

### Query and Inspection Tools

Tools for understanding your build:

| Tool | Description |
|------|-------------|
| `deps` | Show dependencies for a target |
| `query` | Show inputs/outputs for a path |
| `graph` | Generate dependency graph (Graphviz) |
| `path` | Find dependency path between targets |
| `targets` | List all targets |
| `rules` | List all rules |
| `commands` | List commands for targets |
| `inputs` | List all inputs for targets |
| `compdb` | Generate compilation database |

[Query Tools Documentation](query-tools.md)

### Cache Tools

Tools for managing the build cache:

| Tool | Description |
|------|-------------|
| `cache-stats` | Show cache statistics |
| `cache-gc` | Run garbage collection |
| `cache-health` | Check cache integrity |
| `config` | Show/generate configuration |

[Cache Tools Documentation](cache-tools.md)

## Quick Reference

### Common Operations

```bash
# Clean all outputs
rninja -t clean

# Show what would be rebuilt
rninja -n

# Show dependencies for a target
rninja -t deps my_target

# Generate compile_commands.json
rninja -t compdb > compile_commands.json

# Check cache status
rninja -t cache-stats
```

### Debugging Builds

```bash
# Why is this rebuilding?
rninja -t deps target_name
rninja -t query target_name

# Find dependency chain
rninja -t path source.c final_binary

# Visualize dependencies
rninja -t graph | dot -Tpng > deps.png
```

### IDE Integration

```bash
# Generate compilation database for clangd, ccls, etc.
rninja -t compdb > compile_commands.json
```

### Cache Management

```bash
# Check cache health
rninja -t cache-stats
rninja -t cache-health

# Clean up cache
rninja -t cache-gc
```

## Usage Patterns

### Before Commits

```bash
# Clean stale outputs
rninja -t cleandead

# Full clean build
rninja -t clean
rninja
```

### Debugging Build Issues

```bash
# Check target dependencies
rninja -t deps problematic_target

# Check what depends on a file
rninja -t query changed_file

# Visualize the build graph
rninja -t graph target | dot -Tsvg > graph.svg
```

### CI/CD

```bash
# Generate compilation database
rninja -t compdb > compile_commands.json

# Show all commands (for debugging)
rninja -t commands all
```

### Maintenance

```bash
# Optimize build log
rninja -t recompact

# Update timestamps
rninja -t restat

# Cache cleanup
rninja -t cache-gc
```

## Ninja Compatibility

All standard Ninja subtools are supported:

| Ninja Tool | rninja Support |
|------------|----------------|
| `clean` | :material-check: |
| `cleandead` | :material-check: |
| `compdb` | :material-check: |
| `deps` | :material-check: |
| `graph` | :material-check: |
| `query` | :material-check: |
| `targets` | :material-check: |
| `commands` | :material-check: |
| `inputs` | :material-check: |
| `rules` | :material-check: |
| `path` | :material-check: |
| `recompact` | :material-check: |
| `restat` | :material-check: |

### rninja-Specific Tools

These tools are unique to rninja:

| Tool | Description |
|------|-------------|
| `cache-stats` | Cache statistics |
| `cache-gc` | Cache garbage collection |
| `cache-health` | Cache integrity check |
| `config` | Configuration management |

## Next Steps

<div class="grid cards" markdown>

-   :material-hammer-wrench: [__Build Tools__](build-tools.md)

    Clean, restat, and other build management

-   :material-magnify: [__Query Tools__](query-tools.md)

    Inspect dependencies and targets

-   :material-cached: [__Cache Tools__](cache-tools.md)

    Manage the build cache

</div>
