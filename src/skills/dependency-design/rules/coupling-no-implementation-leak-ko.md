---
title: Do Not Leak Implementation Knowledge Through the Interface
impact: HIGH
impactDescription: consumers that reach into concrete internals break on every refactor and freeze the implementation in place
tags: coupling, encapsulation
---

## Do Not Leak Implementation Knowledge Through the Interface

Contents coupling(implementation-knowledge coupling이라고도 한다)은 소비자가 다른 모듈의 구체적인 내부 구현에 의존할 때 발생한다. private 필드, underscore 접두사가 붙은 속성, 내부 데이터 구조, 혹은 애초에 계약의 일부가 아니었던 본문이 그 대상이다. 그 내부를 리팩터링하면 소비자가 깨지므로 구현이 그 자리에 얼어붙는다. 이는 은닉 지식의 직접 누출이다.

대신 추상 interface에 의존하고, 무엇을 노출할지는 소유자가 결정하게 한다. 균형을 맞춰야 할 트레이드오프는 상호작용성 대 누출이다. 유용할 만큼의 동작은 노출하되, 가공되지 않은 내부 구조는 절대 노출하지 않는다. reflection과 dependency injection은 기술적으로는 내부에 침투하지만, DI framework처럼 규칙으로 통제되는 환경 안에서만 허용된다. 그곳에서는 접근이 임의적이지 않고 통제되기 때문이다.

**Incorrect:**

```ts
class OrderService {
  // private-by-convention internals
  _cache: Map<string, Order> = new Map()
  _repo = new OrderRepo()
}

// Consumer reaches into internals and depends on the cache shape.
function warmup(service: OrderService, id: string) {
  const cached = service._cache.get(id) // leaked internal structure
  if (!cached) {
    const order = service._repo.findById(id) // leaked dependency
    service._cache.set(id, order)
  }
}
```

**Correct:**

```ts
// The contract exposes behavior, not structure.
interface OrderLookup {
  get(id: string): Promise<Order>
}

class OrderService implements OrderLookup {
  #cache = new Map<string, Order>()
  #repo = new OrderRepo()

  async get(id: string): Promise<Order> {
    const cached = this.#cache.get(id)
    if (cached) return cached
    const order = await this.#repo.findById(id)
    this.#cache.set(id, order)
    return order
  }
}

// Consumer depends only on the abstract interface.
function warmup(lookup: OrderLookup, id: string) {
  return lookup.get(id)
}
```

Reference: [Coupling types and threat ranking](../references/coupling-models.md)
