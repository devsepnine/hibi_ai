---
description: Summarize recent git history into a QA handoff document — plain-language change summary plus an executable QA checklist for non-developers.
argument-hint: "[range]"
allowed-tools: Read, Write, Grep, Glob, Bash
model: sonnet
effort: high
---

# QA Handoff

Thin entry point for handing finished development work to QA, planners, or CS.
Resolve the commit range — `$ARGUMENTS`, when given, is the range (`v1.14.0..HEAD`,
a branch name, "last week"); otherwise fall back to the skill's default of the
last tag reachable from HEAD to HEAD. Read the diffs — not just commit subjects — and produce one document with two
halves: a summary in product vocabulary, and a checklist each item of which has
preconditions, steps, and an expected result.

How to invoke: run after a feature or release branch is done, or whenever
someone asks what changed and what should be tested. Ask where the output goes
(repo markdown file / conversation only / Confluence) unless the user said.

Non-negotiable: never guess an expected result. Changes you cannot classify
from the diff go to open questions, and changes nobody can observe from outside
go to internal changes — not into the checklist.

**The full method and document template live in the `qa-handoff` skill — follow that as the source of truth.**
