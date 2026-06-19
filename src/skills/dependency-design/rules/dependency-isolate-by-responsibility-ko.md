---
title: Isolate Modules by Responsibility (Change-Rate)
impact: HIGH
impactDescription: co-locating different change-rates couples their lifecycles and widens every change's blast radius
tags: dependency, srp, change-rate
---

## Isolate Modules by Responsibility (Change-Rate)

모듈의 책임은 *변경되는 이유*로 정의하는 것이 가장 좋다. 그리고 변경되는 이유는 실제로는 그 모듈의 변경 빈도(change-rate)로 드러난다. 변동성이 큰 비즈니스 규칙은 매주 바뀌고, 인프라 어댑터는 거의 바뀌지 않는다. 변경 빈도가 다른 두 가지를 한곳에 두면 두 생명주기가 결합된다. 빠르게 변하는 부분을 한 번 수정할 때마다, 움직일 이유가 전혀 없던 느리게 변하는 부분까지 다시 읽고, 다시 테스트하고, 위험을 감수해야 한다.

모듈을 변경 빈도 경계를 따라 분리한다. 분리 후에는 변동성 큰 규칙의 변경이 변동성 모듈만 건드리며, 안정적인 모듈은 손대지 않으므로 신뢰성을 유지한다. 이것은 단일 책임 원칙(Single Responsibility Principle)을 *시간*의 관점에서 읽은 것이다. 함께 변하는 것은 묶고, 따로 변하는 것은 분리한다.

이러한 모듈을 깔끔하게 나누기 위한 네이밍과 파일 구조 규칙은 `coding-standards` skill을 참고한다.

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
