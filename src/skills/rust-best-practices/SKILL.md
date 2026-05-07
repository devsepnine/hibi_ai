---
name: rust-best-practices
description: Comprehensive Rust best practices covering ownership, error handling, async patterns, testing, and project structure. Use when writing Rust code, reviewing Rust, 러스트 코드 작성, Rust 모범사례, Rust 리뷰, 소유권 패턴.
keywords: [rust, 러스트, ownership, 소유권, async, best-practices, 모범사례]
version: 1.0.0
categories:
  - rust
  - best-practices
  - programming
  - systems
---

# Rust Best Practices

Idiomatic, safe, performant Rust across five areas: ownership, errors, async, testing, project structure.

## Default Configuration: Edition 2024

ALWAYS use Edition 2024 for new projects.

```toml
[package]
edition = "2024"
rust-version = "1.85"

# Workspace
[workspace.package]
edition = "2024"
[workspace]
resolver = "2"
```

Edition 2024 brings: native `async fn` in traits (drops `async-trait`), `if let` chains, RPITIT, better lifetime elision, improved const fn, sharper diagnostics. See [edition-2024.md](references/edition-2024.md).

## 1. Ownership & Borrowing

Rules: single owner, dropped at scope end, move or borrow (`&` / `&mut`).

| Use | When |
|-----|------|
| `&T` | read-only access |
| `&mut T` | exclusive write |
| `T` (owned) | transfer ownership / store |
| `Clone` | only when caller AND callee need ownership |

Function param defaults: prefer `&str` over `String`, `&[T]` over `Vec<T>`, `&Path` over `PathBuf`.

```rust
// Good: borrow when reading, return owned when producing
fn process(data: &[u8]) -> Vec<u8> { data.iter().map(|b| b ^ 0xFF).collect() }
fn append(buf: &mut Vec<u8>, data: &[u8]) { buf.extend_from_slice(data); }
```

Details: [ownership-borrowing.md](references/ownership-borrowing.md) · [Rust Book Ch.4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)

## 2. Error Handling

Use `Result<T, E>` for failures, `Option<T>` for absence. Library code → `thiserror`. Application code → `anyhow`.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("io: {0}")]      Io(#[from] std::io::Error),
    #[error("parse: {0}")]   Parse(String),
    #[error("missing: {0}")] Missing(String),
}

fn load(path: &str) -> Result<Config, ConfigError> {
    let s = std::fs::read_to_string(path)?;            // ? converts via From
    serde_json::from_str(&s).map_err(|e| ConfigError::Parse(e.to_string()))
}
```

Rules:
- Never `unwrap()` / `expect()` in production without a documented invariant.
- Propagate with `?`; add context with `map_err` or `anyhow::Context::context`.
- Combine `Option`/`Result` with combinators (`and_then`, `map`, `ok_or`).

Details: [error-handling.md](references/error-handling.md) · [Rust Book Ch.9](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

## 3. Async Patterns

```rust
async fn fetch(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
}

// Concurrent fan-out
let results = futures::future::join_all(urls.iter().map(fetch)).await;

// Background task
tokio::spawn(async move { work(data).await });

// Channels for cross-task communication
let (tx, mut rx) = tokio::sync::mpsc::channel(100);
```

Rules:
- Pick one runtime (tokio for I/O-heavy, smol for embedded, async-std rarely).
- NEVER block in async: use `tokio::time::sleep`, not `std::thread::sleep`. CPU work → `tokio::task::spawn_blocking`.
- Structured concurrency: `join!`, `select!`, `tokio::time::timeout`. Handle cancel via drop.
- Beware task overhead — don't spawn for trivial work.

Details: [async-patterns.md](references/async-patterns.md) · [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

## 4. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn add_works() { assert_eq!(add(2, 2), 4); }

    #[test]
    fn divide_errors_on_zero() {
        assert!(matches!(divide(10, 0), Err(_)));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn panics_on_oob() { let v = vec![1]; let _ = v[99]; }
}
```

Strategy:
- Unit tests live next to code (`#[cfg(test)] mod tests`).
- Integration tests in `tests/` per public API surface.
- Doc tests via `cargo test --doc`.
- Property tests with `proptest`; coverage with `cargo-llvm-cov`.
- Mock external deps via traits, not concrete types.
- Always test failure paths and edge cases (empty/max/concurrent).

Details: [testing.md](references/testing.md) · [Rust Book Ch.11](https://doc.rust-lang.org/book/ch11-00-testing.html)

## 5. Project Structure

```
my-project/
├── Cargo.toml          # edition = "2024"
├── src/
│   ├── main.rs / lib.rs
│   └── <feature>/mod.rs
├── tests/              # integration
├── benches/            # criterion
└── examples/
```

Workspace skeleton:
```toml
[workspace]
members = ["crates/*"]
resolver = "2"
[workspace.package]
edition = "2024"
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

Module rules:
- Pick `mod.rs` OR `module_name.rs`, stay consistent.
- One responsibility per module; re-export with `pub use` at crate root.
- Doc public APIs with `///`; hide internals with `#[doc(hidden)]`.
- Split `[dependencies]` / `[dev-dependencies]` / `[build-dependencies]` correctly.

Details: [project-structure.md](references/project-structure.md) · [Cargo Book](https://doc.rust-lang.org/cargo/)

## Performance Quick Wins

- Iterator chains compile to tight loops — prefer over manual indexing.
- Pre-size: `Vec::with_capacity`, `String::with_capacity` for known bounds.
- `#[inline]` only on small hot functions; trust the compiler otherwise.
- Avoid `.collect()` in middle of pipelines; collect once at the end.

## Security Quick Wins

```rust
// Newtype to prevent confusion
struct UserId(u64);
struct PostId(u64);

// Input validation with `validator`
#[derive(validator::Validate)]
struct Input {
    #[validate(email)] email: String,
    #[validate(length(min = 8, max = 100))] password: String,
}

// Constant-time comparison for secrets
use subtle::ConstantTimeEq;
fn verify(a: &[u8], b: &[u8]) -> bool { a.ct_eq(b).into() }
```

## Anti-Patterns

| Anti-pattern | Fix |
|--------------|-----|
| `fn f(s: String)` then only reading | `fn f(s: &str)` |
| `s.clone()` to "make it work" | borrow, or restructure ownership |
| `pub fn divide(a, b) -> i32 { a / b }` | return `Result` — no panics in libs |
| `let _ = file.write_all(data);` | propagate or log the error |
| `std::thread::sleep` in `async` | `tokio::time::sleep(...).await` |
| `unwrap()` in prod paths | `?`, `ok_or`, or document the invariant |

## Tools

`cargo fmt` · `cargo clippy` · `cargo audit` · `cargo outdated` · `cargo deny` · `cargo watch` · `cargo-llvm-cov`

Recommended lint floor:
```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
enum_glob_use = "deny"
```

## References

- [Edition 2024 Guide](references/edition-2024.md) (start here)
- [Ownership & Borrowing](references/ownership-borrowing.md)
- [Error Handling](references/error-handling.md)
- [Async Patterns](references/async-patterns.md)
- [Testing](references/testing.md)
- [Project Structure](references/project-structure.md)

External: [The Rust Book](https://doc.rust-lang.org/book/) · [API Guidelines](https://rust-lang.github.io/api-guidelines/) · [Performance Book](https://nnethercote.github.io/perf-book/) · [Effective Rust](https://www.lurklurk.org/effective-rust/)
