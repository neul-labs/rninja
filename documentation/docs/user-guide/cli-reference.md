---
title: CLI Reference
description: Complete command-line interface reference
tags:
  - user-guide
  - cli
  - reference
---

# CLI Reference

Complete reference for the rninja command-line interface.

## Synopsis

```
rninja [OPTIONS] [TARGETS...]
```

## Description

rninja is a drop-in replacement for Ninja with built-in caching. It reads `build.ninja` files and executes build commands, using a content-addressed cache to skip redundant work.

## Options

### Build Control

#### `-j, --jobs <N>`

Run N jobs in parallel.

- `0` = use all available CPU cores (default)
- Any positive integer limits parallelism

```bash
rninja -j8      # 8 parallel jobs
rninja -j0      # all cores
rninja -j1      # sequential
```

#### `-k, --keep-going <N>`

Keep going until N jobs fail.

- `1` = stop on first failure (default)
- `0` = keep going indefinitely
- Any positive integer sets the failure limit

```bash
rninja -k0      # never stop
rninja -k5      # stop after 5 failures
```

#### `-l, --load-average <N>`

Do not start new jobs if the system load average is greater than N.

```bash
rninja -l 4.0   # pause if load > 4.0
```

#### `-n, --dry-run`

Dry run mode. Print commands that would be executed but don't run them.

```bash
rninja -n
```

### File Selection

#### `-f, --file <FILE>`

Specify input build file. Default is `build.ninja`.

```bash
rninja -f custom.ninja
```

#### `-C, --dir <DIR>`

Change to DIR before doing anything else.

```bash
rninja -C out/Release
```

### Output Control

#### `-v, --verbose`

Show all command lines while building.

```bash
rninja -v
```

#### `--json`

Output in JSON format for machine consumption. Useful for scripts and AI agents.

```bash
rninja --json
```

### Debugging

#### `-d, --debug <MODE>`

Enable a debugging mode. Available modes:

| Mode | Description |
|------|-------------|
| `explain` | Explain why targets are being rebuilt |
| `keepdepfile` | Don't delete depfiles after processing |
| `stats` | Show execution statistics |

```bash
rninja -d explain
rninja -d stats
```

#### `--trace <FILE>`

Write Chrome trace output to FILE. Open with `chrome://tracing`.

```bash
rninja --trace build_trace.json
```

### Subtools

#### `-t, --tool <TOOL>`

Run a subtool instead of building. See [Subtools](subtools/overview.md) for details.

```bash
rninja -t list          # list tools
rninja -t clean         # clean outputs
rninja -t compdb        # compilation database
```

### Daemon Control

#### `--no-daemon`

Disable daemon mode. Run in single-shot mode without connecting to or spawning a daemon.

```bash
rninja --no-daemon
```

#### `--daemon-socket <PATH>`

Use a custom daemon socket path.

```bash
rninja --daemon-socket /tmp/my-rninja.sock
```

### Other Options

#### `-w, --log <FILE>`

Write build log to FILE. (Experimental)

```bash
rninja -w build.log
```

#### `--version`

Print version information.

```bash
rninja --version
```

#### `--help`

Print help information.

```bash
rninja --help
```

## Arguments

### `[TARGETS...]`

Targets to build. If not specified, builds the default targets defined in the build file.

```bash
rninja                  # default targets
rninja target1 target2  # specific targets
rninja all              # 'all' target if defined
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Build failed |
| 2 | Invalid arguments |

## Environment Variables

rninja respects these environment variables:

| Variable | Description |
|----------|-------------|
| `RNINJA_CACHE_ENABLED` | Enable/disable caching (`0` or `1`) |
| `RNINJA_CACHE_DIR` | Cache directory location |
| `RNINJA_CACHE_MODE` | Cache mode (`local`, `remote`, `auto`) |
| `RNINJA_CACHE_REMOTE_SERVER` | Remote cache server URL |
| `RNINJA_CACHE_TOKEN` | Remote cache authentication token |

See [Environment Variables](configuration/environment-variables.md) for complete list.

## Configuration Files

rninja loads configuration from these locations (in order):

1. `.rninjarc` (project directory)
2. `~/.rninjarc` (home directory)
3. `~/.config/rninja/config.toml` (XDG config)

See [Config Files](configuration/config-files.md) for format details.

## Examples

### Basic Builds

```bash
# Build default targets
rninja

# Build specific target
rninja my_target

# Build multiple targets
rninja target1 target2
```

### Build Configuration

```bash
# Use 4 parallel jobs
rninja -j4

# Build in different directory
rninja -C out/Release

# Use different build file
rninja -f release.ninja
```

### Debugging

```bash
# See why targets rebuild
rninja -d explain

# Verbose output
rninja -v

# Dry run
rninja -n

# Generate trace
rninja --trace trace.json
```

### Automation

```bash
# JSON output for scripts
rninja --json

# CI build
rninja -j0 -k0 --json
```

### Subtools

```bash
# Clean build outputs
rninja -t clean

# Show dependencies
rninja -t deps target

# Generate compile_commands.json
rninja -t compdb > compile_commands.json

# Check cache
rninja -t cache-stats
```

## Ninja Compatibility

rninja supports all standard Ninja flags:

| Ninja Flag | rninja Support |
|------------|----------------|
| `-C DIR` | :material-check: Yes |
| `-f FILE` | :material-check: Yes |
| `-j N` | :material-check: Yes |
| `-k N` | :material-check: Yes |
| `-l N` | :material-check: Yes |
| `-n` | :material-check: Yes |
| `-t TOOL` | :material-check: Yes |
| `-d MODE` | :material-check: Yes |
| `-v` | :material-check: Yes |
| `-w` | :material-check: Yes (experimental) |

## Additional rninja Flags

These flags are rninja extensions:

| Flag | Description |
|------|-------------|
| `--json` | Machine-readable JSON output |
| `--trace FILE` | Chrome trace output |
| `--no-daemon` | Disable daemon mode |
| `--daemon-socket PATH` | Custom daemon socket |

## See Also

- [Basic Usage](basic-usage.md) - Common usage patterns
- [Subtools](subtools/overview.md) - All available subtools
- [Configuration](configuration/overview.md) - Configuration options
