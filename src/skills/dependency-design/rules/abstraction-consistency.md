---
title: Keep Abstraction Criteria and Level Consistent
impact: HIGH
impactDescription: inconsistent abstraction levels make a module split worthless and re-tangle the very complexity modularization was meant to remove
tags: abstraction, consistency, module
---

## Keep Abstraction Criteria and Level Consistent

The only thing that justifies splitting code into a module is abstraction: a
module hides detail behind a single, coherent viewpoint. If the criterion or the
level of that abstraction is inconsistent — high-level orchestration sitting next
to low-level byte fiddling, or a domain API that also speaks raw HTTP — the split
buys nothing. A caller now has to reason about two altitudes at once, so the
module is harder to understand than the flat code it replaced.

Pick one abstraction criterion per module and one level per interface. Everything
a module exposes should describe the problem at the same altitude; push the
lower-level details down into their own module behind their own coherent boundary.

**Incorrect:**

```ts
// One "service" mixes domain orchestration, transport details, and byte work.
class OrderService {
  async place(order: Order): Promise<void> {
    // high-level domain verb
    this.validate(order)

    // mid-level: HTTP transport details leaking into the domain object
    const res = await fetch("https://pay.example.com/v2/charge", {
      method: "POST",
      headers: { "Idempotency-Key": crypto.randomUUID() },
      body: JSON.stringify({ amount_cents: order.totalCents }),
    })
    if (res.status === 429) await this.backoff() // transport concern

    // low-level: manual byte framing for an audit log
    const buf = Buffer.alloc(8)
    buf.writeBigUInt64BE(BigInt(order.id))
    this.auditFd.write(buf)
  }
}
```

**Correct:**

```ts
// OrderService stays at the domain level. Lower altitudes live behind their own
// modules, each with one consistent abstraction.
class OrderService {
  constructor(
    private readonly payments: PaymentGateway, // domain-level contract
    private readonly audit: AuditLog, // domain-level contract
  ) {}

  async place(order: Order): Promise<void> {
    this.validate(order)
    await this.payments.charge(order.id, order.totalCents)
    await this.audit.record("order.placed", order.id)
  }
}

// HTTP/retry details are isolated — that module is consistently transport-level.
class HttpPaymentGateway implements PaymentGateway {
  async charge(orderId: string, amountCents: number): Promise<void> {
    /* fetch, idempotency, backoff live only here */
  }
}

// Byte framing is isolated — that module is consistently encoding-level.
class BinaryAuditLog implements AuditLog {
  async record(event: string, id: string): Promise<void> {
    /* buffer writes live only here */
  }
}
```

For how to choose a single dividing viewpoint and layer modules by it, see the
`backend-patterns` skill.

Reference: [Abstraction and Module Boundaries](../references/abstraction.md)
