# 스타일과 API 명명

## 설정된 포맷 도구로 배치한다

기본 포맷은 [공식 Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)를 따른다. 기본값은 공백 네 칸 들여쓰기, 줄 너비 100자, 여러 줄 목록의 마지막 쉼표를 포함한다. 저장소가 정한 다른 설정도 유효하다.

포맷 명령을 고르기 전에 `rust-toolchain.toml`이나 `rust-toolchain`, `Cargo.toml`, `rustfmt.toml`이나 `.rustfmt.toml`을 확인한다. 고정된 툴체인과 기존 스타일 설정을 쓴다. nightly 문서 URL이 nightly 컴파일러나 새 스타일 에디션을 요구하지는 않는다.

워크스페이스는 루트에서 포맷을 확인한다:

```bash
cargo fmt --all -- --check
```

독립 크레이트는 해당 manifest를 지정한다:

```bash
cargo fmt --manifest-path path/to/Cargo.toml -- --check
```

저장소에 저장된 실행 구성이 있으면 사용한다. 요청한 범위에만 포맷을 적용한다. 동작 변경에 무관한 재포맷을 섞지 않는다. 고정된 포맷 도구와 현재 가이드가 다르면 차이를 기록하고, 설정 변경이 작업에 포함되지 않은 한 기존 툴체인을 유지한다.

## API 이름의 의미를 검수한다

아래 기준은 [Rust API Guidelines의 Naming](https://rust-lang.github.io/api-guidelines/naming.html)을 참고한다.

| 기준 | 확인할 내용 |
|---|---|
| `C-CASE` | 타입, trait, variant는 `Uuid` 같은 `UpperCamelCase`. 함수, 모듈, 지역 변수는 `snake_case`. 상수와 static은 `SCREAMING_SNAKE_CASE`. |
| `C-CONV` | `as_`는 저렴한 빌린 뷰, `to_`는 변환 작업이나 소유한 복사본, `into_`는 non-Copy 값을 소비하며 비용은 가변적이다. |
| `C-GETTER` | `name`과 `name_mut`을 선호한다. 인덱스로 찾는 `get`과 기존 예외는 유효하다. |
| `C-ITER`, `C-ITER-TY` | 동종 컬렉션은 `iter`, `iter_mut`, `into_iter`. 대응 타입은 `Iter`, `IterMut`, `IntoIter`. |
| `C-FEATURE` | 뜻이 있고 함께 켤 수 있는 feature 이름을 쓴다. `use-`, `with-`, 부정형 이름을 피한다. |
| `C-WORD-ORDER` | `ParseAddrError`처럼 관련 이름의 어순을 맞춘다. |

접두사만 보지 말고 실제 시그니처와 동작을 확인한다. `to_` 메서드는 할당 없이 빌린 값을 검증할 수도 있다. 가변 뷰의 이름은 `as_mut_slice`처럼 반환 타입의 어순을 따른다. 명명 패턴에 맞추려고 올바른 `get`의 이름이나 소유권을 바꾸지 않는다. API 변경이 작업에 포함되지 않았다면 공개 호환성을 보존한다. 포맷 통과는 명명 규칙의 정확성을 입증하지 않는다.
