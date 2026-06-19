# Complexity, Cynefin, and Degrees of Freedom

This is judgment material, not a checklist. It explains *why* the dependency rules exist — what coupling actually is, how much understanding you have of the problem you are solving, and how that understanding sets the safe shape of your dependencies. Read it when a design decision feels arbitrary and you want a principle to lean on.

## The harness illusion

It is tempting to believe that a good enough harness — a tight set of prompts, rules, and scaffolding — can drive an AI to produce a sophisticated, maintainable product on its own. It cannot, and the reason is not the harness. The hard part of product development has never been *how* to build; it is defining *what* to build.

A real product is not knowable up front. Its actual shape emerges through iteration: you ship, you get feedback, you refine, and sometimes you pivot to something different from what you started with. Generating code faster does not remove this loop — it just lets you run it faster.

This is exactly where AI-assisted ("vibe") coding tends to fail. The first build succeeds because a greenfield artifact is far less complex than the finished product. Then complexity climbs with each round of edits, every change layered as incremental patches, and the failure rate climbs with it. The way out is structural, not procedural: intervene in the code early, organize it so it is *resistant to modification*, and treat the codebase as a structure that has to absorb change — not as a one-shot output you happen to keep editing.

Maintainable code, stated plainly:

- **Responsibility-isolated** — each piece can change without disturbing the others.
- A *responsibility* is the reason a piece of code changes — its rate of change.
- *Isolated* means dependencies are absent, or minimized and **one-way**.

A markdown rule file can only hand down design *principles*. The concrete structure for any given situation is a separate map you draw yourself. Early on you direct each situation by hand, following its specific design; once the structure matures, you direct the work to follow the structure that already exists.

## Coupling is influence

The precise term is not *couple* but **coupling** — and coupling has two dimensions: **structure** and **intensity**. Coupling is the *influence* one element exerts over others, and because it is influence, it has a strength (how much) and a kind (what sort of ripple it causes). The goal is never "zero coupling" as a slogan; it is understanding what kind of influence exists and how far its ripple travels.

### Why one element influences another

Influence does not appear by accident. It comes from one of three relationships:

- **Following** — one element depends on / conforms to another (it *follows* it).
- **Shared goals** — two elements share a purpose, even partially.
- **Shared resources** — two elements use the same resource.

### Why we bother controlling influence

Uncontrolled influence costs you three concrete things:

- You **cannot use** the element — its behavior is not understandable.
- You **cannot maintain** it — the chain of consequences from a change is unpredictable.
- You **cannot improve or reuse** it in part — everything is entangled.

Two especially costly forms of influence:

- **Lifecycle coupling (shared lifecycle).** Industry code exists to serve a *domain*. Domains change in parts, and when a part of the domain changes, the code that supports it changes with it. If unrelated domain code is wired into one domain's lifecycle, it is dragged along on every change. Lifecycle coupling is what makes a system unable to respond nimbly to domain change.
- **Implementation-knowledge coupling (leaked concrete knowledge).** When concrete internal knowledge leaks across a boundary — even through an interface, depending on how well it abstracts — consumers bind to *how* a thing works, not *what* it offers. This is what makes fixes and improvements hard, because changing the internals breaks the consumers. There is a balance to strike between rich interaction and leakage; see the abstraction discussion in the `coding-standards` skill.

## The Cynefin framework

*Cynefin* (a Welsh word for habitat or the place that shapes you) is a sense-making frame: it sorts a situation into a **context**, and the right strategy follows from the context. There are five — clear, complicated, complex, chaotic, and disorder.

The deciding factor underneath all five is **how well you understand the problem**. That understanding is what we mean by complexity (next section), and it is the relative yardstick for how tightly you can safely couple. When domain understanding is low, coupling tends to rise.

### The five contexts and how to respond

| Context | Confidence in an answer | Strategy |
|---|---|---|
| **Clear** | An answer certainly exists | Classify the problem, apply the known response |
| **Complicated** | An answer probably exists | Search for the answer (analyze), then respond |
| **Complex** | An answer may be discoverable | Experiment to *create* an answer, then respond |
| **Chaotic** | Probably no answer | Act and record the experiment as you go |
| **Disorder** | The situation itself is unclear | Assume a context first, then proceed |

### The same map, for developers

The power of Cynefin for design is that the contexts map directly onto **how visible the ripple of a change is**:

| Context | What you can see about a change |
|---|---|
| **Clear** | You are *certain* a change has no ripple |
| **Complicated** | The structure makes the **compile-time** ripple knowable |
| **Complex** | A **test safety net** makes the **runtime** ripple knowable |
| **Chaotic** | The coupling is uncontrolled — ripple is unknown |

And the matching modification strategy:

- **Clear** — modify safely, as long as you stay within the protocol.
- **Complicated** — identify who will be affected, agree with them, then modify.
- **Complex** — modify, then fix the tests that break or add new ones to cover the change.
- **Chaotic** — modify and fall back on QA to catch the fallout.

```
safer  <-----------------------------------------> more flexible
   clear      complicated      complex      chaotic
 (no ripple) (compile-time)  (runtime/tests) (QA only)
```

The trade-off is real and intentional: moving toward **clear** buys safety of modification; moving toward **chaotic** buys flexibility. Neither end is "correct" — choose per budget, staffing, and the nature of the domain. A high-churn experimental feature may rationally live closer to complex/chaotic; a billing core should be pushed toward clear.

## Complexity is understandability

Strip away the connotations and **complexity = understandability**: the degree to which you can understand the problem you are trying to solve. This is the lens that ties the whole framework together. Low understanding of a domain forces higher coupling (you cannot yet draw clean boundaries); growing understanding is what *lets* you reduce it.

### A system is components + interconnections + purpose

To control complexity, decompose the system the same way every time:

```
System = Components + Interconnections + Purpose
```

- **Purpose** is *why* the components and interconnections exist — the domain / function.
- A **component** (a module) decomposes as:

  ```
  Component = Interface + Context + Implementation
  ```

  - **Interface** — the knowledge exposed for interaction.
  - **Implementation** — the real knowledge hidden inside.
  - **Context** — knowledge assumed but not implemented here.

- An **interconnection** decomposes as:

  ```
  Interconnection = Protocol + Context + Message
  ```

  - **Protocol** — the agreed rules of the exchange.
  - **Message** — what is actually sent.
  - **Context** — what the exchange assumes.

### Where to attack complexity

Each kind of complexity has its own lever, and the levers trade off against each other:

- **Domain complexity** — the goal itself is complex → split the domain → split the system.
- **Component complexity** — reduce it by lowering lifecycle coupling, even at the cost of *more* implementation-knowledge coupling.
- **Interconnection complexity** — reduce it by lowering implementation-knowledge coupling, even at the cost of *more* lifecycle coupling.

Component and interconnection pull in opposite directions; choose whichever side actually lowers total complexity for the case in front of you.

### Why interconnection complexity is the dangerous kind

A component is modularized, so the blast radius of changing it is *bounded* — the damage stays inside the module. Interconnection complexity is different: its ripple **spreads** across the connected parts, which makes it far harder to control. That is why the rules spend most of their force on the shape of connections, not the insides of modules.

## Controlling interconnection complexity

Three techniques keep the ripple of connections in check.

### Linear flow

A *linear* interconnection is one that happens in **time** — a sequence. Time gives you order, order gives you cause-and-effect you can trace step by step. The payoff: you can analyze one stage of the flow without holding the whole phenomenon in your head. Partial analysis becomes valid, complexity is contained, and you can modify a single stage in isolation. This is the pipelining strategy.

```
input -> [stage A] -> [stage B] -> [stage C] -> output
         (analyze and change B without re-reasoning A or C)
```

### One-way direction

Make a connection flow in **one direction** — a request without a coupled response back. To make a request you must know your target, so request direction is the same thing as **dependency direction**: one-way messaging is one-way dependency. Watch for indirect two-way coupling too — even without a direct round-trip, a long enough cycle through the system can close the loop and make the dependency bidirectional again. Direction also pins down cause-and-effect, just as ordering does.

```
good:   A -----> B -----> C
cycle:  A -----> B -----> C -----> A   (bidirectional in disguise)
```

### Message constraints

Tighten the constraints on the messages crossing a connection so malformed messages are rejected up front. Constraining the message narrows what the connection can express, which narrows what can go wrong and shrinks the surface that ripple can travel along.

## How this shows up in code

The ideas above turn into concrete, checkable practices in the dependency-design **rules**:

- *Coupling is influence* and the three reasons for it become the rules on **dependency direction** and **one-way dependencies** — keep arrows pointing one way, break cycles.
- *Lifecycle vs. implementation-knowledge coupling* drives the rules on **module boundaries** and what an interface is allowed to expose; the abstraction trade-off itself is detailed in the `coding-standards` skill.
- *Interconnection over component* maps to the rules on **linear flow / pipelining**, **message constraints**, and validating inputs at boundaries.
- *Cynefin context* is the rule of thumb for **how much test coverage a change needs** — push churning, low-understanding areas toward a runtime safety net (complex) and stabilize cores toward compile-time certainty (clear); the testing mechanics live in the `tdd-workflow` skill.
- *Domain splitting* informs how to decompose larger systems; reusable composition mechanics are in the `composition-patterns` and `backend-patterns` skills.
