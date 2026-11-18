# Rust Edition 2024 Guide

## Overview

Each Rust crate selects its own edition. When changing an existing crate’s edition, use the migration steps below to check source compatibility.

Default to Edition 2024 for a new project when the supported toolchain and dependencies permit it. A new workspace member follows the workspace edition policy.

## Setting Up Edition 2024

### Single Package

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"  # Minimum Rust version for Edition 2024
```

### Workspace

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
# Shared dependencies
```

Member crates inherit workspace settings:

```toml
# crates/my-crate/Cargo.toml
[package]
name = "my-crate"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
```

## Edition and compiler requirements

### Let chains

Let chains require Edition 2024 and Rust 1.88 or later. Use nested conditions with an earlier minimum compiler version.

```rust
fn active_name(user: Option<(&str, bool)>) -> Option<&str> {
    if let Some((name, active)) = user && active {
        Some(name)
    } else {
        None
    }
}
```

### Async methods and opaque return types in traits

Rust 1.75 stabilized native async methods and return position `impl Trait` in traits. Both work in Edition 2021. This caller uses static dispatch. A caller that stores `dyn Source` needs a compatible interface. Its methods could return a pinned boxed future and a boxed iterator, or the project could retain its existing adapter.

```rust
trait Source {
    async fn read(&self) -> usize;
    fn labels(&self) -> impl Iterator<Item = &str>;
}

async fn read_count<S: Source>(source: &S) -> usize {
    source.read().await
}
```

### Async callbacks

Rust 1.85 stabilized async closures and the `AsyncFn` family. This Edition 2021 example imports `AsyncFnMut` explicitly. The callback borrows its captured total on each call, and `process_items` stops at the first error.

```rust
use std::ops::AsyncFnMut;

async fn process_items<T, E>(
    items: impl IntoIterator<Item = T>,
    mut op: impl AsyncFnMut(T) -> Result<(), E>,
) -> Result<(), E> {
    for item in items {
        op(item).await?;
    }
    Ok(())
}

async fn checked_sum(items: &[u8]) -> Result<u8, &'static str> {
    let mut total = 0u8;
    process_items(items.iter().copied(), async |item| {
        total = total.checked_add(item).ok_or("sum overflow")?;
        Ok(())
    }).await?;
    Ok(total)
}
```

### Constant evaluation

A constant can call a stable `const fn` when its operations support constant evaluation on the project’s minimum Rust version. This example computes the buffer size at compile time and returns `None` on overflow.

```rust
const fn buffer_size(count: usize, item_size: usize) -> Option<usize> {
    count.checked_mul(item_size)
}

const BUFFER_SIZE: Option<usize> = buffer_size(100, 64);
```

## Migration from Earlier Editions

### From Edition 2021 → 2024

For a crate in migration scope, run `cargo fix --edition` under the old edition and review its changes. Then set `edition` to `2024` in `Cargo.toml` and run `cargo test` on the minimum supported Rust version. [Edition migration guide](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)

Optional changes after migration

Before changing an async trait, check for callers that use `dyn Trait`. An edition migration alone does not require replacing the existing macro.

**Simplify pattern matching**:
```rust
// ❌ Edition 2021: Nested matches
match user {
    Some(u) => {
        if u.is_admin {
            process_admin(u)
        }
    }
    None => {}
}

// ✅ Edition 2024: if let chains
if let Some(u) = user && u.is_admin {
    process_admin(u)
}
```

### Compatibility Notes

- Review code and macros whose meaning changes across editions.
- **Interoperability**: Edition 2024 crates can depend on earlier editions
- **Incremental migration**: Update crates one at a time in workspaces

```toml
# Workspace with mixed editions
[workspace]
members = ["crate-a", "crate-b"]

# crate-a/Cargo.toml
[package]
edition = "2024"

# crate-b/Cargo.toml (still on 2021)
[package]
edition = "2021"
```

## Best Practices for Edition 2024

### 1. Leverage Async Traits

```rust
// ✅ Define clean async interfaces
trait DataSource {
    async fn fetch(&self, id: &str) -> Result<Data, Error>;
    async fn store(&self, data: Data) -> Result<(), Error>;
}

// ✅ Use in generic contexts
async fn sync_data<S: DataSource>(source: &S, id: &str) -> Result<(), Error> {
    let data = source.fetch(id).await?;
    source.store(data).await
}
```

### 2. Use Pattern Matching Chains

```rust
// ✅ Cleaner conditional logic
fn validate_request(req: &Request) -> Result<(), Error> {
    if let Some(auth) = &req.auth
        && auth.is_valid()
        && !auth.is_expired() {
        Ok(())
    } else {
        Err(Error::Unauthorized)
    }
}

// ✅ Match with complex conditions
match (config.mode, config.level) {
    (Mode::Production, Level::Debug) if !config.allow_debug => {
        return Err(Error::InvalidConfig);
    }
    _ => {}
}
```

### 3. Utilize Improved Type Inference

```rust
// ✅ Let the compiler infer types
let processed = data
    .into_iter()
    .filter(|x| x.score > 80)
    .map(|x| x.into_summary())
    .collect();  // Type inferred

// ✅ Cleaner closure syntax
let handler = |req| async move {
    process(req).await
};
```

### 4. Const Functions for Zero-Cost Abstractions

```rust
// ✅ Compile-time computation
const fn page_size(level: u32) -> Option<usize> {
    match 1usize.checked_shl(level) {
        Some(pages) => pages.checked_mul(4096),
        None => None,
    }
}

const L1_SIZE: Option<usize> = page_size(0); // Some(4096)
const L2_SIZE: Option<usize> = page_size(1); // Some(8192)

// ✅ Const-evaluated collections
const fn create_lookup() -> [u8; 256] {
    let mut lookup = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        lookup[i] = (i * 2) as u8;
        i += 1;
    }
    lookup
}

const LOOKUP: [u8; 256] = create_lookup();
```

### 5. Enhanced Error Handling

```rust
use thiserror::Error;

// Derive errors with thiserror.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("validation failed: {field}: {reason}")]
    Validation { field: String, reason: String },

    #[error("not found: {0}")]
    NotFound(String),
}

// ✅ Use with improved pattern matching
fn handle_error(err: AppError) {
    if let AppError::Validation { field, reason } = err
        && field == "email" {
        // Handle email validation specifically
    }
}
```

## Common Patterns in Edition 2024

### 1. Builder Pattern with Async

```rust
// ✅ Async builder pattern
struct Config {
    host: String,
    port: u16,
}

struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self { host: None, port: None }
    }

    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    async fn build(self) -> Result<Config, Error> {
        let host = self.host.unwrap_or_else(|| "localhost".to_string());
        let port = self.port.unwrap_or(8080);

        // Async validation
        validate_host(&host).await?;

        Ok(Config { host, port })
    }
}
```

### 2. Repository Pattern with Async Traits

```rust
// ✅ Clean async repository
trait Repository<T> {
    async fn find_by_id(&self, id: u64) -> Result<Option<T>, Error>;
    async fn find_all(&self) -> Result<Vec<T>, Error>;
    async fn save(&self, entity: &T) -> Result<(), Error>;
    async fn delete(&self, id: u64) -> Result<(), Error>;
}

struct UserRepository {
    pool: Pool,
}

impl Repository<User> for UserRepository {
    async fn find_by_id(&self, id: u64) -> Result<Option<User>, Error> {
        let user = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn save(&self, user: &User) -> Result<(), Error> {
        sqlx::query("INSERT INTO users (id, name) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET name = excluded.name")
            .bind(user.id)
            .bind(&user.name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Other methods...
}
```

### 3. Service Layer with Dependency Injection

```rust
// ✅ Generic service with async trait bounds
trait Cache {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: String) -> Result<(), Error>;
}

trait Database {
    async fn query(&self, sql: &str) -> Result<Vec<Row>, Error>;
}

struct UserService<C, D>
where
    C: Cache,
    D: Database,
{
    cache: C,
    db: D,
}

impl<C: Cache, D: Database> UserService<C, D> {
    async fn get_user(&self, id: u64) -> Result<User, Error> {
        let cache_key = format!("user:{}", id);

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(serde_json::from_str(&cached)?);
        }

        // Fetch from database
        let rows = self.db.query(&format!("SELECT * FROM users WHERE id = {}", id)).await?;
        let user = parse_user(rows)?;

        // Update cache
        self.cache.set(&cache_key, serde_json::to_string(&user)?).await?;

        Ok(user)
    }
}
```

### 4. Error Context with Chains

```rust
use anyhow::{Context, Result};

// ✅ Rich error context
async fn process_file(path: &str) -> Result<ProcessedData> {
    let content = tokio::fs::read_to_string(path)
        .await
        .context(format!("Failed to read file: {}", path))?;

    let parsed = parse_content(&content)
        .context("Failed to parse content")?;

    if let Some(data) = parsed
        && data.is_valid() {
        process_data(data).await
            .context("Failed to process data")
    } else {
        Err(anyhow::anyhow!("Invalid or missing data"))
    }
}
```

## Performance Considerations

### 1. Const Evaluation for Performance

```rust
// ✅ Compute at compile time
const fn compute_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = expensive_computation(i);
        i += 1;
    }
    table
}

const LOOKUP_TABLE: [u32; 256] = compute_table();

// Runtime: O(1) lookup instead of O(n) computation
fn lookup(index: u8) -> u32 {
    LOOKUP_TABLE[index as usize]
}
```

### 2. Zero-Cost Async Abstractions

```rust
// ✅ Trait abstractions with no runtime overhead
trait Processor {
    async fn process(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// Monomorphization ensures no dynamic dispatch overhead
async fn process_batch<P: Processor>(processor: &P, items: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        results.push(processor.process(item).await?);
    }
    Ok(results)
}
```

### Rounding a buffer capacity

Use `checked_next_power_of_two` when rounding a buffer capacity so the caller can handle a result that exceeds the platform’s integer range.

```rust
assert_eq!(0usize.checked_next_power_of_two(), Some(1));
assert_eq!(17usize.checked_next_power_of_two(), Some(32));
assert_eq!(usize::MAX.checked_next_power_of_two(), None);
```

## Testing in Edition 2024

### Async Tests

```rust
// Tokio provides the async test runtime.
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_operation() {
        let service = MyService::new().await;
        let result = service.fetch_data("key").await.unwrap();
        assert_eq!(result, expected_value());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let service = MyService::new().await;

        let (r1, r2) = tokio::join!(
            service.operation_1(),
            service.operation_2()
        );

        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }
}
```

## Tooling and Lints

### Recommended Clippy Configuration

```toml
# Cargo.toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
# Project lint settings
enum_glob_use = "deny"
unwrap_used = "deny"
expect_used = "warn"
indexing_slicing = "warn"
pedantic = "warn"
nursery = "warn"

# Performance lints
large_stack_arrays = "warn"
large_types_passed_by_value = "warn"
```

### Rustfmt Configuration

```toml
# rustfmt.toml
edition = "2024"
max_width = 100
use_small_heuristics = "Max"
```

## Security Best Practices

```rust
// ✅ Use strong typing to prevent mistakes
struct UserId(u64);
struct SessionToken(String);

impl SessionToken {
    // Validate on construction
    fn new(token: String) -> Result<Self, Error> {
        if token.len() < 32 {
            return Err(Error::InvalidToken);
        }
        Ok(Self(token))
    }
}

// ✅ Zeroize sensitive data
use zeroize::Zeroize;

struct SecretKey {
    key: Vec<u8>,
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

// ✅ Constant-time comparisons
use subtle::ConstantTimeEq;

fn verify_token(provided: &[u8], expected: &[u8]) -> bool {
    provided.ct_eq(expected).into()
}
```

## Summary

After an edition change, verify callers and tests on the minimum supported toolchain.

## Additional Resources

- [Edition Guide](https://doc.rust-lang.org/edition-guide/)
- [Rust 2024 RFC](https://rust-lang.github.io/rfcs/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
