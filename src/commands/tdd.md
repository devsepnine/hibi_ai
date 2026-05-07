---
description: Enforce test-driven development workflow. Scaffold interfaces, generate tests FIRST, then implement minimal code to pass. Ensure 80%+ coverage.
allowed-tools: Task, Read, Write, Edit, Bash, Grep
model: opus
effort: xhigh
---

# TDD Command

Invokes the **tdd-guide** agent (`~/.claude/agents/tdd-guide.md`) to enforce test-driven development.
References the `tdd-workflow` skill (`~/.claude/skills/tdd-workflow/`) and `rules/testing.md` for shared standards.

## TDD Cycle (MANDATORY)

```
RED → GREEN → REFACTOR → REPEAT
```

- **RED**: Write a failing test. Run it; verify it fails for the *right* reason (e.g., `Not implemented`, not a syntax error).
- **GREEN**: Write the minimum code to pass. Nothing more — no premature abstraction.
- **REFACTOR**: Improve while green. Re-run tests after every change. If tests turn red, undo and try smaller steps.

Never skip RED. Never write production code before its test.

## When to Use

- New features, functions, or components
- Bug fixes — the test must reproduce the bug *before* the fix
- Refactors of existing logic (lock behavior with tests, then change)
- Critical business / financial / auth / security code

## Agent Workflow (applies to every cycle)

The `tdd-guide` agent runs this loop per scenario:

1. **Scaffold** types/interfaces; stub the function to throw `Not implemented`.
2. **Write failing tests** — happy path, edge cases (empty/null/zero/max), error paths.
3. **Run tests** → confirm RED. If GREEN unexpectedly, the test is wrong.
4. **Implement minimal code** → run tests → confirm GREEN.
5. **Refactor** (extract constants, helpers, dedupe) → re-run → still GREEN.
6. **Check coverage**; add tests for any uncovered branch until ≥ 80%.

After the loop, the agent reports: tests added, coverage %, files touched.

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

Then iterate: add tests for liquid market, illiquid market, boundary values → implement → refactor (extract `WEIGHTS`, `clamp()`) → verify all GREEN.

## Test Coverage by Type

- **Unit**: function-level — happy path, empty/null/max, error paths, boundaries
- **Integration**: API endpoints, DB ops, external calls, components+hooks
- **E2E**: critical user flows — use the `/e2e` command

## Coverage Targets

- **80% minimum** for all code (see `rules/testing.md`)
- **100% required** for: financial calculations, authentication, security-critical paths, core business logic

## Best Practices

DO:
- Write test first, run it, confirm RED before any implementation.
- Test behavior (inputs → outputs), not implementation details.
- Cover edge cases and error scenarios in the same RED phase.
- Refactor only when green; revert immediately if a refactor turns red.

DON'T:
- Skip RED. Write code before tests. Mock everything.
- Pack multiple assertions of unrelated behaviors into one `it()`.
- Leave `.skip` / `.only` / commented-out tests in commits.

## Safety Guards

- **Never weaken a test** to force GREEN — fix the implementation instead.
- **Bug fix workflow**: regression test reproducing the bug → confirm RED → fix → confirm GREEN.
- **No flaky tests**: if a test passes intermittently, treat it as failing and fix the root cause (timing, async leaks, shared state).
- **Coverage gate**: if a PR drops coverage below 80%, add tests before merging.

## Related Commands

`/plan` (scope before coding) → `/tdd` (this) → `/build-fix` (if build breaks) → `/code-review` → `/test-coverage` → `/e2e`
