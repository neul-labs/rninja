# rninja

[![Version](https://img.shields.io/npm/v/rninja-cli.svg)](https://www.npmjs.com/package/rninja-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**rninja is a drop-in replacement for [Ninja](https://ninja-build.org/) with built-in caching — build faster, cache smarter, change nothing.**

**[Website](https://rninja.neullabs.com)** · **[Documentation](https://docs.neullabs.com/rninja)** · **[GitHub](https://github.com/neul-labs/rninja)**

A Rust-powered build tool with modern scheduling that reads your existing `.ninja` files and cuts build times without changing a line of your build configuration.

## Installation

```bash
npm install -g rninja-cli
```

The post-install script automatically downloads the correct prebuilt binary for your platform (macOS Intel/Apple Silicon, Linux x86_64/aarch64, Windows x86_64).

## Usage

Use `rninja` exactly like `ninja`:

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

## Part of the Neul Labs toolchain

Explore the rest of the Neul Labs developer tools:

| Project | Description |
| --- | --- |
| [rjest](https://github.com/neul-labs/rjest) | A blazing-fast, Jest-compatible test runner — 100x faster warm runs. |
| [rpytest](https://github.com/neul-labs/rpytest) | Run your pytest suite faster. Change nothing. |
| [gity](https://github.com/neul-labs/gity) | Make large Git repositories feel instant. |
| [stkd](https://github.com/neul-labs/stkd) | Stacked diffs for GitHub and GitLab. |
| [grite](https://github.com/neul-labs/grite) | The issue tracker that lives in your repo. Built for AI agents. |

Learn more at [neullabs.com](https://www.neullabs.com).

## License

MIT
