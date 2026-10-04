---
title: Subtract Before You Add
impact: MEDIUM
impactDescription: building on dead code and pass-through layers ties new code to dependencies nobody needs, and every later change must carry them
tags: dependency, dead-code, refactor
---

## Subtract Before You Add

모듈, 파라미터, 레이어를 더하기 전에 이번 변경으로 죽거나 중복이 되는 것을 먼저 지운다: 쓰이지 않는 export, 호출자가 하나뿐인 wrapper, pass-through adapter, 타입이 이미 보장하는 것을 다시 검사하는 validator, 고아 참조. 그 위에 새 코드를 쌓으면 새 코드가 그것들에 의존하게 되고, 이후의 모든 변경이 늘어난 간선을 함께 짊어져야 한다.

순서가 중요하다. 먼저 빼면 바꾸려는 그래프가 작아지므로, 추가가 실제 의존성에만 얹힌다. 빼기는 그 자체로 리뷰하기도 쉽고, 추가를 작게 유지해 준다.

이 규칙은 삭제 방법이 아니라 순서에 관한 것이다. 삭제 자체는 `/refactor-clean`과 그 `refactor-cleaner` 에이전트를 쓴다.

**Incorrect:**

```ts
// The old adapter has one caller left, and the new feature is wired through it.
export function legacyFetchUser(id: string) {
  return userClient.get(id) // pass-through, adds nothing
}

export async function loadProfile(id: string) {
  const user = await legacyFetchUser(id) // new code now depends on the dead layer
  return { ...user, badges: await badgeClient.list(id) }
}
```

**Correct:**

```ts
// Step 1, its own change: delete the pass-through and point its caller at the client.
// Step 2: add the feature against the real dependency.
export async function loadProfile(id: string) {
  const [user, badges] = await Promise.all([userClient.get(id), badgeClient.list(id)])
  return { ...user, badges }
}
```

Reference: [AI-friendly ownership](../references/ai-ownership.md)
