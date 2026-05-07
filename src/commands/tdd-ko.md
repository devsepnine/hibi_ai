---
description: Enforce test-driven development workflow. Scaffold interfaces, generate tests FIRST, then implement minimal code to pass. Ensure 80%+ coverage.
allowed-tools: Task, Read, Write, Edit, Bash, Grep
model: opus
effort: xhigh
---

# TDD Command

**tdd-guide** 에이전트(`~/.claude/agents/tdd-guide.md`)를 호출하여 테스트 주도 개발을 강제한다.
공유 표준을 위해 `tdd-workflow` skill (`~/.claude/skills/tdd-workflow/`)과 `rules/testing.md`를 참조한다.

## TDD Cycle (MANDATORY)

```
RED → GREEN → REFACTOR → REPEAT
```

- **RED**: 실패하는 테스트를 작성한다. 실행하여 *올바른 이유*(예: 문법 오류가 아닌 `Not implemented`)로 실패하는지 확인한다.
- **GREEN**: 통과시키기 위한 최소한의 코드만 작성한다. 그 이상은 안 된다 — 조기 추상화 금지.
- **REFACTOR**: 그린 상태에서 개선한다. 변경할 때마다 테스트를 재실행한다. 테스트가 레드로 바뀌면 되돌리고 더 작은 단계를 시도한다.

RED를 절대 생략하지 않는다. 테스트 없이 프로덕션 코드를 작성하지 않는다.

## When to Use

- 새로운 피처, 함수, 컴포넌트
- 버그 수정 — 테스트는 수정 *전에* 버그를 재현해야 한다
- 기존 로직의 리팩토링 (테스트로 동작을 잠그고 변경)
- 핵심 비즈니스/금융/auth/보안 코드

## Agent Workflow (applies to every cycle)

`tdd-guide` 에이전트는 시나리오마다 이 루프를 실행한다:

1. **Scaffold** types/interfaces; 함수를 `Not implemented`를 throw하도록 stub 처리한다.
2. **Write failing tests** — happy path, edge cases (empty/null/zero/max), error paths.
3. **Run tests** → RED 확인. 예상치 못하게 GREEN이면, 테스트가 잘못된 것이다.
4. **Implement minimal code** → 테스트 실행 → GREEN 확인.
5. **Refactor** (상수, 헬퍼 추출, 중복 제거) → 재실행 → 여전히 GREEN.
6. **Check coverage**; 누락된 분기에 대한 테스트를 ≥ 80%까지 추가한다.

루프 후, 에이전트는 보고한다: 추가된 테스트, 커버리지 %, 변경된 파일.

## Minimal Worked Example

```ts
// 1. Scaffold (interface + stub)
export interface MarketData { totalVolume: number; bidAskSpread: number; activeTraders: number; lastTradeTime: Date }
export function calculateLiquidityScore(m: MarketData): number { throw new Error('Not implemented') }

// 2. Failing test (RED)
it('returns 0 for zero volume', () => {
  const m = { totalVolume: 0, bidAskSpread: 0, activeTraders: 0, lastTradeTime: new Date() }
  expect(calculateLiquidityScore(m)).toBe(0)
})

// 3. Minimal impl (GREEN)
export function calculateLiquidityScore(m: MarketData): number {
  if (m.totalVolume === 0) return 0
  // weighted score of volume / spread / traders / recency, clamped to [0, 100]
}
```

그 후 반복: 유동성 시장, 비유동성 시장, 경계값 테스트 추가 → 구현 → 리팩토링 (`WEIGHTS`, `clamp()` 추출) → 모두 GREEN인지 검증.

## Test Coverage by Type

- **Unit**: 함수 레벨 — happy path, empty/null/max, 오류 경로, 경계
- **Integration**: API 엔드포인트, DB 작업, 외부 호출, components+hooks
- **E2E**: 핵심 사용자 흐름 — `/e2e` 커맨드 사용

## Coverage Targets

- **80% minimum** for all code (see `rules/testing.md`)
- **100% required** for: 금융 계산, 인증, 보안 핵심 경로, 핵심 비즈니스 로직

## Best Practices

DO:
- 테스트를 먼저 작성하고, 실행하고, 어떠한 구현 전에 RED를 확인한다.
- 구현 세부사항이 아닌 동작(입력 → 출력)을 테스트한다.
- 동일한 RED 단계에서 엣지 케이스와 오류 시나리오를 다룬다.
- 그린 상태에서만 리팩토링한다; 리팩토링이 레드로 바뀌면 즉시 되돌린다.

DON'T:
- RED를 생략한다. 테스트 전에 코드를 작성한다. 모든 것을 모킹한다.
- 관련 없는 동작의 여러 어설션을 하나의 `it()`에 묶는다.
- `.skip` / `.only` / 주석 처리된 테스트를 커밋에 남긴다.

## Safety Guards

- **테스트를 약화시키지 않는다** GREEN을 강제하기 위해 — 대신 구현을 수정한다.
- **버그 수정 워크플로우**: 버그를 재현하는 회귀 테스트 → RED 확인 → 수정 → GREEN 확인.
- **flaky 테스트 금지**: 테스트가 간헐적으로 통과하면, 실패로 간주하고 근본 원인(타이밍, async leak, 공유 상태)을 수정한다.
- **커버리지 게이트**: PR이 커버리지를 80% 미만으로 떨어뜨리면, 머지 전에 테스트를 추가한다.

## Related Commands

`/plan` (코딩 전 범위 결정) → `/tdd` (this) → `/build-fix` (빌드 실패 시) → `/code-review` → `/test-coverage` → `/e2e`
