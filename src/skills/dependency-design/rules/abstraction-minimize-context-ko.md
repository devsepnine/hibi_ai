---
title: "Minimize Context: Publish Abstracted, Not Concrete, Knowledge"
impact: MEDIUM
impactDescription: every concrete assumption a module forces its caller to satisfy becomes hidden coupling that breaks silently when the implementation changes
tags: abstraction, interface
---

## Minimize Context: Publish Abstracted, Not Concrete, Knowledge

모듈은 세 종류의 지식으로 이루어진다. 상호작용을 위해 공개하는 **interface**, 내부에 숨기는 실제 지식인 **implementation**, 그리고 모듈이 구현하지 않은 채 주변 환경에 암묵적으로 요구하는 가정인 **context**다. 위험한 부분은 context다. 그것은 문서화되지 않고, 컴파일되지 않으며, 검사되지 않는다. 그래서 호출자는 어떤 오류도 없이 그것을 위반할 수 있고, 문제는 runtime에 가서야 드러난다.

interface가 구체적인 타입과 구체적인 가정을 노출하면, 호출자는 모듈을 올바르게 쓰기 위해 그 context를 흡수해야 하고 복잡성은 사방에서 올라간다. 대신 추상화된 contract를 공개하라. 연산을 호출자의 용어로 명명하고, 상호작용에 필요한 데이터만 정확히 받으며, 순서·단위·생성 세부 사항은 implementation 안에 보관하라. context를 최소화하는 것이야말로 모듈을 명확하고 신뢰할 수 있게 쓰도록 만든다.

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

어떤 지식을 공개할지(general 대 domain-specific)의 선택은 `abstraction-encapsulate-knowledge`에서 다룬다.

Reference: [Abstraction and Module Boundaries](../references/abstraction.md)
