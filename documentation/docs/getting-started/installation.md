---
title: Installation
description: Install rninja on your system
tags:
  - getting-started
  - installation
---

# Installation

rninja can be installed on Linux, macOS, and Windows. Choose the installation method that works best for your environment.

## Requirements

- **Rust toolchain** (for installation via Cargo)
- **Git** (for building from source)

## Installation Methods

### Homebrew (macOS & Linux)

The easiest way to install rninja on macOS and Linux is via Homebrew:

```bash
brew tap neul-labs/tap
brew install rninja
```

This installs the following binaries:

| Binary | Description |
|--------|-------------|
| `rninja` | Main CLI (drop-in Ninja replacement) |
| `rninja-daemon` | Build daemon for faster subsequent builds |
| `rninja-cached` | Remote cache server |

### NPM

You can install rninja globally via npm:

```bash
npm install -g rninja
```

The npm package downloads the correct prebuilt binary for your platform automatically.

### PyPI

You can install rninja via pip:

```bash
pip install rninja
```

The PyPI package downloads the correct prebuilt binary for your platform on first use.

### From Crates.io

Install via Cargo:

```bash
cargo install rninja
```

### From GitHub Releases

Download prebuilt binaries for your platform from the [Releases page](https://github.com/neul-labs/rninja/releases).

### From Source

To build from source with the latest changes:

```bash
# Clone the repository
git clone https://github.com/neul-labs/rninja
cd rninja

# Build and install
cargo install --path .
```

For development builds:

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, faster runtime)
cargo build --release
```

### Verifying Installation

After installation, verify rninja is working:

```bash
rninja --version
```

Expected output:

```
rninja 0.1.1
```

List available subtools:

```bash
rninja -t list
```

## Setting Up as Ninja Replacement

### Option 1: Alias

Add to your shell configuration (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
alias ninja='rninja'
```

### Option 2: Symlink

Create a symlink so tools that call `ninja` use rninja:

=== "Linux/macOS"

    ```bash
    sudo ln -s $(which rninja) /usr/local/bin/ninja
    ```

=== "User-local"

    ```bash
    mkdir -p ~/.local/bin
    ln -s $(which rninja) ~/.local/bin/ninja
    # Ensure ~/.local/bin is in your PATH
    ```

!!! warning "Symlink Priority"
    If you already have Ninja installed, ensure the symlink location comes before Ninja in your `PATH`, or remove the original Ninja binary.

### Option 3: Environment Variable

Some build systems respect environment variables:

```bash
# For CMake
export CMAKE_MAKE_PROGRAM=$(which rninja)

# Or configure CMake directly
cmake -DCMAKE_MAKE_PROGRAM=$(which rninja) ..
```

## Platform-Specific Notes

### Linux

rninja works on all modern Linux distributions. No special configuration needed.

```bash
# Verify installation
which rninja
rninja --version
```

### macOS

rninja works on macOS 10.15 (Catalina) and later.

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install rninja
cargo install rninja
```

### Windows

rninja works on Windows 10 and later with the Rust toolchain installed.

```powershell
# Install via Cargo
cargo install rninja

# Add to PATH if needed
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
```

## Installing Additional Components

### Remote Cache Server

The remote cache server (`rninja-cached`) is installed automatically with the main package. To run it:

```bash
rninja-cached --help
```

See [Remote Cache Deployment](../caching/remote/deployment.md) for setup instructions.

### Daemon

The daemon (`rninja-daemon`) is also installed automatically. It typically auto-starts when you run `rninja`, but can be managed manually:

```bash
rninja-daemon --help
```

See [Daemon Management](../daemon/management.md) for details.

## Updating rninja

To update to the latest version:

```bash
cargo install rninja --force
```

Or if you built from source:

```bash
cd rninja
git pull
cargo install --path . --force
```

## Uninstalling

To remove rninja:

```bash
cargo uninstall rninja
```

If you created a symlink:

```bash
sudo rm /usr/local/bin/ninja  # or wherever you created it
```

## Troubleshooting Installation

### Cargo Not Found

If `cargo` is not found, install the Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build Fails

If the build fails, ensure you have the required build tools:

=== "Ubuntu/Debian"

    ```bash
    sudo apt update
    sudo apt install build-essential pkg-config
    ```

=== "Fedora/RHEL"

    ```bash
    sudo dnf install gcc pkg-config
    ```

=== "macOS"

    ```bash
    xcode-select --install
    ```

### Permission Denied

If you get permission errors when creating symlinks:

```bash
# Use user-local directory instead
mkdir -p ~/.local/bin
ln -s $(which rninja) ~/.local/bin/ninja

# Add to PATH in your shell config
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Next Steps

<div class="grid cards" markdown>

-   :material-rocket-launch: [__Quick Start__](quick-start.md)

    Get your first build running

-   :material-cog: [__Configuration__](../user-guide/configuration/overview.md)

    Configure rninja for your workflow

</div>
