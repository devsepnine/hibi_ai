# Code Ownership in the AI Era

Dependency design is usually framed around coupling and cohesion. But there is a quieter force underneath those metrics: *who owns the code*. The owner — the agent that holds a working mental model of a region of code — determines how change actually happens. AI coding agents own code very differently than humans do, and that difference reshapes what "good structure" means. This document is conceptual judgment material, not a checklist; use it to decide *why* a structure is worth adopting.

## The pre-AI model: humans own code, communication is the tax

Before AI, ownership was simple to describe: a person (or a small team) holds a region of the codebase in their head. They know its invariants, its history, the reason for the ugly workaround on line 200. Change flows through that mental model.

This model has a hard scaling limit. When the code grows beyond what one person can hold, you add people — and the cost is not linear. Every new owner introduces communication edges with every other owner who touches adjacent code. The information that used to live in one head now has to be *transmitted*, and transmission is lossy, slow, and ambiguous.

```
1 owner:   no communication edges
3 owners:  3 edges
6 owners:  15 edges      (n(n-1)/2 grows quadratically)
```

This is why software engineering, at its core, can be read as **the discipline of reducing communication cost**. Modules, interfaces, encapsulation, layering — most of the classic toolkit exists to let one owner change their region without negotiating with everyone else. A clean module boundary is a promise: *you can reason about what's inside without talking to me.* Coupling models (Contents/Common/External/Control/Stamp/Data) and the connascence (symbiosis) model are, in this light, formal measures of how much communication a given coupling forces.

The goal of the pre-AI era was therefore to **minimize the surface across which humans must communicate** — fewer owners per change, smaller interfaces between owners, less implicit shared knowledge.

## The post-AI model: the agent owns no whole

An AI coding agent breaks the central assumption of the pre-AI model. It does not own the codebase. It owns *nothing as a durable whole*. Between sessions there is no retained mental model, no accumulated intuition, no memory of why line 200 looks the way it does.

What the agent does instead:

- It **analyzes only what fits in the current session context.** Its understanding is reconstructed from scratch each time, bounded by the context window.
- It owns code **by purpose, partially and temporarily** — it pulls in exactly the slice relevant to the instructed change, reasons over that slice, and lets it go.

So the scaling pressure inverts. The pre-AI question was "how do we reduce communication cost between many human owners?" The post-AI question is:

> **Can the code support purpose-based partial ownership?**

That is: when an agent is told to change one thing, can it acquire a *complete and correct* understanding of just that thing, from a slice small enough to fit in context — without having to load the whole system to be sure it isn't breaking something invisible?

A codebase can be excellent by classical human metrics and still fail this test. If understanding "the checkout flow" requires holding the inventory module, the pricing module, and three shared singletons in your head simultaneously, a human owner who lives in that code can cope through accumulated memory. An agent with no memory and a finite context cannot. The same hidden coupling that merely *slowed* a human owner can *block* an agent entirely.

## What makes structure AI-friendly

AI-friendliness is not a new aesthetic; it is the same coupling discipline pushed to its logical end, because the agent has none of the slack that human memory provides. Three properties matter most.

### 1. Fully isolated modules, per unit of instructed change

The ideal target for any single instructed change — a fix, a feature addition — is a module the agent can understand and modify *in complete isolation*. Everything the change touches lives inside one boundary; nothing outside needs to be loaded to be confident the change is correct.

When a change maps cleanly onto one isolated module, the agent's partial ownership is also *complete* ownership for the duration of the task. There is no invisible dependency that the context window failed to capture, because there is nothing relevant outside the boundary.

```
Instructed change: "add a discount code field to checkout"

Good:  checkout/ is self-contained → agent loads checkout/, done.
Bad:   checkout reaches into pricing internals, shares a mutable
       cart singleton, and emits an event three modules listen to
       → "just checkout" is not a closed set; correctness depends
         on code the agent may never see.
```

### 2. Unidirectional dependency graph where modules must connect

Real systems cannot be fully isolated; modules must connect. The constraint then is the *shape* of the connections: a **directed acyclic, unidirectional dependency graph.**

Unidirectionality is what makes partial loading sound. If module A depends on B and B never depends back on A, then loading A *plus its downstream dependencies* gives the agent a closed, complete picture for any change to A. Cycles destroy this — once A and B depend on each other, you cannot understand either without the other, and the "slice" needed to safely change one thing quietly expands to engulf the cycle. Bidirectional and circular dependencies are precisely the structures that make purpose-based partial ownership impossible: there is no small closed set to load.

```
A → B → C        unidirectional: to change A, load A,B,C — a closed slice
A ⇄ B            cycle: A and B are one indivisible unit; no partial loading
```

A unidirectional graph also gives the agent a reliable rule for *what to load*: follow the arrows outward from the change site until you hit leaves. That traversal terminates, and what it collects is exactly the relevant set — no more, no less.

### 3. Load only what the task needs, within the context limit

The two properties above exist to serve one operational goal: for any given task, the agent should be able to **load only the code that task requires, and have that loaded set fit within the context limit while still being complete.**

This reframes the context window from a model limitation into a *design constraint on the codebase*. The question "does this fit in context?" becomes a proxy for "is the relevant slice for this change actually small and closed?" If the honest answer to a routine change is "you'd have to load most of the system to be safe," that is not an agent problem — it is a coupling problem the structure is surfacing.

Three forces compound here, and good structure keeps all three small at once:

- **Isolation** shrinks how much a change *can* touch.
- **Unidirectionality** makes the touched set *knowable and closed*.
- **Loading discipline** then keeps each task within the context budget.

## How this shows up in code

The judgment above translates into the concrete, testable rules elsewhere in this skill:

- **Dependency direction and acyclicity** — enforcing the unidirectional, no-cycles graph from property 2. See the module-graph / dependency-direction rules.
- **Module isolation and boundaries** — keeping the change-unit self-contained from property 1. See the module-boundary / encapsulation rules.
- **Coupling reduction** — the classic Contents→Data progression and the connascence (implicit-knowledge) model, which together determine how large and how implicit a loaded slice becomes. See the coupling-model rules.

Read those rules as the *enforcement* of one idea: structure the code so that any instructed change has a small, closed, loadable slice — because that slice is all the ownership an AI agent will ever have.
