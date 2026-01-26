---
title: Meson Integration
description: Using rninja with Meson projects
tags:
  - user-guide
  - generators
  - meson
---

# Meson Integration

rninja works seamlessly with Meson, which uses Ninja as its default backend.

## Basic Usage

### Setup and Build

```bash
# Configure project
meson setup build

# Build with rninja
rninja -C build
```

### Specify rninja Explicitly

```bash
# Tell Meson to use rninja
meson setup build --backend=ninja
# Meson looks for 'ninja' in PATH - use symlink or alias
```

Or build directly:

```bash
meson setup build
rninja -C build  # Use rninja instead of meson compile
```

## Build Types

### Debug Build (Default)

```bash
meson setup build
rninja -C build
```

### Release Build

```bash
meson setup build --buildtype=release
rninja -C build
```

### Other Build Types

```bash
# Plain (no optimization, no debug info)
meson setup build --buildtype=plain

# Debug optimized
meson setup build --buildtype=debugoptimized

# Minimum size
meson setup build --buildtype=minsize
```

## Common Workflows

### Standard Development

```bash
# First time setup
meson setup build
rninja -C build

# After code changes
rninja -C build

# After meson.build changes
meson setup build --reconfigure  # or just re-run setup
rninja -C build
```

### Clean Build

```bash
# Clean outputs
rninja -C build -t clean

# Full reconfigure
rm -rf build
meson setup build
rninja -C build
```

### Running Tests

```bash
# Build and test
rninja -C build
meson test -C build

# Or use ninja target
rninja -C build test
```

### Installing

```bash
rninja -C build
meson install -C build
```

## Using rninja as Default

### Symlink Method

```bash
# Replace ninja with rninja
sudo ln -sf $(which rninja) /usr/local/bin/ninja

# Now Meson automatically uses rninja
meson setup build
meson compile -C build  # Uses rninja
```

### Alias Method

```bash
# In ~/.bashrc
alias ninja='rninja'

# Meson compile will use rninja
meson compile -C build
```

### Direct Invocation

Skip `meson compile` and call rninja directly:

```bash
meson setup build
rninja -C build  # More control over options
```

## Advanced Usage

### Parallel Builds

```bash
# All cores
rninja -C build -j0

# Limited parallelism
rninja -C build -j4
```

### Specific Targets

```bash
# Build specific target
rninja -C build my_library

# Build executable only
rninja -C build my_program
```

### Verbose Output

```bash
# Show commands
rninja -C build -v

# Meson verbose
meson compile -C build -v
```

### Cross-Compilation

```bash
# With cross file
meson setup build --cross-file=arm.txt
rninja -C build
```

## Subprojects

Meson subprojects work normally:

```bash
meson setup build
rninja -C build  # Builds main project and subprojects
```

Build specific subproject target:

```bash
rninja -C build subproject_name:target_name
```

## IDE Integration

### VS Code

`.vscode/settings.json`:

```json
{
  "mesonbuild.buildFolder": "build",
  "mesonbuild.configureOnOpen": true
}
```

Build task (`.vscode/tasks.json`):

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build with rninja",
      "type": "shell",
      "command": "rninja -C build",
      "group": {
        "kind": "build",
        "isDefault": true
      }
    }
  ]
}
```

### GNOME Builder

GNOME Builder auto-detects Meson projects and uses the configured ninja. Create symlink to use rninja:

```bash
sudo ln -sf $(which rninja) /usr/local/bin/ninja
```

## Compilation Database

Meson generates `compile_commands.json` automatically:

```bash
meson setup build
# compile_commands.json is in build/

# Symlink to project root
ln -sf build/compile_commands.json .
```

Or use rninja:

```bash
rninja -C build -t compdb > compile_commands.json
```

## Troubleshooting

### Meson Uses Original Ninja

Ensure rninja is found first:

```bash
# Check which ninja is used
which ninja

# If it's not rninja, update PATH or create symlink
export PATH="$HOME/.cargo/bin:$PATH"
# Or
sudo ln -sf $(which rninja) /usr/local/bin/ninja
```

### Build Fails After Meson Update

```bash
# Full reconfigure
rm -rf build
meson setup build
rninja -C build
```

### Subproject Issues

```bash
# Ensure subprojects are up to date
meson subprojects update
meson setup build --reconfigure
rninja -C build
```

## Tips

### Use meson compile for Portability

For scripts that need to work without rninja:

```bash
meson compile -C build  # Uses whatever ninja is available
```

For maximum performance, call rninja directly:

```bash
rninja -C build  # Full control over flags
```

### Monitor Cache Performance

After builds:

```bash
rninja -C build -t cache-stats
```

### Meson Introspection

```bash
# List all targets
meson introspect build --targets

# Get build options
meson configure build
```

### Development Workflow

```bash
# Watch for changes and rebuild
while inotifywait -r -e modify src/; do
    rninja -C build
done
```
