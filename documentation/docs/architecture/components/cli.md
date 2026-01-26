---
title: rninja CLI Architecture
description: CLI component architecture
tags:
  - architecture
---

# rninja CLI Architecture

The main command-line interface.

## Responsibilities

- Parse command-line arguments
- Connect to daemon (or spawn)
- Forward build requests
- Display output

## Key Modules

| Module | Purpose |
|--------|---------|
| `cli.rs` | Argument parsing (clap) |
| `main.rs` | Entry point |
| `output.rs` | Output formatting |

## Execution Flow

1. Parse arguments
2. Load configuration
3. Connect to daemon
4. Send build request
5. Stream output
6. Report result

## Configuration Loading

Priority order:

1. CLI arguments
2. Environment variables
3. Config files (.rninjarc)
4. Defaults
