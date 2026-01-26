---
title: Development Setup
description: Setting up rninja development environment
tags:
  - contributing
  - development
---

# Development Setup

Set up your environment for rninja development.

## Prerequisites

### Required

- **Rust**: 1.70 or later
- **Git**: For version control
- **Ninja**: For comparison testing

### Optional

- **Docker**: For containerized builds
- **Python 3**: For test scripts
- **CMake/Meson**: For integration tests

## Quick Setup

```bash
# Clone repository
git clone https://github.com/anthropics/rninja
cd rninja

# Build
cargo build

# Run tests
cargo test

# Run rninja
cargo run -- --version
```

## Detailed Setup

### Install Rust

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install stable toolchain
rustup default stable

# Add components
rustup component add rustfmt clippy
```

### Clone Repository

```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/rninja
cd rninja

# Add upstream
git remote add upstream https://github.com/anthropics/rninja

# Keep in sync
git fetch upstream
git merge upstream/main
```

### Build

```bash
# Debug build (fast compile, slow run)
cargo build

# Release build (slow compile, fast run)
cargo build --release

# Build all binaries
cargo build --all
```

Binaries are in `target/debug/` or `target/release/`.

### Run

```bash
# Via cargo
cargo run -- -C /path/to/project

# Or directly
./target/debug/rninja --version
```

## Project Structure

```
rninja/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli.rs           # Argument parsing
│   ├── config.rs        # Configuration
│   ├── build/           # Build execution
│   ├── cache/           # Caching subsystem
│   ├── daemon/          # Daemon process
│   ├── server/          # Remote cache server
│   └── ...
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── docs/                # Internal docs
├── documentation/       # User docs (MkDocs)
├── Cargo.toml           # Dependencies
└── README.md
```

## IDE Setup

### VS Code

Install extensions:
- rust-analyzer
- CodeLLDB (debugging)
- Even Better TOML

Settings (`.vscode/settings.json`):

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all"
}
```

### IntelliJ/CLion

- Install Rust plugin
- Open `Cargo.toml` as project
- Enable clippy in settings

### Vim/Neovim

With rust-analyzer LSP:

```lua
-- init.lua
require'lspconfig'.rust_analyzer.setup{}
```

## Environment Variables

For development:

```bash
# Enable debug logging
export RUST_LOG=rninja=debug

# Use local cache dir
export RNINJA_CACHE_DIR=/tmp/rninja-dev-cache

# Disable daemon for testing
export RNINJA_DAEMON_MODE=off
```

## Test Data

Create test fixtures:

```bash
# Generate test project
./scripts/generate-test-project.sh large /tmp/test-project

# Create build.ninja
cd /tmp/test-project
cmake -G Ninja .
```

## Debugging

### VS Code

Launch config (`.vscode/launch.json`):

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug rninja",
            "cargo": {
                "args": ["build", "--bin=rninja"]
            },
            "args": ["-C", "/path/to/project"],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### Command Line

```bash
# With rust-gdb
rust-gdb target/debug/rninja

# With rust-lldb
rust-lldb target/debug/rninja
```

### Logging

```bash
# Verbose logging
RUST_LOG=trace cargo run -- -C /project

# Specific module
RUST_LOG=rninja::cache=debug cargo run
```

## Building Documentation

```bash
cd documentation

# Install dependencies
pip install -r requirements.txt

# Serve locally
mkdocs serve

# Build static site
mkdocs build
```

## Benchmarking

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench cache_lookup

# Compare with baseline
cargo bench -- --save-baseline before
# Make changes
cargo bench -- --baseline before
```

## Common Tasks

### Update Dependencies

```bash
# Check outdated
cargo outdated

# Update all
cargo update

# Update specific
cargo update -p serde
```

### Format Code

```bash
# Format all
cargo fmt

# Check only
cargo fmt --check
```

### Lint

```bash
# Run clippy
cargo clippy

# Fix automatically
cargo clippy --fix
```

### Generate Docs

```bash
# API docs
cargo doc --open
```

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Test Failures

```bash
# Run single test with output
cargo test test_name -- --nocapture
```

### Linker Errors

On Linux, ensure you have:

```bash
sudo apt install build-essential pkg-config libssl-dev
```

On macOS:

```bash
xcode-select --install
```

## Next Steps

- [Code Style](code-style.md): Coding conventions
- [Testing](testing.md): How to write tests
- [Contributing Guide](guide.md): Contribution process
