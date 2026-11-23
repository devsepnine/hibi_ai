---
name: tdd-workflow
description: "Test-first development: success and failure cases, red/green/refactor, unit/integration/E2E, coverage. Use when designing tests, adding features, fixing bugs, or refactoring. 테스트 설계, 실패 경로, TDD, 테스트 우선."
---

# Test-Driven Development Workflow

Ensures all code follows TDD principles with comprehensive test coverage.

## When to Activate

- New features, bug fixes, refactoring
- API endpoints, components
- Any code that changes runtime behavior
- Test case design and review

## Core Principles

1. **Tests BEFORE code**: write failing test first, then implement
2. **80%+ coverage**: unit + integration + E2E combined
3. **Success and failure considered together**: select reachable error and boundary cases from the changed contract and risk, not from coverage numbers alone
4. **Behavior, not implementation**: call the code the way its users do and assert what they observe against a literal expected value, as described in Assertion Strength

## Red-Green-Refactor Cycle

| Step | Action | Verify |
|------|--------|--------|
| 1. Describe the behavior | Identify the caller and expected outcome | Matches the requirement |
| 2. Red | Run the failing case with the project's test runner | Fails for the intended behavior |
| 3. Green | Implement and rerun the same case | Observed result matches the expectation |
| 4. Refactor | Run the affected tests | Observable behavior stays the same |
| 5. Verify coverage | Use the project's coverage tool | Compare the measurement with the applicable threshold |

## Design failure cases before code

For each changed behavior, consider how its inputs, dependencies, and state transitions can fail before writing the implementation. Select cases that are reachable in this flow. Explain when a category does not apply instead of adding artificial tests to fill a quota.

| Situation | Representative trigger | What to observe |
|---|---|---|
| Input or access rejected | Empty, malformed, boundary value, missing permission | Specific error and absence of unauthorized effects |
| Dependency fails | Read or write error, rejection, malformed response, timeout | Error reaches the caller with context and no false success |
| Work interrupted | Cancel, close, disconnect, dependency disappears | Promised cancellation behavior, resource cleanup, final state |
| Only part completes | First write succeeds, second write fails | Declared atomicity or per-item results and actual persisted state |
| Work overlaps or repeats | Double submission, stale response, concurrent update, retry | Ordering and duplicate-effect behavior required by the contract |
| Work resumes after failure | Retry after a failed attempt | Recovery from the actual failed state, without lost data or extra effects |

For each selected case, define the initial state, failure trigger, expected error or output, and state after failure. Include the data that must survive, effects that must not occur, resources that must be released, and any promised recovery. A thrown error or error banner alone does not prove those guarantees.

Use existing data in the setup when the test must detect data loss. Inject failure at the dependency boundary and call the real operation under test. Mocking the operation itself to reject tests the mock. For partial work, fail after the earlier effect has occurred. Control clocks and completion order instead of depending on sleeps or a live outage.

Derive expectations from the contract. Do not assume rollback, automatic retry, idempotency, or that a timeout means nothing was committed. If the required behavior is undefined, surface that decision before encoding an expectation. Apply the coverage tier and the exceptions in When a Test Is Impractical.

### Shared response handling

When a change adds or tightens a success check in shared code, trace which existing paths reach the check and which provider or legacy implementation produces each materially different response. Read the producer and serialization code or a contract test to verify the response shape. A client type or a fixture with every field populated can hide a valid null, empty, or omitted field.

Test the smallest set of production-valid response variants the new check could reject. For example, a polling path can require a transaction ID while an existing synchronous path can succeed without one. Assert that the synchronous path still reports success and that polling with no ID follows its failure contract.

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

Check that omitting the required behavior or producing a wrong value would fail the test. For an API that returns no value, assert the state or effect its caller observes.

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
- [ ] Failure injection reaches the real operation and asserts its output and state after failure
- [ ] Unit tests use controlled dependencies. An integration test can use an isolated database when it verifies the database contract.
- [ ] Mocks reset between tests, as in `beforeEach(jest.clearAllMocks)`

## Coverage Threshold in jest config

```json
{
  "coverageThreshold": {
    "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 }
  }
}
```

The global setting checks the aggregate. For a changed file threshold, compare the changed paths with the per-file coverage report. Include unexecuted new files in coverage collection and check sensitive paths against their own threshold.

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
- [ ] The test would fail if the required outcome or effect were missing
- [ ] Implementation makes test pass without modifying test
- [ ] Reachable failure and boundary cases selected using [failure case design](#design-failure-cases-before-code), with inapplicable categories explained
- [ ] Failure cases assert the specific error and promised state preservation, cleanup, and recovery rather than only "throws"
- [ ] Coverage ≥ 80% on changed files, tier C+
- [ ] No `.skip` / `.only` / disabled tests
- [ ] Unit tests run in < 30s total
- [ ] E2E covers ≥ 1 success + ≥ 1 failure path per critical flow
- [ ] Completion report names the failing-before run and the passing-after run, or the substitute check and why

## Continuous Testing

Use the repository's test command. For a Python project using standard unittest discovery, this can be `python3 -m unittest discover`. The examples below apply to npm projects that define the corresponding scripts.

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
