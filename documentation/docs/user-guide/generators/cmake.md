---
title: CMake Integration
description: Using rninja with CMake projects
tags:
  - user-guide
  - generators
  - cmake
---

# CMake Integration

rninja works seamlessly with CMake-generated Ninja build files.

## Basic Usage

### Generate Build Files

```bash
# Configure with Ninja generator
cmake -G Ninja -B build

# Build with rninja
rninja -C build
```

### Specify rninja Explicitly

```bash
# Tell CMake to use rninja
cmake -G Ninja -DCMAKE_MAKE_PROGRAM=$(which rninja) -B build

# Build
cmake --build build
# or
rninja -C build
```

## Build Types

### Debug Build

```bash
cmake -G Ninja -DCMAKE_BUILD_TYPE=Debug -B build/debug
rninja -C build/debug
```

### Release Build

```bash
cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -B build/release
rninja -C build/release
```

### Multiple Build Types

```bash
# Create both configurations
cmake -G Ninja -DCMAKE_BUILD_TYPE=Debug -B build/debug
cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -B build/release

# Build both (in parallel)
rninja -C build/debug &
rninja -C build/release &
wait
```

## Common Workflows

### Standard Development

```bash
# First time setup
cmake -G Ninja -B build
rninja -C build

# After code changes
rninja -C build

# After CMakeLists.txt changes
cmake -B build  # Regenerate
rninja -C build
```

### Clean Build

```bash
# Clean outputs (preserves cache)
rninja -C build -t clean

# Full clean (remove build directory)
rm -rf build
cmake -G Ninja -B build
rninja -C build
```

### Generate compilation database

```bash
# Method 1: CMake export
cmake -G Ninja -DCMAKE_EXPORT_COMPILE_COMMANDS=ON -B build
ln -sf build/compile_commands.json .

# Method 2: rninja compdb
rninja -C build -t compdb > compile_commands.json
```

## Using rninja as Default

### CMake Preset

Create `CMakePresets.json`:

```json
{
  "version": 3,
  "configurePresets": [
    {
      "name": "ninja-rninja",
      "generator": "Ninja",
      "binaryDir": "${sourceDir}/build",
      "cacheVariables": {
        "CMAKE_MAKE_PROGRAM": "rninja"
      }
    }
  ]
}
```

Usage:

```bash
cmake --preset ninja-rninja
rninja -C build
```

### Environment Variable

```bash
# In ~/.bashrc
export CMAKE_MAKE_PROGRAM=$(which rninja)
```

Then CMake automatically uses rninja:

```bash
cmake -G Ninja -B build
cmake --build build  # Uses rninja
```

### Symlink

Replace ninja with rninja system-wide:

```bash
sudo ln -sf $(which rninja) /usr/local/bin/ninja
```

## Advanced Usage

### Parallel Builds

```bash
# Use all cores (default)
rninja -C build -j0

# Limit to 4 jobs
rninja -C build -j4
```

### Verbose Output

```bash
# Show commands
rninja -C build -v

# CMake verbose
cmake --build build --verbose
```

### Specific Targets

```bash
# Build specific target
rninja -C build my_target

# Build multiple targets
rninja -C build target1 target2

# Build and run tests
rninja -C build && ctest --test-dir build
```

### Cross-Compilation

```bash
cmake -G Ninja \
  -DCMAKE_TOOLCHAIN_FILE=arm-toolchain.cmake \
  -B build-arm

rninja -C build-arm
```

## IDE Integration

### VS Code

`.vscode/settings.json`:

```json
{
  "cmake.generator": "Ninja",
  "cmake.cmakePath": "cmake",
  "cmake.buildDirectory": "${workspaceFolder}/build",
  "cmake.configureSettings": {
    "CMAKE_MAKE_PROGRAM": "rninja",
    "CMAKE_EXPORT_COMPILE_COMMANDS": "ON"
  }
}
```

### CLion

1. Settings > Build, Execution, Deployment > CMake
2. Add CMake option: `-DCMAKE_MAKE_PROGRAM=rninja`

### Qt Creator

1. Manage Kits > CMake Configuration
2. Add: `CMAKE_MAKE_PROGRAM:FILEPATH=/path/to/rninja`

## Troubleshooting

### CMake Doesn't Find rninja

```bash
# Specify full path
cmake -G Ninja -DCMAKE_MAKE_PROGRAM=/full/path/to/rninja -B build

# Or add to PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

### Regeneration Loop

If CMake keeps regenerating:

```bash
# Touch timestamp file
touch build/CMakeCache.txt

# Or reconfigure
rm -rf build/CMakeFiles
cmake -B build
```

### Cache Not Working

Ensure rninja is actually being used:

```bash
# Check which binary
cmake --build build -- --version

# Should show:
# rninja 0.1.1
```

## Tips

### Use CMake Presets

For reproducible builds:

```json
{
  "version": 3,
  "configurePresets": [
    {
      "name": "dev",
      "generator": "Ninja",
      "binaryDir": "${sourceDir}/build",
      "cacheVariables": {
        "CMAKE_MAKE_PROGRAM": "rninja",
        "CMAKE_BUILD_TYPE": "Debug",
        "CMAKE_EXPORT_COMPILE_COMMANDS": "ON"
      }
    }
  ],
  "buildPresets": [
    {
      "name": "dev",
      "configurePreset": "dev"
    }
  ]
}
```

### Leverage Cache Statistics

After builds:

```bash
rninja -C build -t cache-stats
```

### Multi-Config Generators

For Ninja Multi-Config:

```bash
cmake -G "Ninja Multi-Config" -B build
rninja -C build -f build-Debug.ninja
rninja -C build -f build-Release.ninja
```
