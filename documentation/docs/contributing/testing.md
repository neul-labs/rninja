---
title: Testing
description: How to test rninja
tags:
  - contributing
  - development
---

# Testing

Guide to testing rninja.

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test

```bash
# By name
cargo test test_cache_hit

# By module
cargo test cache::

# With output
cargo test test_name -- --nocapture
```

### Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Doc tests
cargo test --doc
```

## Test Structure

### Unit Tests

In-module tests for internal functions:

```rust
// src/cache/key.rs

pub fn compute_key(input: &[u8]) -> [u8; 32] {
    blake3::hash(input).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_key_deterministic() {
        let input = b"test input";
        let key1 = compute_key(input);
        let key2 = compute_key(input);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_compute_key_different_inputs() {
        let key1 = compute_key(b"input1");
        let key2 = compute_key(b"input2");
        assert_ne!(key1, key2);
    }
}
```

### Integration Tests

In `tests/` directory:

```rust
// tests/integration_test.rs

use rninja::Cache;
use tempfile::TempDir;

#[test]
fn test_cache_roundtrip() {
    let temp = TempDir::new().unwrap();
    let cache = Cache::new(temp.path()).unwrap();

    let key = [0u8; 32];
    let data = b"test data";

    cache.put(&key, data).unwrap();
    let retrieved = cache.get(&key).unwrap();

    assert_eq!(retrieved, data);
}
```

### Async Tests

For async code:

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Test Utilities

### Test Fixtures

```rust
// tests/common/mod.rs

use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestFixture {
    pub dir: TempDir,
    pub build_file: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let build_file = dir.path().join("build.ninja");
        std::fs::write(&build_file, SIMPLE_BUILD_NINJA).unwrap();
        Self { dir, build_file }
    }
}

const SIMPLE_BUILD_NINJA: &str = r#"
rule cc
  command = cc -c $in -o $out

build foo.o: cc foo.c
"#;
```

### Using Fixtures

```rust
// tests/build_test.rs

mod common;
use common::TestFixture;

#[test]
fn test_simple_build() {
    let fixture = TestFixture::new();

    // Create source file
    std::fs::write(fixture.dir.path().join("foo.c"), "int main() {}").unwrap();

    // Run build
    let result = rninja::build(&fixture.build_file, &["foo.o"]);
    assert!(result.is_ok());
}
```

### Mock Objects

```rust
use mockall::automock;

#[automock]
pub trait FileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    fn write(&self, path: &Path, data: &[u8]) -> Result<()>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockFileSystem::new();
    mock.expect_read()
        .returning(|_| Ok(b"content".to_vec()));

    let result = function_under_test(&mock);
    assert!(result.is_ok());
}
```

## Test Patterns

### Arrange-Act-Assert

```rust
#[test]
fn test_example() {
    // Arrange
    let cache = Cache::new_test();
    let key = CacheKey::from_bytes(b"test");

    // Act
    cache.put(&key, b"data").unwrap();
    let result = cache.get(&key);

    // Assert
    assert_eq!(result.unwrap(), b"data");
}
```

### Table-Driven Tests

```rust
#[test]
fn test_parse_size() {
    let cases = vec![
        ("1K", 1024),
        ("1M", 1024 * 1024),
        ("1G", 1024 * 1024 * 1024),
        ("100", 100),
    ];

    for (input, expected) in cases {
        let result = parse_size(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}
```

### Error Testing

```rust
#[test]
fn test_error_case() {
    let result = function_that_fails();

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
}

#[test]
#[should_panic(expected = "invalid input")]
fn test_panic() {
    function_that_panics("invalid");
}
```

## Ninja Compatibility Tests

Compare with real Ninja:

```rust
#[test]
fn test_ninja_compatibility() {
    let fixture = TestFixture::new();

    // Run with ninja
    let ninja_output = Command::new("ninja")
        .args(["-n", "-v"])
        .current_dir(fixture.dir.path())
        .output()
        .unwrap();

    // Run with rninja
    let rninja_output = Command::new("rninja")
        .args(["-n", "-v"])
        .current_dir(fixture.dir.path())
        .output()
        .unwrap();

    assert_eq!(ninja_output.stdout, rninja_output.stdout);
}
```

## Benchmarks

In `benches/`:

```rust
// benches/cache_bench.rs

use criterion::{criterion_group, criterion_main, Criterion};
use rninja::Cache;

fn cache_lookup_benchmark(c: &mut Criterion) {
    let cache = setup_cache_with_entries(1000);
    let key = existing_key();

    c.bench_function("cache_lookup", |b| {
        b.iter(|| cache.get(&key))
    });
}

criterion_group!(benches, cache_lookup_benchmark);
criterion_main!(benches);
```

Run:

```bash
cargo bench
```

## Coverage

Generate coverage report:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## CI Testing

Tests run in CI on every PR:

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
```

## Test Guidelines

### What to Test

- Public API functions
- Edge cases
- Error conditions
- Ninja compatibility
- Performance-critical paths

### What Not to Test

- Internal implementation details
- Trivial getters/setters
- Third-party library behavior

### Test Naming

```rust
#[test]
fn test_function_name_condition_expected_result() {
    // test_cache_get_missing_key_returns_none
    // test_parse_size_invalid_suffix_returns_error
}
```

### Test Independence

Each test should:
- Set up its own state
- Clean up after itself
- Not depend on other tests
- Work in any order

```rust
#[test]
fn test_independent() {
    let temp = TempDir::new().unwrap();  // Own state
    // ... test ...
}  // Cleaned up automatically
```
