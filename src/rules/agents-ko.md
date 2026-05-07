# Agent Orchestration

## 사용 가능한 에이전트

`~/.claude/agents/`에 위치한다. effort와 model 권장사항은 Anthropic Opus 4.7 가이드(`performance.md` 참조)를 따른다.

| Agent | Purpose | When to Use | Effort | Model |
|-------|---------|-------------|--------|-------|
| planner | 구현 계획 수립 | 복잡한 기능, 리팩토링 | high | sonnet-4-6 |
| architect | 시스템 설계 | 아키텍처 의사결정 | xhigh | opus-4-7 |
| tdd-guide | 테스트 주도 개발 | 신규 기능, 버그 수정 | medium | sonnet-4-6 |
| code-reviewer | 코드 리뷰 | 코드 작성 직후 | medium | sonnet-4-6 |
| security-reviewer | 보안 분석 | 커밋 전 | xhigh | opus-4-7 |
| build-error-resolver | 빌드 에러 수정 | 빌드 실패 시 | medium | sonnet-4-6 |
| e2e-runner | E2E 테스트 | 핵심 사용자 플로우 | xhigh | sonnet-4-6 |
| refactor-cleaner | 데드 코드 정리 | 코드 유지보수 | xhigh | sonnet-4-6 |
| doc-updater | 문서화 | 문서 업데이트 | xhigh | sonnet-4-6 |

## Effort 정책 (서브에이전트)

서브에이전트는 독립적인 워커이다. 비용과 지연을 낮게 유지하기 위해 **기본적으로 명시적인 체크리스트와 함께 `low` 또는 `medium` effort를 사용**해야 한다. 서브에이전트가 에이전틱 탐색(다단계 검색, 반복적인 도구 호출)을 수행해야 할 때만 `xhigh`로 올린다.

Anthropic 가이드의 핵심 문구: *"Lower effort is the best fit for subagents."* 다중 섹션 작업에는 `low`와 명시적인 체크리스트를 함께 쓴다.

## 즉시 에이전트 사용 (사용자 프롬프트 불필요)

1. 복잡한 기능 요청 → **planner** 에이전트
2. 코드 작성/수정 직후 → **code-reviewer** 에이전트
3. 버그 수정 또는 신규 기능 → **tdd-guide** 에이전트
4. 아키텍처 의사결정 → **architect** 에이전트
5. 빌드 실패 → **build-error-resolver** 에이전트
6. 커밋 전 보안 점검 → **security-reviewer** 에이전트

## 병렬 Task 실행

독립적인 작업은 항상 병렬 Task 실행을 사용한다.

```
GOOD: Launch 3 agents in parallel in a single message:
1. Agent 1: Security analysis of auth.ts
2. Agent 2: Performance review of cache system
3. Agent 3: Type checking of utils.ts

BAD: Sequential execution when there are no dependencies.
```

## 다중 관점 분석

복잡한 문제에는 역할을 분리한 서브에이전트를 사용한다.
- 사실 검토자 (Factual reviewer)
- 시니어 엔지니어 (Senior engineer)
- 보안 전문가 (Security expert)
- 일관성 검토자 (Consistency reviewer)
- 중복 검토자 (Redundancy checker)

각 관점은 좁게 집중된 범위를 가진 자체 서브에이전트를 부여받는다.
