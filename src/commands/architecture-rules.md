---
description: Build or refresh docs/ARCHITECTURE.md from the code, settling the gray areas by Q&A and drafting lint rules for the mechanical ones.
argument-hint: "[update]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash, AskUserQuestion
model: opus
effort: high
---

# Architecture Rules

Thin entry point for writing the project's architecture rules. With no
document, mine the rules the code proves and ask only about the gray areas;
with an existing document, or when `$ARGUMENTS` is `update`, rerun each rule's
evidence from the recorded SHA and ask about new counter-examples.

Non-negotiable: every rule carries its count and the command that measured it,
every gray area is asked rather than guessed, and lint configs are drafts until
the user agrees.

**The full method and document template live in the `architecture-rules` skill. Follow that as the source of truth.**
