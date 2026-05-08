# rninja

[![PyPI version](https://img.shields.io/pypi/v/rninja.svg)](https://pypi.org/project/rninja/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Build faster. Cache smarter. Drop-in ready.**

A Rust-powered drop-in replacement for [Ninja](https://ninja-build.org/) with built-in caching and modern scheduling. Cut your build times without changing your build files.

## Installation

```bash
pip install rninja
```

The package downloads the correct prebuilt binary for your platform on first use (macOS Intel/Apple Silicon, Linux x86_64/aarch64).

## Usage

After installation, use `rninja` exactly like `ninja`:

```bash
rninja
rninja -C out/Release
rninja -j8 my_target
```

## Features

- **Drop-in compatible** — Works with existing `.ninja` files from CMake, GN, Meson, or any generator
- **Built-in caching** — Content-addressed cache skips redundant work automatically
- **Modern scheduler** — Rust async runtime keeps all cores busy
- **Remote cache ready** — Share cached artifacts across machines and CI runners

## Documentation

- [Full documentation](https://docs.neullabs.com/rninja)
- [GitHub repository](https://github.com/neul-labs/rninja)

## License

MIT
