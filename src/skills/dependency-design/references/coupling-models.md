# Coupling Models: Module, Connascence, and Domain

Coupling is the property that decides how far a change ripples. When you touch one piece of code, the blast radius is determined by what else *knows* about that piece and *how* it knows it. This reference gives you three complementary lenses for reasoning about that blast radius — the **module coupling model** (structure), the **connascence/symbiosis model** (what kind of shared knowledge binds two parts), and the **domain coupling model** (how a unit of code exposes its domain knowledge to collaborators). None of these is a checklist of atomic rules; they are judgment tools. Use them to argue about a design trade-off, not to mechanically grade it.

## Why coupling, not just dependency count

Software engineering, unlike most engineering disciplines, struggles to build models that are both quantitative and broadly applicable. Reality is too varied: a model precise enough to measure one system is usually too narrow to apply to the next, and a model general enough to apply everywhere says nothing useful. Coupling is one of the rare concepts that resists this trap — it can be reasoned about structurally, statically, and at runtime, and it predicts the one thing teams care about most: *how expensive is the next change?*

Two framings sit above the three detailed models and apply to all of them.

### Lifecycle coupling

Two units are lifecycle-coupled when they must change, deploy, or be reasoned about *together over time*. A function and the constant it reads are weakly lifecycle-coupled if the constant is stable; they are strongly lifecycle-coupled if every feature touches both. The structural shape of a dependency matters less than how often the two ends move in lockstep. A "clean" dependency that nonetheless forces synchronized edits on every change is worse than an "ugly" one that never moves. Always weigh coupling against the **rate of change** of the parts involved.

### Implementation-knowledge coupling

This is the deeper question underneath every model below: *how much does one unit need to know about another's internal implementation to work with it?* The danger is rarely the *amount* of shared knowledge — it is the **implicitness** of it: knowledge the compiler cannot see and the next reader cannot infer. A caller that depends on a documented data contract shares a lot of knowledge explicitly and safely. A caller that depends on the order in which a collection happens to be populated shares almost nothing — but shares it invisibly, and that is what breaks in production.

## The module coupling model

The module coupling model is the oldest of the three. It analyzes the **structure and content of code** — what a unit references and how — and in principle lets you score coupling through static analysis: how much does touching one piece of code disturb another? Because it is an early-era model, its original *ranking* of severity does not reflect modern practice, and the section after this one re-interprets it. First, the six classic levels, from worst to best:

| Level | What binds the two units | One-line read |
|---|---|---|
| **Contents** | One unit reaches directly into another's body/internals | Direct leak of hidden internal knowledge |
| **Common** | Units share a single resource and know its *entire* structure | Shared global with full structural exposure |
| **External** | Units share a single resource, but as *purpose-specific* slices | Shared singleton, partitioned by use |
| **Control** | A caller passes flags/names that leak the callee's internal structure and let it be steered from outside | Indirect leak of hidden internal knowledge |
| **Stamp** | A unit receives a whole structure but uses only part of it | More exposed than needed |
| **Data** | Units exchange only the data the interaction actually requires | Minimal, structure-independent coupling |

The classic prescription was simple: measure your coupling and push everything toward **Data**. That advice is still directionally right for terminal/leaf logic, but it is not the whole story today.

### A modern re-reading of each level

- **Contents (now read as "Implement"):** direct penetration of hidden internals. The textbook treats this as the worst sin, yet modern frameworks do it constantly and safely — Spring's DI is built on heavy reflection and runtime class regeneration. The lesson: **internal penetration is acceptable, even encouraged, when it happens inside a controlled, rule-governed environment** (a framework, a DI container). Outside such control it remains dangerous.

- **Common:** sharing the *whole* structure of a single resource that nobody actually needs in full. Almost every singleton resource — an API surface, a DB table, a JSON blob — shares a broad data structure. This is ubiquitous and made workable by control layers (API/DTO boundaries) and concurrency-control patterns. It still tends to be the bottleneck and the place problems originate.

- **External:** sharing one resource but in purpose-specific slices. The guidance splits by layer: **terminal/leaf logic should avoid static structure and prefer instances**, while **upper layers — event loops, global queues, message queues — are legitimately static**.

- **Control:** indirect leak of internal structure via flag arguments, mode names, and the like, letting an outside caller steer the callee's internals. This is treated as the *most severe* coupling in modern terms. Note the duality: localized control coupling (a strategy passed in) versus system-wide inversion of control are two sides of the same mechanism, and IoC sits at the center of modern design.

- **Stamp:** exposing more of a structure than the interaction needs. Modern code does **not** count exposing an *immutable* whole as a leak (think `record` classes / value objects). The real Stamp problem is **bridge exposure that enables train-wreck chains** (`a.getB().getC().doThing()`).

- **Data:** coupling through data with no structural dependency. Emphasized by hexagonal architecture and many others as the ideal — but it is not free. Pushing everything to pure data coupling spawns many hard-to-manage layers and DTOs; the trade-off only pays off when the **rate of change is very high**.

### The modern threat ranking

Re-evaluated against current practice, the severity order is *not* the classic one. From most dangerous to least:

```
Control  >  External  >  Common  >  Contents  >  Stamp  >  Data
```

- **Control (worst):** produces the most complex, often unsolvable coupling. Hidden internals leak indirectly and the callee becomes externally steerable.
- **External:** singleton management is genuinely hard and maintenance is harder still.
- **Common:** the design-scale reason teams reach for MSA. The fix focuses on **breaking up the single shared resource**, not merely on hiding "the whole structure."
- **Contents:** much less threatening now, because it is used almost exclusively inside fully-controlled environments (DI frameworks). The residual real risk is, again, bridge/train-wreck exposure.
- **Stamp:** closer to a design decision than a defect.
- **Data (best):** the idealized direction for terminal logic to aim at.

The headline inversion versus the classic model: **Contents drops from worst to nearly benign, while Control rises to the top.** The reason is uniform — what matters is not the raw act of touching internals but whether that touch happens under control, and whether it leaves the unit steerable from outside in ways no one can see.

## The connascence / symbiosis model

The module model asks *how units are wired*. The connascence model asks a sharper question: **does a passing compile mean the code is safe?** It does not. A compiler checks static types and syntax; runtime and contextual errors slip through. And the real hazard is not the *volume* of shared knowledge but its **implicitness** — the share the compiler cannot catch.

Connascence sorts the kinds of shared knowledge into three bands by how late, and how invisibly, a violation surfaces.

### Static — the compiler catches it

These bind two units through shared knowledge that a rename or type change will break at compile time.

- **Connascence of Name:** both units agree on a name. Rename one side and compilation fails. The safest form — fully explicit, fully checked.
- **Connascence of Type:** both units agree on a type. Change it and compilation fails. Type carries a lot of implied structure, so this is stronger than Name but still caught.
- **Connascence of Meaning:** both units agree on what a *value* means. Change the meaning (a magic number, a sentinel) and it breaks — and the compiler will flag the wrong usage *if* the meaning is encoded in a name/type. The smell here is the magic number that should have been a named constant.

### Compiles-but-unsafe — the compiler is blind, but the knowledge is still code-shaped

Here the build stays green even when the agreement is broken.

- **Connascence of Algorithm:** two units share a processing method — a serialization format, a hashing scheme, a checksum. Change one side's algorithm and it still compiles; it silently disagrees at runtime.
- **Connascence of Position:** two units agree on an *order* — argument order, a position within a collection, column order. Reorder one side and it compiles cleanly while meaning something else. There is no notion of time here; it is purely structural ordering.

### Runtime / implicit — nothing static can catch it

These are entirely implicit. No compiler, no static check reaches them.

- **Connascence of Execution:** the units must run in a specific *order*. Call B before A and the result is wrong, with no signal.
- **Connascence of Timing:** correctness depends on real elapsed time (or ticks) — waiting, debouncing, race windows.
- **Connascence of Value:** values influence each other at runtime — invariants that must hold together, transactional consistency across fields.
- **Connascence of Identity:** the units must operate on *the very same* object instance, not an equal copy. Common in External (shared-singleton) situations.

The practical takeaway: **the further down this list a coupling sits, the more it costs you, because it cannot be found by reading or building — only by failing.** When you cannot remove such a coupling, your job is to drag it *upward* — convert an Execution dependency into a Type dependency (encode the order in the type), convert a Position dependency into a Name dependency (named parameters instead of positional). Promoting connascence toward the static band is one of the highest-leverage refactors available.

## The domain coupling model

Code exists because a domain demanded it. Each unit owns a slice of domain knowledge; to collaborate it must **publish some of that knowledge as shared knowledge**, and *the way it publishes* is what creates coupling. The domain model classifies that publication style by **how the knowledge is encapsulated**.

### Encapsulation modes

- **Intrusive:** no encapsulation and no hiding — the collaborator reads the owner's internals directly. Maximum coupling, but sometimes the right call in fast-moving, exploratory code (see the DDD mapping below).
- **Functional:** the shared knowledge is *implicit* — sequential call requirements, a transaction that must wrap the calls, shared control knowledge. The interface looks clean but the real contract lives in unwritten rules about how to use it. This maps directly onto the runtime/implicit band of connascence.
- **Model:** the collaborator consumes data that was published *for a different purpose* — reusing the owner's model as-is. Convenient, but it ties you to a model shaped by someone else's needs.
- **Contract:** the interaction uses **data published specifically for the interaction**. The quality of this mode hinges entirely on **how well that contract is encapsulated** — a leaky contract is barely better than Model; a tight one is the cleanest coupling available.

### Event-based coupling

Event-driven designs step *outside* compile-time and code-structure analysis altogether. You cannot grade an event integration with the module model, because there is no call edge to inspect. What governs it instead is **the encapsulation level of the event message and the dependencies it carries** — a fat event that ships internal structure couples like Model or worse; a thin, purpose-built event message couples like a clean Contract. Judge events by their payload's encapsulation, not by the absence of a direct call.

### Mapping to DDD Distillation

Domain-Driven Design's distillation of subdomains tells you *which* encapsulation mode is appropriate where. The right coupling style is a function of the subdomain's complexity (read in Cynefin terms) and its rate of change.

- **Core subdomain** — your competitive advantage; Cynefin **complex** or beyond. It changes constantly, so it is healthy for the **core domain module to depend on other modules** (it pulls what it needs) rather than the reverse. In **complex/chaotic** territory, **aggressive intrusive coupling is common and acceptable** — you are still discovering the model, and the cost of premature contracts outweighs the cost of tight coupling.

- **Generic subdomain** — solid infrastructure-grade code; **complicated** or lower. Managed on a regular tech-debt-paydown cycle. Its relationships with other modules should sit at **Model or better** — stable enough to publish a real model, not so volatile that contracts churn.

- **Supporting subdomain** — **clear** complexity, almost no change. A stable, settled solution where **Contract coupling is ideal** — invest in a clean, well-encapsulated boundary because it will hold.

The throughline: **match the encapsulation effort to volatility.** Spend nothing on contracts where the model is still moving (Core → intrusive); spend the most where the model is settled and shared widely (Supporting → contract). This is the same rate-of-change logic that governed lifecycle coupling at the top of this document, now applied to subdomain boundaries.

## How this shows up in code

These three models are the *why*; the skill's rules turn them into concrete *do/don't* you can apply while writing or reviewing:

- **Control coupling and flag arguments, train-wreck/bridge chains, IoC** — handled in the dependency-direction and interface-boundary rules. When you spot a boolean/mode flag steering a callee, or an `a.getB().getC()` chain, that is the module model's worst level surfacing in code.
- **Promoting implicit connascence to static** — handled in the rules on typing and explicit contracts. Positional arguments, magic numbers, and order-dependent calls are connascence smells to lift toward Name/Type.
- **Choosing an encapsulation mode per boundary** — handled in the module-boundary and ownership rules, which lean on the DDD distillation mapping (Core/Generic/Supporting) above.
- **Event payload encapsulation** — see the event/messaging rules.

For broader, non-coupling-specific guidance, defer to the sibling skills rather than duplicating here: see **coding-standards** for naming and code-smell thresholds, **composition-patterns** for replacing flag/boolean proliferation with composition, and **backend-patterns** for service- and resource-boundary design.
