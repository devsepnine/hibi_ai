---
name: tdd-workflow
description: "Test-first development: red/green/refactor, unit/integration/E2E, 80%+ coverage. Use when adding a feature, fixing a bug, or refactoring. TDD, 테스트 주도 개발, 테스트 우선, 커버리지."
---

# Test-Driven Development Workflow

모든 코드가 종합적인 테스트 커버리지와 함께 TDD 원칙을 따르도록 보장한다.

## 활성화 시점

- 새 기능, 버그 수정, 리팩토링
- API 엔드포인트, 컴포넌트
- 런타임 동작이 바뀌는 모든 코드

## 핵심 원칙

1. **코드 전에 테스트**: 실패하는 테스트를 먼저 쓰고, 구현한다
2. **80%+ 커버리지**: unit + integration + E2E 합산
3. **모든 경로 테스트**: happy path, 엣지 케이스, 에러, 경계
4. **구현이 아니라 동작**: 사용자가 호출하는 방식으로 코드를 호출하고, 사용자가 관찰하는 결과를 literal 기대값으로 assert한다. Assertion 강도를 참고한다

## Red-Green-Refactor 사이클

| Step | Action | Verify |
|------|--------|--------|
| 1. User Journey | `As a [role], I want [action], so that [benefit]` | Stakeholder agrees |
| 2. Write Tests | Cases for normal/edge/error/boundary | `npm test` → FAIL, 즉 Red, 의도한 이유로 |
| 3. Implement | Minimal code to pass | `npm test` → PASS, 즉 Green |
| 4. Refactor | Remove dup, improve names, optimize | Tests stay green |
| 5. Verify Coverage | `npm run test:coverage` | ≥ 80% on branches/functions/lines |

## 테스트 타입 매트릭스

| Type | Scope | Speed | Tools |
|------|-------|-------|-------|
| Unit | 순수 함수, 컴포넌트, 헬퍼 | < 50ms | Jest / Vitest + Testing Library |
| Integration | API 라우트, DB ops, 서비스 상호작용 | < 1s | Jest + supertest / NextRequest |
| E2E | 핵심 사용자 플로우, 브라우저 UI | < 30s | Playwright |

## 패턴 스니펫

### 컴포넌트 Unit
```typescript
it('shows the incremented count after a click', () => {
  render(<Counter initial={2} />)
  fireEvent.click(screen.getByRole('button', { name: 'Increment' }))
  expect(screen.getByText('Count: 3')).toBeInTheDocument()
})
```

### API Integration
```typescript
it('returns 400 on invalid query', async () => {
  const req = new NextRequest('http://localhost/api/markets?limit=invalid')
  const res = await GET(req)
  expect(res.status).toBe(400)
})
```

### Playwright E2E
```typescript
test('search returns relevant results', async ({ page }) => {
  await page.goto('/markets')
  await page.fill('input[placeholder="Search markets"]', 'election')
  await page.waitForTimeout(600) // debounce
  await expect(page.locator('[data-testid="market-card"]')).toHaveCount(5, { timeout: 5000 })
})
```

## Assertion 강도

테스트를 남기기 전에 묻는다: **테스트 대상 코드가 `undefined`를 반환해도 이 테스트가 통과하는가?** 그렇다면 아무 동작도 관찰하지 않으므로 결함으로 실패할 수 없다. assertion을 다시 쓰거나 테스트를 삭제한다.

이 검사를 공허하게 통과하는 형태:

| 형태 | 예 | 다시 쓸 방향 |
|------|----|-------------|
| 약한 assertion | `toBeDefined`, `toBeTruthy`, `not.toThrow`, `toBeGreaterThan(0)`뿐 | literal 출력: `expect(slugify('Hello, World!')).toBe('hello-world')` |
| Mock만 확인 | `toHaveBeenCalled` / `toHaveBeenCalledTimes`뿐 | mock이 받은 payload, 즉 `toHaveBeenCalledWith(...)`, 또는 호출 후 상태 |
| 부재만 확인 | `toEqual([])`, `toBeUndefined`, `not.toBe(x)`뿐 | 결과가 비어 있지 않아야 하는 입력에 대한 assertion을 추가 |
| 자기 참조 | `expect(f(a)).toBe(f(a))`, 기대값을 테스트 대상 코드가 만듦 | 손으로 쓴 literal |
| 상수 고정 | `expect(LIMITS.maxTools).toBe(8)`, `expect(DEFAULT_TIMEOUT_MS).toBe(5000)` | 그 상수를 읽는 메커니즘에 입력 하나를 통과시킴 |
| fixture가 fixture를 확인 | `beforeEach`에서 만든 데이터를 assert; 대상은 본문에서 실행되지 않음 | 테스트 본문 안에서 대상을 호출 |

유지: 테이블 행 간 관계 검사, 예를 들어 두 테이블에 같은 키가 존재하는지 확인하는 검사와 `*.test-d.ts`의 컴파일 타임 검사.

## 테스트가 비현실적일 때

**나쁜 테스트보다 새 테스트 없음이 낫다.** 나쁜 테스트란 대부분 mock을 테스트하거나, 구현 세부를 굳히거나, 타이밍·무관한 전역 상태에 의존하거나, 작은 수정에 무거운 인프라가 필요하거나, 수정을 증명한 직후 지워질 테스트다.

가장 싼 실제 테스트가 이 중 하나라면 대상 스크립트, 재현 커맨드, 브라우저 자동화, 로그 assertion 같은 가장 가까운 실행 가능한 검사로 대신하고 그 이유를 완료 보고에 기록한다. 티어가 이 선호보다 우선한다. `do-178c`를 참고한다. 커버리지 게이트가 면제되는 D/E만 대체할 수 있다. C 이상은 아래 커버리지 임계치가 그대로 적용되므로 가장 좁은 실제 테스트를 만든다.

## 파일 레이아웃

```
src/
├── components/Button/Button.test.tsx     # unit (colocated)
├── app/api/markets/route.test.ts         # integration (colocated)
└── e2e/markets.spec.ts                   # E2E (top-level e2e/)
```

## Mocking 체크리스트

- [ ] 외부 API는 모듈 경계에서 mock, 예: `jest.mock('@/lib/...')`
- [ ] mock은 현실적인 shape를 반환, 예: 1536-dim embedding 벡터
- [ ] success path와 failure path 모두 커버
- [ ] unit/integration 테스트에 실제 network/DB 호출 금지
- [ ] 테스트 사이에 mock reset, 예: `beforeEach(jest.clearAllMocks)`

## jest config 커버리지 임계치

```json
{
  "coverageThresholds": {
    "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 }
  }
}
```

**커버리지 계층:**
- 티어 C 이상에 대해 **80% minimum**. D/E는 면제 가능하며, 테스트가 비현실적일 때를 참고한다.
- 금융 계산, 인증, 보안 핵심 경로, 핵심 비즈니스 로직에는 **100% required**.

## 흔한 실수

| Mistake | Result | Fix |
|---------|--------|-----|
| 내부 state 테스트, 예: `component.state.x` | 깨지기 쉽고, 리팩토링이 테스트를 깬다 | 사용자에게 보이는 출력을 테스트, 예: `screen.getByText` |
| CSS 클래스 셀렉터, 예: `.css-xyz` | 스타일 변경에 깨짐 | `data-testid` 또는 semantic role |
| 테스트 간 state 공유 | 순서 의존, flaky | 테스트마다 fresh data 셋업 |
| 한 `it()`에 assert 잔뜩 | 실패 진단이 어려움 | 테스트 하나에 동작 하나 |
| 에러 경로 스킵 | 버그가 프로덕션으로 | throw/reject 분기를 명시적으로 테스트 |
| Mock이 generic shape 반환 | false positive | 실제 schema shape에 맞춤 |
| 테스트에 `console.log` 잔존 | 시끄러운 CI 출력 | 커밋 전에 제거 |
| 구현에 맞추려고 assertion 약화 | 테스트가 버그를 굳힌다 | 의도한 동작이 바뀐 경우에만 테스트를 바꾸고 이유를 남긴다 |

## 작성자 체크리스트

작업을 완료로 표시하기 전에:
- [ ] 테스트를 먼저 작성했고 실패를 확인했다. 의도한 이유로 실패해야 Red가 입증된 것이며, setup·import·오타로 인한 실패는 Red가 아니므로 테스트부터 고친다
- [ ] 모든 assertion이 "테스트 대상이 `undefined` 반환" 검사를 통과한다
- [ ] 구현은 테스트를 수정하지 않고 통과시킨다
- [ ] 엣지 케이스: empty / null / 잘못된 타입 / min-max 경계 / concurrent / partial failure / 대용량 입력 10k+ 항목 / 특수문자 Unicode, 이모지, SQL 메타문자
- [ ] 에러 경로는 단순 "throws"가 아니라 구체적인 assertion으로
- [ ] 변경된 파일에 커버리지 ≥ 80%, 티어 C+
- [ ] `.skip` / `.only` / 비활성 테스트 없음
- [ ] unit 테스트는 합산 < 30s
- [ ] E2E는 critical flow마다 ≥ 1개 success + ≥ 1개 failure 경로
- [ ] 완료 보고에 실패-전 실행과 통과-후 실행, 또는 대체 검사와 그 이유를 명시한다

## 지속적 테스트

```bash
npm test -- --watch              # dev loop
npm test -- --coverage           # CI / pre-commit
npm test && npm run lint         # pre-commit hook
```

## 성공 지표

- 변경 코드에 80%+ 커버리지
- skip/disabled 테스트 0개
- unit suite < 30s
- E2E가 모든 핵심 사용자 플로우 커버
- 테스트가 프로덕션 전에 회귀를 잡아냄

---

**Remember**: 테스트는 선택이 아니다. 자신감 있는 리팩토링과 신뢰성 있는 릴리즈를 위한 안전망이다.
