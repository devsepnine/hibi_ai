---
title: Keep Abstraction Criteria and Level Consistent
impact: HIGH
impactDescription: inconsistent abstraction levels make a module split worthless and re-tangle the very complexity modularization was meant to remove
tags: abstraction, consistency, module
---

## Keep Abstraction Criteria and Level Consistent

코드를 모듈로 나누는 것을 정당화하는 유일한 근거는 추상화다. 모듈은 하나의 일관된 관점 뒤에 세부 사항을 숨긴다. 그 추상화의 기준이나 수준이 일관되지 않으면 — 고수준 orchestration이 저수준 byte 조작과 나란히 놓이거나, 도메인 API가 raw HTTP까지 함께 다루면 — 분리는 아무것도 얻지 못한다. 이제 호출자는 두 고도를 동시에 추론해야 하므로, 그 모듈은 자신이 대체한 평면 코드보다 오히려 이해하기 어려워진다.

모듈마다 추상화 기준을 하나만 선택하고, interface마다 수준을 하나만 선택하라. 모듈이 노출하는 모든 것은 같은 고도에서 문제를 설명해야 한다. 더 낮은 수준의 세부 사항은 그 자체의 일관된 경계 뒤, 별도 모듈로 밀어 넣어라.

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

하나의 분리 관점을 고르고 그 기준으로 모듈을 계층화하는 방법은 `backend-patterns` skill을 참고하라.

Reference: [Abstraction and Module Boundaries](../references/abstraction.md)
