---
description: Safely identify and remove dead code with test verification. Runs knip/depcheck/ts-prune, categorizes findings, deletes only after tests pass.
allowed-tools: Bash, Read, Edit, Grep
model: haiku
effort: low
---

# Refactor Clean

안전한 데드 코드 제거를 위한 얇은 진입점이다. `refactor-cleaner` 에이전트를 실행하면 분석(knip / depcheck / ts-prune)을 수행하고, 위험도별로 발견 사항을 분류하며, 테스트 스위트 통과 후에만 삭제한다.

호출 방법: 데드 코드 정리, 미사용 export, 중복 통합 작업을 `refactor-cleaner` 에이전트에 위임한다. 테스트 통과 없이는 아무것도 삭제하지 않으며, 실패 시 롤백한다.

필수 원칙: 테스트를 먼저 실행하지 않고는 절대로 코드를 삭제하지 않는다.

**전체 워크플로우는 `refactor-cleaner` 에이전트를 source of truth로 삼아 따른다.**
