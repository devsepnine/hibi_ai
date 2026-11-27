---
description: Open, review, or answer comments on a GitHub pull request, covering title format, body written from the diff, pre-PR checklist, necessity and scope checks
argument-hint: "[PR-number]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: medium
---

# Pull Request

PR 작업을 위한 얇은 진입점이다. `pull-request` skill을 로드한다. `$ARGUMENTS`에 PR 번호가 있으면 해당 PR을 리뷰하거나 리뷰 코멘트를 처리하고, 인자가 없으면 현재 브랜치를 PR 준비 상태로 만든다.

**필수 준수 4원칙. 항상 적용하며 절대 제거하지 않는다:**

1. **사용자가 명시적으로 요청한 경우에만 PR을 생성하거나 푸시한다**: 커밋과 동일한 규칙이다. 브랜치를 정리하고 설명을 작성하는 것이 `gh pr create` 실행 권한을 의미하지는 않는다.
2. **규약은 기억이 아니라 저장소에서 읽는다**: base 브랜치를 확인하고, 티켓 ID는 브랜치명이나 히스토리에서 추출하되 없으면 접두사를 생략하며, 저장소에 자체 PR 템플릿이 있으면 그것을 쓴다.
3. **근거를 확인하고 모르는 점은 묻는다**. 본문은 diff를 바탕으로 작성한다. 변경 이유를 확인할 수 없으면 사용자에게 묻는다. 리뷰 코멘트의 범위가 불분명하면 작성자에게 물을 질문을 준비하고, 사용자가 명시적으로 요청할 때 전송한다.
4. **diff에 시크릿, PII, 디버그 코드, `console.log`가 없어야 한다**: 인증·입력·시크릿을 건드리는 변경이면 `/security-review`를 실행한다.

PR을 열기 전에는 `pull-request` 스킬의 검사를 따른다. 피처 브랜치를 확인된 대상 브랜치와 비교하고 저장소의 통합 규칙을 따른다. 이력은 사용자가 명시적으로 요청할 때 다시 쓴다.

PR은 작게 유지한다. 논리적 단위로 나누고, 각 단위가 독립적으로 빌드·테스트 가능해야 한다.

**전체 제목 형식, 설명 템플릿, PR 전 체크리스트, 리뷰 가이드라인은 `pull-request` skill을 source of truth로 삼아 따른다.**
