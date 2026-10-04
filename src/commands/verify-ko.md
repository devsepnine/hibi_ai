---
description: Run comprehensive verification covering build, type check, tests, and lint. Reports failures with file:line context.
argument-hint: "[--quick|--full]"
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

# Verification Command

현재 코드베이스 상태에 대해 빌드, 타입, 린트, 테스트, 시크릿, console.log 감사를 포함한 종합 검증을 실행하고 PASS/FAIL 요약을 보고한다.

커밋이나 PR 전에 코드베이스가 정상인지 확인할 때 `/verify`로 호출한다.

`$ARGUMENTS`로 범위를 선택한다: `quick`은 빌드 + 타입, `full`은 전체 검사이며 기본값, `pre-commit`은 커밋 관련 검사, `pre-pr`은 전체 + 보안 스캔이다.

이 커맨드는 대리 지표 phase 1부터 6까지를 실행하며 최대 `Overall: PARTIAL`까지 보고한다. `READY`에는 스킬의 Phase 7 실제 결과물 확인도 필요하고, 그것은 호출한 쪽이 완료 시점에 실행한다.

전체 워크플로우와 표준 단계는 `verification-loop` 스킬에 있다. 이를 단일 진실 원천으로 따른다.
