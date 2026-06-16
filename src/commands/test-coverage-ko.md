---
description: Analyze test coverage and generate tests for under-covered files. Targets 80%+ coverage threshold.
allowed-tools: Bash, Read, Write, Edit
model: haiku
effort: low
---

# Test Coverage

테스트 커버리지를 분석하고 커버리지가 부족한 파일에 대한 테스트를 생성하여 **80%+ threshold**를 목표로 한다.

## Invoke

커버리지와 함께 테스트를 실행하고(`npm test --coverage` / `pnpm test --coverage`) `coverage/coverage-summary.json`을 읽어, 80% 미만인 각 파일에 대해 unit / integration / E2E 테스트를 생성한다. 새 테스트 통과를 검증하고 변경 전/후 메트릭을 보고한다.

전체 TDD 표준(Red-Green-Refactor 사이클, test-type matrix, mocking checklist, coverage threshold, common mistakes)은 `tdd-workflow` skill(`src/skills/tdd-workflow/SKILL.md`)에 있다. 이를 source of truth로 따른다.
