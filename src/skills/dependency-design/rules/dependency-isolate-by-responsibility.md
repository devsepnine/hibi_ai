---
title: Isolate Modules by Responsibility (Change-Rate)
impact: HIGH
impactDescription: co-locating different change-rates couples their lifecycles and widens every change's blast radius
tags: dependency, srp, change-rate
---

## Isolate Modules by Responsibility (Change-Rate)

A module's responsibility is best defined as *the reason it changes* — and the
reason it changes shows up empirically as its change-rate. Volatile business
rules change weekly; infrastructure adapters change rarely. When you co-locate
two things with different change-rates, you couple their lifecycles: every edit
to the fast-moving part forces you to re-read, re-test, and risk the slow-moving
part, even though it had no reason to move.

Split a module along its change-rate boundaries. After the split, a change to a
volatile rule touches only the volatile module; the stable module is untouched
and stays trustworthy. This is the Single Responsibility Principle read through
the lens of *time*: group what changes together, separate what changes apart.

For the naming and file-structure mechanics of carving these modules cleanly,
see the `coding-standards` skill.

**Incorrect:**

```typescript
// pricing.ts — mixes a volatile rule with stable infra in one module
export class Pricing {
  // volatile: promo logic changes every campaign
  applyDiscount(cart: Cart): number {
    if (cart.total > 100 && isBlackFriday()) return cart.total * 0.7
    return cart.total
  }

  // stable: how we talk to the gateway almost never changes
  async charge(amount: number, card: Card): Promise<Receipt> {
    return this.gateway.post("/charge", { amount, token: card.token })
  }
}
```

**Correct:**

```typescript
// discountPolicy.ts — volatile: the only place campaign rules live
export function applyDiscount(cart: Cart): number {
  if (cart.total > 100 && isBlackFriday()) return cart.total * 0.7
  return cart.total
}

// paymentGateway.ts — stable: untouched when a campaign changes
export class PaymentGateway {
  async charge(amount: number, card: Card): Promise<Receipt> {
    return this.client.post("/charge", { amount, token: card.token })
  }
}

// checkout.ts — composes the two; depends down on both
const total = applyDiscount(cart)
await gateway.charge(total, card)
```

Reference: see the `coding-standards` skill for naming and file-structure conventions.
