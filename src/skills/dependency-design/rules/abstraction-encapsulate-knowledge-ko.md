---
title: "Classify Domain-Specific vs General Knowledge, Share via Contract"
impact: HIGH
impactDescription: reusing data shaped for one purpose as another module's input creates model coupling that turns every internal change into a cross-module break
tags: abstraction, encapsulation, contract
---

## Classify Domain-Specific vs General Knowledge, Share via Contract

모든 모듈은 도메인 지식의 한 조각을 소유하며, 그 안에는 두 종류가 섞여 있다. 폭넓게 적용되고 안정적인 **general** 지식과, 이 문제에만 고유한 **domain-specific** 지식이다. 둘은 공개할 때 서로 반대 방향으로 작용한다. general 지식을 공개하면 재사용성과 호환성이 올라가고, domain-specific 지식을 공개하면 명확성과 신뢰성이 올라간다. 주어진 경계가 어느 쪽을 노출해야 하는지 의도적으로 결정한 뒤, 공개하기 전에 추상화하라.

그 지식을 어떻게 공유하느냐가 결합을 결정한다. **model coupling**은 다른 목적으로 공개된 데이터 — 대개 persistence row나 DTO — 를 두 번째 모듈의 입력으로 재사용한다. 편리해 보이지만 모든 소비자를 남의 내부 형태에 묶어버려서, 저장 방식이 바뀌면 그 여파가 바깥으로 번진다. **contract coupling**은 상호작용만을 위해 만든 데이터를 공개한다. 목적에 맞게 만든 상호작용 contract를 선호하라. 각 측이 contract만 지키면 내부를 자유롭게 바꿀 수 있으므로, 가장 안정적인 공유 형태다.

**Incorrect:**

```ts
// A persistence row is published; other modules couple to its shape (model coupling).
interface UserRow {
  id: number
  pw_hash: string // storage detail
  created_ts: number // storage column name + epoch units
  pref_json: string // serialized blob, schema implicit
}

// Notifications now depends on the database's internal shape.
function sendWelcome(user: UserRow) {
  const prefs = JSON.parse(user.pref_json) // Notifications is bound to the storage row shape, not a contract
  mailer.send(prefs.locale, user.id)
}
```

**Correct:**

```ts
// A purpose-built interaction contract, abstracted from storage (contract coupling).
interface WelcomeRequested {
  userId: string
  locale: string
}

// The User module maps its internal row to the contract; consumers see only that.
function toWelcomeRequested(row: UserRow): WelcomeRequested {
  return { userId: String(row.id), locale: JSON.parse(row.pref_json).locale }
}

// Notifications depends on the stable contract, not on storage internals.
function sendWelcome(evt: WelcomeRequested) {
  mailer.send(evt.locale, evt.userId)
}
```

더 넓은 결합 수준(intrusive, functional, model, contract)과 더 느슨한 쪽으로 옮겨가는 방법은 `backend-patterns` skill을 참고하라.

Reference: [Abstraction and Module Boundaries](../references/abstraction.md)
