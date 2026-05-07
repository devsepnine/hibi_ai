# Code Thresholds

프로젝트 전반의 복잡도 한계. 2단계 모델: **Soft** 목표는 설계를 가이드하고, **Hard** 한계는 머지를 차단한다.

## 임계값 한도

| Metric | Soft (target) | Hard (block) | Why | On Violation |
|--------|---------------|--------------|-----|--------------|
| File Length | ≤ 300 LOC | ≤ 500 LOC | 단일 책임, 한눈에 파악 | 관심사별로 모듈 분리 |
| Function Length | ≤ 50 LOC | ≤ 80 LOC | 테스트, 명명, 재사용 용이 | 헬퍼 추출 |
| Parameters | ≤ 5 | ≤ 7 | 호출부 명료성, 느슨한 결합 | options struct / builder 도입 |
| Cyclomatic Complexity | ≤ 10 | ≤ 15 | 분기 폭발이 테스트 가능 케이스를 제한 | early return, 전략 분리 |
| Nesting Depth | ≤ 4 | ≤ 6 | 선형 읽기 흐름 | guard clause, 함수 추출 |

**Soft** (warning): PR 리뷰에서 논의하고, 가능하면 리팩토링한다.
**Hard** (error): 머지 전에 반드시 리팩토링하거나 예외를 문서화해야 한다.

## 함수 크기 > 파일 크기

작고 응집된 함수들로 구성된 500 LOC 파일이 200 LOC짜리 단일 함수를 가진 250 LOC 파일보다 더 건강하다. 우선순위:

1. **함수 길이** — 복잡도의 가장 강한 신호
2. **순환 복잡도(Cyclomatic complexity)** — 테스트 부담을 예측
3. **중첩 깊이(Nesting depth)** — 독자의 인지 부하를 예측
4. **파일 길이** — 마지막. 응집된 모듈을 분리하면 지역성을 해칠 수 있다

## 측정 규칙

- **LOC**: 빈 줄과 주석 전용 줄은 제외한다.
- **Parameters**: positional + named + optional 합산. 구조 분해된 객체는 1개로 센다.
- **Cyclomatic Complexity**: `if`, `else if`, `match/case` arm, `&&`/`||`, `?:`, loop, `catch` 각각 +1.
- **Nesting Depth**: 함수 본문 내부의 중괄호 깊이(제어 블록, 클로저).

## 허용되는 예외

예외에는 사유를 명시한 파일 최상단 주석이 필요하다. 명시적으로 면제되지 않는 한 Hard 한계는 여전히 적용된다.

- 자동 생성 코드(protobuf, OpenAPI, codegen stub) — 모두 면제
- 테스트 픽스처 / 데이터 테이블 — File LOC 면제, 함수는 여전히 적용
- 타입 정의 파일(types.rs, d.ts) — File LOC Hard 한계를 800으로 상향
- 불가피한 분기 맵(상태 머신, 라우트 테이블) — Complexity 면제
- 일회성 마이그레이션 스크립트 — 모두 면제

## 강제 도구

- **Rust**: `cargo clippy -- -W clippy::cognitive_complexity -W clippy::too_many_arguments`, `tokei`
- **TypeScript**: ESLint `max-lines` (warn:300, error:500), `max-lines-per-function`, `complexity`, `max-params`
- **Python**: `radon cc`, `flake8 --max-complexity=10`

## 리팩토링 트리거

**Rule of Three**를 적용한다: 동일 패턴이 3회 이상 반복되면 추출 신호이다. Soft 임계값 위반과 결합되면 미루지 말고 즉시 리팩토링한다.

## 관련 규칙

- `coding-style.md` — 일반 코딩 스타일과 파일 구성
- `pull-request-rules.md` — 이 임계값을 포함한 PR 체크리스트
- `development-workflow.md` — TDD 및 리뷰 워크플로우
