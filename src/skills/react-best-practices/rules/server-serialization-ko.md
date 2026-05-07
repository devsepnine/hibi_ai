---
title: Minimize Serialization at RSC Boundaries
impact: HIGH
impactDescription: reduces data transfer size
tags: server, rsc, serialization, props
---

## RSC 경계에서 직렬화를 최소화한다

React Server/Client 경계는 모든 객체 속성을 문자열로 직렬화해 HTML 응답과 이후의 RSC 요청에 포함시킨다. 이렇게 직렬화된 데이터는 페이지 무게와 로드 시간에 직접 영향을 미치므로 **크기가 매우 중요하다**. 클라이언트가 실제로 사용하는 필드만 전달한다.

**잘못된 예 (50개 필드 모두 직렬화):**

```tsx
async function Page() {
  const user = await fetchUser()  // 50 fields
  return <Profile user={user} />
}

'use client'
function Profile({ user }: { user: User }) {
  return <div>{user.name}</div>  // uses 1 field
}
```

**올바른 예 (1개 필드만 직렬화):**

```tsx
async function Page() {
  const user = await fetchUser()
  return <Profile name={user.name} />
}

'use client'
function Profile({ name }: { name: string }) {
  return <div>{name}</div>
}
```
