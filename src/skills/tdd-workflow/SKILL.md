---
name: tdd-workflow
description: Use this skill when writing new features, fixing bugs, or refactoring code. Enforces test-driven development with 80%+ coverage including unit, integration, and E2E tests. TDD 워크플로우, 테스트 주도 개발, 테스트 우선 작성, 테스트 커버리지.
keywords: [tdd, test, 테스트, 테스트주도개발, 테스트우선, coverage, 커버리지]
---

# Test-Driven Development Workflow

Ensures all code follows TDD principles with comprehensive test coverage.

## When to Activate

- New features, bug fixes, refactoring
- API endpoints, components
- Any code that changes runtime behavior

## Core Principles

1. **Tests BEFORE code** — write failing test first, then implement
2. **80%+ coverage** — unit + integration + E2E combined
3. **All paths tested** — happy path, edge cases, errors, boundaries

## Red-Green-Refactor Cycle

| Step | Action | Verify |
|------|--------|--------|
| 1. User Journey | `As a [role], I want [action], so that [benefit]` | Stakeholder agrees |
| 2. Write Tests | Cases for normal/edge/error/boundary | `npm test` → FAIL (Red) |
| 3. Implement | Minimal code to pass | `npm test` → PASS (Green) |
| 4. Refactor | Remove dup, improve names, optimize | Tests stay green |
| 5. Verify Coverage | `npm run test:coverage` | ≥ 80% on branches/functions/lines |

## Test Type Matrix

| Type | Scope | Speed | Tools |
|------|-------|-------|-------|
| Unit | Pure functions, components, helpers | < 50ms | Jest / Vitest + Testing Library |
| Integration | API routes, DB ops, service interactions | < 1s | Jest + supertest / NextRequest |
| E2E | Critical user flows, browser UI | < 30s | Playwright |

## Pattern Snippets

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

## File Layout

```
src/
├── components/Button/Button.test.tsx     # unit (colocated)
├── app/api/markets/route.test.ts         # integration (colocated)
└── e2e/markets.spec.ts                   # E2E (top-level e2e/)
```

## Mocking Checklist

- [ ] External APIs mocked at module boundary (`jest.mock('@/lib/...')`)
- [ ] Mock returns realistic shapes (e.g., 1536-dim embedding vector)
- [ ] Both success and failure paths covered
- [ ] No real network/DB calls in unit/integration tests
- [ ] Mocks reset between tests (`beforeEach(jest.clearAllMocks)`)

## Coverage Threshold (jest config)

```json
{
  "coverageThresholds": {
    "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 }
  }
}
```

**Coverage tiers:**
- **80% minimum** for all code.
- **100% required** for financial calculations, authentication, security-critical paths, and core business logic.

## Common Mistakes

| Mistake | Result | Fix |
|---------|--------|-----|
| Testing internal state (`component.state.x`) | Brittle, refactor breaks tests | Test user-visible output (`screen.getByText`) |
| CSS class selectors (`.css-xyz`) | Breaks on style change | `data-testid` or semantic role |
| Tests share state across `it()` | Order-dependent, flaky | Setup fresh data per test |
| One giant `it()` with many asserts | Hard to diagnose failure | One behavior per test |
| Skipping error paths | Bugs ship to prod | Test throw/reject branches explicitly |
| Mock returns generic shape | False positives | Match real schema shape |
| `console.log` left in tests | Noisy CI output | Remove before commit |

## Author Checklist

Before marking work complete:
- [ ] Test written FIRST and saw it fail (Red proven)
- [ ] Implementation makes test pass without modifying test
- [ ] Edge cases: empty / null / max / concurrent / partial failure
- [ ] Error path tested with specific assertion (not just "throws")
- [ ] Coverage ≥ 80% on changed files
- [ ] No `.skip` / `.only` / disabled tests
- [ ] Unit tests run in < 30s total
- [ ] E2E covers ≥ 1 success + ≥ 1 failure path per critical flow

## Continuous Testing

```bash
npm test -- --watch              # dev loop
npm test -- --coverage           # CI / pre-commit
npm test && npm run lint         # pre-commit hook
```

## Success Metrics

- 80%+ coverage on changed code
- Zero skipped/disabled tests
- Unit suite < 30s
- E2E covers all critical user flows
- Tests catch regressions before production

---

**Remember**: Tests are not optional. They are the safety net for confident refactoring and reliable releases.
