---
title: Authenticate Server Actions Like API Routes
impact: CRITICAL
impactDescription: prevents unauthorized access to server mutations
tags: server, server-actions, authentication, security, authorization
---

## Server Action도 API Route처럼 인증한다

**Impact: CRITICAL (서버 mutation에 대한 비인가 접근을 방지)**

Server Actions(`"use server"` 함수)는 API route와 마찬가지로 공개 엔드포인트로 노출된다. 미들웨어, 레이아웃 가드, 페이지 단위 체크에만 의존하지 말고, 항상 각 Server Action **내부에서** 인증과 인가를 검증한다 — Server Actions는 직접 호출이 가능하기 때문이다.

Next.js 공식 문서는 다음과 같이 명시한다: "Treat Server Actions with the same security considerations as public-facing API endpoints, and verify if the user is allowed to perform a mutation."

**잘못된 예 (인증 체크 없음):**

```typescript
'use server'

export async function deleteUser(userId: string) {
  // Anyone can call this! No auth check
  await db.user.delete({ where: { id: userId } })
  return { success: true }
}
```

**올바른 예 (action 내부에서 인증):**

```typescript
'use server'

import { verifySession } from '@/lib/auth'
import { unauthorized } from '@/lib/errors'

export async function deleteUser(userId: string) {
  // Always check auth inside the action
  const session = await verifySession()
  
  if (!session) {
    throw unauthorized('Must be logged in')
  }
  
  // Check authorization too
  if (session.user.role !== 'admin' && session.user.id !== userId) {
    throw unauthorized('Cannot delete other users')
  }
  
  await db.user.delete({ where: { id: userId } })
  return { success: true }
}
```

**입력 검증과 함께 사용하는 예:**

```typescript
'use server'

import { verifySession } from '@/lib/auth'
import { z } from 'zod'

const updateProfileSchema = z.object({
  userId: z.string().uuid(),
  name: z.string().min(1).max(100),
  email: z.string().email()
})

export async function updateProfile(data: unknown) {
  // Validate input first
  const validated = updateProfileSchema.parse(data)
  
  // Then authenticate
  const session = await verifySession()
  if (!session) {
    throw new Error('Unauthorized')
  }
  
  // Then authorize
  if (session.user.id !== validated.userId) {
    throw new Error('Can only update own profile')
  }
  
  // Finally perform the mutation
  await db.user.update({
    where: { id: validated.userId },
    data: {
      name: validated.name,
      email: validated.email
    }
  })
  
  return { success: true }
}
```

참고: [https://nextjs.org/docs/app/guides/authentication](https://nextjs.org/docs/app/guides/authentication)
