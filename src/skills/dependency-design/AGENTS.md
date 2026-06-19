# Dependency & Coupling Design

**Version 1.0.0**  
Engineering  
January 2026

> **Note:**  
> This document is mainly for agents and LLMs to follow when designing,  
> maintaining, or refactoring module boundaries, dependencies, and coupling.  
> Humans may also find it useful, but guidance here is optimized for  
> automation and consistency by AI-assisted workflows.

---

## Abstract

Rules for shaping dependencies and coupling so a system stays changeable as it grows. Match the coupling strategy to how well the problem is understood, rank and reduce harmful coupling, keep dependencies unidirectional and acyclic, publish minimal abstracted knowledge across module boundaries, and lay out layered and monorepo architectures with one-way dependency flow. The final aim is code that an AI agent can partially own — load an isolated subset, understand it in full, and edit it within a bounded context window.

This is a generated artifact that aggregates the individual rule files under `rules/`, grouped by the sections defined in `rules/_sections.md`. Each rule is reproduced with its title, impact, explanation, and Incorrect/Correct examples.

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

Match the coupling strategy to how well the problem is understood, using the Cynefin domains to decide between up-front structure and deferred decisions.

### 1.1 Secure a Test Safety Net Before Changing Complex Code

**Impact: HIGH (prevents silent runtime regressions in code the compiler cannot verify)**

The Cynefin framework sorts a change by how well you understand its impact, and each
band gets a different strategy:

- **Clear** — you are certain the change has no ripple; edit safely within the protocol.
- **Complicated** — the ripple is compile-visible; the type system flags every caller, so agree with the affected parties, then edit.
- **Complex** — the ripple is *runtime* coupling the compiler cannot catch (events, shared state, ordering, side effects). You only know what broke by running the code.
- **Chaotic** — the coupling is uncontrolled; there is no reliable way to predict impact at all.

The trap is treating complex code as if it were clear. When the impact lives at runtime,
editing without a test and "letting QA find it later" is not a strategy — it is operating
in the chaotic band by choice. The discipline that keeps a change *complex* rather than
*chaotic* is a safety net: before you touch runtime-coupled logic, add or confirm a
characterization test that pins the current behavior. Then refactor against it. A failing
test now tells you exactly which runtime contract you broke, instead of a support ticket
a week from now.

So the rule for complex code is: pin behavior first, change second. If existing tests
already cover the path, run them and confirm green. If they do not, write a
characterization test that captures whatever the code does today — even if that behavior
is ugly — and only then make your edit.

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

For the broader test discipline (characterization tests, coverage targets, what to
assert), see the `tdd-workflow` and `coding-standards` skills.

---

## 2. Coupling Types & Threat Ranking

**Impact: CRITICAL**

Identify and reduce harmful coupling; the modern threat order is Control > External > Common > Contents > Stamp > Data.

### 2.1 Avoid Control Coupling (Flag Arguments Leaking Internal Structure)

**Impact: CRITICAL (control flags let a caller steer the callee's internal branches, creating the hardest-to-untangle coupling in modern code)**

Control coupling occurs when a caller passes a flag or mode argument whose only job is to select which internal branch of the callee runs. The flag leaks the callee's internal structure into the call site: the caller now has to know the callee's branches to call it correctly, and every new branch forces a new flag. Among classic coupling types, this is the most severe modern threat, because the resulting dependency is the most complex and the hardest to dissolve. Indirect leakage of hidden knowledge through flags is, in practice, worse than direct leakage.

The fix is to remove the steering: split the behavior into intention-revealing functions, or invert control with a strategy or polymorphism so the caller selects a behavior object instead of toggling internal paths. Localized strategy and broad inversion of control are both core tools of modern design.

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

Contents coupling (also called implementation-knowledge coupling) occurs when a consumer depends on another module's concrete internals: a private field, an underscore-prefixed property, an internal data shape, or a body that was never meant to be part of the contract. Any refactor of those internals breaks the consumer, so the implementation freezes in place. This is direct leakage of hidden knowledge.

Depend on an abstract interface instead, and let the owner decide what to expose. The trade-off to balance is interactivity versus leakage: expose enough behavior to be useful, but never the raw internal structure. Reflection and dependency injection technically reach into internals, but they are acceptable only inside a rule-controlled environment such as a DI framework, where the access is governed rather than ad hoc.

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

Stamp coupling is passing a whole structure when the callee needs only one field of it: the callee is now coupled to a shape it never uses, and any change to that shape ripples outward. Prefer data coupling, where you pass only the value actually required.

In modern code the milder form of stamp coupling is no longer treated as leakage. Exposing an immutable value object (a record-style structure) is closer to a design decision than a defect. The real danger is the train-wreck bridge: a chain like `a.b.c.d` that reaches across several objects and couples the caller to the entire intermediate structure. Each link is a separate thing that can change. Let the owner of the value expose it directly instead of forcing callers to navigate the graph.

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

Two coupling types share one root cause. Common coupling is many modules referencing a single resource and knowing its whole structure (the classic global mutable singleton or shared config read everywhere). External coupling is the same single resource being used for different purposes by different modules. Both are real bottlenecks: shared singletons are hard to manage and harder to maintain.

The key insight is that the cure targets the single-resource sharing, not merely the whole-structure sharing. Hiding the structure behind a DTO does not remove the bottleneck if everyone still contends for the same resource. This is precisely why large-scale designs adopt approaches like MSA: to dissolve the single shared resource. At the code level, scope access behind a narrow owner so each consumer touches only what it needs. Prefer instances for terminal logic; reserve static or global state for genuine upper-layer infrastructure such as the event loop, a global queue, or a message queue, where a single shared instance is the correct model.

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

Keep dependencies unidirectional, acyclic, and isolated by change-rate so volatile parts cannot drag stable parts with them.

### 3.1 Keep Dependencies Unidirectional and Acyclic

**Impact: CRITICAL (cyclic dependencies make change ripple unpredictable and break causal ordering)**

A dependency graph models how change in one module ripples into others. When two
modules depend on each other, or a longer loop closes back on itself
(`A -> B -> C -> A`), the ripple is no longer predictable: a change to any node
in the cycle can propagate all the way around and back. Indirect cycles spanning
many modules are just as harmful as direct ones — the loop count does not soften
the coupling, it hides it.

Cycles also destroy causal order. A unidirectional edge encodes "this is built
on that": the dependency must initialize, run, and reason first. A cycle has no
such ordering, so initialization order, build order, and the mental model all
become ambiguous.

When two modules genuinely need to share something, extract that shared concern
into a lower-level module that both depend on. The dependency still flows one way
— both modules point down at the shared module, and the shared module points at
neither.

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

A module's responsibility is best defined as *the reason it changes* — and the
reason it changes shows up empirically as its change-rate. Volatile business
rules change weekly; infrastructure adapters change rarely. When you co-locate
two things with different change-rates, you couple their lifecycles: every edit
to the fast-moving part forces you to re-read, re-test, and risk the slow-moving
part, even though it had no reason to move.

Split a module along its change-rate boundaries. After the split, a change to a
volatile rule touches only the volatile module; the stable module is untouched
and stays trustworthy. This is the Single Responsibility Principle read through
the lens of *time*: group what changes together, separate what changes apart.

For the naming and file-structure mechanics of carving these modules cleanly,
see the `coding-standards` skill.

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

Dependencies should point toward stability: a module that changes often should
depend on a module that changes rarely, never the reverse. If a stable module
imports a volatile one, every churn in the volatile code drags the stable code
along with it, and the stability you paid for is lost.

DDD distillation gives a practical map of where each kind of code sits:

- **Core subdomain** — your competitive advantage, Cynefin *complex* or above.
  It changes frequently, so it is correct for the Core domain module to depend
  *outward* on the more stable subdomains.
- **Generic subdomain** — solid, infrastructure-like code, *complicated* or
  below, managed on a regular tech-debt cadence. Aim for `model`-level coupling
  or stronger in its relationships.
- **Supporting subdomain** — *clear* complexity, almost never changing. A stable
  solution where `contract` coupling is the ideal.

So the volatile Core points at the stable Generic and Supporting subdomains. A
utility (Generic/Supporting) must stay domain-agnostic — the moment it imports a
fast-changing domain type, it inherits that domain's change-rate.

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

Publish consistent, minimal, abstracted knowledge across module boundaries so callers depend on intent rather than implementation.

### 4.1 Keep Abstraction Criteria and Level Consistent

**Impact: HIGH (inconsistent abstraction levels make a module split worthless and re-tangle the very complexity modularization was meant to remove)**

The only thing that justifies splitting code into a module is abstraction: a
module hides detail behind a single, coherent viewpoint. If the criterion or the
level of that abstraction is inconsistent — high-level orchestration sitting next
to low-level byte fiddling, or a domain API that also speaks raw HTTP — the split
buys nothing. A caller now has to reason about two altitudes at once, so the
module is harder to understand than the flat code it replaced.

Pick one abstraction criterion per module and one level per interface. Everything
a module exposes should describe the problem at the same altitude; push the
lower-level details down into their own module behind their own coherent boundary.

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

For how to choose a single dividing viewpoint and layer modules by it, see the
`backend-patterns` skill.

### 4.2 Minimize Context: Publish Abstracted, Not Concrete, Knowledge

**Impact: MEDIUM (every concrete assumption a module forces its caller to satisfy becomes hidden coupling that breaks silently when the implementation changes)**

A module is three kinds of knowledge: the **interface** (what it publishes for
interaction), the **implementation** (the real knowledge it hides), and the
**context** (the assumptions it does not implement and silently requires of its
surroundings). Context is the dangerous part — it is unwritten, uncompiled, and
unchecked, so the caller can violate it without any error until runtime.

When the interface exposes concrete types and concrete assumptions, callers must
absorb that context to use the module correctly, and complexity rises everywhere.
Publish an abstracted contract instead: name the operation in the caller's terms,
accept exactly the data the interaction needs, and keep ordering, units, and
construction details inside the implementation. Minimizing context is what makes a
module clear and reliable to use.

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

The choice of *which* knowledge to publish (general vs domain-specific) is covered
by `abstraction-encapsulate-knowledge`.

### 4.3 Classify Domain-Specific vs General Knowledge, Share via Contract

**Impact: HIGH (reusing data shaped for one purpose as another module's input creates model coupling that turns every internal change into a cross-module break)**

Every module owns a slice of domain knowledge that blends two kinds: **general**
knowledge (broadly applicable, stable) and **domain-specific** knowledge (peculiar
to this problem). The two pull in opposite directions when published. Publishing
general knowledge raises reuse and compatibility; publishing domain-specific
knowledge raises clarity and reliability. Decide deliberately which kind a given
boundary should expose, then abstract it before publishing.

How you share that knowledge decides the coupling. **Model coupling** reuses data
that was published for another purpose — most often a persistence row or DTO — as
a second module's input. It looks convenient but binds every consumer to the
shape of someone else's internals, so a storage change ripples outward. **Contract
coupling** publishes data made only for the interaction. Prefer a purpose-built
interaction contract: it is the most stable form of sharing because each side can
change its internals freely as long as the contract holds.

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

For broader coupling levels (intrusive, functional, model, contract) and how to
move toward looser ones, see the `backend-patterns` skill.

---

## 5. Layered & Monorepo Architecture

**Impact: MEDIUM**

Apply layered structure and a Turbo monorepo with one-way dependency flow from apps to shared packages.

### 5.1 Layered Architecture: One-Way Dependency, Watch N:N Mapping

**Impact: HIGH (layers make one-way dependency easy but produce N:N mapping that erodes the separation they promised)**

A layer separates the system by a single abstraction viewpoint. Because every
layer sits above or below another, layering always introduces functional
coupling — an upper layer cannot do its job without the layer beneath it. That is
not a defect; it is the deal a layer makes. Layering is most effective in a
**fixed domain** that maps onto a traditional org structure, where the
viewpoint is stable and the boundaries rarely move.

The strength of layering is that one-way dependency is trivial to enforce: each
layer is only allowed to point downward. The recurring failure mode is **N:N
mapping** — when concerns are not assigned to a layer cleanly, every upper module
ends up touching every lower module, and the layer boundary stops carrying any
meaning. Keep the mapping narrow: a layer should depend on the layer directly
below it, not reach across or skip levels.

Pick the layer viewpoint deliberately. There are three common ones, and mixing
them inconsistently is what creates the N:N tangle:

- **Lifecycle** — how long an object lives.
  - `presentation`: created on demand, destroyed once the interaction is handled.
  - `application`: lives from request to response, then is discarded.
  - `business`: relatively long-lived, stateless invariants.
  - `data-access`: relatively long-lived, owns only persistent state.
- **Functional role** — what part each layer plays.
  - `interface` (presentation): raises the initial event and receives the result.
  - `orchestrator` (application): gathers and relays features to drive them.
  - `provider` (business + data-access): the real feature-providing layers.
- **Domain role** — how general the knowledge is.
  - `domain`: handles per-domain interaction.
  - `function`: neutral capability the domain reuses.
  - `foundation`: base capability that the function layer runs on.

Choose one viewpoint per axis and stay consistent. The canonical lifecycle flow
is `presentation -> application -> business -> data-access`, one way only.

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

A Turborepo monorepo is a compound layering applied at the workspace level. Each
workspace folder plays one role, and dependencies flow in one direction only:

1. **`apps/*`** — orchestrators that wire `packages` features together, and each
   app is an independently buildable/deployable unit (an application). The rule:
   `app -> packages` is always one-way. An app never imports from another app.
2. **`packages/<domain>*`** — the domain layer: business-logic (the domain's
   invariants) and data-access (the domain's long-term state).
3. **`packages/lib*`** — the foundation layer: pure capability that
   business-logic and data-access depend on. `packages/domain -> packages/lib` is
   one-way.

The discouraged edge is **`packages/lib -> packages/lib`**. A foundation package
should not depend on another foundation package — if `lib2` is general enough to
be a shared dependency of `lib1`, it does not belong in `packages` at all; it
belongs in `node_modules` (publish it, or treat it as a third-party dep). Keeping
`lib` packages leaf-level keeps the foundation flat and acyclic.

Split each domain package into three sub-areas so front and server share exactly
the contract and nothing more:

```
packages/<domain>/common  - protocol, type, rule, invariant (shared both ways)
packages/<domain>/front   - front-side domain rule, invariant
packages/<domain>/server  - server-side domain rule, invariant
```

A concrete tree:

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

For how to structure the components *inside* an app or package — context,
compound components, and prop boundaries — see the `composition-patterns` skill.

### 5.3 Control Interconnection Complexity: Linear Flow + Message Constraints

**Impact: MEDIUM (interconnection complexity ripples across modules and cannot be contained the way modular components can)**

A system is **components + interconnections + purpose**. A component is modular,
so the blast radius of a change is bounded inside it. Interconnection complexity
is different: it is not contained by a module boundary, so its ripple spreads
across everything it touches. That is why connection design needs its own
discipline — you cannot rely on encapsulation to absorb it.

Three constraints keep interconnection complexity in check:

- **Make connections linear.** Linear interconnection is a temporal idea: the
  connection happens in a definite order (sequential / pipelined). Order encodes
  causality, so you can reason about one stage at a time without holding the
  whole flow in your head. This is what makes partial analysis and partial
  edits possible — the pipelining strategy.
- **Keep connections unidirectional.** A connection is request -> response one
  way, not a two-way chatty conversation. To send a request you must know the
  target, so direction is the same thing as dependency direction; one-way
  direction also fixes the causal order. Watch for large cycles that become
  bidirectional indirectly even when no single edge is.
- **Constrain messages.** Tighten the schema of what crosses the connection so
  invalid input is rejected at the boundary, before it propagates. A permissive
  payload pushes validation downstream where the ripple is already wide.

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

Structure code so an AI agent can own and modify isolated parts within a limited context window.

### 6.1 Structure Code for AI Partial Ownership

**Impact: HIGH (code an AI can edit by loading only the task-relevant subset stays changeable; code that drags in a web of cyclic files does not)**

Before AI, a human owned a fixed slice of the codebase, and software engineering
was largely the craft of reducing the communication cost between those owners. An
AI agent owns nothing in that sense. It does not hold the whole repository in its
head. For any given instruction it analyzes only what fits the session context
window, reconstructs just enough understanding to act, and discards the rest.

This reframes the central design question: is the code structured so it can be
*partially owned by purpose*? A change is cheap when the agent can load one
isolated module, understand it in full, and edit it. A change is expensive when
touching one file forces it to pull in a web of cyclically coupled files just to
form a correct mental model — and the relevant context may not even fit.

The optimal target, in order of preference:

1. Per-instruction fully isolated modules — a feature or fix lives in a module
   that can be loaded and edited on its own.
2. When connection is unavoidable, a graph of unidirectional dependencies — so
   the agent traverses dependencies one direction only and the closure stays
   bounded.
3. Load only what the task needs — the minimal closure stays small enough to fit
   the context window.

Cyclic coupling defeats all three: there is no minimal closure, so the agent
must load the whole cluster to safely change any part of it.

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

The corrected version lets an agent load `invoice.ts` by itself to change billing
math, with a minimal, bounded context — exactly the partial ownership AI relies
on.

---

## References

1. `tdd-workflow` skill — test discipline and characterization tests
2. `coding-standards` skill — naming, file structure, clean code
3. `backend-patterns` skill — coupling levels and module layering
4. `composition-patterns` skill — component structure inside an app or package
