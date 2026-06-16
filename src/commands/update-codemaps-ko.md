---
description: Scan codebase and generate token-lean architecture codemaps. Detects >30% drift, requests user approval before update.
allowed-tools: Read, Write, Bash, Grep, Glob
model: sonnet
effort: medium
---

# Update Codemaps

코드베이스 구조를 스캔하여 토큰 효율적인 아키텍처 codemap을 재생성하며, 30%를 초과하는 drift를 감지하면 큰 업데이트를 적용하기 전에 사용자 승인을 요청한다.

중요한 아키텍처 변경 후 `/update-codemaps`로 호출하거나 `doc-updater` 에이전트에 위임한다.

전체 워크플로우, codemap 형식, drift 승인 규칙은 `doc-updater` 에이전트에 있다 — 이를 단일 진실 원천으로 따른다.
