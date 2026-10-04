---
description: Find what a diff could break beyond itself before it merges, and prove the one fact it is safe because of by running real code.
argument-hint: "[file | symbol | PR number]"
allowed-tools: Read, Write, Grep, Glob, Bash, Agent
model: sonnet
effort: high
---

# Blast Radius

Thin entry point for a pre-merge breakage hunt. `$ARGUMENTS` names the target,
which is a file, a symbol, or a PR number; with none, use the uncommitted diff, then
`git diff main...HEAD`.

Non-negotiable: name the one fact the change is safe because of and take it
down the evidence ladder to a script or test that runs the real code, or mark
it `UNPROVEN`. Never invent a caller or an API; do not commit the proof script.

**The full method, evidence ladder, and report format live in the `blast-radius` skill. Follow that as the source of truth.**
