---
title: Use flatMap to Map and Filter in One Pass
impact: LOW-MEDIUM
impactDescription: eliminates intermediate array
tags: javascript, arrays, flatMap, filter, performance
---

## Use flatMap to Map and Filter in One Pass

**Impact: LOW-MEDIUM (eliminates intermediate array)**

`.map().filter(Boolean)` 체인은 중간 배열을 만들고 두 번 순회한다. `.flatMap()`을 사용해 한 번의 순회로 변환과 필터링을 동시에 한다.

**Incorrect (2 iterations, intermediate array):**

```typescript
const userNames = users
  .map(user => user.isActive ? user.name : null)
  .filter(Boolean)
```

**Correct (1 iteration, no intermediate array):**

```typescript
const userNames = users.flatMap(user =>
  user.isActive ? [user.name] : []
)
```

**More examples:**

```typescript
// Extract valid emails from responses
// Before
const emails = responses
  .map(r => r.success ? r.data.email : null)
  .filter(Boolean)

// After
const emails = responses.flatMap(r =>
  r.success ? [r.data.email] : []
)

// Parse and filter valid numbers
// Before
const numbers = strings
  .map(s => parseInt(s, 10))
  .filter(n => !isNaN(n))

// After
const numbers = strings.flatMap(s => {
  const n = parseInt(s, 10)
  return isNaN(n) ? [] : [n]
})
```

**When to use:**
- 일부를 걸러내면서 나머지를 변환할 때
- 일부 입력은 출력이 없는 조건부 매핑
- 잘못된 입력은 건너뛰는 파싱·검증
