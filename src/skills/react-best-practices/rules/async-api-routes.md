---
title: Prevent Waterfall Chains in API Routes
impact: CRITICAL
impactDescription: 2-10× improvement
tags: api-routes, server-actions, waterfalls, parallelization
---

## Prevent Waterfall Chains in API Routes

Start independent async operations together and attach them to `Promise.all` before the first `await`. A rejection from either operation then reaches the caller.

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

For operations with more complex dependency chains, use `better-all` to automatically maximize parallelism. See Dependency-Based Parallelization.
