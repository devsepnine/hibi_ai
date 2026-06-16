---
description: Scan codebase and generate token-lean architecture codemaps. Detects >30% drift, requests user approval before update.
allowed-tools: Read, Write, Bash, Grep, Glob
model: sonnet
effort: medium
---

# Update Codemaps

Scans the codebase structure and regenerates token-lean architecture codemaps, detecting >30% drift and requesting user approval before applying large updates.

Invoke with `/update-codemaps` after notable architectural changes, or delegate to the `doc-updater` agent.

Full workflow, codemap formats, and drift-approval rules live in the `doc-updater` agent — follow that as the source of truth.
