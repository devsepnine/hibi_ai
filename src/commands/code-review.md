---
description: Comprehensive security and quality review of uncommitted changes. Checks hardcoded secrets, input validation, injection risks, code style violations.
allowed-tools: Read, Grep, Bash, Glob
model: sonnet
effort: medium
---

# Code Review

Thin entry point for reviewing uncommitted changes. Run the `code-reviewer` agent, which scans the diff (`git diff --name-only HEAD`) for security, quality, and best-practice issues, then reports findings by severity (CRITICAL / HIGH / MEDIUM / LOW) with file:line and a suggested fix.

How to invoke: hand the task to the `code-reviewer` agent right after writing or modifying code. Block the commit if CRITICAL or HIGH issues are found.

Non-negotiable: never approve code with security vulnerabilities.

**Full review criteria live in the `coding-standards` skill (references/review-checklist.md) — follow that as the source of truth.**
