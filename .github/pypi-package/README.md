# rninja

A drop-in replacement for the Ninja build system with built-in caching and improved scheduling.

This Python package provides a thin wrapper that downloads the correct prebuilt binary for your platform on first use.

## Installation

```bash
pip install rninja
```

## Usage

After installation, use `rninja` exactly like `ninja`:

```bash
rninja
rninja -C out/Release
rninja -j8 my_target
```

Binaries are downloaded automatically on first invocation.
