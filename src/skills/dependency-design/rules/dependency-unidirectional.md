---
title: Keep Dependencies Unidirectional and Acyclic
impact: CRITICAL
impactDescription: cyclic dependencies make change ripple unpredictable and break causal ordering
tags: dependency, cycle, graph
---

## Keep Dependencies Unidirectional and Acyclic

A dependency graph models how change in one module ripples into others. When two
modules depend on each other, or a longer loop closes back on itself
(`A -> B -> C -> A`), the ripple is no longer predictable: a change to any node
in the cycle can propagate all the way around and back. Indirect cycles spanning
many modules are just as harmful as direct ones — the loop count does not soften
the coupling, it hides it.

Cycles also destroy causal order. A unidirectional edge encodes "this is built
on that": the dependency must initialize, run, and reason first. A cycle has no
such ordering, so initialization order, build order, and the mental model all
become ambiguous.

When two modules genuinely need to share something, extract that shared concern
into a lower-level module that both depend on. The dependency still flows one way
— both modules point down at the shared module, and the shared module points at
neither.

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
