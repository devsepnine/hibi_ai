---
description: Generate and run end-to-end tests with Playwright. Creates test journeys, runs tests, captures screenshots/videos/traces, and uploads artifacts.
allowed-tools: Task, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# E2E Command

**e2e-runner** 에이전트를 디스패치하여 Playwright E2E 테스트를 생성, 유지, 실행한다.

## Invoke

`/e2e <journey description>` (자유 형식). 선택적 플래그:
- `--file <path>` 생성 대신 기존 spec 실행
- `--headed` / `--debug` Playwright로 전달
- `--repeat-each=N` 플레이크 탐지 (flake hunt 시 기본 10)

설명도 `--file`도 없으면 어떤 journey를 테스트할지 사용자에게 확인한다. 이후 `Task(subagent_type="e2e-runner", prompt=<journey + flags + repo context>)`를 디스패치하고 그 보고서를 전달한다.

전체 워크플로우(POM 작성, config, flake 격리, 아티팩트, safety guard, result format)는 `e2e-runner` 에이전트(`src/agents/e2e-runner.md`)에 있다. 이를 source of truth로 따른다.
