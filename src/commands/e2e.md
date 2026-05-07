---
description: Generate and run end-to-end tests with Playwright. Creates test journeys, runs tests, captures screenshots/videos/traces, and uploads artifacts.
allowed-tools: Task, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# E2E Command

Invokes the **e2e-runner** agent (`~/.claude/agents/e2e-runner.md`) to generate, maintain, and execute Playwright E2E tests.

## Argument Parsing

Input form: `/e2e <journey description>` (free-form). Optional flags:
- `--file <path>` run an existing spec instead of generating
- `--headed` / `--debug` forwarded to Playwright
- `--repeat-each=N` flake detection (defaults: 10 if unset on flake hunt)

If no description and no `--file`, ask the user which journey to test.

## Invocation Flow

1. Parse args → journey description + flags.
2. Locate Playwright setup: `playwright.config.ts`, `tests/e2e/`, `tests/pages/`.
3. Dispatch `Task(subagent_type="e2e-runner", prompt=<journey + repo context>)`.
4. Receive generated/updated spec + run results.
5. Verify artifacts exist (HTML report, JUnit XML, screenshots/videos/traces on failure).
6. Format final report (see Result Format below).

## When to Use

- Critical user journeys (login, trading, payments, wallet, search)
- Multi-step flows requiring frontend↔backend integration
- Pre-deployment regression sweeps
- Flaky test triage (`--repeat-each=10`)

## Core Playwright Settings (delegated to agent — do not duplicate)

The agent owns POM authoring, config tuning, and flake quarantine. Reference:
[Playwright config](https://playwright.dev/docs/test-configuration), [POM guide](https://playwright.dev/docs/pom).

Minimum config the agent enforces:

```typescript
use: { baseURL, trace: 'on-first-retry', screenshot: 'only-on-failure', video: 'retain-on-failure' }
retries: process.env.CI ? 2 : 0
reporter: [['html'], ['junit', { outputFile: 'playwright-results.xml' }]]
```

## Test Authoring Rules (must hold for every generated spec)

- [ ] Page Object Model (`tests/pages/*.ts`)
- [ ] `data-testid` locators only — no CSS-class / text-only matching
- [ ] `waitForResponse` / `waitForLoadState` — never `waitForTimeout`
- [ ] `test.skip(process.env.NODE_ENV === 'production')` on financial / trading flows
- [ ] Screenshot at meaningful checkpoints
- [ ] No hardcoded URLs — `baseURL` from config

## Journey Coverage (PMX)

Detailed pattern (one example):

**Market Search & View** — navigate `/markets` → `searchMarkets('election')` → wait `/api/markets/search` 200 → assert `marketCards.count() > 0` → click first → assert URL `/markets/[slug]` + chart visible. Screenshot results + detail.

Other critical journeys (1-line): wallet connect (mock `window.ethereum`), market creation (auth-gated), trade (testnet only, NODE_ENV guard), withdrawal, market resolution. Important: profile updates, real-time price, filters, mobile layout.

## Result Format

```
E2E Test Results
================
Status:    [PASS|FAIL|FLAKY]
Total:     N tests
Passed:    N (P%)
Failed:    N
Flaky:     N
Duration:  Ts

Spec: tests/e2e/<area>/<flow>.spec.ts

Artifacts:
- playwright-report/index.html
- playwright-results.xml (JUnit)
- artifacts/*.png (screenshots)
- test-results/*.webm (videos, on failure)
- test-results/*.zip (traces, on failure)

View report: npx playwright show-report
```

On failure include for each: `file:line`, error message, artifact paths, recommended fix.

On flaky tests: list pass rate per test, root-cause hypothesis, quarantine recommendation (`test.fixme()` + issue link).

## Quick Commands (forwarded to Bash)

```bash
npx playwright test                          # all
npx playwright test <path>                   # single
npx playwright test --headed --debug         # visual debug
npx playwright test --repeat-each=10         # flake detection
npx playwright codegen <url>                 # record actions
npx playwright show-report                   # view HTML
```

## Safety Rules

- NEVER run tests against production
- Trading / financial specs MUST have `test.skip(NODE_ENV === 'production')`
- Use testnet wallets with test funds only
- Do NOT auto-commit generated specs — wait for user review

## Integration

- `/plan` → identify journeys before `/e2e`
- `/tdd` → unit tests (faster, granular)
- `/e2e` → integration / user journeys
- `/code-review` → review generated test quality
