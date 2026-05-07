---
title: Parallel Nested Data Fetching
impact: CRITICAL
impactDescription: eliminates server-side waterfalls
tags: server, rsc, parallel-fetching, promise-chaining
---

## 병렬 중첩 데이터 페칭

중첩된 데이터를 병렬로 fetch할 때는 의존하는 fetch를 각 항목의 promise 안에 체인으로 묶어, 느린 항목 하나가 나머지를 막지 않도록 한다.

**잘못된 예 (한 개의 느린 항목이 모든 중첩 fetch를 차단):**

```tsx
const chats = await Promise.all(
  chatIds.map(id => getChat(id))
)

const chatAuthors = await Promise.all(
  chats.map(chat => getUser(chat.author))
)
```

100개 중 단 하나의 `getChat(id)`가 매우 느리면, 나머지 99개의 chat 작성자 정보가 이미 준비되어 있어도 로드를 시작할 수 없다.

**올바른 예 (각 항목이 자신의 중첩 fetch를 체인):**

```tsx
const chatAuthors = await Promise.all(
  chatIds.map(id => getChat(id).then(chat => getUser(chat.author)))
)
```

각 항목이 독립적으로 `getChat` → `getUser`를 체인하므로, 느린 chat 하나가 다른 항목의 author fetch를 차단하지 않는다.
