---
description: Generate and run end-to-end tests with Playwright. Creates test journeys, runs tests, captures screenshots/videos/traces, and uploads artifacts.
allowed-tools: Task, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# E2E Command

Generate, maintain, and run Playwright E2E tests by dispatching the **e2e-runner** agent.

## Invoke

`/e2e <journey description>` (free-form). Optional flags:
- `--file <path>` run an existing spec instead of generating
- `--headed` / `--debug` forwarded to Playwright
- `--repeat-each=N` flake detection (default 10 on a flake hunt)

If no description and no `--file`, ask the user which journey to test. Then dispatch `Task(subagent_type="e2e-runner", prompt=<journey + flags + repo context>)` and relay its report.

Full workflow — POM authoring, config, flake quarantine, artifacts, safety guards, result format — lives in the `e2e-runner` agent (`src/agents/e2e-runner.md`). Follow that as the source of truth.
