---
description: Open, review, or answer comments on a GitHub pull request, covering title format, body written from the diff, pre-PR checklist, necessity and scope checks
argument-hint: "[PR-number]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: medium
---

# Pull Request

Thin entry point for PR work. Loads the `pull-request` skill. With a PR number in `$ARGUMENTS`, review that PR or work through its review comments; with no argument, prepare the current branch for one.

**Four non-negotiable reminders. They always apply and are never stripped:**

1. **Open or push a PR ONLY when the user explicitly asks**: the same rule that governs commits. Preparing the branch and drafting the description is not permission to run `gh pr create`.
2. **Read the conventions from the repo, not from memory**: resolve the base branch, extract the ticket ID from the branch or history and omit the prefix when there is none, and use the repository's own PR template when it has one.
3. **Resolve uncertainty from evidence**. Write the body from the diff. Ask the user when you cannot recover why a change was made. If a review comment's scope is unclear, draft a question for the commenter and send it only on explicit request.
4. **No secrets, PII, debug code, or `console.log` in the diff**: run `/security-review` if the change touches auth, input, or secrets.

Use the pre-PR checks in the `pull-request` skill. Compare the feature branch with the verified target and follow the repository's integration policy. Rewrite history only on explicit request.

Keep PRs small: split into logical units, each independently buildable and testable.

**Full title format, description template, pre-PR checklist, and review guidelines live in the `pull-request` skill. Follow that as the source of truth.**
