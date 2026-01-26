---
title: Build Tools
description: Subtools for managing build outputs
tags:
  - user-guide
  - subtools
---

# Build Tools

Tools for managing build outputs and the build state.

## clean

Remove built files.

### Usage

```bash
# Clean all outputs
rninja -t clean

# Clean specific targets
rninja -t clean target1 target2
```

### Description

Removes all files that are outputs of build rules. This is equivalent to deleting object files, executables, and other generated files.

!!! note "Cache Preserved"
    `clean` removes output files but preserves the cache. Rebuilding will restore cached artifacts.

### Examples

```bash
# Clean everything
rninja -t clean

# Clean specific target and its dependencies
rninja -t clean my_binary

# Clean then rebuild
rninja -t clean && rninja
```

## cleandead

Remove outputs no longer in the build manifest.

### Usage

```bash
rninja -t cleandead
```

### Description

Removes files that were previously built but are no longer outputs of any build rule. This happens when:

- A source file was removed
- A build rule was deleted
- Output paths changed

### Examples

```bash
# Remove stale outputs
rninja -t cleandead

# Useful after refactoring
git checkout new-branch
rninja -t cleandead
rninja
```

### When to Use

- After switching branches
- After renaming files
- After removing build targets
- Before committing to ensure clean state

## restat

Update file timestamps in the build log.

### Usage

```bash
# Restat all outputs
rninja -t restat

# Restat specific files
rninja -t restat file1.o file2.o
```

### Description

Re-reads the modification times of all output files and updates the build log. This is useful when files were modified outside of the build system.

### Examples

```bash
# After manually modifying outputs
touch output.o
rninja -t restat

# Force rebuild check
rninja -t restat
rninja
```

### When to Use

- After modifying outputs manually
- After restoring files from backup
- When build log seems out of sync

## recompact

Optimize ninja-internal data structures.

### Usage

```bash
rninja -t recompact
```

### Description

Recompacts the `.ninja_log` and `.ninja_deps` files, removing obsolete entries and optimizing for faster reads.

### Examples

```bash
# Optimize build log
rninja -t recompact

# Check size before and after
ls -la .ninja_log
rninja -t recompact
ls -la .ninja_log
```

### When to Use

- After large refactorings
- If build log grows very large
- Periodically for maintenance

## Workflow Examples

### Full Clean Build

```bash
# Remove all outputs
rninja -t clean

# Rebuild (uses cache if available)
rninja
```

### Clean Build Without Cache

```bash
# Disable caching for true clean build
RNINJA_CACHE_ENABLED=0 rninja -t clean
RNINJA_CACHE_ENABLED=0 rninja
```

### Post-Refactoring Cleanup

```bash
# Remove stale outputs
rninja -t cleandead

# Optimize data structures
rninja -t recompact

# Build
rninja
```

### Branch Switching

```bash
git checkout feature-branch

# Clean stale files from previous branch
rninja -t cleandead

# Build new branch
rninja
```

### Maintenance Routine

```bash
# Weekly maintenance
rninja -t cleandead
rninja -t recompact
rninja -t cache-gc
```

## Comparison with Ninja

| Tool | Ninja | rninja |
|------|-------|--------|
| `clean` | :material-check: | :material-check: |
| `cleandead` | :material-check: | :material-check: |
| `restat` | :material-check: | :material-check: |
| `recompact` | :material-check: | :material-check: |

rninja's implementations are fully compatible with Ninja's behavior.

## Tips

### Don't Clean Unnecessarily

With caching, you rarely need to clean:

```bash
# Usually sufficient - cache handles changes
rninja

# Only clean for specific issues
rninja -t clean
```

### Use cleandead Regularly

After branch switches or refactoring:

```bash
rninja -t cleandead
```

### Automate Maintenance

Add to your CI or git hooks:

```bash
# In post-checkout hook
rninja -t cleandead 2>/dev/null || true
```
