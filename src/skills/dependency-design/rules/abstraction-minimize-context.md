---
title: "Minimize Context: Publish Abstracted, Not Concrete, Knowledge"
impact: MEDIUM
impactDescription: every concrete assumption a module forces its caller to satisfy becomes hidden coupling that breaks silently when the implementation changes
tags: abstraction, interface
---

## Minimize Context: Publish Abstracted, Not Concrete, Knowledge

A module is three kinds of knowledge: the **interface** (what it publishes for
interaction), the **implementation** (the real knowledge it hides), and the
**context** (the assumptions it does not implement and silently requires of its
surroundings). Context is the dangerous part — it is unwritten, uncompiled, and
unchecked, so the caller can violate it without any error until runtime.

When the interface exposes concrete types and concrete assumptions, callers must
absorb that context to use the module correctly, and complexity rises everywhere.
Publish an abstracted contract instead: name the operation in the caller's terms,
accept exactly the data the interaction needs, and keep ordering, units, and
construction details inside the implementation. Minimizing context is what makes a
module clear and reliable to use.

**Incorrect:**

```ts
// Interface leaks concrete types and unstated assumptions (the "context").
interface PriceCalculator {
  // Caller must know: cents not dollars, that taxTable is pre-sorted,
  // that init() must run first, and how to build a raw DiscountRow.
  taxTable: TaxRow[]
  init(): void
  calc(amountCents: number, region: string, rows: DiscountRow[]): number
}

// Every caller now carries that hidden context:
calculator.init() // forget this and you get NaN
const total = calculator.calc(1999, "US-CA", buildDiscountRows(cart)) // cents? rows shape?
```

**Correct:**

```ts
// Abstracted contract: caller speaks in domain terms, context stays internal.
interface PriceCalculator {
  // Money is a self-describing value object; no init order, no raw rows.
  total(cart: Cart, region: Region): Money
}

// The caller needs no hidden knowledge — the interface says everything.
const total = calculator.total(cart, region)
```

The choice of *which* knowledge to publish (general vs domain-specific) is covered
by `abstraction-encapsulate-knowledge`.

Reference: [Abstraction and Module Boundaries](../references/abstraction.md)
