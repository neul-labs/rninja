---
title: Your First Build
description: Hands-on tutorial building a sample project
tags:
  - getting-started
  - tutorial
---

# Your First Build

This tutorial walks you through building a sample C project with rninja, demonstrating caching and common workflows.

## Prerequisites

- rninja installed ([Installation Guide](installation.md))
- A C compiler (gcc or clang)
- Basic familiarity with build systems

## Create a Sample Project

Let's create a simple C project to demonstrate rninja's features.

### Step 1: Create Project Structure

```bash
mkdir rninja-demo
cd rninja-demo
```

### Step 2: Create Source Files

Create `main.c`:

```c title="main.c"
#include <stdio.h>
#include "greet.h"

int main() {
    greet("World");
    return 0;
}
```

Create `greet.h`:

```c title="greet.h"
#ifndef GREET_H
#define GREET_H

void greet(const char *name);

#endif
```

Create `greet.c`:

```c title="greet.c"
#include <stdio.h>
#include "greet.h"

void greet(const char *name) {
    printf("Hello, %s!\n", name);
}
```

### Step 3: Create the Build File

Create `build.ninja`:

```ninja title="build.ninja"
# Ninja build file for demo project

# Variables
cc = gcc
cflags = -Wall -O2

# Rules
rule cc
  command = $cc $cflags -c $in -o $out
  description = CC $out

rule link
  command = $cc $in -o $out
  description = LINK $out

# Build edges
build main.o: cc main.c | greet.h
build greet.o: cc greet.c | greet.h
build hello: link main.o greet.o

# Default target
default hello
```

## Build with rninja

### First Build (Cold)

Run your first build:

```bash
rninja
```

Expected output:

```
[1/3] CC main.o
[2/3] CC greet.o
[3/3] LINK hello
```

Run the program:

```bash
./hello
```

Output:

```
Hello, World!
```

### Second Build (No-op)

Run rninja again without changes:

```bash
rninja
```

Expected output:

```
ninja: no work to do.
```

!!! note "Instant Detection"
    rninja detected nothing changed in under 10 milliseconds.

### Cached Rebuild

Now clean and rebuild to see caching in action:

```bash
# Clean all outputs
rninja -t clean

# Rebuild
rninja
```

Notice how the rebuild completes almost instantly? rninja restored the cached artifacts instead of recompiling.

## Explore Build Information

### View Dependencies

```bash
rninja -t deps main.o
```

Output:

```
main.o: #deps 2, deps mtime ...
    main.c
    greet.h
```

### Query a Target

```bash
rninja -t query hello
```

Output:

```
hello:
  input: link
    main.o
    greet.o
```

### Show All Targets

```bash
rninja -t targets
```

### Generate Compilation Database

For IDE integration (clangd, ccls, etc.):

```bash
rninja -t compdb > compile_commands.json
```

## Modify and Rebuild

### Change a Source File

Edit `greet.c` to change the message:

```c title="greet.c" hl_lines="5"
#include <stdio.h>
#include "greet.h"

void greet(const char *name) {
    printf("Greetings, %s!\n", name);  // Changed message
}
```

### Incremental Rebuild

```bash
rninja
```

Only the changed file and dependent targets rebuild:

```
[1/2] CC greet.o
[2/2] LINK hello
```

!!! tip "Incremental Builds"
    rninja tracks dependencies and only rebuilds what's necessary.

## View Cache Statistics

```bash
rninja -t cache-stats
```

Example output:

```
Cache Statistics:
  Enabled: true
  Mode: local
  Directory: /home/user/.cache/rninja

  Local Cache:
    Total entries: 6
    Total size: 24.3 KB
    Hit rate: 66.7%
```

## Debug Build Issues

### Explain Rebuilds

If you're unsure why something is rebuilding:

```bash
rninja -d explain
```

Output shows why each target needs rebuilding:

```
ninja explain: main.o is dirty
  greet.h has changed
```

### Verbose Output

See all commands being executed:

```bash
rninja -v
```

### Dry Run

See what would be built without building:

```bash
rninja -n
```

## Build with Different Options

### Parallel Jobs

Control parallelism:

```bash
# Use all CPU cores (default)
rninja -j0

# Limit to 2 parallel jobs
rninja -j2
```

### Continue on Errors

Keep building other targets if one fails:

```bash
rninja -k0  # Keep going indefinitely
rninja -k5  # Stop after 5 failures
```

## Clean Up

### Clean Build Outputs

```bash
rninja -t clean
```

### Clean Stale Outputs

Remove outputs that are no longer in the build:

```bash
rninja -t cleandead
```

### Clear Cache (if needed)

```bash
rninja -t cache-gc
```

## Using CMake Instead

If you prefer CMake, here's the equivalent workflow:

### Create CMakeLists.txt

```cmake title="CMakeLists.txt"
cmake_minimum_required(VERSION 3.10)
project(hello C)

add_executable(hello main.c greet.c)
```

### Generate and Build

```bash
# Generate Ninja build files
cmake -G Ninja -B build

# Build with rninja
rninja -C build
```

## Next Steps

<div class="grid cards" markdown>

-   :material-swap-horizontal: [__Migration Guide__](migration.md)

    Learn how to switch your team from Ninja to rninja

-   :material-cog: [__Configuration__](../user-guide/configuration/overview.md)

    Customize rninja's behavior

-   :material-cloud-upload: [__Remote Caching__](../caching/remote/quick-setup.md)

    Share cache across machines

-   :material-tools: [__All Subtools__](../user-guide/subtools/overview.md)

    Explore available tools

</div>
