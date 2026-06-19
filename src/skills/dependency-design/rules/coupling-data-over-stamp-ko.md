---
title: Pass Only the Data Needed (Data over Stamp), Avoid Train-Wreck Bridges
impact: MEDIUM
impactDescription: passing whole structures and chaining through them couples callers to shapes they never use
tags: coupling, interface
---

## Pass Only the Data Needed (Data over Stamp), Avoid Train-Wreck Bridges

Stamp coupling은 피호출자가 한 필드만 필요한데 전체 구조를 넘기는 것이다. 이제 피호출자는 사용하지도 않는 구조에 결합되고, 그 구조가 바뀌면 파장이 바깥으로 퍼진다. 실제로 필요한 값만 넘기는 data coupling을 선호하라.

현대 개발에서 stamp coupling의 약한 형태는 더 이상 누출로 취급되지 않는다. 불변 값 객체(record 스타일 구조)를 노출하는 것은 결함이라기보다 설계 의사결정에 가깝다. 진짜 위험은 train-wreck bridge다. `a.b.c.d`처럼 여러 객체를 가로질러 뻗어 나가며 호출자를 중간 구조 전체에 결합시키는 체인이다. 각 연결 고리는 따로따로 바뀔 수 있는 별개의 대상이다. 호출자가 그래프를 탐색하도록 강요하지 말고, 값의 소유자가 그 값을 직접 노출하게 하라.

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
