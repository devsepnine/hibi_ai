---
title: Use toSorted() Instead of sort() for Immutability
impact: MEDIUM-HIGH
impactDescription: prevents mutation bugs in React state
tags: javascript, arrays, immutability, react, state, mutation
---

## Use toSorted() Instead of sort() for Immutability

`.sort()`는 배열을 in-place로 변경하므로 React의 state·props에서 버그를 유발할 수 있다. `.toSorted()`를 사용해 변형 없이 새 정렬 배열을 만든다.

**Incorrect (mutates original array):**

```typescript
function UserList({ users }: { users: User[] }) {
  // Mutates the users prop array!
  const sorted = useMemo(
    () => users.sort((a, b) => a.name.localeCompare(b.name)),
    [users]
  )
  return <div>{sorted.map(renderUser)}</div>
}
```

**Correct (creates new array):**

```typescript
function UserList({ users }: { users: User[] }) {
  // Creates new sorted array, original unchanged
  const sorted = useMemo(
    () => users.toSorted((a, b) => a.name.localeCompare(b.name)),
    [users]
  )
  return <div>{sorted.map(renderUser)}</div>
}
```

**Why this matters in React:**

1. props·state 변경은 React의 불변성 모델을 깨뜨린다 — React는 props와 state를 read-only로 취급한다고 가정한다
2. stale closure 버그를 유발한다 — closure(callback, effect) 내부에서 배열을 변경하면 예측하지 못한 동작으로 이어질 수 있다

**Browser support (fallback for older browsers):**

`.toSorted()`는 모든 최신 브라우저에서 사용 가능하다(Chrome 110+, Safari 16+, Firefox 115+, Node.js 20+). 구형 환경에서는 spread operator를 사용한다.

```typescript
// Fallback for older browsers
const sorted = [...items].sort((a, b) => a.value - b.value)
```

**Other immutable array methods:**

- `.toSorted()` - 불변 정렬
- `.toReversed()` - 불변 reverse
- `.toSpliced()` - 불변 splice
- `.with()` - 불변 원소 교체
