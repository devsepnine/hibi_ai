---
title: Migrate Every Caller, Then Delete the Old API
impact: HIGH
impactDescription: keeping old and new APIs side by side doubles the dependency graph and lets new callers keep landing on the path you meant to remove
tags: dependency, migration, api
---

## Migrate Every Caller, Then Delete the Old API

When a new internal API replaces an old one, migrate every caller and delete the old API in the same change. Two parallel paths mean two sets of edges into every consumer, two behaviors to keep consistent, and a default that new code will keep picking by accident.

Keep a compatibility shim only when an external consumer that you cannot change in this repo depends on the old API. Then give the shim an owner and a removal date, and make it forward to the new API rather than keep its own logic.

**Incorrect:**

```ts
type Money = { value: number; currency: string; locale: string }

// New API added, old one kept "for now" with its own implementation.
export function formatPrice(cents: number) {
  return `$${(cents / 100).toFixed(2)}`
}

export function formatMoney(amount: Money) {
  return new Intl.NumberFormat(amount.locale, { style: 'currency', currency: amount.currency }).format(amount.value)
}
// Half the callers use formatPrice, half use formatMoney, and they disagree on locale.
```

**Correct:**

```ts
// Every caller now passes Money; formatPrice is deleted in the same change.
export function formatMoney(amount: Money) {
  return new Intl.NumberFormat(amount.locale, { style: 'currency', currency: amount.currency }).format(amount.value)
}
```

After the change, grep the old name across the tree, including strings, docs, and comments, to confirm nothing still refers to it.

Reference: [Abstraction and module boundary](../references/abstraction.md)
