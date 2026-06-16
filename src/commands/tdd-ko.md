---
description: Enforce test-driven development workflow. Scaffold interfaces, generate tests FIRST, then implement minimal code to pass. Ensure 80%+ coverage.
allowed-tools: Task, Read, Write, Edit, Bash, Grep
model: opus
effort: xhigh
---

# TDD Command

테스트 주도 개발을 강제한다: 실패하는 테스트를 FIRST 작성하고, 그 후 통과시킬 최소한의 코드를 작성한다.

## Invoke

실행을 위해 **tdd-guide** 에이전트를 디스패치한다. 에이전트는 시나리오마다 Red-Green-Refactor 루프(scaffold → failing test → minimal impl → refactor → coverage check)를 실행하고, 추가된 테스트 / 커버리지 % / 변경된 파일을 보고한다.

전체 TDD 표준(cycle 정의, test-type matrix, 커버리지 계층(80% 최소 / 금융·인증·보안·핵심 로직 100%), pattern snippet, mocking checklist, common mistakes, author checklist)은 `tdd-workflow` skill(`src/skills/tdd-workflow/SKILL.md`)에 있다. 이를 source of truth로 따른다.
