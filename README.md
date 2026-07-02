# rninja

[![Crates.io](https://img.shields.io/crates/v/rninja.svg)](https://crates.io/crates/rninja)
[![npm](https://img.shields.io/npm/v/rninja-cli)](https://www.npmjs.com/package/rninja-cli)
[![PyPI](https://img.shields.io/pypi/v/rninja-cli)](https://pypi.org/project/rninja-cli/)
[![CI](https://github.com/neul-labs/rninja/actions/workflows/ci.yml/badge.svg)](https://github.com/neul-labs/rninja/actions)
[![Documentation](https://img.shields.io/badge/docs-neullabs.com-blue)](https://docs.neullabs.com/rninja)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**rninja is a drop-in replacement for [Ninja](https://ninja-build.org/) with built-in caching — build faster, cache smarter, change nothing.**

**[Website](https://rninja.neullabs.com)** · **[Documentation](https://docs.neullabs.com/rninja)** · **[GitHub](https://github.com/neul-labs/rninja)**

A Rust-powered build tool with modern scheduling that reads your existing `.ninja` files and cuts build times without changing a line of your build configuration.

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap neul-labs/tap
brew install rninja
```

### NPM

```bash
npm install -g rninja-cli
```

### PyPI

```bash
pip install rninja-cli
```

### Cargo

```bash
cargo install rninja
```

### From GitHub Releases

Download prebuilt binaries from the [Releases page](https://github.com/neul-labs/rninja/releases).

### Build from source

```bash
git clone https://github.com/neul-labs/rninja
cd rninja
cargo install --path .
```

## Usage

rninja works exactly like ninja—just swap the binary:

```bash
# Use directly
rninja

# Or with your existing workflow
rninja -C out/Release
rninja -j8 my_target

# Symlink as ninja for seamless integration
ln -s $(which rninja) /usr/local/bin/ninja
```

All ninja flags work: `-C`, `-j`, `-k`, `-d`, `-t`, and more.

## Why rninja?

| Feature | Benefit |
|---------|---------|
| **Drop-in compatible** | Works with existing `.ninja` files from CMake, GN, Meson, or any generator |
| **Built-in caching** | Content-addressed cache skips redundant work automatically |
| **Modern scheduler** | Rust async runtime keeps all cores busy, minimizing idle time |
| **Remote cache ready** | Share cached artifacts across machines and CI runners |

### Performance

| Scenario | Speedup |
|----------|---------|
| Warm incremental builds | 2× – 5× |
| CI with shared cache | 2× – 5× |
| Cold builds | 1.3× – 2× |

See [BENCHMARK.md](BENCHMARK.md) for detailed comparisons.

## Configuration

rninja works out of the box with sensible defaults. For customization:

```bash
# Generate a sample config
rninja -t config -v
```

Config files are loaded from:
1. `.rninjarc` (project)
2. `~/.rninjarc` (user)
3. `~/.config/rninja/config.toml` (XDG)

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RNINJA_CACHE_DIR` | Cache directory location |
| `RNINJA_CACHE_ENABLED` | Enable/disable caching (`0` or `1`) |
| `RNINJA_REMOTE_URL` | Remote cache server URL |
| `RNINJA_CACHE_MODE` | `local`, `remote`, or `both` |

## Subtools

All ninja subtools plus extras:

```bash
rninja -t clean       # Remove built files
rninja -t compdb      # Dump JSON compilation database
rninja -t graph       # Output graphviz dot file
rninja -t deps        # Show stored dependencies
rninja -t query       # Show inputs/outputs for a path
rninja -t config      # Show/generate configuration
```

Run `rninja -t list` for the complete list.

## Who is rninja for?

- **C/C++ projects** with multi-minute incremental builds
- **CI pipelines** running many builds per day
- **Monorepos** with shared code across teams
- **Game studios** and performance-sensitive teams already using Ninja
- **Anyone** who wants faster builds without changing their workflow

## How it works

1. **Parse** your existing `build.ninja` file (no changes needed)
2. **Hash** inputs, compiler flags, and environment
3. **Check cache** for matching artifacts
4. **Build** only what's actually changed
5. **Store** results for next time

The local cache uses [sled](https://github.com/spacejam/sled) for metadata and content-addressed blob storage for artifacts. Optional remote caching uses [nng](https://nng.nanomsg.org/) for high-throughput artifact sharing.

## Contributing

Contributions welcome! Please see our [GitHub repository](https://github.com/neul-labs/rninja) for:

- Bug reports and feature requests
- Pull requests
- Architecture discussions

## Documentation

- [Performance deep-dive](docs/performance.md)
- [Architecture overview](docs/architecture.md)
- [Drop-in compatibility guide](docs/dropin.md)
- [Roadmap](docs/roadmap.md)

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

MIT License - see [LICENSE](LICENSE) for details.

---

Built with Rust by [Neul Labs](https://www.neullabs.com)
