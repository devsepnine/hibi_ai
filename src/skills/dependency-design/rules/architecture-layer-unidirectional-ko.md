---
title: "Layered Architecture: One-Way Dependency, Watch N:N Mapping"
impact: HIGH
impactDescription: layers make one-way dependency easy but produce N:N mapping that erodes the separation they promised
tags: architecture, layer
---

## Layered Architecture: One-Way Dependency, Watch N:N Mapping

레이어는 단일한 추상화 관점으로 시스템을 분리한다. 모든 레이어는 다른 레이어의 위 또는 아래에 놓이므로, 레이어링은 언제나 functional coupling을 만든다. 상위 레이어는 그 아래 레이어 없이는 자신의 일을 할 수 없다. 이것은 결함이 아니라 레이어가 받아들이는 거래다. 레이어링은 관점이 안정적이고 경계가 거의 움직이지 않는 **고정된 도메인**에서 가장 효과적이며, 전통적인 조직 구조에 잘 대응된다.

레이어링의 강점은 one-way dependency를 강제하기가 매우 쉽다는 점이다. 각 레이어는 아래쪽만 가리킬 수 있다. 반복해서 나타나는 실패 양상은 **N:N mapping**이다. 관심사가 레이어에 깔끔하게 배정되지 않으면, 모든 상위 모듈이 모든 하위 모듈을 건드리게 되고 레이어 경계는 더 이상 아무 의미도 가지지 못한다. 매핑을 좁게 유지하라. 레이어는 바로 아래 레이어에 의존해야 하며, 가로질러 닿거나 중간 단계를 건너뛰어서는 안 된다.

레이어 관점은 의도적으로 골라야 한다. 흔히 쓰이는 관점은 세 가지이며, 이들을 일관성 없이 섞는 것이 바로 N:N 엉킴을 만든다.

- **Lifecycle** — 객체가 얼마나 오래 사는가.
  - `presentation`: 필요할 때 생성되고 상호작용 처리가 끝나면 파기된다.
  - `application`: request에서 response까지 살아 있다가 폐기된다.
  - `business`: 상대적으로 long-term으로 유지되는, 상태 없는 invariant.
  - `data-access`: 상대적으로 long-term으로 유지되며 영속 상태만 소유한다.
- **Functional role** — 각 레이어가 맡는 역할.
  - `interface` (presentation): 최초 이벤트를 발생시키고 결과를 수령한다.
  - `orchestrator` (application): 기능을 모아 중계하여 동작시킨다.
  - `provider` (business + data-access): 실제 기능을 제공하는 레이어.
- **Domain role** — 지식이 얼마나 일반적인가.
  - `domain`: 도메인별 상호작용을 담당한다.
  - `function`: 도메인이 재사용하는 중립적인 기능.
  - `foundation`: function 레이어가 동작할 수 있는 기반 기능.

축마다 하나의 관점을 골라 일관되게 유지하라. 표준적인 lifecycle 흐름은 `presentation -> application -> business -> data-access`이며, 오직 한 방향이다.

**Incorrect:**

```typescript
// business/orderRules.ts  — a business-layer module reaching UP into presentation
import { showToast } from "../presentation/toast"          // upward dependency
import { db } from "../dataAccess/db"

export function applyDiscount(order: Order) {
  const next = { ...order, total: order.total * 0.9 }
  db.save(next)
  showToast("Discount applied")   // business now knows about the UI layer
  return next
}
```

**Correct:**

```typescript
// presentation/orderView.ts  — top layer, points down at application only
import { placeOrder } from "../application/orderService"
import { showToast } from "./toast"
async function onSubmit(order: Order) {
  const result = await placeOrder(order)   // down to application
  showToast(`Saved (total ${result.total})`)   // UI stays in the UI layer
}

// application/orderService.ts  — orchestrator, points down at business
import { applyDiscount } from "../business/orderRules"
export async function placeOrder(order: Order) {
  return applyDiscount(order)
}

// business/orderRules.ts  — business invariant, points down at data-access only
import { save } from "../dataAccess/orderRepo"
export function applyDiscount(order: Order): Order {
  const next = { ...order, total: order.total * 0.9 }
  save(next)
  return next   // returns a value; never reaches up to the UI
}

// dataAccess/orderRepo.ts  — bottom layer, depends on nothing above it
export function save(order: Order): void {
  /* persist */
}
```

Reference: [Layered and Turbo Monorepo Architecture](../references/monorepo.md)
