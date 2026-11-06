---
description: Summarize recent git history into a QA handoff document with a plain-language change summary plus an executable QA checklist for non-developers.
argument-hint: "[range]"
allowed-tools: Read, Write, Grep, Glob, Bash
model: sonnet
effort: high
---

# QA Handoff

Thin entry point for handing finished development work to QA, planners, or CS.
Resolve the commit range. `$ARGUMENTS`, when given, is the range, such as `v1.14.0..HEAD`,
a branch name, or "last week". For work just completed, use its known commits.
Otherwise fall back to the skill's default of the last tag reachable from HEAD to HEAD.
Read the diffs, not just commit subjects, and produce one document with two
halves: a summary in product vocabulary, and a checklist each item of which has
setup, reproduction steps, and an expected result. Compare previous and current
behavior, omit internal maintenance, and do not assign checklist priorities.
When an API contract changes, explain which values changed and where the product uses them.

How to invoke: run after a feature or release branch is done, or whenever
someone asks what changed and what should be tested. Ask where the output goes,
either a repo markdown file, conversation only, Jira, or Confluence, unless the user said.

Non-negotiable: never guess an expected result. Changes you cannot classify
from the diff go to developer follow-up. Omit changes with no externally observable
effect, while keeping negative-path checks for security and data integrity.

**The full method and document template live in the `qa-handoff` skill. Follow that as the source of truth.**
