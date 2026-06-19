---
title: Secure a Test Safety Net Before Changing Complex Code
impact: HIGH
impactDescription: prevents silent runtime regressions in code the compiler cannot verify
tags: testing, complexity, cynefin
---

## Secure a Test Safety Net Before Changing Complex Code

The Cynefin framework sorts a change by how well you understand its impact, and each
band gets a different strategy:

- **Clear** — you are certain the change has no ripple; edit safely within the protocol.
- **Complicated** — the ripple is compile-visible; the type system flags every caller, so agree with the affected parties, then edit.
- **Complex** — the ripple is *runtime* coupling the compiler cannot catch (events, shared state, ordering, side effects). You only know what broke by running the code.
- **Chaotic** — the coupling is uncontrolled; there is no reliable way to predict impact at all.

The trap is treating complex code as if it were clear. When the impact lives at runtime,
editing without a test and "letting QA find it later" is not a strategy — it is operating
in the chaotic band by choice. The discipline that keeps a change *complex* rather than
*chaotic* is a safety net: before you touch runtime-coupled logic, add or confirm a
characterization test that pins the current behavior. Then refactor against it. A failing
test now tells you exactly which runtime contract you broke, instead of a support ticket
a week from now.

So the rule for complex code is: pin behavior first, change second. If existing tests
already cover the path, run them and confirm green. If they do not, write a
characterization test that captures whatever the code does today — even if that behavior
is ugly — and only then make your edit.

**Incorrect:**

```ts
// Complex code: an order's total quietly depends on the discount engine,
// the tax service, and a mutable cart cache — none of it compile-checked.
// Editing it with no test and trusting QA to catch fallout = chaotic by choice.
function applyLoyaltyBonus(order: Order): Order {
  // New rule: loyalty members get an extra 5% off.
  order.discountRate += 0.05            // mutates shared cart-cache state
  order.total = order.subtotal * (1 - order.discountRate)
  // Forgot: tax is computed downstream from `total` in another module.
  // Forgot: discountRate is also read by the invoice PDF generator.
  return order
}
// Shipped. No test pinned the old total or tax behavior, so the
// double-applied discount and wrong tax only surface in production.
```

**Correct:**

```ts
// 1. Pin current behavior with a characterization test BEFORE editing.
describe('applyLoyaltyBonus (characterization)', () => {
  it('keeps total, tax, and invoice in sync for a member order', () => {
    const order = makeOrder({ subtotal: 100, discountRate: 0.1, loyalty: true })

    const result = applyLoyaltyBonus(order)

    // Lock the runtime contracts the compiler cannot see:
    expect(result.total).toBe(85)              // 100 * (1 - 0.15)
    expect(computeTax(result)).toBe(8.5)       // downstream module
    expect(renderInvoice(result).discountLine).toBe('15% off') // PDF generator
  })
})

// 2. Now refactor against the net. A broken runtime contract fails loudly here,
//    not in production.
function applyLoyaltyBonus(order: Order): Order {
  const discountRate = order.discountRate + (order.loyalty ? 0.05 : 0)
  const total = order.subtotal * (1 - discountRate)
  return { ...order, discountRate, total } // no shared-state mutation
}
```

For the broader test discipline (characterization tests, coverage targets, what to
assert), see the `tdd-workflow` and `coding-standards` skills.

Reference: [Complexity strategy ladder](../references/complexity.md)
