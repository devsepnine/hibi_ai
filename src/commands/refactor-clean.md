---
description: Safely identify and remove dead code with test verification. Runs knip/depcheck/ts-prune, categorizes findings, deletes only after tests pass.
allowed-tools: Bash, Read, Edit, Grep
model: haiku
effort: low
---

# Refactor Clean

Thin entry point for safe dead-code removal. Run the `refactor-cleaner` agent, which analyzes (knip / depcheck / ts-prune), categorizes findings by risk, and deletes only after the test suite passes.

How to invoke: hand the task to the `refactor-cleaner` agent (e.g. on dead-code cleanup, unused exports, or duplicate consolidation). It deletes nothing without a passing test run and rolls back on failure.

Non-negotiable: never delete code without running tests first.

**Full workflow lives in the `refactor-cleaner` agent — follow that as the source of truth.**
