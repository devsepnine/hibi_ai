---
description: Analyze test coverage and generate tests for under-covered files. Targets 80%+ coverage threshold.
allowed-tools: Bash, Read, Write, Edit
model: haiku
effort: low
---

# Test Coverage

Analyze coverage and generate tests for under-covered files, targeting the **80%+ threshold**.

## Invoke

Run tests with coverage with `npm test --coverage` or `pnpm test --coverage`, read `coverage/coverage-summary.json`, and for each file below 80% generate unit / integration / E2E tests. Every generated test must pass the skill's Assertion Strength check. A test written only to raise the number is the bad test the skill forbids. Verify new tests pass and report before/after metrics.

Full TDD standards, covering Red-Green-Refactor cycle, test-type matrix, mocking checklist, coverage thresholds, and common mistakes, live in the `tdd-workflow` skill at `src/skills/tdd-workflow/SKILL.md`. Follow that as the source of truth.
