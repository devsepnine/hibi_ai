---
description: Manage eval-driven development workflow. Define evals, check pass@k metrics, generate regression reports.
argument-hint: "[define|check|report|list] [feature-name]"
allowed-tools: Read, Write, Bash, Edit, Grep
model: sonnet
effort: medium
---

# Eval Command

Eval 기반 개발 워크플로우를 관리한다: eval 정의, pass@k 메트릭 확인, 회귀 보고서 생성.

## Usage

`/eval [define|check|report|list] [feature-name]`

`$ARGUMENTS` 서브커맨드:
- `define <name>` - 새로운 eval 정의 생성
- `check <name>` - eval 실행 및 확인
- `report <name>` - 전체 보고서 생성
- `list` - 모든 eval 표시
- `clean` - 오래된 eval 로그 제거 (최근 10회 실행 유지)

전체 워크플로우, 템플릿, 메트릭 형식은 `eval-harness` 스킬에 있다 — 이를 단일 진실 원천으로 따른다.
