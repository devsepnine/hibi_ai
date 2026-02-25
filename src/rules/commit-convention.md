# Commit Convention

## Commit Message Format

```
<type>: [<ticket-number>] <title>

<body content>
- Specific changes
- Key logic explanation
```

## Commit Types

- feat: Add new feature
- fix: Bug fix
- refactor: Code refactoring (no functionality change)
- style: Code formatting, missing semicolons, etc. (no logic change)
- docs: Documentation updates
- test: Add/modify test code
- chore: Build scripts, package manager, and other tasks

## Ticket Number Format

- `[PP-XXXX]`: Project ticket number (e.g., PP-6050)
- Ticket number can be found in branch name
- Must match feature branch name

## Mandatory Rules

**CRITICAL: Only create commits when explicitly requested by the user. Never automatically commit after completing work unless the user specifically asks for it.**

### Pre-commit Checklist

- Keep work, commits, and PRs small.
- Read entire files thoroughly and understand impact.
- Ensure tests pass (include new tests for new code).
- Record assumptions in Issues/PRs/ADRs.

### ABSOLUTE Security Check

- NEVER: Commit secrets (passwords/API keys/tokens) in code/logs/environment variables/.env files.
- NEVER: Commit sensitive data (PII/credit cards/SSN).
- Stop commit immediately and specify location if secrets are found.

### Commit Message Rules

- Keep title under 50 characters and concise
- Body should specifically explain changes and reasons
- Write in English
- **ABSOLUTE PROHIBITION: NO emojis (🤖, ✅, etc.)**
- **ABSOLUTE PROHIBITION: NO generation markers (Co-Authored-By, Generated with Claude Code, etc.)**
- Write clear explanations that reveal intent

### Commit Process

- Split commits into logical units (follow ≤ 300 LOC file limit).
- Explain commit plan and proceed after approval.
- Each commit should be independently buildable and testable.

## FORBIDDEN PATTERNS

**NEVER add these to commit messages:**

```
❌ 🤖 Generated with [Claude Code](https://claude.com/claude-code)
❌ Co-Authored-By: Claude <noreply@anthropic.com>
❌ Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
❌ Any emojis (🎉, 🐛, ✨, etc.)
❌ Any generation markers or AI attribution
```

**Correct format:**

```
✅ chore: update installer binary

- Remove debug logs from installer.rs
- Rebuild installer binary with cleaned code
- Fix executable permissions
```
