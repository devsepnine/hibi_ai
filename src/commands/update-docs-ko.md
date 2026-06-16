---
description: Update README, CHANGELOG, and project documentation to reflect current state. Reads package.json, .env.example, route definitions.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: medium
---

# Update Project Documentation

프로젝트의 현재 상태를 반영하도록 프로젝트 문서(README, CONTRIB, RUNBOOK, CHANGELOG)를 업데이트하며, `package.json`과 `.env.example` 같은 매니페스트 소스를 기반으로 한다.

중요한 구조 변경이나 의존성 변경 후 `/update-docs`로 호출하거나 `doc-updater` 에이전트에 위임한다.

전체 워크플로우, 문서 형식, 가이드라인은 `doc-updater` 에이전트에 있다 — 이를 단일 진실 원천으로 따른다.
