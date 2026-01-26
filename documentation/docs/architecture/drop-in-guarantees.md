---
title: Drop-in Guarantees
description: Ninja compatibility guarantees
tags:
  - architecture
  - compatibility
---

# Drop-in Guarantees

rninja's compatibility with Ninja.

## Full Compatibility

### Build Files

- Reads `.ninja` files
- Same syntax
- Same semantics

### CLI Flags

| Flag | Support |
|------|---------|
| `-C`, `-f`, `-j`, `-k` | Full |
| `-l`, `-n`, `-v`, `-d` | Full |
| `-t`, `-w` | Full |

### Subtools

All Ninja subtools supported:

- clean, compdb, deps
- graph, query, targets
- restat, recompact

### File Formats

| File | Compatible |
|------|------------|
| build.ninja | Yes |
| .ninja_log | Yes |
| .ninja_deps | Yes |

## Exit Codes

Same as Ninja:

- 0: Success
- 1: Build failed
- 2: Invalid arguments

## Additions (Non-Breaking)

rninja adds:

- `--json` output
- `--trace` profiling
- `--no-daemon` mode
- Cache subtools

These are extensions, not changes.

## Migration

Replace ninja with rninja:

```bash
ln -s $(which rninja) /usr/local/bin/ninja
```

No other changes needed.
