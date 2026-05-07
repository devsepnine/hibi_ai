---
title: Cross-Request LRU Caching
impact: HIGH
impactDescription: caches across requests
tags: server, cache, lru, cross-request
---

## 요청 간 LRU 캐싱

`React.cache()`는 한 요청 내에서만 동작한다. 순차적인 요청들 사이에서 공유될 데이터(사용자가 버튼 A를 누르고 다음에 버튼 B를 누르는 시나리오)에는 LRU 캐시를 사용한다.

**구현:**

```typescript
import { LRUCache } from 'lru-cache'

const cache = new LRUCache<string, any>({
  max: 1000,
  ttl: 5 * 60 * 1000  // 5 minutes
})

export async function getUser(id: string) {
  const cached = cache.get(id)
  if (cached) return cached

  const user = await db.user.findUnique({ where: { id } })
  cache.set(id, user)
  return user
}

// Request 1: DB query, result cached
// Request 2: cache hit, no DB query
```

수 초 이내에 여러 엔드포인트가 동일한 데이터를 필요로 하는 순차적 사용자 액션에서 사용한다.

**Vercel [Fluid Compute](https://vercel.com/docs/fluid-compute) 환경:** 여러 동시 요청이 동일 함수 인스턴스와 캐시를 공유할 수 있어 LRU 캐싱 효과가 특히 크다. Redis 같은 외부 저장소 없이도 요청 간에 캐시가 유지된다.

**전통적인 서버리스 환경:** 각 invocation이 격리되어 실행되므로, 프로세스 간 캐싱이 필요하다면 Redis를 고려한다.

참고: [https://github.com/isaacs/node-lru-cache](https://github.com/isaacs/node-lru-cache)
