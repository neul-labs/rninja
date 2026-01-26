---
title: Exit Codes
description: rninja exit code reference
tags:
  - reference
---

# Exit Codes

rninja exit codes and their meanings.

## Standard Exit Codes

rninja uses the same exit codes as Ninja for compatibility:

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Success | Build completed successfully |
| `1` | Build failure | One or more targets failed to build |
| `2` | Invalid arguments | Command-line argument error |

## Extended Exit Codes

rninja adds additional codes for specific scenarios:

| Code | Meaning | Description |
|------|---------|-------------|
| `3` | Configuration error | Invalid configuration file |
| `4` | Cache error | Cache operation failed |
| `5` | Daemon error | Daemon communication failed |
| `6` | Internal error | Unexpected internal error |

## Build Failure Details

Exit code `1` indicates build failure. Check output for details:

```bash
rninja
echo $?  # 1 = build failed

# Get more details
rninja -v  # Verbose shows failed commands
rninja -d explain  # Explains rebuild reasons
```

## Usage in Scripts

### Basic Error Handling

```bash
#!/bin/bash
set -e  # Exit on error

rninja || {
    echo "Build failed!"
    exit 1
}

echo "Build succeeded"
```

### Detailed Error Handling

```bash
#!/bin/bash

rninja
status=$?

case $status in
    0)
        echo "Build succeeded"
        ;;
    1)
        echo "Build failed - check compiler errors"
        exit 1
        ;;
    2)
        echo "Invalid arguments - check command"
        exit 2
        ;;
    3)
        echo "Configuration error - check config files"
        exit 1
        ;;
    4)
        echo "Cache error - try --no-cache"
        # Retry without cache
        rninja --no-cache
        ;;
    5)
        echo "Daemon error - try --no-daemon"
        rninja --no-daemon
        ;;
    *)
        echo "Unknown error: $status"
        exit 1
        ;;
esac
```

### CI/CD Usage

```yaml
# GitHub Actions
- name: Build
  run: |
    rninja || exit 1
  continue-on-error: false
```

## Subtool Exit Codes

Subtools use the same convention:

| Command | Success | Failure |
|---------|---------|---------|
| `rninja -t clean` | 0 | 1 |
| `rninja -t compdb` | 0 | 1 |
| `rninja -t query TARGET` | 0 | 1 (not found) |
| `rninja -t cache-stats` | 0 | 4 (cache error) |

## Dry Run

Dry run (`-n`) returns success if the build *would* succeed:

```bash
# Check if build is up-to-date
rninja -n
if [ $? -eq 0 ]; then
    echo "Build is current"
fi
```

## JSON Output

With `--json`, exit codes are included in output:

```bash
rninja --json 2>/dev/null; echo "Exit: $?"
```

```json
{
  "success": false,
  "exit_code": 1,
  "failed_targets": ["src/foo.o"],
  "error": "compilation failed"
}
```

## Ninja Compatibility

rninja maintains Ninja exit code compatibility:

| Ninja | rninja | Meaning |
|-------|--------|---------|
| 0 | 0 | Success |
| 1 | 1 | Build failure |
| 2 | 2 | Invalid args |

Scripts written for Ninja work unchanged with rninja.

## Common Scenarios

### Build Succeeded

```
$ rninja
[100/100] Linking program
$ echo $?
0
```

### Compilation Error

```
$ rninja
[50/100] Compiling foo.c
FAILED: foo.o
error: undefined reference to 'bar'
$ echo $?
1
```

### Invalid Flag

```
$ rninja --invalid-flag
error: unknown option '--invalid-flag'
$ echo $?
2
```

### Config Error

```
$ rninja --config invalid.toml
error: failed to parse config: invalid TOML
$ echo $?
3
```

## Signaling

If rninja receives a signal:

| Signal | Exit Code |
|--------|-----------|
| SIGINT (Ctrl+C) | 130 |
| SIGTERM | 143 |
| SIGKILL | 137 |

```bash
# Ctrl+C during build
$ rninja
[50/100] Compiling...
^C
$ echo $?
130
```
