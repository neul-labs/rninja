---
title: Query Tools
description: Subtools for inspecting builds
tags:
  - user-guide
  - subtools
---

# Query Tools

Tools for inspecting and understanding your build.

## deps

Show dependencies stored in the deps log.

### Usage

```bash
# Show deps for a target
rninja -t deps target.o

# Show all deps
rninja -t deps
```

### Output

```
target.o: #deps 5, deps mtime 1234567890
    source.c
    header1.h
    header2.h
    config.h
    types.h
```

### Examples

```bash
# Check what a target depends on
rninja -t deps main.o

# Debug include issues
rninja -t deps problematic.o | grep missing_header
```

## query

Show inputs and outputs for a path.

### Usage

```bash
rninja -t query path/to/file
```

### Output

```
path/to/file:
  input: rule_name
    input1
    input2
  outputs:
    output1
    output2
```

### Examples

```bash
# What produces this file?
rninja -t query libfoo.a

# What depends on this file?
rninja -t query common.h
```

## graph

Generate a Graphviz dependency graph.

### Usage

```bash
# Graph for all targets
rninja -t graph > deps.dot

# Graph for specific target
rninja -t graph my_target > target.dot
```

### Rendering

```bash
# Generate PNG
rninja -t graph | dot -Tpng > deps.png

# Generate SVG (better for large graphs)
rninja -t graph | dot -Tsvg > deps.svg

# Generate PDF
rninja -t graph | dot -Tpdf > deps.pdf
```

### Examples

```bash
# Visualize entire build
rninja -t graph | dot -Tpng > full_build.png

# Visualize specific target's dependencies
rninja -t graph my_executable | dot -Tsvg > my_exe.svg

# Open in browser (SVG)
rninja -t graph | dot -Tsvg > deps.svg && xdg-open deps.svg
```

!!! tip "Large Graphs"
    For large projects, use SVG format and a viewer that supports panning/zooming.

## path

Find the dependency path between two targets.

### Usage

```bash
rninja -t path source target
```

### Output

```
source.c
  header.h
    types.h
      target.o
```

### Examples

```bash
# Why does changing this file rebuild that target?
rninja -t path common.h final_binary

# Find connection between files
rninja -t path utils.c main
```

## targets

List targets by rule or depth.

### Usage

```bash
# List all targets
rninja -t targets all

# List targets by rule
rninja -t targets rule rule_name

# List targets by depth
rninja -t targets depth N
```

### Examples

```bash
# All targets
rninja -t targets all

# All compilation targets
rninja -t targets rule cc

# All link targets
rninja -t targets rule link

# Targets at depth 1 (direct dependencies of default)
rninja -t targets depth 1
```

## rules

List all rules defined in the build file.

### Usage

```bash
rninja -t rules
```

### Output

```
cc
cxx
link
ar
```

### Examples

```bash
# See what rules are available
rninja -t rules

# Check if a rule exists
rninja -t rules | grep custom_rule
```

## commands

List commands for rebuilding targets.

### Usage

```bash
# Commands for all targets
rninja -t commands

# Commands for specific target
rninja -t commands my_target
```

### Output

```
gcc -c -o main.o main.c
gcc -c -o util.o util.c
gcc -o myprogram main.o util.o
```

### Examples

```bash
# See exact compilation commands
rninja -t commands main.o

# Debug compiler flags
rninja -t commands | grep problematic_file

# Export for scripting
rninja -t commands > build_commands.txt
```

## inputs

List all inputs required to build targets.

### Usage

```bash
# Inputs for all targets
rninja -t inputs

# Inputs for specific target
rninja -t inputs my_target
```

### Examples

```bash
# What files are needed?
rninja -t inputs final_binary

# Find all source files
rninja -t inputs | grep '\.c$'
```

## compdb

Generate a JSON compilation database.

### Usage

```bash
rninja -t compdb > compile_commands.json

# Filter by rule
rninja -t compdb cc cxx > compile_commands.json
```

### Output Format

```json
[
  {
    "directory": "/path/to/project",
    "command": "gcc -c -o main.o main.c",
    "file": "main.c",
    "output": "main.o"
  }
]
```

### Examples

```bash
# Generate for IDE/editor
rninja -t compdb > compile_commands.json

# For clangd
ln -s compile_commands.json .

# For ccls
rninja -t compdb > compile_commands.json

# Only C++ compilations
rninja -t compdb cxx > compile_commands.json
```

### IDE Integration

Most C/C++ language servers use `compile_commands.json`:

- **clangd**: Reads from project root
- **ccls**: Reads from project root
- **VS Code C/C++**: Configure `compileCommands` setting
- **CLion**: Auto-detects in project root

## Workflow Examples

### Understanding a Target

```bash
# What is it?
rninja -t query target

# What are its dependencies?
rninja -t deps target

# What command builds it?
rninja -t commands target

# Visualize its dependency tree
rninja -t graph target | dot -Tpng > target.png
```

### Debugging Build Issues

```bash
# Why is X rebuilding?
rninja -t deps X

# What depends on changed file?
rninja -t query changed_file

# Path from change to rebuild
rninja -t path changed_file rebuilt_target
```

### Build Analysis

```bash
# How many targets?
rninja -t targets all | wc -l

# What rules are used?
rninja -t rules

# Compilation database
rninja -t compdb > compile_commands.json
```

## Tips

### Use `query` First

When debugging, start with `query`:

```bash
rninja -t query mystery_file
```

### Pipe to Tools

These tools output text that works well with standard tools:

```bash
# Search deps
rninja -t deps | grep header.h

# Count targets
rninja -t targets all | wc -l

# Sort rules
rninja -t rules | sort
```

### Generate Compilation Database Once

After CMake/Meson regeneration:

```bash
# In project root
rninja -t compdb > compile_commands.json
```
