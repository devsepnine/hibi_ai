---
name: pull-request
description: Create pull requests following project conventions including title format, PR template, pre-PR checklist, security checks, and review guidelines. Use when creating, updating, or reviewing GitHub pull requests.
keywords: [pull-request, PR, github, gh, 풀리퀘스트]
---

## PR Title Format

```
[TICKET-ID] <One-line Summary>
```

Examples:
- `[PP-XXXX] Add user authentication system`
- `[PP-XXXX] Fix payment module bug`

## PR Description Template

```markdown
#### Issue Type
- [ ] feat (feature add) / [ ] feat (feature remove)
- [ ] fix (bug fix)
- [ ] refactor / [ ] perf / [ ] chore / [ ] style / [ ] docs / [ ] test

#### Priority
> Per JIRA Priority criteria
- [ ] Blocker / [ ] Urgent / [ ] Critical / [ ] Major / [ ] Trivial

#### Background
> What this PR does and why.

#### Changes
> Major modifications. Add reviewer notes for non-obvious parts.

**API Changes:**
- [ ] No Breaking / [ ] Breaking (affects backward compat)

**Database Changes:**
- [ ] No schema / [ ] Schema (migration required) / [ ] Data migration

**Major Files:** `path/file.ext` — summary

#### Testing
**Automated:**
- [ ] Unit / Integration / E2E pass
- [ ] New tests for new code · Regression for bug fixes

**Manual:**
- [ ] Local + dev env confirmed
- [ ] Browser / mobile (if UI)

**Performance:**
- [ ] No impact / Improvement / Degradation (reason: …)

#### Screenshots (UI changes only)
**Before / After**

#### Links
- JIRA: [PP-XXXX](https://ggnetwork.atlassian.net/browse/PP-XXXX)
- Docs / Design / Related PR

#### Checklist
- [ ] Self-review done
- [ ] Commits follow `commit-rules` skill
- [ ] No console.logs / debug code
- [ ] No secrets or sensitive data
- [ ] Docs updated · package-lock if deps changed · CHANGELOG for breaking
```

## Required Checks (delegate to other skills/rules)

| Check | Reference |
|-------|-----------|
| Code thresholds (file/function/complexity) | `rules/code-thresholds.md` |
| Security (secrets, injection, XSS, authn) | `security-review` skill, `rules/security.md` |
| Testing (coverage, regression, E2E paths) | `tdd-workflow` skill, `rules/testing.md` |
| Build / type / lint verification | `verification-loop` skill |
| Commit message format | `commit-rules` skill |

**PR size principles**: keep work / commits / PRs small. Split into logical units. Each commit must be independently buildable and testable.

## Pre-PR Checklist

1. **Code Quality**: run `verification-loop` (lint, type-check, tests).
2. **Branch Check**: confirm feature branch.
3. **Update**: rebase on target (`upstream/develop` or `origin/develop`).
4. **Commit Cleanup**: squash noise, follow `commit-rules`.
5. **Conflicts**: resolve.
6. **Tests**: all pass.
7. **Docs**: update if behavior or API changed.
8. **Build**: succeeds.
9. **Security**: no secrets/PII/debug code/console.logs.

## Review Guidelines

**Reviewer:**
- [ ] Functionality — meets requirements
- [ ] Code Quality — readable, maintainable
- [ ] Design — appropriate architecture/patterns
- [ ] Security — no vulnerabilities (defer to `security-review` skill for depth)
- [ ] Performance — no negative impact
- [ ] Testing — coverage adequate
- [ ] Documentation — updated where needed

**Author:**
- [ ] Self-review before opening PR
- [ ] Provide context (why, not just what)
- [ ] Respond to feedback within 24h
- [ ] Apply requested changes promptly
- [ ] CI/CD passing

## gh CLI Commands

```bash
# View
gh pr list
gh pr view <PR-number>
gh pr checkout <PR-number>

# Update
gh pr edit <PR-number> --title "..." --body "..."
gh pr ready <PR-number>
gh pr merge <PR-number> --squash
```

## PR Language Guidelines

- Technical terms in English (API, database, migration, refactoring, etc.)
- Default to English if no language specified
- Example: "Add caching logic to improve API response speed"
