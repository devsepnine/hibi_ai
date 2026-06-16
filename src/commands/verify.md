---
description: Run comprehensive verification (build, type check, tests, lint). Reports failures with file:line context.
argument-hint: "[--quick|--full]"
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

# Verification Command

Runs comprehensive verification (build, types, lint, tests, secrets, console.log audit) on the current codebase state and reports a PASS/FAIL summary.

Invoke with `/verify` when you need to confirm the codebase is sound before a commit or PR.

`$ARGUMENTS` selects scope: `quick` (build + types), `full` (all checks, default), `pre-commit` (commit-relevant checks), `pre-pr` (full + security scan).

Full workflow and canonical steps live in the `verification-loop` skill — follow that as the source of truth.
