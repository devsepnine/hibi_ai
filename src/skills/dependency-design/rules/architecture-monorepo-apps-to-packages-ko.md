---
title: "Turbo Monorepo: apps -> packages One-Way, No lib -> lib"
impact: HIGH
impactDescription: a clean apps/packages/lib gradient keeps deploy units independent and stops dependency cycles from forming across the workspace
tags: architecture, monorepo, turborepo
---

## Turbo Monorepo: apps -> packages One-Way, No lib -> lib

Turborepo 모노레포는 workspace 수준에 적용된 복합 레이어링이다. 각 workspace 폴더는 하나의 역할을 맡고, 의존성은 오직 한 방향으로만 흐른다.

1. **`apps/*`** — `packages` 기능을 엮는 orchestrator이며, 각 app은 독립적으로 빌드/배포 가능한 단위(application)다. 규칙은 `app -> packages`가 언제나 one-way라는 것이다. app은 다른 app을 import하지 않는다.
2. **`packages/<domain>*`** — domain 레이어로, business-logic(도메인의 invariant)와 data-access(도메인의 long-term 상태)를 담는다.
3. **`packages/lib*`** — foundation 레이어로, business-logic과 data-access가 의존하는 순수 기능이다. `packages/domain -> packages/lib`는 one-way다.

지양해야 하는 간선은 **`packages/lib -> packages/lib`**이다. foundation package는 다른 foundation package에 의존해서는 안 된다. `lib2`가 `lib1`의 공유 의존성이 될 만큼 충분히 일반적이라면, 그것은 애초에 `packages`에 속하지 않고 `node_modules` 대상이다(배포하거나 third-party 의존성으로 다룬다). `lib` package를 leaf 수준으로 유지하면 foundation이 평평하고 acyclic하게 유지된다.

각 domain package를 세 영역으로 나눠 front와 server가 계약(contract)만 정확히 공유하고 그 이상은 공유하지 않도록 한다.

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

**Incorrect:**

```jsonc
// packages/pdfUtil/package.json  — a lib depending on another lib
{
  "name": "@acme/pdfUtil",
  "dependencies": {
    "@acme/llmApi": "workspace:*"   // lib -> lib: foundation is no longer flat
  }
}

// apps/ndxServer/src/index.ts  — an app reaching into another app
import { renderPdf } from "../../pdfServer/src/render"   // app -> app: forbidden
```

**Correct:**

```jsonc
// packages/pdf/server/package.json  — domain depends DOWN on foundation libs
{
  "name": "@acme/pdf-server",
  "dependencies": {
    "@acme/pdfUtil": "workspace:*",   // domain -> lib, one-way
    "@acme/fetch": "workspace:*"
  }
}

// packages/pdfUtil/package.json  — a foundation lib is a leaf: no workspace deps
{
  "name": "@acme/pdfUtil",
  "dependencies": {
    "pdf-lib": "^1.17.0"   // truly shared general code lives in node_modules
  }
}

// apps/ndxServer/package.json  — app depends DOWN on packages only
{
  "name": "ndx-server",
  "dependencies": {
    "@acme/ndx-server": "workspace:*",
    "@acme/llmApi": "workspace:*"
  }
}
```

app이나 package *내부*의 컴포넌트 구조 — context, compound component, prop 경계 — 를 어떻게 잡을지는 `composition-patterns` skill을 참고하라.

Reference: [Layered and Turbo Monorepo Architecture](../references/monorepo.md)
