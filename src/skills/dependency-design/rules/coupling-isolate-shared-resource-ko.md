---
title: Isolate Shared-Resource Coupling (Common/External)
impact: MEDIUM
impactDescription: many modules sharing one mutable resource creates bottlenecks and maintenance traps that only resource isolation resolves
tags: coupling, singleton, state
---

## Isolate Shared-Resource Coupling (Common/External)

두 가지 결합 유형은 하나의 근본 원인을 공유한다. Common coupling은 여러 모듈이 단일 자원을 참조하면서 그 자원의 전체 구조를 알고 있는 경우다(여기저기서 읽는 전형적인 global mutable singleton이나 공유 config). External coupling은 같은 단일 자원을 서로 다른 모듈이 목적별로 다르게 사용하는 경우다. 둘 다 실제 병목이다. 공유 singleton은 관리하기 어렵고 유지보수는 더 어렵다.

핵심 통찰은, 치료의 대상이 단순한 "전체 구조 공유"가 아니라 "단일 자원 공유"라는 점이다. 모두가 여전히 같은 자원을 두고 경쟁한다면 구조를 DTO 뒤로 숨겨도 병목은 사라지지 않는다. 대규모 설계가 MSA 같은 접근을 도입하는 이유가 바로 이것이다. 단일 공유 자원을 해소하기 위함이다. 코드 수준에서는 접근을 좁은 소유자 뒤로 한정해 각 소비자가 필요한 것만 건드리게 한다. 터미널 로직에는 instance를 선호하라. static이나 global 상태는 event loop, global queue, message queue처럼 단일 공유 instance가 올바른 모델인 진짜 상위 레이어 인프라에만 남겨 둔다.

**Incorrect:**

```ts
// Common/External: one global mutable object, read and written everywhere.
export const appState = {
  currentUser: null as User | null,
  db: null as DbConnection | null,
  featureFlags: {} as Record<string, boolean>,
}

function checkout() {
  if (!appState.currentUser) throw new Error("no user")
  appState.db!.insert(/* ... */) // every module shares the same connection object
}
```

**Correct:**

```ts
// Each consumer depends on a narrow owner, not a shared global blob.
interface UserContext {
  current(): User
}
interface OrderStore {
  insert(order: Order): Promise<void>
}

// Terminal logic uses injected instances scoped to the request.
function checkout(users: UserContext, orders: OrderStore, order: Order) {
  const user = users.current()
  return orders.insert({ ...order, userId: user.id })
}

// Static/global is reserved for genuine upper-layer infrastructure.
class MessageBus {
  static readonly instance = new MessageBus() // one queue is the correct model
  publish(event: DomainEvent) {
    /* ... */
  }
}
```

Reference: [Coupling types and threat ranking](../references/coupling-models.md)
