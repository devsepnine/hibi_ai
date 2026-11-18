---
title: "Workspace Package Boundaries"
impact: HIGH
impactDescription: a clean apps/packages/lib gradient keeps deploy units independent and stops dependency cycles from forming across the workspace
tags: architecture, monorepo, turborepo
---

## Workspace Package Boundaries

저장소의 아키텍처와 패키지 경계를 먼저 확인한다. 다음 구조에서는 앱이 배포 단위를 소유하고 패키지가 재사용 가능한 기능을 제공한다.

1. **`apps/*`**: `packages` 기능을 엮는 orchestrator이며, 각 app은 독립적으로 빌드/배포 가능한 단위, 즉 application이다. 규칙은 `app -> packages`가 언제나 one-way라는 것이다. app은 다른 app을 import하지 않는다.
2. **`packages/<domain>*`**: domain 레이어로, 도메인의 invariant인 business-logic과 도메인의 long-term 상태인 data-access를 담는다.
3. **`packages/lib*`**: foundation 레이어로, business-logic과 data-access가 의존하는 순수 기능이다. `packages/domain -> packages/lib`는 one-way다.

프로젝트가 허용하면 기반 패키지는 다른 기반 패키지의 공개 API를 사용할 수 있다. 워크스페이스 의존성을 선언하고 그래프에 순환이 없는지 확인한다. 프로젝트가 기반 패키지를 말단으로 정했다면 그 규칙을 따른다. 외부 배포 여부는 따로 판단한다.

각 domain package를 세 영역으로 나눠 front와 server가 contract만 정확히 공유하고 그 이상은 공유하지 않도록 한다.

```
packages/<domain>/common  - protocol, type, rule, invariant (shared both ways)
packages/<domain>/front   - front-side domain rule, invariant
packages/<domain>/server  - server-side domain rule, invariant
```

구체적인 트리는 다음과 같다.

```
apps/
  ndxFront     - presentation + domain + orchestrator
  ndxServer    - application + orchestrator
  pdfFront
  pdfServer
packages/
  ndx/                     # domain layer
    common/  - protocol, type, rule, invariant
    front/   - front domain rule, invariant
    server/  - server domain rule, invariant
  pdf/
    common/
    front/
    server/
  pdfUtil      # foundation (lib)
  llmApi       # foundation (lib)
  socket       # foundation (lib)
  fetch        # foundation (lib)
```

클라이언트 패키지에서 통신 패키지를 사용하는 예시:

```json
{
  "name": "@acme/api-client",
  "dependencies": {
    "@acme/transport": "workspace:*"
  }
}
```

`@acme/transport`가 다시 `@acme/api-client`를 import하면 순환이 생긴다. 클라이언트에만 필요한 동작을 클라이언트로 옮기거나 공통 계약을 두 패키지의 하위 의존성으로 분리한다.

app이나 package *내부*의 컴포넌트 구조, 즉 context, compound component, prop 경계를 어떻게 잡을지는 `composition-patterns` skill을 참고하라.

Reference: [Layered and Turbo Monorepo Architecture](../references/monorepo.md)
