---
name: tdd-workflow
description: "Test-first development: success and failure cases, red/green/refactor, unit/integration/E2E, coverage. Use when designing tests, adding features, fixing bugs, or refactoring. 테스트 설계, 실패 경로, TDD, 테스트 우선."
---

# Test-Driven Development Workflow

모든 코드가 종합적인 테스트 커버리지와 함께 TDD 원칙을 따르도록 보장한다.

## 활성화 시점

- 새 기능, 버그 수정, 리팩토링
- API 엔드포인트, 컴포넌트
- 런타임 동작이 바뀌는 모든 코드
- 테스트 사례 설계와 리뷰

## 핵심 원칙

1. **코드 전에 테스트**: 실패하는 테스트를 먼저 쓰고, 구현한다
2. **80%+ 커버리지**: unit + integration + E2E 합산
3. **정상과 실패를 함께 고려**: 커버리지 수치만 보지 않고 바뀐 계약과 위험을 기준으로 도달 가능한 오류와 경계 사례를 고른다
4. **구현이 아니라 동작**: 사용자가 호출하는 방식으로 코드를 호출하고, 사용자가 관찰하는 결과를 literal 기대값으로 assert한다. Assertion 강도를 참고한다

## Red-Green-Refactor 사이클

| Step | Action | Verify |
|------|--------|--------|
| 1. 동작 정의 | 호출자와 기대 결과를 식별한다 | 요구사항과 일치한다 |
| 2. Red | 프로젝트의 테스트 러너로 재현 사례를 실행한다 | 의도한 동작 차이로 실패한다 |
| 3. Green | 필요한 만큼 구현하고 같은 사례를 실행한다 | 기대 결과를 얻는다 |
| 4. 리팩토링 | 영향받는 테스트를 실행한다 | 관찰 가능한 동작을 유지한다 |
| 5. 커버리지 | 프로젝트의 커버리지 도구로 측정한다 | 적용할 기준과 실제 수치를 비교한다 |

## 구현 전에 실패 사례를 설계한다

동작이 바뀔 때마다 구현 전에 입력, 의존성, 상태 전이가 어떻게 실패할 수 있는지 살핀다. 해당 흐름에서 실제로 도달할 수 있는 사례를 고른다. 개수를 채우려고 억지 테스트를 추가하지 말고, 적용되지 않는 유형은 이유를 설명한다.

| 상황 | 대표적인 유발 조건 | 관찰할 결과 |
|---|---|---|
| 입력이나 접근 거부 | 빈 값, 잘못된 형식, 경곗값, 권한 부족 | 구체적인 오류와 허용되지 않은 부수 효과의 부재 |
| 의존성 실패 | 읽기나 쓰기 오류, 요청 거부, 잘못된 응답, 타임아웃 | 맥락이 있는 오류가 호출자에게 전달되고 성공으로 표시되지 않음 |
| 작업 중단 | 취소, 닫기, 연결 해제, 의존 대상 소멸 | 약속한 취소 동작, 자원 정리, 최종 상태 |
| 일부만 완료 | 첫 쓰기는 성공하고 두 번째 쓰기는 실패 | 약속한 원자성이나 항목별 결과와 실제 저장 상태 |
| 작업 중복이나 경합 | 중복 제출, 오래된 응답, 동시 수정, 재시도 | 계약에서 정한 순서와 중복 효과 처리 |
| 실패 후 작업 재개 | 실패한 작업을 다시 시도 | 실제 실패 상태에서 데이터 손실이나 추가 효과 없이 복구 |

선택한 사례마다 초기 상태, 실패를 일으킬 조건, 예상 오류나 출력, 실패 후 상태를 정한다. 보존할 데이터, 발생하면 안 되는 효과, 해제할 자원, 약속한 복구 동작을 포함한다. 예외 발생이나 오류 배너만 확인해서는 이 보장을 입증할 수 없다.

데이터 손실을 잡아야 하는 테스트는 기존 데이터가 있는 상태에서 시작한다. 의존성과 만나는 지점에서 실패를 주입하고 테스트 대상인 실제 작업을 호출한다. 대상 작업 자체를 실패하도록 mock하면 mock을 테스트하는 셈이다. 부분 완료는 앞선 효과가 발생한 뒤 실패시킨다. sleep이나 실제 장애에 기대지 말고 시계와 완료 순서를 제어한다.

기대값은 계약에서 도출한다. 롤백, 자동 재시도, 멱등성, 타임아웃이면 아무것도 저장되지 않았다는 가정을 넣지 않는다. 필요한 동작이 정해지지 않았다면 기대값으로 굳히기 전에 결정할 사안으로 알린다. 커버리지 티어와 테스트가 비현실적일 때의 예외를 적용한다.

### 공용 응답 처리

공용 코드의 성공 조건을 추가하거나 강화할 때는 이 조건에 도달하는 기존 경로와 서로 다른 응답을 만드는 제공자 또는 레거시 구현을 추적한다. 응답을 만드는 코드와 직렬화 코드 또는 계약 테스트에서 실제 응답 형식을 확인한다. 클라이언트 타입이나 모든 필드를 채운 fixture는 정상 응답에 포함될 수 있는 `null`, 빈 값, 필드 생략을 가릴 수 있다.

새 조건에서 거부될 수 있는 실제 정상 응답을 대표하는 최소 사례를 테스트한다. 예를 들어 폴링에는 거래 ID가 필요해도 기존 동기 경로는 ID 없이 성공할 수 있다. 이 경우 동기 경로의 성공 처리와 ID가 없을 때 폴링 흐름이 계약에 따라 실패를 처리하는지 확인한다.

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

요구한 동작을 빼거나 틀린 값으로 바꾸면 테스트가 실패하는지 확인한다. 반환값이 없는 API는 호출자가 관찰하는 상태나 효과를 검증한다.

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
- [ ] 실제 작업까지 실패를 주입하고 출력과 실패 후 상태를 검증
- [ ] 단위 테스트는 의존성을 통제한다. DB 계약을 검증하는 통합 테스트는 격리된 데이터베이스를 사용할 수 있다.
- [ ] 테스트 사이에 mock reset, 예: `beforeEach(jest.clearAllMocks)`

## jest config 커버리지 임계치

```json
{
  "coverageThreshold": {
    "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 }
  }
}
```

위 설정은 전체 파일 합계에 적용된다. 변경 파일에 기준이 있으면 변경 목록과 파일별 커버리지 보고서를 대조한다. 실행되지 않은 새 파일도 측정 대상으로 포함하고, 민감 경로의 기준은 해당 파일에서 확인한다.

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
- [ ] 요구한 결과나 효과가 빠지면 테스트가 실패한다
- [ ] 구현은 테스트를 수정하지 않고 통과시킨다
- [ ] [실패 사례 설계](#구현-전에-실패-사례를-설계한다)에 따라 도달 가능한 실패와 경계 사례를 고르고, 적용되지 않는 유형의 이유를 설명했다
- [ ] 실패 사례에서 단순 "throws" 대신 구체적인 오류와 약속한 상태 보존, 정리, 복구를 검증했다
- [ ] 변경된 파일에 커버리지 ≥ 80%, 티어 C+
- [ ] `.skip` / `.only` / 비활성 테스트 없음
- [ ] unit 테스트는 합산 < 30s
- [ ] E2E는 critical flow마다 ≥ 1개 success + ≥ 1개 failure 경로
- [ ] 완료 보고에 실패-전 실행과 통과-후 실행, 또는 대체 검사와 그 이유를 명시한다

## 지속적 테스트

저장소의 테스트 명령을 사용한다. 기본 `unittest` 프로젝트에서는 `python3 -m unittest discover`를 사용할 수 있다. 다음은 npm과 해당 스크립트를 사용하는 프로젝트의 예다.

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
