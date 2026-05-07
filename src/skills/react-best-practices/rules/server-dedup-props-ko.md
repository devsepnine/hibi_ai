---
title: Avoid Duplicate Serialization in RSC Props
impact: LOW
impactDescription: reduces network payload by avoiding duplicate serialization
tags: server, rsc, serialization, props, client-components
---

## RSC props에서 중복 직렬화를 피한다

**Impact: LOW (중복 직렬화를 피해 네트워크 payload를 감소)**

RSC→client 직렬화는 객체 참조 기준으로 중복 제거한다 — 값 기준이 아니다. 동일 참조 = 한 번만 직렬화, 새 참조 = 다시 직렬화. `.toSorted()`, `.filter()`, `.map()` 같은 변환은 서버가 아닌 클라이언트에서 수행한다.

**잘못된 예 (배열을 중복 전송):**

```tsx
// RSC: sends 6 strings (2 arrays × 3 items)
<ClientList usernames={usernames} usernamesOrdered={usernames.toSorted()} />
```

**올바른 예 (3개 문자열만 전송):**

```tsx
// RSC: send once
<ClientList usernames={usernames} />

// Client: transform there
'use client'
const sorted = useMemo(() => [...usernames].sort(), [usernames])
```

**중첩 중복 제거 동작:**

중복 제거는 재귀적으로 적용된다. 데이터 타입에 따라 영향이 다르다.

- `string[]`, `number[]`, `boolean[]`: **HIGH 영향** — 배열과 모든 primitive가 완전히 중복됨
- `object[]`: **LOW 영향** — 배열 자체는 중복되지만 중첩 객체는 참조 기준으로 중복 제거됨

```tsx
// string[] - duplicates everything
usernames={['a','b']} sorted={usernames.toSorted()} // sends 4 strings

// object[] - duplicates array structure only
users={[{id:1},{id:2}]} sorted={users.toSorted()} // sends 2 arrays + 2 unique objects (not 4)
```

**중복 제거를 깨뜨리는 연산 (새 참조 생성):**

- 배열: `.toSorted()`, `.filter()`, `.map()`, `.slice()`, `[...arr]`
- 객체: `{...obj}`, `Object.assign()`, `structuredClone()`, `JSON.parse(JSON.stringify())`

**추가 예시:**

```tsx
// Bad
<C users={users} active={users.filter(u => u.active)} />
<C product={product} productName={product.name} />

// Good
<C users={users} />
<C product={product} />
// Do filtering/destructuring in client
```

**예외:** 변환 비용이 크거나 클라이언트가 원본을 필요로 하지 않을 때는 derived data를 그대로 전달한다.
