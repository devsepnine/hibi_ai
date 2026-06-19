# Dependency & Coupling Design

**Version 1.0.0**  
Engineering  
January 2026

> **Note:**  
> 이 문서는 주로 에이전트와 LLM이 모듈 경계, dependency, coupling을 설계,  
> 유지보수, 리팩토링할 때 따르도록 작성되었다. 사람도 유용하게 활용할 수  
> 있지만, 여기의 가이드는 AI 보조 워크플로우의 자동화와 일관성에 최적화되어  
> 있다.

---

## Abstract

시스템이 확장되어도 변경 가능한 상태를 유지하도록 dependency와 coupling을 잡아 가는 규칙이다. 문제를 얼마나 잘 이해하고 있는지에 coupling 전략을 맞추고, 해로운 coupling을 순위 매겨 줄이며, dependency를 단방향이고 비순환적으로 유지하고, 모듈 경계 전반에 최소한의 추상화된 지식을 공개하며, app에서 공유 package로 흐르는 단방향 dependency로 layered 및 monorepo 아키텍처를 구성한다. 궁극적 목표는 AI agent가 부분 소유할 수 있는 코드다 — 격리된 부분 집합을 로딩해 온전히 이해하고 제한된 context window 안에서 수정할 수 있어야 한다.

이 문서는 `rules/` 아래의 개별 rule 파일들을 `rules/_sections.md`에 정의된 섹션별로 묶어 집계한 생성 산출물이다. 각 rule은 제목, impact, 설명, Incorrect/Correct 예제와 함께 재수록된다.

---

## Table of Contents

1. [Complexity & Context (Cynefin)](#1-complexity--context-cynefin) — **HIGH**
   - 1.1 [Secure a Test Safety Net Before Changing Complex Code](#11-secure-a-test-safety-net-before-changing-complex-code)
2. [Coupling Types & Threat Ranking](#2-coupling-types--threat-ranking) — **CRITICAL**
   - 2.1 [Avoid Control Coupling (Flag Arguments Leaking Internal Structure)](#21-avoid-control-coupling-flag-arguments-leaking-internal-structure)
   - 2.2 [Do Not Leak Implementation Knowledge Through the Interface](#22-do-not-leak-implementation-knowledge-through-the-interface)
   - 2.3 [Pass Only the Data Needed (Data over Stamp), Avoid Train-Wreck Bridges](#23-pass-only-the-data-needed-data-over-stamp-avoid-train-wreck-bridges)
   - 2.4 [Isolate Shared-Resource Coupling (Common/External)](#24-isolate-shared-resource-coupling-commonexternal)
3. [Dependency Direction & Structure](#3-dependency-direction--structure) — **CRITICAL**
   - 3.1 [Keep Dependencies Unidirectional and Acyclic](#31-keep-dependencies-unidirectional-and-acyclic)
   - 3.2 [Isolate Modules by Responsibility (Change-Rate)](#32-isolate-modules-by-responsibility-change-rate)
   - 3.3 [Depend in the Direction of Stability (DDD Subdomains)](#33-depend-in-the-direction-of-stability-ddd-subdomains)
4. [Abstraction & Module Boundary](#4-abstraction--module-boundary) — **HIGH**
   - 4.1 [Keep Abstraction Criteria and Level Consistent](#41-keep-abstraction-criteria-and-level-consistent)
   - 4.2 [Minimize Context: Publish Abstracted, Not Concrete, Knowledge](#42-minimize-context-publish-abstracted-not-concrete-knowledge)
   - 4.3 [Classify Domain-Specific vs General Knowledge, Share via Contract](#43-classify-domain-specific-vs-general-knowledge-share-via-contract)
5. [Layered & Monorepo Architecture](#5-layered--monorepo-architecture) — **MEDIUM**
   - 5.1 [Layered Architecture: One-Way Dependency, Watch N:N Mapping](#51-layered-architecture-one-way-dependency-watch-nn-mapping)
   - 5.2 [Turbo Monorepo: apps -> packages One-Way, No lib -> lib](#52-turbo-monorepo-apps---packages-one-way-no-lib---lib)
   - 5.3 [Control Interconnection Complexity: Linear Flow + Message Constraints](#53-control-interconnection-complexity-linear-flow--message-constraints)
6. [AI-Friendly Ownership](#6-ai-friendly-ownership) — **MEDIUM**
   - 6.1 [Structure Code for AI Partial Ownership](#61-structure-code-for-ai-partial-ownership)

---

## 1. Complexity & Context (Cynefin)

**Impact: HIGH**

문제를 얼마나 잘 이해하고 있는지에 coupling 전략을 맞추며, Cynefin 도메인을 사용해 사전 구조화와 결정 보류 사이를 선택한다.

### 1.1 Secure a Test Safety Net Before Changing Complex Code

**Impact: HIGH (prevents silent runtime regressions in code the compiler cannot verify)**

Cynefin 프레임워크는 변경의 영향을 얼마나 잘 이해하고 있는지에 따라 상황을 분류하며,
각 구간마다 다른 전략을 적용한다.

- **Clear** — 변경이 아무런 파급을 일으키지 않음을 확신한다. 프로토콜 범위 안에서 안전하게 수정한다.
- **Complicated** — 파급이 컴파일 시점에 드러난다. 타입 시스템이 모든 caller를 짚어 주므로, 영향받는 대상과 합의한 뒤 수정한다.
- **Complex** — 파급이 컴파일러가 잡지 못하는 *런타임* 결합이다(event, 공유 상태, 순서, side effect). 코드를 실행해야만 무엇이 깨졌는지 알 수 있다.
- **Chaotic** — 결합이 통제되지 않은 상태다. 영향을 예측할 신뢰할 만한 방법이 전혀 없다.

함정은 complex 코드를 clear인 것처럼 다루는 것이다. 영향이 런타임에 존재하는데도
테스트 없이 수정하고 "나중에 QA가 찾겠지"라고 미루는 것은 전략이 아니라, 스스로
chaotic 구간에서 작업하기를 선택하는 것이다. 변경을 *chaotic*이 아니라 *complex*로
유지하는 규율은 안전망이다. runtime-coupled 로직을 건드리기 전에, 현재 동작을 고정하는
characterization test를 추가하거나 확인하라. 그런 다음 그 위에서 refactor하라. 실패하는
테스트는 일주일 뒤 도착할 지원 티켓 대신, 지금 당장 어떤 런타임 계약을 깨뜨렸는지를
정확히 알려 준다.

따라서 complex 코드에 대한 규칙은 이렇다. 동작을 먼저 고정하고, 그다음에 변경하라.
기존 테스트가 이미 해당 경로를 덮고 있다면 실행해서 green을 확인한다. 덮고 있지 않다면,
오늘의 동작이 비록 보기 좋지 않더라도 그것을 그대로 포착하는 characterization test를
작성한 뒤에만 수정한다.

**Incorrect:**

```ts
// Complex code: an order's total quietly depends on the discount engine,
// the tax service, and a mutable cart cache — none of it compile-checked.
// Editing it with no test and trusting QA to catch fallout = chaotic by choice.
function applyLoyaltyBonus(order: Order): Order {
  // New rule: loyalty members get an extra 5% off.
  order.discountRate += 0.05            // mutates shared cart-cache state
  order.total = order.subtotal * (1 - order.discountRate)
  // Forgot: tax is computed downstream from `total` in another module.
  // Forgot: discountRate is also read by the invoice PDF generator.
  return order
}
// Shipped. No test pinned the old total or tax behavior, so the
// double-applied discount and wrong tax only surface in production.
```

**Correct:**

```ts
// 1. Pin current behavior with a characterization test BEFORE editing.
describe('applyLoyaltyBonus (characterization)', () => {
  it('keeps total, tax, and invoice in sync for a member order', () => {
    const order = makeOrder({ subtotal: 100, discountRate: 0.1, loyalty: true })

    const result = applyLoyaltyBonus(order)

    // Lock the runtime contracts the compiler cannot see:
    expect(result.total).toBe(85)              // 100 * (1 - 0.15)
    expect(computeTax(result)).toBe(8.5)       // downstream module
    expect(renderInvoice(result).discountLine).toBe('15% off') // PDF generator
  })
})

// 2. Now refactor against the net. A broken runtime contract fails loudly here,
//    not in production.
function applyLoyaltyBonus(order: Order): Order {
  const discountRate = order.discountRate + (order.loyalty ? 0.05 : 0)
  const total = order.subtotal * (1 - discountRate)
  return { ...order, discountRate, total } // no shared-state mutation
}
```

characterization test, coverage 목표, 무엇을 단언할지 등 더 넓은 테스트 규율은
`tdd-workflow`와 `coding-standards` skill을 참고한다.

---

## 2. Coupling Types & Threat Ranking

**Impact: CRITICAL**

해로운 coupling을 식별하고 줄인다; 현대적 위협 순서는 Control > External > Common > Contents > Stamp > Data이다.

### 2.1 Avoid Control Coupling (Flag Arguments Leaking Internal Structure)

**Impact: CRITICAL (control flags let a caller steer the callee's internal branches, creating the hardest-to-untangle coupling in modern code)**

Control coupling은 호출자가 오직 피호출자의 어떤 내부 분기를 실행할지 고르기 위한 flag나 mode 인자를 넘길 때 발생한다. 이 flag는 피호출자의 내부 구조를 호출 지점으로 유출시킨다. 즉, 호출자가 올바르게 호출하려면 피호출자의 분기들을 알아야 하고, 분기가 하나 늘 때마다 새로운 flag가 강요된다. 고전적인 결합 유형 중에서 이것이 가장 심각한 현대적 위협이다. 그 결과 생기는 의존성이 가장 복잡하고 해소하기 가장 어렵기 때문이다. 실제로 flag를 통한 은닉 지식의 간접 누출은 직접 누출보다 더 나쁘다.

해결책은 조종 자체를 제거하는 것이다. 동작을 의도를 드러내는 함수들로 분리하거나, strategy 혹은 polymorphism으로 제어를 역전시켜 호출자가 내부 경로를 토글하는 대신 동작 객체를 선택하게 한다. 지엽적인 strategy와 전체적인 inversion of control은 모두 현대 설계의 핵심 도구다.

**Incorrect:**

```ts
// The boolean flags drive a switch on the callee's internal structure.
// Callers must know every branch to call this correctly.
function generateReport(
  data: SalesRow[],
  isAdmin: boolean,
  exportAsPdf: boolean,
): Buffer | string {
  let rows = data
  if (isAdmin) {
    rows = data // admins see raw rows including margins
  } else {
    rows = data.map((r) => ({ ...r, margin: undefined }))
  }

  if (exportAsPdf) {
    return renderPdf(rows)
  } else {
    return renderCsv(rows)
  }
}

// Call sites are unreadable and must track flag order/meaning.
generateReport(rows, true, false)
generateReport(rows, false, true)
```

**Correct:**

```ts
// Intention-revealing functions: no flag steers the internals.
function adminRows(data: SalesRow[]): SalesRow[] {
  return data
}
function viewerRows(data: SalesRow[]): SalesRow[] {
  return data.map((r) => ({ ...r, margin: undefined }))
}

// Strategy object: the caller injects the format behavior
// instead of toggling an internal branch.
interface ReportFormat {
  render(rows: SalesRow[]): Buffer | string
}
const PdfFormat: ReportFormat = { render: (rows) => renderPdf(rows) }
const CsvFormat: ReportFormat = { render: (rows) => renderCsv(rows) }

function generateReport(rows: SalesRow[], format: ReportFormat) {
  return format.render(rows)
}

// Call sites read as intent, and new formats add no flags.
generateReport(adminRows(rows), CsvFormat)
generateReport(viewerRows(rows), PdfFormat)
```

### 2.2 Do Not Leak Implementation Knowledge Through the Interface

**Impact: HIGH (consumers that reach into concrete internals break on every refactor and freeze the implementation in place)**

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

### 2.3 Pass Only the Data Needed (Data over Stamp), Avoid Train-Wreck Bridges

**Impact: MEDIUM (passing whole structures and chaining through them couples callers to shapes they never use)**

Stamp coupling은 피호출자가 한 필드만 필요한데 전체 구조를 넘기는 것이다. 이제 피호출자는 사용하지도 않는 구조에 결합되고, 그 구조가 바뀌면 파장이 바깥으로 퍼진다. 실제로 필요한 값만 넘기는 data coupling을 선호하라.

현대 개발에서 stamp coupling의 약한 형태는 더 이상 누출로 취급되지 않는다. 불변 값 객체(record 스타일 구조)를 노출하는 것은 결함이라기보다 설계 의사결정에 가깝다. 진짜 위험은 train-wreck bridge다. `a.b.c.d`처럼 여러 객체를 가로질러 뻗어 나가며 호출자를 중간 구조 전체에 결합시키는 체인이다. 각 연결 고리는 따로따로 바뀔 수 있는 별개의 대상이다. 호출자가 그래프를 탐색하도록 강요하지 말고, 값의 소유자가 그 값을 직접 노출하게 하라.

**Incorrect:**

```ts
// Stamp: the whole order is passed to compute one number.
function computeTax(order: Order): number {
  return order.total * 0.1
}

// Train-wreck bridge: the caller is coupled to four nested shapes.
const zip = order.customer.address.location.zipCode
```

**Correct:**

```ts
// Data: pass only the value the function needs.
function computeTax(orderTotal: number): number {
  return orderTotal * 0.1
}
computeTax(order.total)

// The owner exposes the needed value; no deep navigation at the call site.
class Order {
  constructor(private readonly customer: Customer) {}
  get shippingZip(): string {
    return this.customer.shippingZip()
  }
}
const zip = order.shippingZip

// Immutable exposure of a small value object is fine, not leakage.
type Money = Readonly<{ amount: number; currency: string }>
function format(price: Money): string {
  return `${price.amount} ${price.currency}`
}
```

### 2.4 Isolate Shared-Resource Coupling (Common/External)

**Impact: MEDIUM (many modules sharing one mutable resource creates bottlenecks and maintenance traps that only resource isolation resolves)**

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

---

## 3. Dependency Direction & Structure

**Impact: CRITICAL**

dependency를 단방향이고 비순환적이며 변경 빈도별로 격리되도록 유지해, 변동이 잦은 부분이 안정적인 부분을 끌고 가지 못하게 한다.

### 3.1 Keep Dependencies Unidirectional and Acyclic

**Impact: CRITICAL (cyclic dependencies make change ripple unpredictable and break causal ordering)**

의존성 그래프는 한 모듈의 변경이 다른 모듈로 어떻게 파급되는지를 나타내는 모델이다. 두 모듈이 서로를 의존하거나, 더 긴 순환이 자기 자신으로 되돌아오면(`A -> B -> C -> A`) 파급은 더 이상 예측할 수 없게 된다. 순환에 속한 어떤 노드를 바꾸든 변경이 고리를 한 바퀴 돌아 다시 자신에게 돌아올 수 있다. 여러 모듈을 거치는 간접 순환도 직접 순환만큼 해롭다. 고리의 길이가 결합을 완화해 주는 것이 아니라 단지 숨길 뿐이다.

순환은 인과적 순서도 무너뜨린다. 단방향 간선은 "이것이 저것 위에 세워졌다"는 의미를 담는다. 즉 의존 대상이 먼저 초기화되고, 실행되고, 추론되어야 한다. 순환에는 그런 순서가 없으므로 초기화 순서, 빌드 순서, 그리고 머릿속 이해 모델까지 모두 모호해진다.

두 모듈이 정말로 무언가를 공유해야 한다면, 그 공유 관심사를 두 모듈이 함께 의존하는 더 낮은 수준의 모듈로 추출한다. 이렇게 하면 의존성은 여전히 한 방향으로 흐른다. 두 모듈은 모두 공유 모듈을 아래로 가리키고, 공유 모듈은 그 어느 쪽도 가리키지 않는다.

**Incorrect:**

```typescript
// orderModule.ts
import { notifyUser } from "./userModule"
export function placeOrder(order: Order) {
  save(order)
  notifyUser(order.userId, "Order placed")
}

// userModule.ts  ->  cycle: orderModule <-> userModule
import { getOrdersFor } from "./orderModule"
export function notifyUser(userId: string, msg: string) {
  const open = getOrdersFor(userId)
  send(userId, `${msg} (${open.length} open)`)
}
```

**Correct:**

```typescript
// notifier.ts  — shared lower module, depends on neither caller
export function notifyUser(userId: string, msg: string) {
  send(userId, msg)
}

// orderModule.ts  — depends down on notifier and userQuery
import { notifyUser } from "./notifier"
import { getOrdersFor } from "./userQuery"
export function placeOrder(order: Order) {
  save(order)
  const open = getOrdersFor(order.userId)
  notifyUser(order.userId, `Order placed (${open.length} open)`)
}

// userQuery.ts  — depends on nothing above it
export function getOrdersFor(userId: string): Order[] {
  return query(userId)
}
```

### 3.2 Isolate Modules by Responsibility (Change-Rate)

**Impact: HIGH (co-locating different change-rates couples their lifecycles and widens every change's blast radius)**

모듈의 책임은 *변경되는 이유*로 정의하는 것이 가장 좋다. 그리고 변경되는 이유는 실제로는 그 모듈의 변경 빈도(change-rate)로 드러난다. 변동성이 큰 비즈니스 규칙은 매주 바뀌고, 인프라 어댑터는 거의 바뀌지 않는다. 변경 빈도가 다른 두 가지를 한곳에 두면 두 생명주기가 결합된다. 빠르게 변하는 부분을 한 번 수정할 때마다, 움직일 이유가 전혀 없던 느리게 변하는 부분까지 다시 읽고, 다시 테스트하고, 위험을 감수해야 한다.

모듈을 변경 빈도 경계를 따라 분리한다. 분리 후에는 변동성 큰 규칙의 변경이 변동성 모듈만 건드리며, 안정적인 모듈은 손대지 않으므로 신뢰성을 유지한다. 이것은 단일 책임 원칙(Single Responsibility Principle)을 *시간*의 관점에서 읽은 것이다. 함께 변하는 것은 묶고, 따로 변하는 것은 분리한다.

이러한 모듈을 깔끔하게 나누기 위한 네이밍과 파일 구조 규칙은 `coding-standards` skill을 참고한다.

**Incorrect:**

```typescript
// pricing.ts — mixes a volatile rule with stable infra in one module
export class Pricing {
  // volatile: promo logic changes every campaign
  applyDiscount(cart: Cart): number {
    if (cart.total > 100 && isBlackFriday()) return cart.total * 0.7
    return cart.total
  }

  // stable: how we talk to the gateway almost never changes
  async charge(amount: number, card: Card): Promise<Receipt> {
    return this.gateway.post("/charge", { amount, token: card.token })
  }
}
```

**Correct:**

```typescript
// discountPolicy.ts — volatile: the only place campaign rules live
export function applyDiscount(cart: Cart): number {
  if (cart.total > 100 && isBlackFriday()) return cart.total * 0.7
  return cart.total
}

// paymentGateway.ts — stable: untouched when a campaign changes
export class PaymentGateway {
  async charge(amount: number, card: Card): Promise<Receipt> {
    return this.client.post("/charge", { amount, token: card.token })
  }
}

// checkout.ts — composes the two; depends down on both
const total = applyDiscount(cart)
await gateway.charge(total, card)
```

### 3.3 Depend in the Direction of Stability (DDD Subdomains)

**Impact: HIGH (pointing dependencies from volatile to stable code keeps churn from rippling into trusted modules)**

의존성은 안정성을 향해야 한다. 자주 변하는 모듈이 거의 변하지 않는 모듈을 의존해야 하며, 그 반대가 되어서는 안 된다. 안정적인 모듈이 변동성 큰 모듈을 import하면, 변동성 코드가 흔들릴 때마다 안정적인 코드까지 함께 끌려가서 애써 얻은 안정성을 잃게 된다.

DDD distillation은 각 종류의 코드가 어디에 위치하는지에 대한 실용적인 지도를 제공한다.

- **Core subdomain** — 핵심 경쟁력이며 Cynefin 기준 *complex* 이상의 복잡성을 가진다. 자주 변경되므로 Core 도메인 모듈이 더 안정적인 하위 도메인을 *바깥으로* 의존하는 것이 바람직하다.
- **Generic subdomain** — 굳건한 인프라성 코드로 *complicated* 이하이며, 정기적인 기술 부채 해소 주기로 관리된다. 관계에서 `model` 이상의 결합을 지향한다.
- **Supporting subdomain** — *clear* 복잡성으로 거의 변동이 없다. 안정적인 솔루션이며 `contract` 결합이 이상적이다.

따라서 변동성 큰 Core가 안정적인 Generic 및 Supporting 하위 도메인을 가리킨다. 유틸리티(Generic/Supporting)는 반드시 도메인에 비종속적으로 유지되어야 한다. 빠르게 변하는 도메인 타입을 import하는 순간, 그 도메인의 변경 빈도를 그대로 물려받기 때문이다.

**Incorrect:**

```typescript
// dateUtils.ts — a stable, generic utility...
import { Invoice } from "../billing/invoice" // ...importing a volatile domain type

// now every billing rule change can force dateUtils to recompile and retest
export function dueDateFor(invoice: Invoice): Date {
  return addDays(invoice.issuedAt, invoice.terms.netDays)
}
```

**Correct:**

```typescript
// dateUtils.ts — generic, domain-agnostic; depends on nothing above it
export function addDays(date: Date, days: number): Date {
  const d = new Date(date)
  d.setDate(d.getDate() + days)
  return d
}

// billing/invoice.ts — volatile Core, depends DOWN on the stable utility
import { addDays } from "../utils/dateUtils"
export function dueDateFor(invoice: Invoice): Date {
  return addDays(invoice.issuedAt, invoice.terms.netDays)
}
```

---

## 4. Abstraction & Module Boundary

**Impact: HIGH**

모듈 경계 전반에 일관되고 최소이며 추상화된 지식을 공개해, 호출자가 구현이 아니라 의도에 의존하도록 한다.

### 4.1 Keep Abstraction Criteria and Level Consistent

**Impact: HIGH (inconsistent abstraction levels make a module split worthless and re-tangle the very complexity modularization was meant to remove)**

코드를 모듈로 나누는 것을 정당화하는 유일한 근거는 추상화다. 모듈은 하나의 일관된 관점 뒤에 세부 사항을 숨긴다. 그 추상화의 기준이나 수준이 일관되지 않으면 — 고수준 orchestration이 저수준 byte 조작과 나란히 놓이거나, 도메인 API가 raw HTTP까지 함께 다루면 — 분리는 아무것도 얻지 못한다. 이제 호출자는 두 고도를 동시에 추론해야 하므로, 그 모듈은 자신이 대체한 평면 코드보다 오히려 이해하기 어려워진다.

모듈마다 추상화 기준을 하나만 선택하고, interface마다 수준을 하나만 선택하라. 모듈이 노출하는 모든 것은 같은 고도에서 문제를 설명해야 한다. 더 낮은 수준의 세부 사항은 그 자체의 일관된 경계 뒤, 별도 모듈로 밀어 넣어라.

**Incorrect:**

```ts
// One "service" mixes domain orchestration, transport details, and byte work.
class OrderService {
  async place(order: Order): Promise<void> {
    // high-level domain verb
    this.validate(order)

    // mid-level: HTTP transport details leaking into the domain object
    const res = await fetch("https://pay.example.com/v2/charge", {
      method: "POST",
      headers: { "Idempotency-Key": crypto.randomUUID() },
      body: JSON.stringify({ amount_cents: order.totalCents }),
    })
    if (res.status === 429) await this.backoff() // transport concern

    // low-level: manual byte framing for an audit log
    const buf = Buffer.alloc(8)
    buf.writeBigUInt64BE(BigInt(order.id))
    this.auditFd.write(buf)
  }
}
```

**Correct:**

```ts
// OrderService stays at the domain level. Lower altitudes live behind their own
// modules, each with one consistent abstraction.
class OrderService {
  constructor(
    private readonly payments: PaymentGateway, // domain-level contract
    private readonly audit: AuditLog, // domain-level contract
  ) {}

  async place(order: Order): Promise<void> {
    this.validate(order)
    await this.payments.charge(order.id, order.totalCents)
    await this.audit.record("order.placed", order.id)
  }
}

// HTTP/retry details are isolated — that module is consistently transport-level.
class HttpPaymentGateway implements PaymentGateway {
  async charge(orderId: string, amountCents: number): Promise<void> {
    /* fetch, idempotency, backoff live only here */
  }
}

// Byte framing is isolated — that module is consistently encoding-level.
class BinaryAuditLog implements AuditLog {
  async record(event: string, id: string): Promise<void> {
    /* buffer writes live only here */
  }
}
```

하나의 분리 관점을 고르고 그 기준으로 모듈을 계층화하는 방법은 `backend-patterns` skill을 참고하라.

### 4.2 Minimize Context: Publish Abstracted, Not Concrete, Knowledge

**Impact: MEDIUM (every concrete assumption a module forces its caller to satisfy becomes hidden coupling that breaks silently when the implementation changes)**

모듈은 세 종류의 지식으로 이루어진다. 상호작용을 위해 공개하는 **interface**, 내부에 숨기는 실제 지식인 **implementation**, 그리고 모듈이 구현하지 않은 채 주변 환경에 암묵적으로 요구하는 가정인 **context**다. 위험한 부분은 context다. 그것은 문서화되지 않고, 컴파일되지 않으며, 검사되지 않는다. 그래서 호출자는 어떤 오류도 없이 그것을 위반할 수 있고, 문제는 runtime에 가서야 드러난다.

interface가 구체적인 타입과 구체적인 가정을 노출하면, 호출자는 모듈을 올바르게 쓰기 위해 그 context를 흡수해야 하고 복잡성은 사방에서 올라간다. 대신 추상화된 contract를 공개하라. 연산을 호출자의 용어로 명명하고, 상호작용에 필요한 데이터만 정확히 받으며, 순서·단위·생성 세부 사항은 implementation 안에 보관하라. context를 최소화하는 것이야말로 모듈을 명확하고 신뢰할 수 있게 쓰도록 만든다.

**Incorrect:**

```ts
// Interface leaks concrete types and unstated assumptions (the "context").
interface PriceCalculator {
  // Caller must know: cents not dollars, that taxTable is pre-sorted,
  // that init() must run first, and how to build a raw DiscountRow.
  taxTable: TaxRow[]
  init(): void
  calc(amountCents: number, region: string, rows: DiscountRow[]): number
}

// Every caller now carries that hidden context:
calculator.init() // forget this and you get NaN
const total = calculator.calc(1999, "US-CA", buildDiscountRows(cart)) // cents? rows shape?
```

**Correct:**

```ts
// Abstracted contract: caller speaks in domain terms, context stays internal.
interface PriceCalculator {
  // Money is a self-describing value object; no init order, no raw rows.
  total(cart: Cart, region: Region): Money
}

// The caller needs no hidden knowledge — the interface says everything.
const total = calculator.total(cart, region)
```

어떤 지식을 공개할지(general 대 domain-specific)의 선택은 `abstraction-encapsulate-knowledge`에서 다룬다.

### 4.3 Classify Domain-Specific vs General Knowledge, Share via Contract

**Impact: HIGH (reusing data shaped for one purpose as another module's input creates model coupling that turns every internal change into a cross-module break)**

모든 모듈은 도메인 지식의 한 조각을 소유하며, 그 안에는 두 종류가 섞여 있다. 폭넓게 적용되고 안정적인 **general** 지식과, 이 문제에만 고유한 **domain-specific** 지식이다. 둘은 공개할 때 서로 반대 방향으로 작용한다. general 지식을 공개하면 재사용성과 호환성이 올라가고, domain-specific 지식을 공개하면 명확성과 신뢰성이 올라간다. 주어진 경계가 어느 쪽을 노출해야 하는지 의도적으로 결정한 뒤, 공개하기 전에 추상화하라.

그 지식을 어떻게 공유하느냐가 결합을 결정한다. **model coupling**은 다른 목적으로 공개된 데이터 — 대개 persistence row나 DTO — 를 두 번째 모듈의 입력으로 재사용한다. 편리해 보이지만 모든 소비자를 남의 내부 형태에 묶어버려서, 저장 방식이 바뀌면 그 여파가 바깥으로 번진다. **contract coupling**은 상호작용만을 위해 만든 데이터를 공개한다. 목적에 맞게 만든 상호작용 contract를 선호하라. 각 측이 contract만 지키면 내부를 자유롭게 바꿀 수 있으므로, 가장 안정적인 공유 형태다.

**Incorrect:**

```ts
// A persistence row is published; other modules couple to its shape (model coupling).
interface UserRow {
  id: number
  pw_hash: string // storage detail
  created_ts: number // storage column name + epoch units
  pref_json: string // serialized blob, schema implicit
}

// Notifications now depends on the database's internal shape.
function sendWelcome(user: UserRow) {
  const prefs = JSON.parse(user.pref_json) // Notifications is bound to the storage row shape, not a contract
  mailer.send(prefs.locale, user.id)
}
```

**Correct:**

```ts
// A purpose-built interaction contract, abstracted from storage (contract coupling).
interface WelcomeRequested {
  userId: string
  locale: string
}

// The User module maps its internal row to the contract; consumers see only that.
function toWelcomeRequested(row: UserRow): WelcomeRequested {
  return { userId: String(row.id), locale: JSON.parse(row.pref_json).locale }
}

// Notifications depends on the stable contract, not on storage internals.
function sendWelcome(evt: WelcomeRequested) {
  mailer.send(evt.locale, evt.userId)
}
```

더 넓은 결합 수준(intrusive, functional, model, contract)과 더 느슨한 쪽으로 옮겨가는 방법은 `backend-patterns` skill을 참고하라.

---

## 5. Layered & Monorepo Architecture

**Impact: MEDIUM**

layered 구조와 Turbo monorepo를 적용해 app에서 공유 package로 흐르는 단방향 dependency 흐름을 구성한다.

### 5.1 Layered Architecture: One-Way Dependency, Watch N:N Mapping

**Impact: HIGH (layers make one-way dependency easy but produce N:N mapping that erodes the separation they promised)**

레이어는 단일한 추상화 관점으로 시스템을 분리한다. 모든 레이어는 다른 레이어의 위 또는 아래에 놓이므로, 레이어링은 언제나 functional coupling을 만든다. 상위 레이어는 그 아래 레이어 없이는 자신의 일을 할 수 없다. 이것은 결함이 아니라 레이어가 받아들이는 거래다. 레이어링은 관점이 안정적이고 경계가 거의 움직이지 않는 **고정된 도메인**에서 가장 효과적이며, 전통적인 조직 구조에 잘 대응된다.

레이어링의 강점은 one-way dependency를 강제하기가 매우 쉽다는 점이다. 각 레이어는 아래쪽만 가리킬 수 있다. 반복해서 나타나는 실패 양상은 **N:N mapping**이다. 관심사가 레이어에 깔끔하게 배정되지 않으면, 모든 상위 모듈이 모든 하위 모듈을 건드리게 되고 레이어 경계는 더 이상 아무 의미도 가지지 못한다. 매핑을 좁게 유지하라. 레이어는 바로 아래 레이어에 의존해야 하며, 가로질러 닿거나 중간 단계를 건너뛰어서는 안 된다.

레이어 관점은 의도적으로 골라야 한다. 흔히 쓰이는 관점은 세 가지이며, 이들을 일관성 없이 섞는 것이 바로 N:N 엉킴을 만든다.

- **Lifecycle** — 객체가 얼마나 오래 사는가.
  - `presentation`: 필요할 때 생성되고 상호작용 처리가 끝나면 파기된다.
  - `application`: request에서 response까지 살아 있다가 폐기된다.
  - `business`: 상대적으로 long-term으로 유지되는, 상태 없는 invariant.
  - `data-access`: 상대적으로 long-term으로 유지되며 영속 상태만 소유한다.
- **Functional role** — 각 레이어가 맡는 역할.
  - `interface` (presentation): 최초 이벤트를 발생시키고 결과를 수령한다.
  - `orchestrator` (application): 기능을 모아 중계하여 동작시킨다.
  - `provider` (business + data-access): 실제 기능을 제공하는 레이어.
- **Domain role** — 지식이 얼마나 일반적인가.
  - `domain`: 도메인별 상호작용을 담당한다.
  - `function`: 도메인이 재사용하는 중립적인 기능.
  - `foundation`: function 레이어가 동작할 수 있는 기반 기능.

축마다 하나의 관점을 골라 일관되게 유지하라. 표준적인 lifecycle 흐름은 `presentation -> application -> business -> data-access`이며, 오직 한 방향이다.

**Incorrect:**

```typescript
// business/orderRules.ts  — a business-layer module reaching UP into presentation
import { showToast } from "../presentation/toast"          // upward dependency
import { db } from "../dataAccess/db"

export function applyDiscount(order: Order) {
  const next = { ...order, total: order.total * 0.9 }
  db.save(next)
  showToast("Discount applied")   // business now knows about the UI layer
  return next
}
```

**Correct:**

```typescript
// presentation/orderView.ts  — top layer, points down at application only
import { placeOrder } from "../application/orderService"
import { showToast } from "./toast"
async function onSubmit(order: Order) {
  const result = await placeOrder(order)   // down to application
  showToast(`Saved (total ${result.total})`)   // UI stays in the UI layer
}

// application/orderService.ts  — orchestrator, points down at business
import { applyDiscount } from "../business/orderRules"
export async function placeOrder(order: Order) {
  return applyDiscount(order)
}

// business/orderRules.ts  — business invariant, points down at data-access only
import { save } from "../dataAccess/orderRepo"
export function applyDiscount(order: Order): Order {
  const next = { ...order, total: order.total * 0.9 }
  save(next)
  return next   // returns a value; never reaches up to the UI
}

// dataAccess/orderRepo.ts  — bottom layer, depends on nothing above it
export function save(order: Order): void {
  /* persist */
}
```

### 5.2 Turbo Monorepo: apps -> packages One-Way, No lib -> lib

**Impact: HIGH (a clean apps/packages/lib gradient keeps deploy units independent and stops dependency cycles from forming across the workspace)**

Turborepo 모노레포는 workspace 수준에 적용된 복합 레이어링이다. 각 workspace 폴더는 하나의 역할을 맡고, 의존성은 오직 한 방향으로만 흐른다.

1. **`apps/*`** — `packages` 기능을 엮는 orchestrator이며, 각 app은 독립적으로 빌드/배포 가능한 단위(application)다. 규칙은 `app -> packages`가 언제나 one-way라는 것이다. app은 다른 app을 import하지 않는다.
2. **`packages/<domain>*`** — domain 레이어로, business-logic(도메인의 invariant)와 data-access(도메인의 long-term 상태)를 담는다.
3. **`packages/lib*`** — foundation 레이어로, business-logic과 data-access가 의존하는 순수 기능이다. `packages/domain -> packages/lib`는 one-way다.

지양해야 하는 간선은 **`packages/lib -> packages/lib`**이다. foundation package는 다른 foundation package에 의존해서는 안 된다. `lib2`가 `lib1`의 공유 의존성이 될 만큼 충분히 일반적이라면, 그것은 애초에 `packages`에 속하지 않고 `node_modules` 대상이다(배포하거나 third-party 의존성으로 다룬다). `lib` package를 leaf 수준으로 유지하면 foundation이 평평하고 acyclic하게 유지된다.

각 domain package를 세 영역으로 나눠 front와 server가 계약(contract)만 정확히 공유하고 그 이상은 공유하지 않도록 한다.

```
packages/<domain>/common  - protocol, type, rule, invariant (shared both ways)
packages/<domain>/front   - front-side domain rule, invariant
packages/<domain>/server  - server-side domain rule, invariant
```

구체적인 트리는 다음과 같다.

```
apps/
  ndxFront     - presentation + domain + orchestrator
  ndxServer    - application + orchestrator
  pdfFront
  pdfServer
packages/
  ndx/                     # domain layer
    common/  - protocol, type, rule, invariant
    front/   - front domain rule, invariant
    server/  - server domain rule, invariant
  pdf/
    common/
    front/
    server/
  pdfUtil      # foundation (lib)
  llmApi       # foundation (lib)
  socket       # foundation (lib)
  fetch        # foundation (lib)
```

**Incorrect:**

```jsonc
// packages/pdfUtil/package.json  — a lib depending on another lib
{
  "name": "@acme/pdfUtil",
  "dependencies": {
    "@acme/llmApi": "workspace:*"   // lib -> lib: foundation is no longer flat
  }
}

// apps/ndxServer/src/index.ts  — an app reaching into another app
import { renderPdf } from "../../pdfServer/src/render"   // app -> app: forbidden
```

**Correct:**

```jsonc
// packages/pdf/server/package.json  — domain depends DOWN on foundation libs
{
  "name": "@acme/pdf-server",
  "dependencies": {
    "@acme/pdfUtil": "workspace:*",   // domain -> lib, one-way
    "@acme/fetch": "workspace:*"
  }
}

// packages/pdfUtil/package.json  — a foundation lib is a leaf: no workspace deps
{
  "name": "@acme/pdfUtil",
  "dependencies": {
    "pdf-lib": "^1.17.0"   // truly shared general code lives in node_modules
  }
}

// apps/ndxServer/package.json  — app depends DOWN on packages only
{
  "name": "ndx-server",
  "dependencies": {
    "@acme/ndx-server": "workspace:*",
    "@acme/llmApi": "workspace:*"
  }
}
```

app이나 package *내부*의 컴포넌트 구조 — context, compound component, prop 경계 — 를 어떻게 잡을지는 `composition-patterns` skill을 참고하라.

### 5.3 Control Interconnection Complexity: Linear Flow + Message Constraints

**Impact: MEDIUM (interconnection complexity ripples across modules and cannot be contained the way modular components can)**

시스템은 **components + interconnections + purpose**다. component는 모듈화되어 있으므로 변경의 blast radius가 그 내부로 한정된다. interconnection 복잡성은 다르다. 모듈 경계로 가둘 수 없기 때문에, 그 여파는 닿는 모든 것으로 번진다. 그래서 연결 설계에는 별도의 규율이 필요하다. encapsulation이 그것을 흡수해 주리라 기대할 수 없다.

세 가지 제약이 interconnection 복잡성을 통제한다.

- **연결을 linear하게 만들라.** linear interconnection은 시간적 개념이다. 연결이 정해진 순서(sequential / pipelined)로 일어난다. 순서는 인과관계를 담으므로, 전체 흐름을 머릿속에 담지 않고도 한 단계씩 추론할 수 있다. 이것이 부분 분석과 부분 수정을 가능하게 하는 pipelining 전략이다.
- **연결을 unidirectional하게 유지하라.** 연결은 request -> response 한 방향이지, 양방향으로 수다스럽게 주고받는 대화가 아니다. request를 보내려면 대상을 알아야 하므로, 방향은 곧 dependency 방향과 같으며 one-way 방향은 인과적 순서도 확정한다. 단일 간선만으로는 양방향이 아니어도 큰 순환을 통해 간접적으로 양방향이 되는 경우를 경계하라.
- **메시지를 제약하라.** 연결을 가로지르는 메시지의 schema를 좁혀, 잘못된 입력이 퍼지기 전에 경계에서 차단되게 하라. 느슨한 payload는 검증을 하류로 밀어내고, 그곳에서는 이미 여파가 넓어져 있다.

**Incorrect:**

```typescript
// Bidirectional, chatty, permissive — the caller and worker call back and forth,
// and the message is an open bag of optional fields validated nowhere.
interface Job {
  kind?: string
  payload?: unknown        // anything goes; errors surface deep downstream
  onProgress?: (pct: number) => void
}

class Worker {
  constructor(private caller: Caller) {}            // worker knows the caller
  run(job: Job) {
    this.caller.notifyStarted()                     // back-edge to caller
    const data = this.caller.fetchMore(job.kind)    // pulls more mid-run
    job.onProgress?.(50)                            // calls back in
    this.caller.notifyDone(data)                    // and again -> cycle
  }
}
```

**Correct:**

```typescript
// One-way pipeline stage with a strict, validated message schema.
import { z } from "zod"

const RenderRequest = z.object({
  documentId: z.string().uuid(),
  pages: z.array(z.number().int().positive()).nonempty(),
})
type RenderRequest = z.infer<typeof RenderRequest>

interface RenderResult {
  documentId: string
  url: string
}

// Each stage takes a validated input and RETURNS an output. No call-backs,
// no reference to the caller. The pipeline composes the stages in order.
function renderStage(input: unknown): RenderResult {
  const req = RenderRequest.parse(input)   // invalid message rejected at the edge
  const url = render(req.documentId, req.pages)
  return { documentId: req.documentId, url }
}

// request -> response, one way; the orchestrator owns the ordering.
const result = renderStage(incomingMessage)
```

---

## 6. AI-Friendly Ownership

**Impact: MEDIUM**

AI agent가 제한된 context window 안에서 격리된 부분을 소유하고 수정할 수 있도록 코드를 구조화한다.

### 6.1 Structure Code for AI Partial Ownership

**Impact: HIGH (code an AI can edit by loading only the task-relevant subset stays changeable; code that drags in a web of cyclic files does not)**

AI 이전에는 사람이 코드베이스의 고정된 한 부분을 소유했고, 소프트웨어 공학은 대체로 그 소유자들 사이의 커뮤니케이션 비용을 줄이는 기술이었다. AI 에이전트는 그런 의미의 소유를 하지 않는다. 저장소 전체를 머릿속에 담지 않는다. 주어진 지시마다 세션 context window에 들어가는 만큼만 분석하고, 행동에 필요한 만큼의 이해만 재구성한 뒤 나머지는 버린다.

이는 핵심 설계 질문을 다시 정의한다. 코드가 *목적별로 부분 소유*될 수 있도록 구조화되어 있는가? 에이전트가 격리된 모듈 하나를 로딩해 온전히 이해하고 수정할 수 있을 때 변경 비용은 낮다. 파일 하나를 건드리는 데에도 올바른 멘탈 모델을 세우기 위해 cyclic하게 결합된 파일의 그물을 끌어와야 한다면 변경 비용은 높다 — 게다가 그 context가 애초에 다 들어가지 않을 수도 있다.

선호 순서대로 정리한 최적의 목표는 다음과 같다.

1. 지시 단위로 완전히 격리된 모듈 — 기능이나 수정이 독립적으로 로딩하고 편집할 수 있는 모듈 안에 존재한다.
2. 연결이 불가피하다면, unidirectional dependency의 graph 구조 — 에이전트가 의존성을 한 방향으로만 따라가고 closure가 제한된 상태로 유지된다.
3. 작업에 필요한 만큼만 로딩 — 최소 closure가 context window에 들어갈 만큼 작게 유지된다.

Cyclic coupling은 이 세 가지를 모두 무너뜨린다. 최소 closure 자체가 존재하지 않으므로, 어느 한 부분을 안전하게 변경하려면 에이전트가 클러스터 전체를 로딩해야 한다.

**Incorrect:**

```typescript
// invoice.ts — to understand invoicing you must load customer.ts...
import { Customer } from "./customer"

export class Invoice {
  constructor(public customer: Customer) {}
  recalculate() {
    this.customer.refreshOutstandingBalance() // reaches back into Customer
  }
}

// customer.ts — ...which reaches back into invoice.ts (cycle)
import { Invoice } from "./invoice"

export class Customer {
  invoices: Invoice[] = []
  refreshOutstandingBalance() {
    this.invoices.forEach((i) => i.recalculate()) // back into Invoice
  }
}
// Editing either file requires loading BOTH (and whatever they each pull in).
// The "minimal context" for a one-line change is the entire cluster.
```

**Correct:**

```typescript
// invoice.ts — a self-contained module, editable in isolation.
// It depends on plain data, not on a class that depends back on it.
export interface InvoiceLine {
  amount: number
}

export function invoiceTotal(lines: InvoiceLine[]): number {
  return lines.reduce((sum, line) => sum + line.amount, 0)
}

// balance.ts — depends on invoice.ts in ONE direction only.
// To change balance logic, load balance.ts (+ the small invoice.ts type).
// To change invoice math, load invoice.ts ALONE — no cycle to drag in.
import { type InvoiceLine, invoiceTotal } from "./invoice"

export function outstandingBalance(openInvoices: InvoiceLine[][]): number {
  return openInvoices.reduce((sum, lines) => sum + invoiceTotal(lines), 0)
}
```

수정된 버전에서는 에이전트가 `invoice.ts` 하나만 로딩해 청구 계산 로직을 변경할 수 있으며, context가 최소이고 제한되어 있다 — 이것이 바로 AI가 의존하는 부분 소유다.

---

## References

1. `tdd-workflow` skill — 테스트 규율과 characterization test
2. `coding-standards` skill — 네이밍, 파일 구조, clean code
3. `backend-patterns` skill — 결합 수준과 모듈 계층화
4. `composition-patterns` skill — app 또는 package 내부의 컴포넌트 구조
