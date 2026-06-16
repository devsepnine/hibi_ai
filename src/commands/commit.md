---
description: Create a commit following project conventions and security rules
keywords: [커밋, commit, コミット]
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

Thin entry point for creating a commit. Invoke when the user explicitly asks to commit.

**Three non-negotiable reminders (always apply, never strip):**
1. **NO emojis, NO generation markers** — never add `Co-Authored-By` or "Generated with Claude Code" / AI attribution.
2. **Commit ONLY when the user explicitly asks** — never auto-commit after finishing work.
3. **Format:** `<type>: [<ticket>] <title>` (types: feat, fix, refactor, style, docs, test, chore; ticket `[PP-XXXX]` matches the feature branch).

If secrets are found, **stop the commit immediately** and specify the location.

**The full commit convention lives in the `commit-rules` skill — follow that as the source of truth.**
