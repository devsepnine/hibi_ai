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

다섯 영역(소유권, 에러, async, 테스트, 프로젝트 구조)에 걸친 관용적이고 안전하며 성능 좋은 Rust 코드 작성법.

## 기본 설정: Edition 2024

새 프로젝트는 항상 Edition 2024를 사용한다.

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

Edition 2024는 다음을 지원한다: trait의 native `async fn` (`async-trait` 의존 제거), `if let` chain, RPITIT, 개선된 lifetime elision, 향상된 const fn, 더 정확한 진단. 자세한 내용은 [edition-2024.md](references/edition-2024.md) 참고.

## 1. Ownership & Borrowing

규칙: 단일 소유자, scope 종료 시 drop, move 또는 borrow (`&` / `&mut`).

| Use | When |
|-----|------|
| `&T` | 읽기 전용 접근 |
| `&mut T` | 배타적 쓰기 |
| `T` (owned) | 소유권 이전 / 저장 |
| `Clone` | 호출자와 피호출자 모두 소유권이 필요할 때만 |

함수 파라미터 기본값: `String`보다 `&str`, `Vec<T>`보다 `&[T]`, `PathBuf`보다 `&Path`를 선호한다.

```rust
// Good: borrow when reading, return owned when producing
fn process(data: &[u8]) -> Vec<u8> { data.iter().map(|b| b ^ 0xFF).collect() }
fn append(buf: &mut Vec<u8>, data: &[u8]) { buf.extend_from_slice(data); }
```

자세한 내용: [ownership-borrowing.md](references/ownership-borrowing.md) · [Rust Book Ch.4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)

## 2. Error Handling

실패에는 `Result<T, E>`, 부재에는 `Option<T>`를 사용한다. 라이브러리 코드 → `thiserror`. 애플리케이션 코드 → `anyhow`.

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

규칙:
- 문서화된 invariant 없이 프로덕션에서 `unwrap()` / `expect()` 금지.
- `?`로 propagate; `map_err` 또는 `anyhow::Context::context`로 컨텍스트를 더한다.
- `Option`/`Result`는 combinator(`and_then`, `map`, `ok_or`)로 합성한다.

자세한 내용: [error-handling.md](references/error-handling.md) · [Rust Book Ch.9](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

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

규칙:
- 런타임은 하나만 고른다 (I/O 중심에는 tokio, 임베디드에는 smol, async-std는 거의 안 씀).
- async 안에서 절대 블로킹 금지: `std::thread::sleep` 대신 `tokio::time::sleep`을 쓴다. CPU 작업 → `tokio::task::spawn_blocking`.
- 구조적 동시성: `join!`, `select!`, `tokio::time::timeout`. drop을 통한 cancel을 처리한다.
- 태스크 오버헤드를 의식한다 — 사소한 작업에 spawn 하지 않는다.

자세한 내용: [async-patterns.md](references/async-patterns.md) · [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

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

전략:
- 유닛 테스트는 코드 옆에 둔다 (`#[cfg(test)] mod tests`).
- 통합 테스트는 public API surface 단위로 `tests/`에 둔다.
- Doc test는 `cargo test --doc`으로.
- 프로퍼티 테스트는 `proptest`, 커버리지는 `cargo-llvm-cov`.
- 외부 의존성은 구체 타입이 아닌 trait로 mocking 한다.
- 항상 실패 경로와 엣지 케이스(empty/max/concurrent)를 테스트한다.

자세한 내용: [testing.md](references/testing.md) · [Rust Book Ch.11](https://doc.rust-lang.org/book/ch11-00-testing.html)

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

Workspace 스켈레톤:
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

모듈 규칙:
- `mod.rs` 또는 `module_name.rs` 중 하나를 골라 일관성을 유지한다.
- 모듈당 하나의 책임; crate 루트에서 `pub use`로 재노출한다.
- public API는 `///`로 문서화하고, 내부는 `#[doc(hidden)]`으로 숨긴다.
- `[dependencies]` / `[dev-dependencies]` / `[build-dependencies]`를 정확히 분리한다.

자세한 내용: [project-structure.md](references/project-structure.md) · [Cargo Book](https://doc.rust-lang.org/cargo/)

## 성능 빠른 개선

- iterator chain은 타이트한 루프로 컴파일된다 — 수동 인덱싱보다 선호하라.
- 미리 사이즈를 잡는다: 알려진 경계에는 `Vec::with_capacity`, `String::with_capacity`.
- `#[inline]`은 작고 hot한 함수에만; 그 외에는 컴파일러를 신뢰한다.
- 파이프라인 중간에서 `.collect()` 금지; 마지막에 한 번만 collect 한다.

## 보안 빠른 개선

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

## 안티패턴

| Anti-pattern | Fix |
|--------------|-----|
| `fn f(s: String)` 인데 읽기만 함 | `fn f(s: &str)` |
| 동작시키려고 `s.clone()` | borrow 하거나 소유권 구조를 다시 잡는다 |
| `pub fn divide(a, b) -> i32 { a / b }` | `Result` 반환 — lib에서는 panic 금지 |
| `let _ = file.write_all(data);` | 에러를 propagate 하거나 log 한다 |
| `async`에서 `std::thread::sleep` | `tokio::time::sleep(...).await` |
| 프로덕션 경로에서 `unwrap()` | `?`, `ok_or`, 또는 invariant 문서화 |

## 도구

`cargo fmt` · `cargo clippy` · `cargo audit` · `cargo outdated` · `cargo deny` · `cargo watch` · `cargo-llvm-cov`

권장 lint 베이스라인:
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
