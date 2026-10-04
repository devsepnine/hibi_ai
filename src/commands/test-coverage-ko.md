---
description: Analyze test coverage and generate tests for under-covered files. Targets 80%+ coverage threshold.
allowed-tools: Bash, Read, Write, Edit
model: haiku
effort: low
---

# Test Coverage

테스트 커버리지를 분석하고 커버리지가 부족한 파일에 대한 테스트를 생성하여 **80%+ threshold**를 목표로 한다.

## Invoke

`npm test --coverage` 또는 `pnpm test --coverage`로 커버리지와 함께 테스트를 실행하고 `coverage/coverage-summary.json`을 읽어, 80% 미만인 각 파일에 대해 unit / integration / E2E 테스트를 생성한다. 생성한 모든 테스트는 skill의 Assertion 강도 검사를 통과해야 한다. 숫자를 올리려고만 쓴 테스트는 skill이 금지하는 나쁜 테스트다. 새 테스트 통과를 검증하고 변경 전/후 메트릭을 보고한다.

전체 TDD 표준은 Red-Green-Refactor 사이클, test-type matrix, mocking checklist, coverage threshold, common mistakes를 포함하며 `tdd-workflow` skill의 `src/skills/tdd-workflow/SKILL.md`에 있다. 이를 source of truth로 따른다.
