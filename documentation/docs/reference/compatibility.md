---
title: Compatibility Matrix
description: rninja compatibility reference
tags:
  - reference
  - compatibility
---

# Compatibility Matrix

rninja compatibility with Ninja and build systems.

## Ninja Compatibility

### Command-Line Options

| Option | Ninja | rninja | Notes |
|--------|-------|--------|-------|
| `-C DIR` | ✓ | ✓ | Change directory |
| `-f FILE` | ✓ | ✓ | Build file |
| `-j N` | ✓ | ✓ | Parallel jobs |
| `-k N` | ✓ | ✓ | Keep going |
| `-l N` | ✓ | ✓ | Load limit |
| `-n` | ✓ | ✓ | Dry run |
| `-v` | ✓ | ✓ | Verbose |
| `-d MODE` | ✓ | ✓ | Debug mode |
| `-t TOOL` | ✓ | ✓ | Subtools |
| `-w FLAG` | ✓ | ✓ | Warnings |

### Subtools

| Tool | Ninja | rninja | Notes |
|------|-------|--------|-------|
| `clean` | ✓ | ✓ | Remove outputs |
| `cleandead` | ✓ | ✓ | Remove stale outputs |
| `compdb` | ✓ | ✓ | Compilation database |
| `deps` | ✓ | ✓ | Show dependencies |
| `graph` | ✓ | ✓ | GraphViz output |
| `query` | ✓ | ✓ | Target info |
| `targets` | ✓ | ✓ | List targets |
| `commands` | ✓ | ✓ | Show commands |
| `recompact` | ✓ | ✓ | Compact log |
| `restat` | ✓ | ✓ | Update timestamps |
| `rules` | ✓ | ✓ | List rules |
| `cache-*` | ✗ | ✓ | rninja extension |
| `daemon-*` | ✗ | ✓ | rninja extension |

### File Formats

| File | Ninja | rninja | Notes |
|------|-------|--------|-------|
| `build.ninja` | ✓ | ✓ | Identical |
| `.ninja_log` | ✓ | ✓ | Compatible |
| `.ninja_deps` | ✓ | ✓ | Compatible |
| `compile_commands.json` | ✓ | ✓ | Standard format |

### Exit Codes

| Code | Ninja | rninja | Meaning |
|------|-------|--------|---------|
| 0 | ✓ | ✓ | Success |
| 1 | ✓ | ✓ | Build failed |
| 2 | ✓ | ✓ | Invalid arguments |

### Debug Modes

| Mode | Ninja | rninja |
|------|-------|--------|
| `stats` | ✓ | ✓ |
| `explain` | ✓ | ✓ |
| `keepdepfile` | ✓ | ✓ |
| `keeprsp` | ✓ | ✓ |

## Build System Compatibility

### CMake

| Feature | Support | Notes |
|---------|---------|-------|
| Ninja generator | ✓ | `cmake -G Ninja` |
| Multi-config | ✓ | Via ninja-multi |
| compile_commands.json | ✓ | Standard format |
| Custom commands | ✓ | Full support |

```bash
cmake -G Ninja -B build
rninja -C build
```

### Meson

| Feature | Support | Notes |
|---------|---------|-------|
| Ninja backend | ✓ | Default backend |
| Subprojects | ✓ | Full support |
| compile_commands.json | ✓ | Auto-generated |
| Cross-compilation | ✓ | Full support |

```bash
meson setup build
rninja -C build
```

### GN (Generate Ninja)

| Feature | Support | Notes |
|---------|---------|-------|
| Build files | ✓ | Full support |
| Toolchains | ✓ | All supported |
| Actions | ✓ | Full support |

```bash
gn gen out/Default
rninja -C out/Default
```

### Bazel

| Feature | Support | Notes |
|---------|---------|-------|
| Direct | ✗ | Uses own executor |
| Via rules_foreign_cc | ✓ | CMake/Ninja builds |

### Buck2

| Feature | Support | Notes |
|---------|---------|-------|
| Direct | ✗ | Uses own executor |

## Platform Compatibility

### Operating Systems

| Platform | Support | Notes |
|----------|---------|-------|
| Linux (x86_64) | ✓ | Full support |
| Linux (aarch64) | ✓ | Full support |
| macOS (x86_64) | ✓ | Full support |
| macOS (Apple Silicon) | ✓ | Full support |
| Windows (x86_64) | ✓ | Full support |
| FreeBSD | ✓ | Full support |

### CI/CD Platforms

| Platform | Support | Notes |
|----------|---------|-------|
| GitHub Actions | ✓ | Full support, examples provided |
| GitLab CI | ✓ | Full support, examples provided |
| Jenkins | ✓ | Full support |
| CircleCI | ✓ | Full support |
| Azure Pipelines | ✓ | Full support |
| Buildkite | ✓ | Full support |

### Container Platforms

| Platform | Support | Notes |
|----------|---------|-------|
| Docker | ✓ | Official images available |
| Podman | ✓ | Compatible with Docker images |
| Kubernetes | ✓ | Deployment guides provided |

## Compiler Compatibility

### Depfile Formats

| Compiler | Format | Support |
|----------|--------|---------|
| GCC | GCC (`deps = gcc`) | ✓ |
| Clang | GCC (`deps = gcc`) | ✓ |
| MSVC | MSVC (`deps = msvc`) | ✓ |
| Intel ICC | GCC | ✓ |
| Intel ICX | GCC | ✓ |

### Response Files

| Compiler | Support |
|----------|---------|
| GCC | ✓ `@file` |
| Clang | ✓ `@file` |
| MSVC | ✓ `@file` |

## Known Limitations

### Ninja Features

| Feature | Status | Notes |
|---------|--------|-------|
| Pools | ✓ | Full support |
| Console pool | ✓ | Full support |
| Phony targets | ✓ | Full support |
| Default targets | ✓ | Full support |
| Depfile parsing | ✓ | All formats |
| Dynamic dependencies | ✓ | `dyndep` supported |

### Edge Cases

| Scenario | Ninja | rninja | Notes |
|----------|-------|--------|-------|
| Very long command lines | ✓ | ✓ | Uses response files |
| Unicode paths | ✓ | ✓ | UTF-8 support |
| Symlinks | ✓ | ✓ | Followed correctly |
| Network paths | ✓ | ✓ | Supported |

## Migration Path

### Drop-in Replacement

```bash
# Create symlink
sudo ln -sf $(which rninja) /usr/local/bin/ninja

# Or alias
alias ninja=rninja
```

### Verification

```bash
# Compare behavior
ninja -n -v > ninja.log 2>&1
rninja -n -v > rninja.log 2>&1
diff ninja.log rninja.log
```

### Rollback

```bash
# Remove symlink
sudo rm /usr/local/bin/ninja
sudo ln -sf $(which ninja.real) /usr/local/bin/ninja
```

## Version Compatibility

| rninja | Ninja | Status |
|--------|-------|--------|
| 0.1.x | 1.11+ | Full |
| 0.1.x | 1.10.x | Full |
| 0.1.x | 1.9.x | Full |
| 0.1.x | 1.8.x | Compatible |
