---
title: Do Not Leak Implementation Knowledge Through the Interface
impact: HIGH
impactDescription: consumers that reach into concrete internals break on every refactor and freeze the implementation in place
tags: coupling, encapsulation
---

## Do Not Leak Implementation Knowledge Through the Interface

Contents coupling (also called implementation-knowledge coupling) occurs when a consumer depends on another module's concrete internals: a private field, an underscore-prefixed property, an internal data shape, or a body that was never meant to be part of the contract. Any refactor of those internals breaks the consumer, so the implementation freezes in place. This is direct leakage of hidden knowledge.

Depend on an abstract interface instead, and let the owner decide what to expose. The trade-off to balance is interactivity versus leakage: expose enough behavior to be useful, but never the raw internal structure. Reflection and dependency injection technically reach into internals, but they are acceptable only inside a rule-controlled environment such as a DI framework, where the access is governed rather than ad hoc.

**Incorrect:**

```ts
class OrderService {
  // private-by-convention internals
  _cache: Map<string, Order> = new Map()
  _repo = new OrderRepo()
}

// Consumer reaches into internals and depends on the cache shape.
function warmup(service: OrderService, id: string) {
  const cached = service._cache.get(id) // leaked internal structure
  if (!cached) {
    const order = service._repo.findById(id) // leaked dependency
    service._cache.set(id, order)
  }
}
```

**Correct:**

```ts
// The contract exposes behavior, not structure.
interface OrderLookup {
  get(id: string): Promise<Order>
}

class OrderService implements OrderLookup {
  #cache = new Map<string, Order>()
  #repo = new OrderRepo()

  async get(id: string): Promise<Order> {
    const cached = this.#cache.get(id)
    if (cached) return cached
    const order = await this.#repo.findById(id)
    this.#cache.set(id, order)
    return order
  }
}

// Consumer depends only on the abstract interface.
function warmup(lookup: OrderLookup, id: string) {
  return lookup.get(id)
}
```

Reference: [Coupling types and threat ranking](../references/coupling-models.md)
