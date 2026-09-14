---
description: Open, review, or answer comments on a GitHub pull request — title format, body written from the diff, pre-PR checklist, necessity and scope checks
argument-hint: "[PR-number]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: medium
---

# Pull Request

Thin entry point for PR work. Loads the `pull-request` skill. With a PR number in `$ARGUMENTS`, review that PR or work through its review comments; with no argument, prepare the current branch for one.

**Four non-negotiable reminders (always apply, never strip):**

1. **Open or push a PR ONLY when the user explicitly asks** — the same rule that governs commits. Preparing the branch and drafting the description is not permission to run `gh pr create`.
2. **Read the conventions from the repo, not from memory** — resolve the base branch, extract the ticket ID from the branch or history (omit the prefix when there is none), and use the repository's own PR template when it has one.
3. **Ask instead of guessing** — write the body from the diff, not from the branch name; when a change's reason cannot be recovered, ask rather than inventing one. Before applying a review comment, check whether it belongs to this PR; ask the commenter when it is unclear.
4. **No secrets, PII, debug code, or `console.log` in the diff** — run `/security-review` if the change touches auth, input, or secrets.

Before opening: `verification-loop` (lint, type-check, tests) passes, the branch is a feature branch rebased on its target, commits follow `commit-rules`, and docs are updated if behavior or an API changed.

Keep PRs small — split into logical units, each independently buildable and testable.

**Full title format, description template, pre-PR checklist, and review guidelines live in the `pull-request` skill — follow that as the source of truth.**
