---
description: Build or refresh docs/ARCHITECTURE.md from the code, settling the gray areas by Q&A and drafting lint rules for the mechanical ones.
argument-hint: "[update]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash, AskUserQuestion
model: opus
effort: high
---

# Architecture Rules

프로젝트의 아키텍처 규칙을 작성하는 얇은 진입점이다. 문서가 없으면 코드가
증명하는 규칙을 추출하고 회색 지대만 묻는다. 문서가 이미 있거나 `$ARGUMENTS`가
`update`면 기록된 SHA부터 각 규칙의 근거를 다시 측정하고 새 반례만 묻는다.

타협 불가: 모든 규칙은 개수와 그것을 측정한 커맨드를 함께 적고, 회색 지대는
추측하지 않고 물으며, lint 설정은 사용자가 동의할 때까지 초안이다.

**전체 방법과 문서 템플릿은 `architecture-rules` 스킬에 있다. 그것을 단일 진실 원천으로 따른다.**
