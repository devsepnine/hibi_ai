---
name: tdd-workflow
description: Use this skill when writing new features, fixing bugs, or refactoring code. Enforces test-driven development with 80%+ coverage including unit, integration, and E2E tests. TDD 워크플로우, 테스트 주도 개발, 테스트 우선 작성, 테스트 커버리지.
keywords: [tdd, test, 테스트, 테스트주도개발, 테스트우선, coverage, 커버리지]
---

# Test-Driven Development Workflow

모든 코드가 종합적인 테스트 커버리지와 함께 TDD 원칙을 따르도록 보장한다.

## 활성화 시점

- 새 기능, 버그 수정, 리팩토링
- API 엔드포인트, 컴포넌트
- 런타임 동작이 바뀌는 모든 코드

## 핵심 원칙

1. **코드 전에 테스트** — 실패하는 테스트를 먼저 쓰고, 구현한다
2. **80%+ 커버리지** — unit + integration + E2E 합산
3. **모든 경로 테스트** — happy path, 엣지 케이스, 에러, 경계

## Red-Green-Refactor 사이클

| Step | Action | Verify |
|------|--------|--------|
| 1. User Journey | `As a [role], I want [action], so that [benefit]` | Stakeholder agrees |
| 2. Write Tests | Cases for normal/edge/error/boundary | `npm test` → FAIL (Red) |
| 3. Implement | Minimal code to pass | `npm test` → PASS (Green) |
| 4. Refactor | Remove dup, improve names, optimize | Tests stay green |
| 5. Verify Coverage | `npm run test:coverage` | ≥ 80% on branches/functions/lines |

## 테스트 타입 매트릭스

| Type | Scope | Speed | Tools |
|------|-------|-------|-------|
| Unit | 순수 함수, 컴포넌트, 헬퍼 | < 50ms | Jest / Vitest + Testing Library |
| Integration | API 라우트, DB ops, 서비스 상호작용 | < 1s | Jest + supertest / NextRequest |
| E2E | 핵심 사용자 플로우, 브라우저 UI | < 30s | Playwright |

## 패턴 스니펫

### Unit (component)
```typescript
it('calls onClick when clicked', () => {
  const handleClick = jest.fn()
  render(<Button onClick={handleClick}>Click</Button>)
  fireEvent.click(screen.getByRole('button'))
  expect(handleClick).toHaveBeenCalledTimes(1)
})
```

### Integration (API)
```typescript
it('returns 400 on invalid query', async () => {
  const req = new NextRequest('http://localhost/api/markets?limit=invalid')
  const res = await GET(req)
  expect(res.status).toBe(400)
})
```

### E2E (Playwright)
```typescript
test('search returns relevant results', async ({ page }) => {
  await page.goto('/markets')
  await page.fill('input[placeholder="Search markets"]', 'election')
  await page.waitForTimeout(600) // debounce
  await expect(page.locator('[data-testid="market-card"]')).toHaveCount(5, { timeout: 5000 })
})
```

## 파일 레이아웃

```
src/
├── components/Button/Button.test.tsx     # unit (colocated)
├── app/api/markets/route.test.ts         # integration (colocated)
└── e2e/markets.spec.ts                   # E2E (top-level e2e/)
```

## Mocking 체크리스트

- [ ] 외부 API는 모듈 경계에서 mock (`jest.mock('@/lib/...')`)
- [ ] mock은 현실적인 shape를 반환 (예: 1536-dim embedding 벡터)
- [ ] success path와 failure path 모두 커버
- [ ] unit/integration 테스트에 실제 network/DB 호출 금지
- [ ] 테스트 사이에 mock reset (`beforeEach(jest.clearAllMocks)`)

## 커버리지 임계치 (jest config)

```json
{
  "coverageThresholds": {
    "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 }
  }
}
```

## 흔한 실수

| Mistake | Result | Fix |
|---------|--------|-----|
| 내부 state 테스트 (`component.state.x`) | 깨지기 쉽고, 리팩토링이 테스트를 깬다 | 사용자에게 보이는 출력을 테스트 (`screen.getByText`) |
| CSS 클래스 셀렉터 (`.css-xyz`) | 스타일 변경에 깨짐 | `data-testid` 또는 semantic role |
| 테스트 간 state 공유 | 순서 의존, flaky | 테스트마다 fresh data 셋업 |
| 한 `it()`에 assert 잔뜩 | 실패 진단이 어려움 | 테스트 하나에 동작 하나 |
| 에러 경로 스킵 | 버그가 프로덕션으로 | throw/reject 분기를 명시적으로 테스트 |
| Mock이 generic shape 반환 | false positive | 실제 schema shape에 맞춤 |
| 테스트에 `console.log` 잔존 | 시끄러운 CI 출력 | 커밋 전에 제거 |

## 작성자 체크리스트

작업을 완료로 표시하기 전에:
- [ ] 테스트를 먼저 작성했고 실패를 확인했다 (Red 입증)
- [ ] 구현은 테스트를 수정하지 않고 통과시킨다
- [ ] 엣지 케이스: empty / null / max / concurrent / partial failure
- [ ] 에러 경로는 구체적인 assertion으로 (단순 "throws"가 아니라)
- [ ] 변경된 파일에 커버리지 ≥ 80%
- [ ] `.skip` / `.only` / 비활성 테스트 없음
- [ ] unit 테스트는 합산 < 30s
- [ ] E2E는 critical flow마다 ≥ 1개 success + ≥ 1개 failure 경로

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
