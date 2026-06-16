---
description: Enforce test-driven development workflow. Scaffold interfaces, generate tests FIRST, then implement minimal code to pass. Ensure 80%+ coverage.
allowed-tools: Task, Read, Write, Edit, Bash, Grep
model: opus
effort: xhigh
---

# TDD Command

Enforce test-driven development: write a failing test FIRST, then the minimal code to pass.

## Invoke

For execution, dispatch the **tdd-guide** agent, which runs the Red-Green-Refactor loop per scenario (scaffold → failing test → minimal impl → refactor → coverage check) and reports tests added, coverage %, and files touched.

Full TDD standards — cycle definition, test-type matrix, coverage tiers (80% min, 100% for financial/auth/security/core), pattern snippets, mocking checklist, common mistakes, author checklist — live in the `tdd-workflow` skill (`src/skills/tdd-workflow/SKILL.md`). Follow that as the source of truth.
