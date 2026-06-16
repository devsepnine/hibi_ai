---
description: Sequential multi-agent workflow for complex tasks. Coordinates the built-in Plan agent, tdd-guide, and code-reviewer in handoff chain.
argument-hint: "[workflow-type] [task-description]"
allowed-tools: Task, Read, Bash, Grep
model: opus
effort: xhigh
---

# Orchestrate Command

순차 에이전트 워크플로우. 사용법: `/orchestrate [workflow-type] [task-description]`

## Workflow Types

| Type | Agent Chain | Use Case |
|------|-------------|----------|
| `feature` | the built-in `Plan` agent -> tdd-guide -> code-reviewer | 전체 피처 빌드 |
| `bugfix` | the built-in `Explore` agent -> tdd-guide -> code-reviewer | 버그 조사 + 수정 |
| `refactor` | architect -> code-reviewer -> tdd-guide | 안전한 리팩토링 |
| `security` | code-reviewer -> architect | 보안 감사 |
| `custom` | user-defined CSV list | 임시 시퀀스 |

## Execution Loop

체인의 각 에이전트에 대해:
1. 이전 핸드오프를 컨텍스트로 호출한다
2. 출력을 구조화된 핸드오프로 수집한다
3. 다음 에이전트로 전달한다
4. 최종 보고서로 집계한다

## Handoff Format

```markdown
## HANDOFF: [prev-agent] -> [next-agent]
### Context        — what was done
### Findings       — discoveries / decisions
### Files Modified — list of touched files
### Open Questions — unresolved items
### Recommendations — suggested next steps
```

## Agent Responsibilities

| Agent | Reads | Produces |
|-------|-------|----------|
| the built-in `Plan` agent | requirements | 구현 계획, 의존성, 위험 |
| architect | requirements / plan | 설계 결정, 구조 |
| the built-in `Explore` agent | bug report | 재현 단계, 근본 원인 |
| tdd-guide | plan / handoff | 테스트 우선, 그 다음 최소 구현 |
| code-reviewer | impl | 품질 이슈, 제안, 보안 취약점 스캔 (심층 OWASP/CWE 분석은 `security-review` skill 에 위임) |

## Final Report

```
ORCHESTRATION REPORT
Workflow: <type> | Task: <desc>
Chain: <agent -> agent -> ...>

SUMMARY        — one paragraph
AGENT OUTPUTS  — per-agent summary
FILES CHANGED  — list
TEST RESULTS   — pass/fail
SECURITY       — findings
RECOMMENDATION — SHIP / NEEDS WORK / BLOCKED
```

## Parallel Phase

독립적인 검사의 경우 동시에 fan out 후 병합한다:
- code-reviewer (quality + security) + architect (design) -> 단일 병합 보고서

## Examples

```
/orchestrate feature "Add user authentication"
/orchestrate bugfix "Login fails on empty email"
/orchestrate custom "architect,tdd-guide,code-reviewer" "Redesign caching layer"
```

## Tips

- 복잡한 피처는 built-in `Plan` 에이전트로 시작; 설계 중심 작업은 `architect`로 시작한다
- 머지 전 항상 `code-reviewer`를 포함한다
- auth, payment, PII 경로에는 `code-reviewer`를 사용한다 (`security-review` skill 을 통해 보안 검토를 담당)
- 핸드오프는 간결하게 유지한다 — 다음 에이전트가 필요한 것만
- 위험한 전환 사이에는 검증(빌드/테스트)을 실행한다
