---
title: Depend in the Direction of Stability (DDD Subdomains)
impact: HIGH
impactDescription: pointing dependencies from volatile to stable code keeps churn from rippling into trusted modules
tags: dependency, ddd, stability
---

## Depend in the Direction of Stability (DDD Subdomains)

의존성은 안정성을 향해야 한다. 자주 변하는 모듈이 거의 변하지 않는 모듈을 의존해야 하며, 그 반대가 되어서는 안 된다. 안정적인 모듈이 변동성 큰 모듈을 import하면, 변동성 코드가 흔들릴 때마다 안정적인 코드까지 함께 끌려가서 애써 얻은 안정성을 잃게 된다.

DDD distillation은 각 종류의 코드가 어디에 위치하는지에 대한 실용적인 지도를 제공한다.

- **Core subdomain** — 핵심 경쟁력이며 Cynefin 기준 *complex* 이상의 복잡성을 가진다. 자주 변경되므로 Core 도메인 모듈이 더 안정적인 하위 도메인을 *바깥으로* 의존하는 것이 바람직하다.
- **Generic subdomain** — 굳건한 인프라성 코드로 *complicated* 이하이며, 정기적인 기술 부채 해소 주기로 관리된다. 관계에서 `model` 이상의 결합을 지향한다.
- **Supporting subdomain** — *clear* 복잡성으로 거의 변동이 없다. 안정적인 솔루션이며 `contract` 결합이 이상적이다.

따라서 변동성 큰 Core가 안정적인 Generic 및 Supporting 하위 도메인을 가리킨다. 유틸리티(Generic/Supporting)는 반드시 도메인에 비종속적으로 유지되어야 한다. 빠르게 변하는 도메인 타입을 import하는 순간, 그 도메인의 변경 빈도를 그대로 물려받기 때문이다.

**Incorrect:**

```typescript
// dateUtils.ts — a stable, generic utility...
import { Invoice } from "../billing/invoice" // ...importing a volatile domain type

// now every billing rule change can force dateUtils to recompile and retest
export function dueDateFor(invoice: Invoice): Date {
  return addDays(invoice.issuedAt, invoice.terms.netDays)
}
```

**Correct:**

```typescript
// dateUtils.ts — generic, domain-agnostic; depends on nothing above it
export function addDays(date: Date, days: number): Date {
  const d = new Date(date)
  d.setDate(d.getDate() + days)
  return d
}

// billing/invoice.ts — volatile Core, depends DOWN on the stable utility
import { addDays } from "../utils/dateUtils"
export function dueDateFor(invoice: Invoice): Date {
  return addDays(invoice.issuedAt, invoice.terms.netDays)
}
```

Reference: [Coupling Models: Module, Connascence, and Domain](../references/coupling-models.md)
