---
title: Isolate Shared-Resource Coupling (Common/External)
impact: MEDIUM
impactDescription: many modules sharing one mutable resource creates bottlenecks and maintenance traps that only resource isolation resolves
tags: coupling, singleton, state
---

## Isolate Shared-Resource Coupling (Common/External)

Two coupling types share one root cause. Common coupling is many modules referencing a single resource and knowing its whole structure (the classic global mutable singleton or shared config read everywhere). External coupling is the same single resource being used for different purposes by different modules. Both are real bottlenecks: shared singletons are hard to manage and harder to maintain.

The key insight is that the cure targets the single-resource sharing, not merely the whole-structure sharing. Hiding the structure behind a DTO does not remove the bottleneck if everyone still contends for the same resource. This is precisely why large-scale designs adopt approaches like MSA: to dissolve the single shared resource. At the code level, scope access behind a narrow owner so each consumer touches only what it needs. Prefer instances for terminal logic; reserve static or global state for genuine upper-layer infrastructure such as the event loop, a global queue, or a message queue, where a single shared instance is the correct model.

**Incorrect:**

```ts
// Common/External: one global mutable object, read and written everywhere.
export const appState = {
  currentUser: null as User | null,
  db: null as DbConnection | null,
  featureFlags: {} as Record<string, boolean>,
}

function checkout() {
  if (!appState.currentUser) throw new Error("no user")
  appState.db!.insert(/* ... */) // every module shares the same connection object
}
```

**Correct:**

```ts
// Each consumer depends on a narrow owner, not a shared global blob.
interface UserContext {
  current(): User
}
interface OrderStore {
  insert(order: Order): Promise<void>
}

// Terminal logic uses injected instances scoped to the request.
function checkout(users: UserContext, orders: OrderStore, order: Order) {
  const user = users.current()
  return orders.insert({ ...order, userId: user.id })
}

// Static/global is reserved for genuine upper-layer infrastructure.
class MessageBus {
  static readonly instance = new MessageBus() // one queue is the correct model
  publish(event: DomainEvent) {
    /* ... */
  }
}
```

Reference: [Coupling types and threat ranking](../references/coupling-models.md)
