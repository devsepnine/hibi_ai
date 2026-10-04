---
description: Enforce test-driven development workflow. Scaffold interfaces, generate tests FIRST, then implement minimal code to pass. Ensure 80%+ coverage.
allowed-tools: Agent, Read, Write, Edit, Bash, Grep
model: opus
effort: xhigh
---

# TDD Command

Enforce test-driven development: write a failing test FIRST, then the minimal code to pass.

## Invoke

For execution, dispatch the **tdd-guide** agent, which runs the Red-Green-Refactor loop per scenario, moving through scaffold → failing test → minimal impl → refactor → coverage check, and reports tests added, coverage %, and files touched.

Full TDD standards, covering cycle definition, test-type matrix, coverage tiers of 80% minimum and 100% for financial/auth/security/core, pattern snippets, mocking checklist, common mistakes, and author checklist, live in the `tdd-workflow` skill at `src/skills/tdd-workflow/SKILL.md`. Follow that as the source of truth.
