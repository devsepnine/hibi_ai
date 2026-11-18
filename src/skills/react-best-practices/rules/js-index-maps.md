---
title: Build Index Maps for Repeated Lookups
impact: LOW-MEDIUM
impactDescription: 1M ops to 2K ops
tags: javascript, map, indexing, optimization, performance
---

## Build Index Maps for Repeated Lookups

These orders look up users by string ID. When an ID occurs more than once, keep the first user in the index to preserve the result of `.find()`.

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

Build the map once in `O(n)`, then all lookups are `O(1)`.
For 1000 orders × 1000 users: 1M ops → 2K ops.
