---
title: Code Style
description: rninja coding conventions
tags:
  - contributing
  - development
---

# Code Style

Coding conventions for rninja development.

## Rust Style

### Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

All code must pass `cargo fmt --check` in CI.

### Linting

Code must pass `clippy` without warnings:

```bash
cargo clippy -- -D warnings
```

### Naming Conventions

Follow Rust conventions:

| Item | Convention | Example |
|------|------------|---------|
| Types | PascalCase | `CacheEntry` |
| Functions | snake_case | `get_cache_key` |
| Variables | snake_case | `cache_hit` |
| Constants | SCREAMING_SNAKE | `MAX_CACHE_SIZE` |
| Modules | snake_case | `cache_manager` |

### Module Organization

```rust
// 1. Standard library
use std::collections::HashMap;
use std::path::PathBuf;

// 2. External crates
use serde::{Deserialize, Serialize};
use tokio::fs;

// 3. Internal modules
use crate::cache::CacheKey;
use crate::config::Config;
```

### Error Handling

Use `thiserror` for errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("cache entry not found: {0}")]
    NotFound(String),

    #[error("cache corrupted: {0}")]
    Corrupted(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

Use `?` for propagation:

```rust
fn read_cache(key: &CacheKey) -> Result<Vec<u8>, CacheError> {
    let path = get_cache_path(key)?;
    let data = std::fs::read(&path)?;
    Ok(data)
}
```

### Documentation

Document public APIs:

```rust
/// Computes the cache key for a build target.
///
/// The key is a BLAKE3 hash of:
/// - Rule name
/// - Command line
/// - Input file contents
///
/// # Arguments
///
/// * `target` - The build target
/// * `inputs` - Input files
///
/// # Returns
///
/// A 32-byte cache key.
///
/// # Examples
///
/// ```
/// let key = compute_cache_key(&target, &inputs)?;
/// assert_eq!(key.len(), 32);
/// ```
pub fn compute_cache_key(
    target: &Target,
    inputs: &[PathBuf],
) -> Result<CacheKey, CacheError> {
    // ...
}
```

### Testing

See [Testing](testing.md) for full guide.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        // Arrange
        let cache = Cache::new_test();
        let key = CacheKey::from_bytes(b"test");

        // Act
        cache.put(&key, b"data").unwrap();
        let result = cache.get(&key).unwrap();

        // Assert
        assert_eq!(result, b"data");
    }
}
```

## Code Organization

### File Size

- Keep files under 500 lines
- Split large modules into submodules

### Function Size

- Keep functions under 50 lines
- Extract complex logic into helpers

### Struct Design

Prefer composition over large structs:

```rust
// Good
struct BuildExecutor {
    scheduler: Scheduler,
    cache: CacheManager,
    reporter: ProgressReporter,
}

// Avoid
struct BuildExecutor {
    // 20 fields...
}
```

## Performance Guidelines

### Avoid Allocations in Hot Paths

```rust
// Good - reuse buffer
fn process_many(items: &[Item], buffer: &mut Vec<u8>) {
    for item in items {
        buffer.clear();
        process_one(item, buffer);
    }
}

// Avoid - allocates each iteration
fn process_many(items: &[Item]) {
    for item in items {
        let buffer = Vec::new();
        process_one(item, &buffer);
    }
}
```

### Use Appropriate Data Structures

```rust
// HashSet for O(1) lookups
let built: HashSet<PathBuf> = HashSet::new();

// BTreeMap for sorted iteration
let targets: BTreeMap<String, Target> = BTreeMap::new();
```

### Parallelize with Rayon

```rust
use rayon::prelude::*;

// Parallel iteration
let results: Vec<_> = inputs
    .par_iter()
    .map(|input| process(input))
    .collect();
```

## Safety Guidelines

### No Unsafe Without Justification

If `unsafe` is needed, document why:

```rust
// SAFETY: We verified that ptr is non-null and properly aligned
// in the check above. The lifetime is tied to self.
unsafe {
    &*ptr
}
```

### Handle Errors

Never use `.unwrap()` in library code:

```rust
// Good
let value = map.get(&key).ok_or(Error::NotFound)?;

// Avoid in library code
let value = map.get(&key).unwrap();
```

### Validate Input

```rust
pub fn set_jobs(n: u32) -> Result<(), ConfigError> {
    if n == 0 {
        return Err(ConfigError::InvalidValue("jobs must be > 0"));
    }
    // ...
}
```

## Comments

### When to Comment

- Explain "why", not "what"
- Document non-obvious behavior
- Reference issues or design docs

```rust
// Good
// Use BTreeMap to ensure deterministic iteration order,
// which is required for reproducible cache keys.
let env_vars: BTreeMap<String, String> = ...

// Avoid
// Create a map
let env_vars: BTreeMap<String, String> = ...
```

### TODO Comments

Include issue reference:

```rust
// TODO(#123): Implement retry logic
```

## Commit Messages

Format:

```
type(scope): description

Longer explanation if needed.

Closes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

```
feat(cache): add LRU eviction policy

Implement LRU eviction when cache exceeds max_size.
Entries are evicted based on last access time.

Closes #45
```

```
fix(daemon): handle socket permission errors

Check socket permissions before attempting to connect.
Provides better error message for permission issues.

Fixes #78
```

## Review Checklist

Before submitting PR:

- [ ] `cargo fmt` passes
- [ ] `cargo clippy` has no warnings
- [ ] Tests pass (`cargo test`)
- [ ] New code has tests
- [ ] Public APIs documented
- [ ] Commit messages follow format
