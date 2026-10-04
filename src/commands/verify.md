---
description: Run comprehensive verification covering build, type check, tests, and lint. Reports failures with file:line context.
argument-hint: "[--quick|--full]"
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

# Verification Command

Runs comprehensive verification covering build, types, lint, tests, secrets, and console.log audit on the current codebase state and reports a PASS/FAIL summary.

Invoke with `/verify` when you need to confirm the codebase is sound before a commit or PR.

`$ARGUMENTS` selects scope: `quick` covers build + types, `full` covers all checks and is the default, `pre-commit` covers commit-relevant checks, and `pre-pr` covers full + security scan.

This command runs the proxy phases 1 to 6 and reports `Overall: PARTIAL` at most; `READY` also needs the skill's Phase 7 real-artifact check, which the caller runs at completion.

Full workflow and canonical steps live in the `verification-loop` skill. Follow that as the source of truth.
