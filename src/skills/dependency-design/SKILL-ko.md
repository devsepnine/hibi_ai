---
name: dependency-design
description: A methodology for managing dependencies, coupling, and abstraction so that vibe-coded software stays modifiable and AI-ownable as it grows. Modification-resistant code is code isolated by responsibility, where each part can change without rippling into others — achieved by minimizing dependencies and keeping them one-directional. Use when designing modules, deciding dependency direction, judging whether a coupling is acceptable, structuring a monorepo, defining abstraction boundaries, or reviewing architecture. 의존성 설계, 결합도 관리, 모듈 설계, 단방향 의존성, 모노레포 구조, 추상화 경계, 바이브코딩 의존성.
keywords: [dependency-design, coupling, connascence, cynefin, unidirectional-dependency, abstraction, monorepo, 의존성, 결합도, 추상화, 모듈화, 단방향 의존성, 모노레포]
---

# Dependency Design

harness 단독으로는 수정에 강한 코드를 만들 수 없다. 어려운 것은 도구가 아니라, 만들려는 제품이 무엇인지 정의하고 그것이 지속적인 변경을 견디도록 코드를 구조화하는 일이다. 바이브코딩 프로젝트는 최초 구축에서는 성공하지만(복잡성이 낮다) 열 번째 수정에서 실패한다(복잡성이 누적되고 증분 수정이 연쇄적으로 번진다). 방어책은 구조에 일찍 개입하는 것이다. 책임을 격리하고, 의존성을 최소화하고, 의존성을 단방향으로 강제해서 어떤 모듈이든 제한된 컨텍스트 안에서 — 사람이든 AI든 — 로딩하고 수정할 수 있게 만든다.

이 skill은 고정된 레시피가 아니라 의사결정 방법론이다. 제품마다 책임과 변화율이 다르므로 조직화 구조도 매번 달라진다. 이식 가능한 것은 추론 방식이다. 문제의 이해 가능성을 분류하고, 그에 맞는 결합 강도를 선택하고, 단방향 의존성을 강제하고, 추상화의 일관성을 유지한다.

## When to Apply

아래 상황이 나타나면 해당하는 reference를 참조한다.

| 상황 | Reference / rules |
|---|---|
| 문제가 가진 복잡성의 정도를 판단하고, 수정을 얼마나 안전하게 할 수 있는지 결정할 때 | `references/complexity.md` (Cynefin 분류) |
| 결합에 이름을 붙이고 등급을 매길 때 (어떤 종류인지, 얼마나 강한지, 무엇이 번지는지) | `references/coupling-models.md` (module/connascence/domain 모델) |
| 모듈이 무엇을 공개하고 무엇을 은닉할지 선택하고, 추상화 수준을 일관되게 유지할 때 | `references/abstraction.md` |
| AI가 전체를 로딩하지 않고도 일부분을 소유하고 수정할 수 있도록 코드를 구조화할 때 | `references/ai-ownership.md` |
| 레이어 간 단방향 의존성을 강제할 수 있는 monorepo를 배치할 때 | `references/monorepo.md` |
| 리뷰 중에 컴파일된 규칙을 강제할 때 | `AGENTS.md` (전체 rule set) |

## Core decision flow

1. **컨텍스트를 분류한다 (Cynefin).** 문제가 얼마나 이해되었는지 가늠한다. `clear` → `complicated` → `complex` → `chaotic`. 이해도가 낮을수록 더 느슨하고 유연한 결합이 정당화되고, 이해도가 높을수록 더 단단하고 안전한 결합이 정당화된다. 이후의 모든 선택에 대한 상대적 기준이다. `references/complexity.md` 참조.
2. **결합 전략을 고른다.** 지금 만들고 있는 결합에 이름을 붙이고 module, connascence, domain 모델로 그 강도를 평가한다. 변화율이 비용을 정당화하지 않는 한 더 약하고 더 명시적인 결합 쪽으로 밀어붙인다. `references/coupling-models.md` 참조.
3. **단방향 의존성을 강제한다.** 요청하려면 대상을 알아야 하므로, 상호작용의 방향이 의존성의 방향을 확정한다. 의존성을 비순환적이고 선형적으로(pipelining) 유지해서 인과관계를 추적 가능하게 하고 부분 수정을 안전하게 유지한다. `references/monorepo.md`와 `references/ai-ownership.md` 참조.
4. **추상화를 일관되게 유지한다.** 구체적인 내부가 아니라 추상화된 지식을 공개하고, 모듈/레이어마다 하나의 일관된 추상화 기준을 적용한다. 추상화가 일관되지 않으면 모듈화는 무의미해진다. `references/abstraction.md` 참조.

## Rule Categories

| # | Category | Impact | Prefix |
| --- | --- | --- | --- |
| 1 | Complexity & Context (Cynefin) | HIGH | `complexity-` |
| 2 | Coupling Types & Threat Ranking | CRITICAL | `coupling-` |
| 3 | Dependency Direction & Structure | CRITICAL | `dependency-` |
| 4 | Abstraction & Module Boundary | HIGH | `abstraction-` |
| 5 | Layered & Monorepo Architecture | MEDIUM | `architecture-` |
| 6 | AI-Friendly Ownership | MEDIUM | `ai-` |

## Quick Reference

### Complexity & Context (HIGH)

- `complexity-test-safety-for-complex` — runtime 결합(complex) 코드를 수정하기 전에 테스트 안전망을 확보한다

### Coupling Types & Threat Ranking (CRITICAL)

- `coupling-avoid-control-coupling` — callee의 내부 분기를 조종하는 flag/mode 인자 금지 (Control 결합)
- `coupling-no-implementation-leak` — 인터페이스로 구현 지식을 누출하지 않는다
- `coupling-data-over-stamp` — 필요한 데이터만 전달하고 train-wreck 브릿지를 피한다
- `coupling-isolate-shared-resource` — 공유 자원(Common/External) 결합을 격리한다

### Dependency Direction & Structure (CRITICAL)

- `dependency-unidirectional` — 의존성을 단방향·비순환으로 유지한다
- `dependency-isolate-by-responsibility` — 책임(변화율)별로 모듈을 격리한다
- `dependency-stable-direction` — 안정성 방향으로 의존한다 (DDD subdomain)

### Abstraction & Module Boundary (HIGH)

- `abstraction-consistency` — 추상화 기준과 수준을 일관되게 유지한다
- `abstraction-minimize-context` — 구체가 아닌 추상화된 지식을 공개하고 context를 최소화한다
- `abstraction-encapsulate-knowledge` — 도메인 특화 지식과 일반 지식을 분류해 contract로 공유한다

### Layered & Monorepo Architecture (MEDIUM)

- `architecture-layer-unidirectional` — 레이어 아키텍처: 단방향 의존, N:N 매핑 주의
- `architecture-monorepo-apps-to-packages` — Turbo monorepo: apps→packages 단방향, lib→lib 금지
- `architecture-linear-interconnection` — 상호연결 복잡성 통제: 선형 흐름 + 메시지 제약

### AI-Friendly Ownership (MEDIUM)

- `ai-partial-ownership` — AI 부분 소유를 위한 코드 구조화

## Deep references

- [references/complexity.md](references/complexity.md) — 복잡성, 이해 가능성, 그리고 결합 강도를 고르기 위한 Cynefin framework.
- [references/coupling-models.md](references/coupling-models.md) — module coupling, connascence (컴파일 타임 vs. 암묵적), domain coupling 모델.
- [references/abstraction.md](references/abstraction.md) — interface/implementation/context 분리, 지식 분류, modeling/categorization/grouping, 추상화 일관성.
- [references/ai-ownership.md](references/ai-ownership.md) — AI 이전과 이후의 code ownership, 그리고 부분적이고 컨텍스트로 제한된 소유를 위한 코드 구조화.
- [references/monorepo.md](references/monorepo.md) — 레이어 추상화와, 단방향 의존성을 강제하는 Turbo monorepo 배치 (`apps/` → `packages/`).

전체 컴파일된 rule set은 `AGENTS.md`를 참조한다.

## Related skills

clean-code, 네이밍, code-smell 표준은 `coding-standards`를, React 컴포넌트 수준의 composition (compound component, state lifting)은 `composition-patterns`를, 서버 측 모듈 및 API 설계는 `backend-patterns`를 참조한다. 여기서 그 내용을 중복하지 말고 링크로 연결한다.
