---
description: Analyze test coverage and generate tests for under-covered files. Targets 80%+ coverage threshold.
allowed-tools: Bash, Read, Write, Edit
model: haiku
effort: low
---

# Test Coverage

Analyze coverage and generate tests for under-covered files, targeting the **80%+ threshold**.

## Invoke

Run tests with coverage (`npm test --coverage` / `pnpm test --coverage`), read `coverage/coverage-summary.json`, and for each file below 80% generate unit / integration / E2E tests. Verify new tests pass and report before/after metrics.

Full TDD standards — Red-Green-Refactor cycle, test-type matrix, mocking checklist, coverage thresholds, common mistakes — live in the `tdd-workflow` skill (`src/skills/tdd-workflow/SKILL.md`). Follow that as the source of truth.
