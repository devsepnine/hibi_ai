---
description: Build or refresh docs/FEATURES.md — a feature map indexed by screen region, user words, and literal UI strings, so a vague report lands on the right files.
argument-hint: "[update]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash
model: sonnet
effort: high
---

# Feature Map

Thin entry point for writing the project's symptom-to-code map. With no map,
build one from the user-facing surface inward; with an existing map, or when
`$ARGUMENTS` is `update`, diff from the recorded SHA and re-verify only the
entries whose files changed.

How to invoke: run once to bootstrap, then after features ship or UI copy
changes. The output is `docs/FEATURES.md` (split into `docs/features/` when it
grows); propose a pointer line in `CLAUDE.md` / `AGENTS.md` and add it only when
the user agrees.

Non-negotiable: every path comes from a file read in this run, every literal UI
string greps in the source, and every pitfall cites its evidence — no guesses.

**The full method and document template live in the `feature-map` skill — follow that as the source of truth.**
