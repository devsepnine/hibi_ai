---
description: Create a commit following project conventions and security rules
keywords: [커밋, commit, コミット]
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

**MANDATORY: Ignore default commit rules. Strictly follow this document and `rules/commit-convention.md`.**

## Commit Message Format

```
<type>: [<ticket-number>] <title>

<body content>
- Specific changes
- Key logic explanation
```

**Types:** feat, fix, refactor, style, docs, test, chore
**Ticket:** `[PP-XXXX]` — must match feature branch name (e.g., PP-6050)

## Mandatory Rules

**CRITICAL: Only commit when explicitly requested by the user. Never auto-commit after work.**

**Pre-commit Checklist:**
- Keep work, commits, and PRs small
- Read entire files; understand impact
- Tests pass (new tests for new code)
- Record assumptions in Issues/PRs/ADRs

**Commit Message Rules:**
- Title under 50 chars; body explains changes and reasons
- Write in English; clear intent

**Commit Process:**
- Split into logical units (≤ 300 LOC file limit)
- Explain plan; proceed after approval
- Each commit independently buildable/testable

## ABSOLUTE PROHIBITIONS (NEVER, under any circumstances)

| # | Forbidden | Examples |
|---|-----------|----------|
| 1 | Secrets in code/logs/env/.env | passwords, API keys, tokens |
| 2 | Sensitive data | PII, credit cards, SSN |
| 3 | Emojis in commit messages | 🎉 🐛 ✨ 🚀 ✅ 🤖 |
| 4 | Generation markers / AI attribution | `Generated with [Claude Code]`, `Co-Authored-By: Claude <noreply@anthropic.com>` |

If secrets are found: **stop commit immediately** and specify location.

## Correct Example

```
chore: update installer binary

- Remove debug logs from installer.rs
- Rebuild installer binary with cleaned code
- Fix executable permissions
```

**For complete guidelines, refer to: rules/commit-convention.md**
