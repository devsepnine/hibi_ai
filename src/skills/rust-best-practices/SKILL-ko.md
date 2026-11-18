---
name: rust-best-practices
description: "Rust practices: ownership, errors, async, tests, formatting, API naming, project layout. Use when writing or reviewing Rust. 러스트 코드 작성, Rust 모범사례, API 명명, rustfmt, Rust 리뷰. NOT iced/ratatui specifics."
metadata:
  version: "1.0.0"
---

# Rust Best Practices

다섯 영역인 소유권, 에러, async, 테스트, 프로젝트 구조에 걸친 관용적이고 안전하며 성능 좋은 Rust 코드 작성법.

**코드를 작성하기 전에 프로젝트의 개발·코딩 지침과 아키텍처 문서에서 변경 대상에 적용되는 규칙과 예외를 확인한다.** 설계는 그 규칙을 따르고, 선택이 열려 있는 부분에서는 함수형 설계를 기본값으로 삼는다. 리뷰할 때는 변경한 코드를 해당 규칙과 대조한다.

## 호출자가 필요로 하는 표현 고르기

유효한 상태를 명시하고, 가능한 곳에서는 데이터 변환을 순수 함수로 작성하며, 입출력은 경계에 모은다. 이 기본값은 이번 변경 범위 안에서 적용한다.

### trait 대신 enum을 선택할 때

도메인이 가능한 경우를 모두 정의하면 enum으로 합 타입을 표현한다. 예를 들어 `enum Task { Open, Done { completed_at: Timestamp } }`는 완료한 작업에만 완료 시각을 요구한다. 각 변형을 `match`에 명시하면, 나중에 `Cancelled`를 추가할 때 새 분기가 필요한 곳마다 컴파일 오류가 난다. 플러그인이 독립적으로 구현을 추가해야 하고 호출자는 공통 연산만 사용한다면 그 연산을 trait로 정의한다.

enum에도 메서드를 둘 수 있다. 여러 호출자가 같은 분기를 반복하면 그 연산을 enum의 메서드로 모은다. 값의 부재는 `Option<T>`, 호출자가 처리해야 할 실패는 `Result<T, E>`로 표현한다. 호출 지점에서 성공과 실패 사례를 하나씩 시도하고 타입, 테스트, 사용 예시를 함께 고친다. AI가 작성한 Rust 코드를 리뷰할 때도 그 호출에서 값이 어떻게 전달되는지 확인한 뒤 추상화를 채택한다.

### 컬렉션에 맞는 디스패치 고르기

두 경우에 별도 데이터가 필요 없으면 불리언 분기로 충분하다. 서로 다른 데이터를 가진 변형이 섞인다면 enum을 패턴 매칭한다. 한 묶음의 원소가 모두 같은 구체 타입이면 `&[T]`를 받는 제네릭 함수가 루프 전체에서 해당 타입으로 정적 디스패치할 수 있다. 실행 중 구현을 선택하면 `dyn Trait`를 고려한다. TypeScript와 Python의 제네릭도 컬렉션의 타입 제약을 표현할 수 있다. 타입 인자만으로 각 타입에 특화된 기계어가 생성되지는 않는다.

### 생성 중 변경과 이후의 공유 읽기 분리하기

두 컴포넌트가 같은 데이터를 변경해야 한다면 누가 만들고 누가 이후에 읽기만 하는지 추적한다. 가능하면 생성 단계에서 변경을 마친다. 두 호출자가 독립적인 값을 필요로 하면 작은 값을 복제해 양쪽 API를 단순하게 유지할 수 있다. 변경 결과를 공유해야 하면 쓰기를 담당하는 주체를 명시한다.

### 순환 관계와 수명을 함께 모델링하기

부모를 참조하는 트리에서는 부모가 자식을 소유하고 자식이 부모를 약한 참조로 가리킬 수 있다. 강한 `Rc` 참조가 순환하면 노드가 해제되지 않고, `RefCell`을 쓰면 빌림 규칙을 실행 중에 검사한다. 노드의 수명이 같다면 아레나가 노드를 소유하고 ID로 연결하는 방법을 검토한다. 벡터에 모아 저장하면 개별 할당을 줄일 수 있고, 저장 순서대로 순회할 때 캐시 지역성도 좋아질 수 있다. 삭제한 슬롯을 재사용하면 세대 번호로 오래된 ID를 검출한다. 같은 노드 이름이 반복된다면 인터닝으로 중복 저장을 줄일 수 있다. 인터닝의 조회 비용은 아레나 선택과 별개로 판단한다.

### 선언적 프레임워크로 경계에서 한 번 파싱하기

Serde의 derive는 컴파일 중에 직렬화와 역직렬화 구현을 생성한다. clap의 derive를 쓰면 CLI 타입 선언과 문서 주석으로 도움말도 구성할 수 있다. 실행 중에는 실제 입력을 파싱하고, 도메인 타입으로 변환할 때 제약을 검사한 뒤, 처리 함수에 그 값을 전달한다. 기존 derive가 계약을 표현할 수 있으면 이를 활용한다. 직접 만든 매크로는 문법과 오류 메시지까지 유지해야 한다.

### 성능을 측정해 가능한 설계 찾기

검색 가능한 JSON 인덱스에 쓸 수 있는 메모리나 대화형 쿼리의 응답 시간처럼 기능이 요구하는 한도를 먼저 정한다. 대표 데이터로 단순한 구현을 측정한다. 해시 조회, 복제, 할당, 간접 호출 중 어느 하나가 병목이면 그 부분을 바꾸고 전체 연산을 다시 측정한다. 인덱스를 만드는 시간과 최대 메모리 사용량도 비교한다. 측정값이 한도 안에 들어오면 메모리 안에서 검색하는 설계를 선택할 수 있다. 단순한 표현으로 한도를 충족하면 그대로 유지한다.

## 스타일과 API 명명

Rust를 작성하거나 리뷰할 때 [스타일과 API 명명](references/style-and-api-naming-ko.md)을 적용한다. 공식 포맷과 명명 규칙의 링크와 검수 기준을 담았다. 저장소의 툴체인, 에디션, 포맷 설정을 따른다. nightly 문서 URL은 이 설정들을 올리라는 지시가 아니다. 변환 메서드는 소유권과 비용을 확인하고 이름을 정한다. 포맷 도구는 그 의미를 검증하지 못한다. 기존 공개 API의 이름을 바꾸기 전에 호출자와 호환성을 확인한다.

## 기본 설정: Edition 2024

새 프로젝트에서는 지원할 툴체인과 의존성이 허용할 때 Edition 2024를 기본값으로 쓴다. 워크스페이스에 크레이트를 추가하면 해당 워크스페이스의 에디션 정책을 따른다.

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

에디션과 최소 컴파일러 버전을 함께 확인한다. 예를 들어 `if let` 체인은 Edition 2024와 Rust 1.88 이상이 필요하다. [에디션 설정](references/edition-2024-ko.md)

## 1. Ownership & Borrowing

규칙: 단일 소유자, scope 종료 시 drop, move 또는 borrow, `&` / `&mut`.

| Use | When |
|-----|------|
| `&T` | 읽기 전용 접근 |
| `&mut T` | 배타적 쓰기 |
| `T`, owned | 소유권 이전 / 저장 |
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
- `Option`/`Result`는 combinator인 `and_then`, `map`, `ok_or`로 합성한다.

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
- 런타임은 하나만 고른다, I/O 중심에는 tokio, 임베디드에는 smol, async-std는 거의 안 씀.
- async 안에서 절대 블로킹 금지: `std::thread::sleep` 대신 `tokio::time::sleep`을 쓴다. CPU 작업 → `tokio::task::spawn_blocking`.
- 구조적 동시성: `join!`, `select!`, `tokio::time::timeout`. drop을 통한 cancel을 처리한다.
- 태스크 오버헤드를 의식한다, 사소한 작업에 spawn 하지 않는다.

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
- 유닛 테스트는 코드 옆에 둔다, `#[cfg(test)] mod tests`.
- 통합 테스트는 public API surface 단위로 `tests/`에 둔다.
- Doc test는 `cargo test --doc`으로.
- 프로퍼티 테스트는 `proptest`, 커버리지는 `cargo-llvm-cov`.
- 외부 의존성은 구체 타입이 아닌 trait로 mocking 한다.
- 항상 실패 경로와 엣지 케이스인 empty/max/concurrent를 테스트한다.
- 실패 사례 설계에는 `tdd-workflow`, 실패 후 상태를 포함한 셀프리뷰에는 `coding-standards`를 적용한다.

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

- iterator chain은 타이트한 루프로 컴파일된다, 수동 인덱싱보다 선호하라.
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
| `pub fn divide(a, b) -> i32 { a / b }` | `Result` 반환, lib에서는 panic 금지 |
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

- [스타일과 API 명명](references/style-and-api-naming-ko.md)
- [Edition 2024 Guide](references/edition-2024.md), start here
- [Ownership & Borrowing](references/ownership-borrowing.md)
- [Error Handling](references/error-handling.md)
- [Async Patterns](references/async-patterns.md)
- [Testing](references/testing.md)
- [Project Structure](references/project-structure.md)

External: [The Rust Book](https://doc.rust-lang.org/book/) · [API Guidelines](https://rust-lang.github.io/api-guidelines/) · [Performance Book](https://nnethercote.github.io/perf-book/) · [Effective Rust](https://www.lurklurk.org/effective-rust/)
