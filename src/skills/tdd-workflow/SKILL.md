---
name: tdd-workflow
description: "Test-first development: red/green/refactor, unit/integration/E2E, 80%+ coverage. Use when adding a feature, fixing a bug, or refactoring. TDD, 테스트 주도 개발, 테스트 우선, 커버리지."
---

# Test-Driven Development Workflow

Ensures all code follows TDD principles with comprehensive test coverage.

## When to Activate

- New features, bug fixes, refactoring
- API endpoints, components
- Any code that changes runtime behavior

## Core Principles

1. **Tests BEFORE code**: write failing test first, then implement
2. **80%+ coverage**: unit + integration + E2E combined
3. **All paths tested**: happy path, edge cases, errors, boundaries
4. **Behavior, not implementation**: call the code the way its users do and assert what they observe against a literal expected value, as described in Assertion Strength

## Red-Green-Refactor Cycle

| Step | Action | Verify |
|------|--------|--------|
| 1. User Journey | `As a [role], I want [action], so that [benefit]` | Stakeholder agrees |
| 2. Write Tests | Cases for normal/edge/error/boundary | `npm test` → FAIL as Red, for the intended reason |
| 3. Implement | Minimal code to pass | `npm test` → PASS as Green |
| 4. Refactor | Remove dup, improve names, optimize | Tests stay green |
| 5. Verify Coverage | `npm run test:coverage` | ≥ 80% on branches/functions/lines |

## Test Type Matrix

| Type | Scope | Speed | Tools |
|------|-------|-------|-------|
| Unit | Pure functions, components, helpers | < 50ms | Jest / Vitest + Testing Library |
| Integration | API routes, DB ops, service interactions | < 1s | Jest + supertest / NextRequest |
| E2E | Critical user flows, browser UI | < 30s | Playwright |

## Pattern Snippets

### Unit for components
```typescript
it('shows the incremented count after a click', () => {
  render(<Counter initial={2} />)
  fireEvent.click(screen.getByRole('button', { name: 'Increment' }))
  expect(screen.getByText('Count: 3')).toBeInTheDocument()
})
```

### Integration for APIs
```typescript
it('returns 400 on invalid query', async () => {
  const req = new NextRequest('http://localhost/api/markets?limit=invalid')
  const res = await GET(req)
  expect(res.status).toBe(400)
})
```

### E2E with Playwright
```typescript
test('search returns relevant results', async ({ page }) => {
  await page.goto('/markets')
  await page.fill('input[placeholder="Search markets"]', 'election')
  await page.waitForTimeout(600) // debounce
  await expect(page.locator('[data-testid="market-card"]')).toHaveCount(5, { timeout: 5000 })
})
```

## Assertion Strength

Before keeping a test, ask: **would it still pass if the code under test returned `undefined`?** If yes, it observes no behavior and cannot fail for a defect, so rewrite the assertion or delete the test.

Shapes that pass that check vacuously:

| Shape | Example | Rewrite to |
|-------|---------|-----------|
| Weak assertion | only `toBeDefined`, `toBeTruthy`, `not.toThrow`, `toBeGreaterThan(0)` | the literal output: `expect(slugify('Hello, World!')).toBe('hello-world')` |
| Mock-only | only `toHaveBeenCalled` / `toHaveBeenCalledTimes` | the payload the mock received, via `toHaveBeenCalledWith(...)`, or the state after the call |
| Absence-only | only `toEqual([])`, `toBeUndefined`, `not.toBe(x)` | add an assertion on an input that must produce a non-empty result |
| Self-referential | `expect(f(a)).toBe(f(a))`, expected value built by the code under test | a hand-written literal |
| Constant pin | `expect(LIMITS.maxTools).toBe(8)`, `expect(DEFAULT_TIMEOUT_MS).toBe(5000)` | one input through the mechanism that reads the constant |
| Fixture asserts fixture | asserts data built in `beforeEach`; the subject never runs in the body | call the subject inside the test body |

Keep: relation checks across table rows, such as a key present in two tables, and compile-time checks in `*.test-d.ts`.

## When a Test Is Impractical

**Prefer no new test over a bad test.** A bad test mostly tests mocks, encodes implementation details, depends on timing or unrelated global state, needs heavy infrastructure for a small fix, or would be deleted right after proving the fix.

When the cheapest real test is one of those, use the closest executable check instead, such as a targeted script, a reproduction command, browser automation, or a log assertion, and record why in the completion report. The tier overrides this preference; see `do-178c`. Only D/E, where the coverage gate is waived, may substitute. From C up the coverage thresholds below still hold, so build the narrowest real test instead.

## File Layout

```
src/
├── components/Button/Button.test.tsx     # unit (colocated)
├── app/api/markets/route.test.ts         # integration (colocated)
└── e2e/markets.spec.ts                   # E2E (top-level e2e/)
```

## Mocking Checklist

- [ ] External APIs mocked at module boundary, as in `jest.mock('@/lib/...')`
- [ ] Mock returns realistic shapes, e.g., a 1536-dim embedding vector
- [ ] Both success and failure paths covered
- [ ] No real network/DB calls in unit/integration tests
- [ ] Mocks reset between tests, as in `beforeEach(jest.clearAllMocks)`

## Coverage Threshold in jest config

```json
{
  "coverageThresholds": {
    "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 }
  }
}
```

**Coverage tiers:**
- **80% minimum** for tier C and above. D/E may waive, see When a Test Is Impractical.
- **100% required** for financial calculations, authentication, security-critical paths, and core business logic.

## Common Mistakes

| Mistake | Result | Fix |
|---------|--------|-----|
| Testing internal state, e.g. `component.state.x` | Brittle, refactor breaks tests | Test user-visible output, e.g. `screen.getByText` |
| CSS class selectors such as `.css-xyz` | Breaks on style change | `data-testid` or semantic role |
| Tests share state across `it()` | Order-dependent, flaky | Setup fresh data per test |
| One giant `it()` with many asserts | Hard to diagnose failure | One behavior per test |
| Skipping error paths | Bugs ship to prod | Test throw/reject branches explicitly |
| Mock returns generic shape | False positives | Match real schema shape |
| `console.log` left in tests | Noisy CI output | Remove before commit |
| Weakening an assertion to match the implementation | Test now encodes the bug | Change a test only when the intended behavior changed, and say why |

## Author Checklist

Before marking work complete:
- [ ] Test written FIRST and saw it fail, which proves Red, for the intended reason; a failure from setup, import, or typo is not Red, fix the test first
- [ ] Every assertion survives the "code under test returns `undefined`" check
- [ ] Implementation makes test pass without modifying test
- [ ] Edge cases: empty / null / wrong type / min-max boundary / concurrent / partial failure / large input of 10k+ items / special characters such as Unicode, emoji, SQL metacharacters
- [ ] Error path tested with a specific assertion, not just "throws"
- [ ] Coverage ≥ 80% on changed files, tier C+
- [ ] No `.skip` / `.only` / disabled tests
- [ ] Unit tests run in < 30s total
- [ ] E2E covers ≥ 1 success + ≥ 1 failure path per critical flow
- [ ] Completion report names the failing-before run and the passing-after run, or the substitute check and why

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
