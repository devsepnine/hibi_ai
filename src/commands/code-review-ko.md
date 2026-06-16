---
description: Comprehensive security and quality review of uncommitted changes. Checks hardcoded secrets, input validation, injection risks, code style violations.
allowed-tools: Read, Grep, Bash, Glob
model: sonnet
effort: medium
---

# Code Review

커밋되지 않은 변경 사항 검토를 위한 얇은 진입점이다. `code-reviewer` 에이전트를 실행하면 diff(`git diff --name-only HEAD`)에서 보안, 품질, 베스트 프랙티스 이슈를 점검하고, 심각도별(CRITICAL / HIGH / MEDIUM / LOW)로 파일:라인과 수정안을 포함한 보고서를 생성한다.

호출 방법: 코드를 작성하거나 수정한 직후 `code-reviewer` 에이전트에 위임한다. CRITICAL 또는 HIGH 이슈 발견 시 커밋을 차단한다.

필수 원칙: 보안 취약점이 있는 코드는 절대 승인하지 않는다.

**전체 리뷰 기준은 `coding-standards` skill(references/review-checklist.md)을 source of truth로 삼아 따른다.**
