---
description: Open or review a GitHub pull request per project convention — title format, description template, pre-PR checklist, reviewer guidance
argument-hint: "[PR-number]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: medium
---

# Pull Request

Thin entry point for PR work. Loads the `pull-request` skill. With a PR number in `$ARGUMENTS`, review that PR; with no argument, prepare the current branch for one.

**Three non-negotiable reminders (always apply, never strip):**

1. **Open or push a PR ONLY when the user explicitly asks** — the same rule that governs commits. Preparing the branch and drafting the description is not permission to run `gh pr create`.
2. **Title format:** `[TICKET-ID] <One-line Summary>`.
3. **No secrets, PII, debug code, or `console.log` in the diff** — run `/security-review` if the change touches auth, input, or secrets.

Before opening: `verification-loop` (lint, type-check, tests) passes, the branch is a feature branch rebased on its target, commits follow `commit-rules`, and docs are updated if behavior or an API changed.

Keep PRs small — split into logical units, each independently buildable and testable.

**Full title format, description template, pre-PR checklist, and review guidelines live in the `pull-request` skill — follow that as the source of truth.**
