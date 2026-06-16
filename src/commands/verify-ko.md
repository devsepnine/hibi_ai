---
description: Run comprehensive verification (build, type check, tests, lint). Reports failures with file:line context.
argument-hint: "[--quick|--full]"
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

# Verification Command

현재 코드베이스 상태에 대해 종합 검증(빌드, 타입, 린트, 테스트, 시크릿, console.log 감사)을 실행하고 PASS/FAIL 요약을 보고한다.

커밋이나 PR 전에 코드베이스가 정상인지 확인할 때 `/verify`로 호출한다.

`$ARGUMENTS`로 범위를 선택한다: `quick`(빌드 + 타입), `full`(전체 검사, 기본값), `pre-commit`(커밋 관련 검사), `pre-pr`(전체 + 보안 스캔).

전체 워크플로우와 표준 단계는 `verification-loop` 스킬에 있다 — 이를 단일 진실 원천으로 따른다.
