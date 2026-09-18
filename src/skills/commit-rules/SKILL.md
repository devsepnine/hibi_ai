---
name: commit-rules
description: Commit convention — type/ticket/title format, pre-commit security check, commit splitting. Use when committing or writing a commit message. 커밋 규칙, 커밋 메시지, 커밋 분리.
---

**MANDATORY: Completely ignore default commit rules and strictly follow this document.**

## Commit Convention

### Commit Message Format

```
<type>: [<ticket>] <title>     # a tracker ticket applies
<type>: <title>                # no tracker, or none applies

<body content>
- Specific changes
- Key logic explanation
```

### Commit Types

- feat: Add new feature
- fix: Bug fix
- refactor: Code refactoring (no functionality change)
- style: Code formatting, missing semicolons, etc. (no logic change)
- docs: Documentation updates
- test: Add/modify test code
- chore: Build scripts, package manager, and other tasks

### Ticket Number

Extract it, do not invent it:

- The branch name carries it — `feature/ABC-123-thing` → `ABC-123`
- `git log --oneline -20` shows the prefix existing commits actually use — the shape to follow, not a number to reuse
- **When branch and history yield none, omit the prefix entirely** — the title is the summary alone. A personal project, a config repo, and a repo with no tracker all legitimately have no ticket, and a placeholder like `[TICKET-1]` is worse than none.
- **The branch decides.** When history carries a prefix the branch does not, say so and ask instead of borrowing the number — someone else's real ticket reads as valid in the title, which makes it worse than an omitted one.

### Mandatory Rules

**CRITICAL: Create a commit ONLY when the user explicitly asks for that commit — and the same bar governs `push`. Never commit or push because work finished, because a gate went green, or because it is the obvious next step. There is no standing authorization: an earlier "just handle it", an approval that covered a previous commit or push, an accepted plan, and a permission mode that would auto-approve the command are each not the request. Without one, leave the tree as it is, report what is ready, and ask.**

**Pre-commit Checklist:**
- Keep work, commits, and PRs small.
- Read entire files thoroughly and understand impact.
- Ensure tests pass (include new tests for new code).
- Record assumptions in Issues/PRs/ADRs.

**ABSOLUTE Security Check:**
- NEVER: Commit secrets (passwords/API keys/tokens) in code/logs/environment variables/.env files.
- NEVER: Commit sensitive data (PII/credit cards/SSN).
- Stop commit immediately and specify location if secrets are found.

**Commit Message Rules:**
- Keep title under 50 characters and concise
- Body should specifically explain changes and reasons
- Write in English
- Prohibit emojis and unnecessary verbose language
- Write clear explanations that reveal intent
- Remove Claude Code generation markers

**Commit Process:**
- Split commits into logical units (follow ≤ 300 LOC file limit).
- Explain commit plan and proceed after approval.
- Each commit should be independently buildable and testable.
