---
description: Create a commit following project conventions and security rules
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

Thin entry point for creating a commit. Invoke when the user explicitly asks to commit.

**Three non-negotiable reminders. They always apply and are never stripped:**
1. **NO emojis, NO generation markers**. Never add `Co-Authored-By` or "Generated with Claude Code" / AI attribution.
2. **Commit ONLY when the user explicitly asks**. Never auto-commit after finishing work.
3. **Format:** `<type>: [<ticket>] <title>`. The types are feat, fix, refactor, style, docs, test, chore. Extract the ticket from the branch and history; **omit the prefix entirely when neither yields one**. Never invent a placeholder.

If secrets are found, **stop the commit immediately** and specify the location.

**The full commit convention lives in the `commit-rules` skill. Follow that as the source of truth.**
