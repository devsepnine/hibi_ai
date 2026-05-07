---
description: Create a commit following project conventions and security rules
keywords: [커밋, commit, コミット]
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

**MANDATORY: 기본 커밋 규칙을 무시한다. 본 문서 및 `rules/commit-convention.md`를 엄격히 준수한다.**

## Commit Message Format

```
<type>: [<ticket-number>] <title>

<body content>
- Specific changes
- Key logic explanation
```

**Types:** feat, fix, refactor, style, docs, test, chore
**Ticket:** `[PP-XXXX]` — 피처 브랜치명과 일치해야 한다 (예: PP-6050)

## Mandatory Rules

**CRITICAL: 사용자가 명시적으로 요청한 경우에만 커밋한다. 작업 후 자동 커밋은 절대 금지.**

**Pre-commit Checklist:**
- 작업, 커밋, PR을 작게 유지한다
- 파일 전체를 읽고 영향 범위를 이해한다
- 테스트 통과 (신규 코드에는 신규 테스트)
- Issue/PR/ADR에 가정사항을 기록한다

**Commit Message Rules:**
- 제목은 50자 이내; 본문은 변경사항과 이유를 설명한다
- 영어로 작성하며, 의도를 명확히 드러낸다

**Commit Process:**
- 논리적 단위로 분할한다 (≤ 300 LOC 파일 제한)
- 계획을 설명하고 승인 후 진행한다
- 각 커밋은 독립적으로 빌드/테스트 가능해야 한다

## ABSOLUTE PROHIBITIONS (NEVER, under any circumstances)

| # | Forbidden | Examples |
|---|-----------|----------|
| 1 | Secrets in code/logs/env/.env | passwords, API keys, tokens |
| 2 | Sensitive data | PII, credit cards, SSN |
| 3 | Emojis in commit messages | 🎉 🐛 ✨ 🚀 ✅ 🤖 |
| 4 | Generation markers / AI attribution | `Generated with [Claude Code]`, `Co-Authored-By: Claude <noreply@anthropic.com>` |

시크릿 발견 시: **즉시 커밋을 중단하고** 위치를 명시한다.

## Correct Example

```
chore: update installer binary

- Remove debug logs from installer.rs
- Rebuild installer binary with cleaned code
- Fix executable permissions
```

**전체 가이드라인은 다음을 참조한다: rules/commit-convention.md**
