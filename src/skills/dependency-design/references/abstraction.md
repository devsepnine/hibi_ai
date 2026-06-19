# Abstraction and Module Boundaries

A module is the unit you reason about when you control complexity. Drawing its boundary well is the single highest-leverage decision in dependency design, because the boundary determines what the rest of the system can see, depend on, and break against. This document is conceptual judgment material: it explains how to think about what a module exposes and why, not a list of mechanical rules.

## A module is three kinds of knowledge

Every module partitions the domain knowledge it touches into three buckets:

- **Interface** — the knowledge it *publishes* so others can interact with it. This is the contract: types, signatures, names, guarantees. Once published, others depend on it.
- **Implementation** — the actual knowledge it *hides*. The real logic, data layout, and algorithms. Nobody outside should know or care that this exists.
- **Context** — the knowledge it *does not implement* and instead assumes or requires from the outside. Environment, preconditions, injected collaborators, ambient assumptions.

```
        ┌─────────────────────────────┐
        │            Module           │
        │                             │
  ──────┤  Interface  (published)     │   what callers see and depend on
        │ ─────────────────────────── │
        │  Implementation (hidden)    │   the real work, invisible outside
        │ ─────────────────────────── │
        │  Context    (unimplemented) │   what it assumes from the world
        └──────────────▲──────────────┘
                       │
                supplied by the environment
```

Designing a module is the act of deciding which knowledge goes in which bucket. The same fact can be published, hidden, or required — and the choice has consequences.

### The interface is a promise you cannot quietly take back

Anything in the interface is something callers will couple to. The cost of publishing is not the act of writing the signature; it is that every future change to that signature ripples outward. So the question for each piece of knowledge is not "is this useful to expose?" but "am I willing to be bound by this for as long as the module lives?"

### Implementation is where you keep your freedom

Knowledge kept in the implementation can change without telling anyone, because nobody depends on it. The more you can push into the implementation, the more freely the module evolves. Hiding is not secrecy for its own sake — it is the mechanism that preserves your ability to refactor.

### Context is borrowed complexity

Context is knowledge the module refuses to own and demands from its surroundings. Some context is unavoidable (a module needs *something* to act on). But every requirement you place on the environment is a precondition the caller must satisfy, and an assumption that can silently become false. Context is the quietest source of coupling because it is invisible in the signature yet real at runtime.

## Classifying knowledge: domain-specific vs general

Inside any module, domain knowledge is a blend of two strands tangled together:

- **Domain-specific knowledge** — facts true only of *this* problem. "An order can't ship before payment clears." "Markets settle at the close price."
- **General knowledge** — facts true regardless of domain. Sorting, retrying, caching, validation, serialization.

When you decide what to publish, you are implicitly deciding which strand to expose. Each choice buys something and costs something.

### Publishing general knowledge

Exposing the general strand (a generic `Repository<T>`, a generic `retry(fn)`) raises **reusability and compatibility**. More callers can connect to it, across more domains, because nothing in the interface ties them to your specific problem.

The cost: a general interface says little about *your* domain, so callers must supply the meaning themselves, and the interface can drift toward anemic, low-information shapes.

### Publishing domain-specific knowledge

Exposing the specific strand (a `MarketSettlementService` with named, domain-shaped operations) raises **clarity and reliability**. The interface tells the reader exactly what is and isn't allowed; illegal states become hard to express; the compiler and the names carry intent.

The cost: a domain-specific interface is reusable only within that domain. You trade breadth of connection for depth of meaning.

```typescript
// General — high connectivity, low domain meaning
interface Repository<T> {
  find(id: string): Promise<T | null>
  save(entity: T): Promise<void>
}

// Domain-specific — high clarity, narrow reuse
interface SettlementLedger {
  recordSettlement(marketId: MarketId, price: ClosePrice): Promise<Receipt>
  // illegal to "save" a half-built market; the interface won't let you
}
```

Neither is "better." The right call depends on whether this module sits on a reuse seam (favor general) or encodes a hard domain rule that must not be violated (favor specific).

### Minimize context for clarity and reliability

Independently of the specific/general axis, **shrinking the context bucket almost always helps**. The fewer assumptions a module makes about the outside world, the easier it is to understand in isolation and the fewer ways it can quietly break. A module that requires three ambient globals and a particular call order is fragile no matter how clean its interface looks. Prefer to make required context *explicit and minimal* — passed in, named, and as small as the job allows.

## Publish abstracted knowledge, not concrete knowledge

Here is the rule that ties the buckets together: **whichever strand you publish, publish it abstracted, never concrete.**

Concrete knowledge in an interface — a leaked data shape, an exposed internal enum, a method that only makes sense given the current implementation — drags implementation detail across the boundary. Even when the signature is technically "an interface," it can leak so much that callers end up coupled to *how* you work, not just *what* you promise. That is concrete-knowledge leakage, and it raises complexity for everyone downstream.

Abstracted knowledge, by contrast, communicates the *what* while keeping the *how* private. And critically: **abstraction must be consistent.** An interface that is abstract in one method and concrete in the next is worse than a uniformly concrete one, because callers can no longer predict the level they are working at. Pick an altitude and hold it across the whole boundary.

## Three ways to abstract

Abstraction is not vagueness. Knowledge has a *specific* part and a *general* part; the two together make meaning. But only the general part transmits across a boundary — the specific part stays behind. To interact, you must hand over knowledge, so you redefine a specific-plus-general concept into one that leads with its generality. There are three distinct methods for doing that, each operating on a different aspect.

### Modeling — reinterpret from a viewpoint

Take the knowledge and re-express it from a chosen point of view, rearranging it around what that viewpoint cares about. You keep the same underlying thing but re-place its parts so the relevant aspects come forward and the irrelevant ones recede.

> A bank transfer *modeled as* an "event with a source, target, and amount" — same reality, reinterpreted through the viewpoint that matters to an audit log.

Modeling abstracts by **generalizing the viewpoint**: choose a vantage point broad enough that many concrete cases look the same through it.

### Categorization — simplify toward generality

Drop the distinctions you don't need until what remains is a simpler, more general concept. You are deliberately throwing away specificity to gain reach.

> `CreditCard`, `BankTransfer`, and `Wallet` categorized as `PaymentMethod` — the differences are discarded; the shared capability remains.

Categorization abstracts by **generalizing the concept itself**: fewer attributes, broader applicability.

### Grouping — incorporate from a different ontology

Bring the knowledge in under a completely different frame of reference — change *what kind of thing* it is. This is the most aggressive move: it re-homes the concept in another ontology entirely.

> Treating a "user," a "service account," and an "API key" all as `Principal` in a security model — they are different kinds of entities, regrouped under an authorization ontology where the only thing that matters is "something that can be granted permissions."

Grouping abstracts by **generalizing the existence**: changing the category of being, not just the viewpoint or the attribute set.

## Generality improves connectivity

The payoff across all three methods is the same: **the more general (and the smaller) the published knowledge, the higher the connectivity.** Less knowledge to satisfy means more things can connect to the boundary, across more contexts, with less friction.

This is why each method only pays off when it pushes in its native direction:

- Modeling must generalize the **viewpoint**.
- Categorization must generalize the **concept**.
- Grouping must generalize the **existence**.

An abstraction that adds detail instead of removing it, or that mixes levels, lowers connectivity and raises coupling — the opposite of the goal. Abstraction earns its keep only when it leaves the boundary smaller and more general than before.

## How this shows up in code

The judgment on this page becomes concrete in the rules:

- **Choosing what a module publishes vs hides** — see the *Module boundaries and interface design* rules for the mechanics of keeping interfaces narrow and abstraction consistent.
- **Keeping implementation detail from leaking** — see the *Concrete-knowledge leakage* rules.
- **Minimizing and making context explicit** — see the rules on dependency direction and injected context; the `coding-standards` skill covers naming and the `composition-patterns` skill covers exposing capability through composition rather than wide prop/parameter surfaces.
- **Where the seam between general and domain-specific lives** — see the `backend-patterns` skill for repository/service layering that puts general persistence behind domain-shaped services.
