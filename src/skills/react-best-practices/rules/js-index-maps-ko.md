---
title: Build Index Maps for Repeated Lookups
impact: LOW-MEDIUM
impactDescription: 1M ops to 2K ops
tags: javascript, map, indexing, optimization, performance
---

## Build Index Maps for Repeated Lookups

이 예시에서 주문은 문자열 사용자 ID로 조회한다. 같은 ID가 여러 번 나오면 `.find()`가 반환할 첫 사용자를 인덱스에 유지한다.

**Incorrect, `O(n)` per lookup:**

```typescript
function processOrders(orders: Order[], users: User[]) {
  return orders.map(order => ({
    ...order,
    user: users.find(u => u.id === order.userId)
  }))
}
```

**Correct, `O(1)` per lookup:**

```typescript
function processOrders(orders: Order[], users: User[]) {
  const userById = new Map<User['id'], User>()
  for (const user of users) {
    if (!userById.has(user.id)) userById.set(user.id, user)
  }

  return orders.map(order => ({
    ...order,
    user: userById.get(order.userId)
  }))
}
```

map을 `O(n)`에 한 번 만들고 나면 이후 모든 조회는 `O(1)`이다.
주문 1000개 × 사용자 1000명: 1M ops → 2K ops.
