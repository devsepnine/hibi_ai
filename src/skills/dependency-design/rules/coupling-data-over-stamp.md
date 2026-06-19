---
title: Pass Only the Data Needed (Data over Stamp), Avoid Train-Wreck Bridges
impact: MEDIUM
impactDescription: passing whole structures and chaining through them couples callers to shapes they never use
tags: coupling, interface
---

## Pass Only the Data Needed (Data over Stamp), Avoid Train-Wreck Bridges

Stamp coupling is passing a whole structure when the callee needs only one field of it: the callee is now coupled to a shape it never uses, and any change to that shape ripples outward. Prefer data coupling, where you pass only the value actually required.

In modern code the milder form of stamp coupling is no longer treated as leakage. Exposing an immutable value object (a record-style structure) is closer to a design decision than a defect. The real danger is the train-wreck bridge: a chain like `a.b.c.d` that reaches across several objects and couples the caller to the entire intermediate structure. Each link is a separate thing that can change. Let the owner of the value expose it directly instead of forcing callers to navigate the graph.

**Incorrect:**

```ts
// Stamp: the whole order is passed to compute one number.
function computeTax(order: Order): number {
  return order.total * 0.1
}

// Train-wreck bridge: the caller is coupled to four nested shapes.
const zip = order.customer.address.location.zipCode
```

**Correct:**

```ts
// Data: pass only the value the function needs.
function computeTax(orderTotal: number): number {
  return orderTotal * 0.1
}
computeTax(order.total)

// The owner exposes the needed value; no deep navigation at the call site.
class Order {
  constructor(private readonly customer: Customer) {}
  get shippingZip(): string {
    return this.customer.shippingZip()
  }
}
const zip = order.shippingZip

// Immutable exposure of a small value object is fine, not leakage.
type Money = Readonly<{ amount: number; currency: string }>
function format(price: Money): string {
  return `${price.amount} ${price.currency}`
}
```

Reference: [Coupling types and threat ranking](../references/coupling-models.md)
