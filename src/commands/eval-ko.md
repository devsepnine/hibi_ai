---
description: Manage eval-driven development workflow. Define evals, check pass@k metrics, generate regression reports.
argument-hint: "[define|check|report|list] [feature-name]"
allowed-tools: Read, Write, Bash, Edit, Grep
model: sonnet
effort: medium
---

# Eval Command

Eval 기반 개발 워크플로우를 관리한다.

## Usage

`/eval [define|check|report|list] [feature-name]`

## Define Evals

`/eval define feature-name`

새로운 eval 정의를 생성한다:

1. 다음 템플릿으로 `.claude/evals/feature-name.md`를 생성한다:

```markdown
## EVAL: feature-name
Created: $(date)

### Capability Evals
- [ ] [Description of capability 1]
- [ ] [Description of capability 2]

### Regression Evals
- [ ] [Existing behavior 1 still works]
- [ ] [Existing behavior 2 still works]

### Success Criteria
- pass@3 > 90% for capability evals
- pass^3 = 100% for regression evals
```

2. 사용자에게 구체적인 기준을 채워달라고 요청한다

## Check Evals

`/eval check feature-name`

피처에 대한 eval을 실행한다:

1. `.claude/evals/feature-name.md`에서 eval 정의를 읽는다
2. 각 capability eval에 대해:
   - 기준 검증을 시도한다
   - PASS/FAIL을 기록한다
   - `.claude/evals/feature-name.log`에 시도를 로깅한다
3. 각 regression eval에 대해:
   - 관련 테스트를 실행한다
   - 베이스라인과 비교한다
   - PASS/FAIL을 기록한다
4. 현재 상태 보고:

```
EVAL CHECK: feature-name
========================
Capability: X/Y passing
Regression: X/Y passing
Status: IN PROGRESS / READY
```

## Report Evals

`/eval report feature-name`

종합 eval 보고서를 생성한다:

```
EVAL REPORT: feature-name
=========================
Generated: $(date)

CAPABILITY EVALS
----------------
[eval-1]: PASS (pass@1)
[eval-2]: PASS (pass@2) - required retry
[eval-3]: FAIL - see notes

REGRESSION EVALS
----------------
[test-1]: PASS
[test-2]: PASS
[test-3]: PASS

METRICS
-------
Capability pass@1: 67%
Capability pass@3: 100%
Regression pass^3: 100%

NOTES
-----
[Any issues, edge cases, or observations]

RECOMMENDATION
--------------
[SHIP / NEEDS WORK / BLOCKED]
```

## List Evals

`/eval list`

모든 eval 정의를 표시한다:

```
EVAL DEFINITIONS
================
feature-auth      [3/5 passing] IN PROGRESS
feature-search    [5/5 passing] READY
feature-export    [0/4 passing] NOT STARTED
```

## Arguments

$ARGUMENTS:
- `define <name>` - 새로운 eval 정의 생성
- `check <name>` - eval 실행 및 확인
- `report <name>` - 전체 보고서 생성
- `list` - 모든 eval 표시
- `clean` - 오래된 eval 로그 제거 (최근 10회 실행 유지)
