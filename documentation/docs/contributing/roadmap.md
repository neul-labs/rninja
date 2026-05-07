---
title: Roadmap
description: rninja development roadmap
tags:
  - contributing
---

# Roadmap

rninja development plans and future direction.

## Current Status

rninja is production-ready with core features:

- Full Ninja compatibility
- Local caching
- Remote caching
- Daemon mode
- CLI tooling

## Near-term Goals

### Performance

- [ ] Parallel build file parsing
- [ ] Improved scheduling algorithm
- [ ] Memory usage optimization
- [ ] Faster cache lookups

### Caching

- [ ] S3-compatible storage backend
- [ ] GCS storage backend
- [ ] Cache warming tools
- [ ] Better cache key debugging

### Usability

- [ ] Improved error messages
- [ ] Progress bar improvements
- [ ] Better IDE integration
- [ ] Shell completions

### Operations

- [ ] Prometheus metrics endpoint
- [ ] Health check improvements
- [ ] Log aggregation support
- [ ] Better Docker support

## Medium-term Goals

### Distributed Builds

Distribute build execution across machines:

- Remote execution protocol
- Worker node management
- Load balancing
- Failure handling

### Advanced Caching

- Content-aware deduplication
- Predictive cache warming
- Cache analytics
- Multi-tier caching

### Build Intelligence

- Build time predictions
- Bottleneck detection
- Optimization suggestions
- Trend analysis

### Ecosystem

- Language server protocol support
- Editor plugins
- Build system integrations
- Cloud platform integrations

## Long-term Vision

### Goals

1. **Fastest builds**: Always faster than alternatives
2. **Zero configuration**: Works perfectly out of the box
3. **Universal caching**: Share builds across the world
4. **Build insights**: Deep understanding of your builds

### Non-Goals

- Replacing higher-level build systems (CMake, Meson)
- Build file generation
- Package management
- Dependency resolution

## Contributing to Roadmap

### Suggest Features

Open an issue with:
- Use case description
- Proposed solution
- Impact assessment

### Vote on Features

React with 👍 on issues you want prioritized.

### Implement Features

Check issues labeled:
- `good first issue`: Great for newcomers
- `help wanted`: Ready for contribution
- `roadmap`: Planned features

## Version History

### v0.1.1 (Current)

- Initial release
- Ninja compatibility
- Local caching
- Remote caching
- Daemon mode

### Future Releases

Versions follow semantic versioning:

- **Major** (1.0, 2.0): Breaking changes
- **Minor** (0.2, 0.3): New features
- **Patch** (0.1.1): Bug fixes

## Release Cadence

- **Patch releases**: As needed for bug fixes
- **Minor releases**: Monthly with new features
- **Major releases**: When breaking changes needed

## Deprecation Policy

- Features deprecated with 2 minor version warning
- Breaking changes only in major versions
- Migration guides provided

## Community Input

Roadmap is influenced by:

- GitHub issues and discussions
- User feedback
- Production usage patterns
- Ecosystem needs

Your input shapes rninja's future!

## Getting Involved

### Feature Development

1. Check roadmap issues
2. Comment your interest
3. Discuss approach
4. Submit PR

### Testing

- Try beta releases
- Report issues
- Benchmark in your environment

### Documentation

- Improve guides
- Add examples
- Translate content

See [Contributing Guide](guide.md) to get started.
