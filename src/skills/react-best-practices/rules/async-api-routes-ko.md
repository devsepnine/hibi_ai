---
title: Prevent Waterfall Chains in API Routes
impact: CRITICAL
impactDescription: 2-10× improvement
tags: api-routes, server-actions, waterfalls, parallelization
---

## Prevent Waterfall Chains in API Routes

독립적인 비동기 작업은 함께 시작하고 첫 `await` 전에 `Promise.all`에 연결한다. 먼저 실패한 작업의 오류도 라우트의 호출자에게 전달된다.

**Incorrect, config waits for auth, data waits for both:**

```typescript
export async function GET(request: Request) {
  const session = await auth()
  const config = await fetchConfig()
  const data = await fetchData(session.user.id)
  return Response.json({ data, config })
}
```

**Correct, auth and config start immediately:**

```typescript
export async function GET(request: Request) {
  const [config, data] = await Promise.all([
    fetchConfig(),
    auth().then(session => fetchData(session.user.id)),
  ])
  return Response.json({ data, config })
}
```

더 복잡한 의존성 체인을 가진 작업이라면 `better-all`을 사용해 자동으로 병렬성을 극대화한다. Dependency-Based Parallelization을 참고한다.
