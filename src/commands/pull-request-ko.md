---
description: Open or review a GitHub pull request per project convention — title format, description template, pre-PR checklist, reviewer guidance
argument-hint: "[PR-number]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: medium
---

# Pull Request

PR 작업을 위한 얇은 진입점이다. `pull-request` skill을 로드한다. `$ARGUMENTS`에 PR 번호가 있으면 해당 PR을 리뷰하고, 인자가 없으면 현재 브랜치를 PR 준비 상태로 만든다.

**필수 준수 3원칙 (항상 적용, 절대 제거 금지):**

1. **사용자가 명시적으로 요청한 경우에만 PR을 생성하거나 푸시한다** — 커밋과 동일한 규칙이다. 브랜치를 정리하고 설명을 작성하는 것이 `gh pr create` 실행 권한을 의미하지는 않는다.
2. **제목 형식:** `[TICKET-ID] <One-line Summary>`.
3. **diff에 시크릿, PII, 디버그 코드, `console.log`가 없어야 한다** — 인증·입력·시크릿을 건드리는 변경이면 `/security-review`를 실행한다.

PR 생성 전 조건: `verification-loop`(lint, type-check, 테스트) 통과, 대상 브랜치에 리베이스된 피처 브랜치, `commit-rules`를 따르는 커밋, 동작이나 API가 바뀐 경우 문서 갱신.

PR은 작게 유지한다 — 논리적 단위로 나누고, 각 단위가 독립적으로 빌드·테스트 가능해야 한다.

**전체 제목 형식, 설명 템플릿, PR 전 체크리스트, 리뷰 가이드라인은 `pull-request` skill을 source of truth로 삼아 따른다.**
