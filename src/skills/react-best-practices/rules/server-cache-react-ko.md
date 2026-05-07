---
title: Per-Request Deduplication with React.cache()
impact: MEDIUM
impactDescription: deduplicates within request
tags: server, cache, react-cache, deduplication
---

## React.cache()로 요청 단위 중복 제거

서버 측 요청 중복 제거에는 `React.cache()`를 사용한다. 인증과 데이터베이스 쿼리에서 가장 큰 효과를 본다.

**사용 예:**

```typescript
import { cache } from 'react'

export const getCurrentUser = cache(async () => {
  const session = await auth()
  if (!session?.user?.id) return null
  return await db.user.findUnique({
    where: { id: session.user.id }
  })
})
```

단일 요청 내에서 `getCurrentUser()`를 여러 번 호출해도 쿼리는 한 번만 실행된다.

**인자로 인라인 객체를 사용하지 않는다:**

`React.cache()`는 캐시 hit 여부를 shallow equality(`Object.is`)로 판단한다. 인라인 객체는 호출마다 새 참조를 만들어 캐시 hit를 막는다.

**잘못된 예 (항상 캐시 miss):**

```typescript
const getUser = cache(async (params: { uid: number }) => {
  return await db.user.findUnique({ where: { id: params.uid } })
})

// Each call creates new object, never hits cache
getUser({ uid: 1 })
getUser({ uid: 1 })  // Cache miss, runs query again
```

**올바른 예 (캐시 hit):**

```typescript
const getUser = cache(async (uid: number) => {
  return await db.user.findUnique({ where: { id: uid } })
})

// Primitive args use value equality
getUser(1)
getUser(1)  // Cache hit, returns cached result
```

객체를 꼭 전달해야 한다면 동일한 참조를 전달한다.

```typescript
const params = { uid: 1 }
getUser(params)  // Query runs
getUser(params)  // Cache hit (same reference)
```

**Next.js 관련 참고:**

Next.js에서는 `fetch` API가 자동으로 request memoization으로 확장된다. 동일한 URL과 옵션을 가진 요청은 단일 요청 내에서 자동으로 중복 제거되므로, `fetch` 호출에는 `React.cache()`가 필요 없다. 다만 다음과 같은 다른 비동기 작업에는 여전히 `React.cache()`가 필수다:

- 데이터베이스 쿼리 (Prisma, Drizzle 등)
- 무거운 계산
- 인증 체크
- 파일 시스템 연산
- 그 밖의 fetch가 아닌 비동기 작업

이런 연산들을 컴포넌트 트리 전체에서 중복 제거하려면 `React.cache()`를 사용한다.

참고: [React.cache documentation](https://react.dev/reference/react/cache)
