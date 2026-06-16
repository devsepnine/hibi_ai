---
description: Create a commit following project conventions and security rules
keywords: [커밋, commit, コミット]
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

커밋 생성을 위한 얇은 진입점이다. 사용자가 명시적으로 커밋을 요청할 때 호출한다.

**필수 준수 3원칙 (항상 적용, 절대 제거 금지):**
1. **이모지 금지, 생성 마커 금지** — `Co-Authored-By`나 "Generated with Claude Code" 등 AI 귀속 표기를 절대 추가하지 않는다.
2. **사용자가 명시적으로 요청한 경우에만 커밋** — 작업 완료 후 자동 커밋은 절대 금지.
3. **형식:** `<type>: [<ticket>] <title>` (types: feat, fix, refactor, style, docs, test, chore; ticket `[PP-XXXX]`는 피처 브랜치명과 일치).

시크릿 발견 시 **즉시 커밋을 중단하고** 위치를 명시한다.

**전체 커밋 컨벤션은 `commit-rules` skill을 source of truth로 삼아 따른다.**
