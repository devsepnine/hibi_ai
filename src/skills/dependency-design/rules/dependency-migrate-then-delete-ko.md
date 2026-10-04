---
title: Migrate Every Caller, Then Delete the Old API
impact: HIGH
impactDescription: keeping old and new APIs side by side doubles the dependency graph and lets new callers keep landing on the path you meant to remove
tags: dependency, migration, api
---

## Migrate Every Caller, Then Delete the Old API

새 내부 API가 옛 API를 대체하면, 같은 변경 안에서 모든 호출자를 옮기고 옛 API를 지운다. 경로가 두 개면 모든 소비자로 들어가는 간선도 두 벌이고, 일관되게 유지해야 할 동작도 두 개이며, 새 코드가 실수로 계속 고르게 되는 기본값이 생긴다.

이 저장소 안에서 바꿀 수 없는 외부 소비자가 옛 API에 의존할 때만 호환 shim을 둔다. 그때는 shim에 소유자와 제거 날짜를 정하고, 자체 로직을 두지 말고 새 API로 넘기게 한다.

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

변경 후에는 문자열, 문서, 주석까지 트리 전체에서 옛 이름을 grep해 아무것도 그것을 가리키지 않는지 확인한다.

Reference: [Abstraction and module boundary](../references/abstraction.md)
