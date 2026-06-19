---
title: Secure a Test Safety Net Before Changing Complex Code
impact: HIGH
impactDescription: prevents silent runtime regressions in code the compiler cannot verify
tags: testing, complexity, cynefin
---

## Secure a Test Safety Net Before Changing Complex Code

Cynefin 프레임워크는 변경의 영향을 얼마나 잘 이해하고 있는지에 따라 상황을 분류하며,
각 구간마다 다른 전략을 적용한다.

- **Clear** — 변경이 아무런 파급을 일으키지 않음을 확신한다. 프로토콜 범위 안에서 안전하게 수정한다.
- **Complicated** — 파급이 컴파일 시점에 드러난다. 타입 시스템이 모든 caller를 짚어 주므로, 영향받는 대상과 합의한 뒤 수정한다.
- **Complex** — 파급이 컴파일러가 잡지 못하는 *런타임* 결합이다(event, 공유 상태, 순서, side effect). 코드를 실행해야만 무엇이 깨졌는지 알 수 있다.
- **Chaotic** — 결합이 통제되지 않은 상태다. 영향을 예측할 신뢰할 만한 방법이 전혀 없다.

함정은 complex 코드를 clear인 것처럼 다루는 것이다. 영향이 런타임에 존재하는데도
테스트 없이 수정하고 "나중에 QA가 찾겠지"라고 미루는 것은 전략이 아니라, 스스로
chaotic 구간에서 작업하기를 선택하는 것이다. 변경을 *chaotic*이 아니라 *complex*로
유지하는 규율은 안전망이다. runtime-coupled 로직을 건드리기 전에, 현재 동작을 고정하는
characterization test를 추가하거나 확인하라. 그런 다음 그 위에서 refactor하라. 실패하는
테스트는 일주일 뒤 도착할 지원 티켓 대신, 지금 당장 어떤 런타임 계약을 깨뜨렸는지를
정확히 알려 준다.

따라서 complex 코드에 대한 규칙은 이렇다. 동작을 먼저 고정하고, 그다음에 변경하라.
기존 테스트가 이미 해당 경로를 덮고 있다면 실행해서 green을 확인한다. 덮고 있지 않다면,
오늘의 동작이 비록 보기 좋지 않더라도 그것을 그대로 포착하는 characterization test를
작성한 뒤에만 수정한다.

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

characterization test, coverage 목표, 무엇을 단언할지 등 더 넓은 테스트 규율은
`tdd-workflow`와 `coding-standards` skill을 참고한다.

Reference: [Complexity strategy ladder](../references/complexity.md)
