---
description: Restate requirements, assess risks, and create step-by-step implementation plan. WAIT for user CONFIRM before touching any code.
allowed-tools: Agent, Read, Grep, Glob
model: sonnet
effort: high
---

# Plan Command

harness built-in `Plan` 에이전트를 호출해(별도 커스텀 에이전트 파일 불필요) 코드를 쓰기 전에 구현 계획을 만든다.

## When to Use

새 기능, 중요한 아키텍처 변경, 복잡한 리팩토링, 여러 파일에 걸친 변경, 또는 요구사항이 아직 모호할 때. 파일 하나를 고치는 자명한 변경에는 쓰지 않는다 — 그런 경우의 계획은 절약하는 것보다 더 많은 비용을 쓴다.

## What the agent produces

1. **재진술된 요구사항** — 모호한 지점은 추측하지 않고 모호하다고 지목한다
2. **단계** — 구체적이고 실행 가능한 단계를 의존성 순서로
3. **위험과 차단 요인** — 각각 주목할 이유가 되는 심각도와 함께
4. **복잡도 추정** — High / Medium / Low
5. **정지** — 계획을 제시하고, 사용자가 확인할 때까지 아무것도 쓰지 않는다

## Important Notes

**CRITICAL**: 사용자가 "yes", "proceed" 같은 명시적 긍정으로 답할 때까지 에이전트는 **어떤 코드도 쓰지 않는다**.

승인 대신 방향을 바꾸려면:
- `modify: <your changes>`
- `different approach: <alternative>`
- `skip phase 2 and do phase 3 first`

## Integration with Other Commands

계획 이후: 테스트 우선 구현은 `/tdd` · 빌드가 깨지면 `/build-fix` · 완료된 구현 검토는 `/code-review`.
