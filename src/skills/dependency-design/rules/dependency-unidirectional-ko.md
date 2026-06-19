---
title: Keep Dependencies Unidirectional and Acyclic
impact: CRITICAL
impactDescription: cyclic dependencies make change ripple unpredictable and break causal ordering
tags: dependency, cycle, graph
---

## Keep Dependencies Unidirectional and Acyclic

의존성 그래프는 한 모듈의 변경이 다른 모듈로 어떻게 파급되는지를 나타내는 모델이다. 두 모듈이 서로를 의존하거나, 더 긴 순환이 자기 자신으로 되돌아오면(`A -> B -> C -> A`) 파급은 더 이상 예측할 수 없게 된다. 순환에 속한 어떤 노드를 바꾸든 변경이 고리를 한 바퀴 돌아 다시 자신에게 돌아올 수 있다. 여러 모듈을 거치는 간접 순환도 직접 순환만큼 해롭다. 고리의 길이가 결합을 완화해 주는 것이 아니라 단지 숨길 뿐이다.

순환은 인과적 순서도 무너뜨린다. 단방향 간선은 "이것이 저것 위에 세워졌다"는 의미를 담는다. 즉 의존 대상이 먼저 초기화되고, 실행되고, 추론되어야 한다. 순환에는 그런 순서가 없으므로 초기화 순서, 빌드 순서, 그리고 머릿속 이해 모델까지 모두 모호해진다.

두 모듈이 정말로 무언가를 공유해야 한다면, 그 공유 관심사를 두 모듈이 함께 의존하는 더 낮은 수준의 모듈로 추출한다. 이렇게 하면 의존성은 여전히 한 방향으로 흐른다. 두 모듈은 모두 공유 모듈을 아래로 가리키고, 공유 모듈은 그 어느 쪽도 가리키지 않는다.

**Incorrect:**

```typescript
// orderModule.ts
import { notifyUser } from "./userModule"
export function placeOrder(order: Order) {
  save(order)
  notifyUser(order.userId, "Order placed")
}

// userModule.ts  ->  cycle: orderModule <-> userModule
import { getOrdersFor } from "./orderModule"
export function notifyUser(userId: string, msg: string) {
  const open = getOrdersFor(userId)
  send(userId, `${msg} (${open.length} open)`)
}
```

**Correct:**

```typescript
// notifier.ts  — shared lower module, depends on neither caller
export function notifyUser(userId: string, msg: string) {
  send(userId, msg)
}

// orderModule.ts  — depends down on notifier and userQuery
import { notifyUser } from "./notifier"
import { getOrdersFor } from "./userQuery"
export function placeOrder(order: Order) {
  save(order)
  const open = getOrdersFor(order.userId)
  notifyUser(order.userId, `Order placed (${open.length} open)`)
}

// userQuery.ts  — depends on nothing above it
export function getOrdersFor(userId: string): Order[] {
  return query(userId)
}
```

Reference: [Complexity, Cynefin, and Degrees of Freedom](../references/complexity.md)
