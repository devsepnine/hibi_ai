---
title: "Layered Architecture: One-Way Dependency, Watch N:N Mapping"
impact: HIGH
impactDescription: layers make one-way dependency easy but produce N:N mapping that erodes the separation they promised
tags: architecture, layer
---

## Layered Architecture: One-Way Dependency, Watch N:N Mapping

A layer separates the system by a single abstraction viewpoint. Because every
layer sits above or below another, layering always introduces functional
coupling — an upper layer cannot do its job without the layer beneath it. That is
not a defect; it is the deal a layer makes. Layering is most effective in a
**fixed domain** that maps onto a traditional org structure, where the
viewpoint is stable and the boundaries rarely move.

The strength of layering is that one-way dependency is trivial to enforce: each
layer is only allowed to point downward. The recurring failure mode is **N:N
mapping** — when concerns are not assigned to a layer cleanly, every upper module
ends up touching every lower module, and the layer boundary stops carrying any
meaning. Keep the mapping narrow: a layer should depend on the layer directly
below it, not reach across or skip levels.

Pick the layer viewpoint deliberately. There are three common ones, and mixing
them inconsistently is what creates the N:N tangle:

- **Lifecycle** — how long an object lives.
  - `presentation`: created on demand, destroyed once the interaction is handled.
  - `application`: lives from request to response, then is discarded.
  - `business`: relatively long-lived, stateless invariants.
  - `data-access`: relatively long-lived, owns only persistent state.
- **Functional role** — what part each layer plays.
  - `interface` (presentation): raises the initial event and receives the result.
  - `orchestrator` (application): gathers and relays features to drive them.
  - `provider` (business + data-access): the real feature-providing layers.
- **Domain role** — how general the knowledge is.
  - `domain`: handles per-domain interaction.
  - `function`: neutral capability the domain reuses.
  - `foundation`: base capability that the function layer runs on.

Choose one viewpoint per axis and stay consistent. The canonical lifecycle flow
is `presentation -> application -> business -> data-access`, one way only.

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
