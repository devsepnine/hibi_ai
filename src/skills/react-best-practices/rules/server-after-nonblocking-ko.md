---
title: Use after() for Non-Blocking Operations
impact: MEDIUM
impactDescription: faster response times
tags: server, async, logging, analytics, side-effects
---

## 논블로킹 작업에는 after()를 사용한다

응답이 전송된 후 실행되어야 할 작업은 Next.js의 `after()`로 스케줄링한다. 이를 통해 로깅, analytics 같은 사이드 이펙트가 응답을 차단하지 않도록 한다.

**잘못된 예 (응답을 차단):**

```tsx
import { logUserAction } from '@/app/utils'

export async function POST(request: Request) {
  // Perform mutation
  await updateDatabase(request)
  
  // Logging blocks the response
  const userAgent = request.headers.get('user-agent') || 'unknown'
  await logUserAction({ userAgent })
  
  return new Response(JSON.stringify({ status: 'success' }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  })
}
```

**올바른 예 (논블로킹):**

```tsx
import { after } from 'next/server'
import { headers, cookies } from 'next/headers'
import { logUserAction } from '@/app/utils'

export async function POST(request: Request) {
  // Perform mutation
  await updateDatabase(request)
  
  // Log after response is sent
  after(async () => {
    const userAgent = (await headers()).get('user-agent') || 'unknown'
    const sessionCookie = (await cookies()).get('session-id')?.value || 'anonymous'
    
    logUserAction({ sessionCookie, userAgent })
  })
  
  return new Response(JSON.stringify({ status: 'success' }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  })
}
```

응답은 즉시 전송되고 로깅은 백그라운드에서 처리된다.

**대표적인 사용 사례:**

- Analytics 추적
- 감사 로그(audit logging)
- 알림 전송
- 캐시 무효화
- 정리(cleanup) 작업

**중요 사항:**

- `after()`는 응답이 실패하거나 redirect되어도 실행된다
- Server Actions, Route Handlers, Server Components에서 동작한다

참고: [https://nextjs.org/docs/app/api-reference/functions/after](https://nextjs.org/docs/app/api-reference/functions/after)
