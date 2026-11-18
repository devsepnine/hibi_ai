---
name: rust-best-practices
description: "Rust practices: ownership, errors, async, tests, formatting, API naming, project layout. Use when writing or reviewing Rust. 러스트 코드 작성, Rust 모범사례, API 명명, rustfmt, Rust 리뷰. NOT iced/ratatui specifics."
metadata:
  version: "1.0.0"
---

# Rust Best Practices

Idiomatic, safe, performant Rust across five areas: ownership, errors, async, testing, project structure.

**Before writing code, read the project’s development, coding, and architecture guidance for the area you will change, including any exceptions.** Follow those rules when choosing a design. Where they leave a choice, default to a functional design. During review, check the diff against the applicable rules.

## Choose a representation from the caller

Model valid states explicitly, keep transformations pure where practical, and isolate I/O at the boundary. Apply this default within the scope of the change.

### When to choose an enum over a trait

Model alternatives with an enum when the domain defines the full set. For example, `enum Task { Open, Done { completed_at: Timestamp } }` requires a timestamp only for completed tasks. Match each variant explicitly. Adding `Cancelled` then produces a compiler error at every match that needs a new arm. If plugins must add implementations independently and callers only need a shared operation, define a trait for that operation.

An enum can also own methods shared by its variants. Put a shared operation there when callers would otherwise repeat the same match. Use `Option<T>` for absence and `Result<T, E>` for a failure the caller must handle. Try one success and one failure at a call site, then revise the type, test, and usage example together. When reviewing generated Rust, follow a value through that call before accepting the proposed abstraction.

### Choose dispatch for the collection

For two cases, a boolean branch suffices when each case needs no extra data. For mixed values with distinct payloads, match the enum. If a batch contains one concrete type, a generic function over `&[T]` can use static dispatch for that type throughout the loop. Use `dyn Trait` for implementations chosen at runtime. TypeScript and Python generics can describe the same collection constraint. Their type arguments alone do not generate specialized machine code.

### Separate construction from shared reads

When two components need mutable access, trace who constructs the data and who only reads it afterward. Finish mutation during construction when possible. If two callers genuinely need independent values, a small clone can keep both APIs simple. When they need shared updates, give the write operation an explicit owner.

### Model cycles and lifetimes together

For a tree with parent links, children may own their descendants and hold a weak link back. A cycle of strong `Rc` links keeps nodes alive, and `RefCell` moves borrow checks to runtime. When nodes share a lifetime, consider an arena that owns the nodes and represents edges with IDs. A packed vector can reduce individual allocations and improve locality when traversal follows storage order. If slots are reused, check generations so stale IDs cannot reach new nodes. For repeated node names, interning adds a lookup and can save space independently of the arena.

### Parse once with a declarative framework

Serde derives generate serialization and deserialization implementations during compilation. With clap derive, a typed CLI declaration can also supply help text from doc comments. At runtime, parse the actual input, enforce domain constraints in the conversion, and pass the resulting value to the handler. Prefer the framework’s existing derive when it expresses the contract. A custom macro brings its own syntax, diagnostics, and maintenance cost.

### Measure which designs become feasible

Set a budget from the feature, such as the memory available for a searchable JSON index or the latency of an interactive query. Measure a simple implementation on representative data. If a hash lookup, clone, allocation, or indirect call dominates, change that part and measure the whole operation again. Include index construction and peak memory in the comparison. A result that fits the budget can make an in memory design feasible. Keep the simpler representation while it meets the budget.

## Style and API naming

When writing or reviewing Rust, use [Style and API naming](references/style-and-api-naming.md). It links the official formatting and naming rules and gives focused review checks. Follow the repository's toolchain, edition, and formatter configuration. The nightly documentation URL is not an instruction to upgrade them. Check ownership and cost when naming conversions; a formatter cannot validate those meanings. Before renaming an existing public API, check callers and compatibility.

## Default Configuration: Edition 2024

Default to Edition 2024 for a new project when the supported toolchain and dependencies permit it. A new workspace member follows the workspace edition policy.

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

Check both the edition and the minimum compiler version. For example, let chains require Edition 2024 and Rust 1.88 or later. [Edition settings](references/edition-2024.md)

## 1. Ownership & Borrowing

Rules: single owner, dropped at scope end, move or borrow, `&` / `&mut`.

| Use | When |
|-----|------|
| `&T` | read-only access |
| `&mut T` | exclusive write |
| `T`, owned | transfer ownership / store |
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
- Combine `Option`/`Result` with combinators, `and_then`, `map`, `ok_or`.

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
- Pick one runtime, tokio for I/O-heavy, smol for embedded, async-std rarely.
- NEVER block in async: use `tokio::time::sleep`, not `std::thread::sleep`. CPU work → `tokio::task::spawn_blocking`.
- Structured concurrency: `join!`, `select!`, `tokio::time::timeout`. Handle cancel via drop.
- Beware task overhead, don't spawn for trivial work.

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
- Unit tests live next to code, `#[cfg(test)] mod tests`.
- Integration tests in `tests/` per public API surface.
- Doc tests via `cargo test --doc`.
- Property tests with `proptest`; coverage with `cargo-llvm-cov`.
- Mock external deps via traits, not concrete types.
- Always test failure paths and edge cases, empty/max/concurrent.
- Use `tdd-workflow` for failure case design and `coding-standards` for self-review, including state after failure.

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

- Iterator chains compile to tight loops, prefer over manual indexing.
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
| `pub fn divide(a, b) -> i32 { a / b }` | return `Result`, no panics in libs |
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

- [Style and API naming](references/style-and-api-naming.md)
- [Edition 2024 Guide](references/edition-2024.md), start here
- [Ownership & Borrowing](references/ownership-borrowing.md)
- [Error Handling](references/error-handling.md)
- [Async Patterns](references/async-patterns.md)
- [Testing](references/testing.md)
- [Project Structure](references/project-structure.md)

External: [The Rust Book](https://doc.rust-lang.org/book/) · [API Guidelines](https://rust-lang.github.io/api-guidelines/) · [Performance Book](https://nnethercote.github.io/perf-book/) · [Effective Rust](https://www.lurklurk.org/effective-rust/)
