# Layered and Turbo Monorepo Architecture

## Modularization is abstraction with a consistent criterion

Splitting a system into modules is only useful when there is a single,
consistent criterion behind the split — and that criterion is abstraction.
A module earns its boundary by hiding a coherent body of knowledge behind a
smaller surface. When the *level* or the *kind* of abstraction varies from
one module to the next, the boundaries stop meaning anything: a reader can no
longer predict what lives where, and the "modularization" is decorative.

So before drawing any boundary, name the abstraction it represents and check
that sibling modules sit at the same level. Mixing "the PDF domain" with "a
date-formatting helper" as peers is a sign the criterion has drifted.

## Layered architecture: separation by viewpoint

A layer is a module boundary drawn along a single *viewpoint* — one chosen
abstraction axis applied uniformly across the whole system. Because every
layer shares that axis, layering always produces **functional coupling**:
each layer exists to perform one category of function for the layers above it.

Layering has a recognizable trade profile:

- It mirrors how traditional organizations split work, so it reads naturally
  in well-understood (fixed) domains.
- One-way dependency is easy to express — higher layers depend downward, never
  the reverse.
- But **N:N coupling is common**: many callers in an upper layer touch many
  providers in a lower one, so the edges multiply.
- It scales gracefully with resource consumption and makes it easy to *pinpoint*
  where a problem originated, because each concern lives in exactly one layer.

The N:N tendency is the price of a pure horizontal slice. It is acceptable as
long as every edge still points one direction; it becomes a problem only when
the slicing axis is inconsistent (see the modularization criterion above).

## Three viewpoints you can layer along

The same system can be sliced by more than one viewpoint. Three are useful,
and the real power comes from composing them.

### 1. Lifecycle viewpoint

Layers by *how long an instance lives and what state it carries*:

- **Presentation** — created on demand for a situation, then destroyed once
  handled. Shortest-lived.
- **Application** — lives for the span of one request→response, then destroyed.
- **Business logic** — relatively long-lived; stateless invariants (rules that
  hold regardless of request).
- **Data access** — relatively long-lived; handles only persistent state
  management.

### 2. Functional-role viewpoint

Layers by *what part each plays in getting work done*:

- **Interface** (presentation) — raises the initial event and receives the
  result.
- **Orchestrator** (application) — gathers capabilities and relays/sequences
  them; owns no real work of its own.
- **Providers** (business logic + data access) — the layers that actually
  perform the work.

### 3. Domain-role viewpoint

Layers by *how domain-aware each part is*:

- **Domain layer** — owns interactions specific to a domain.
- **Function layer** — domain-neutral capability a domain can draw on.
- **Foundation layer** — base capability that lets the function layer run at
  all.

These viewpoints are orthogonal. A single physical unit can occupy a different
position under each — which is exactly what the monorepo layout below exploits.

## The Turbo monorepo as a composite layering

A Turbo monorepo collapses all three viewpoints into one physical structure of
`apps/*` and `packages/*`. Each top-level location maps to a known position in
each viewpoint:

```
apps/<app>          orchestrator + independently deployable application
packages/<domain>   business-logic + data-access (the domain's invariants and
                    long-term state)
packages/<lib>      pure foundation capability
```

### `apps/*` — orchestrator and deploy unit

An `app` is an **orchestrator**: it composes the capabilities published by
`packages` rather than implementing them. It is also the **application** in the
lifecycle/deployment sense — an independently buildable, deployable unit
(a front-end, a server, etc.).

Hard rule: `app → packages` is **always** one-directional. An app may depend on
packages; a package may never depend on an app.

### `packages/<domain>*` — business logic + data access

A domain package holds a domain's **invariants** (stateless business rules) and
its **long-term state management** (data access). This is where the real
domain work lives.

### `packages/<lib>*` — pure foundation

A lib package is a **foundation layer**: neutral, reusable capability that
business-logic and data-access code depend on (HTTP fetch, sockets, an LLM API
client, and so on).

### Dependency rules between packages

```
apps/*  ──────────────▶  packages/*          (always allowed, one-way)
packages/<domain>  ───▶  packages/<lib>       (allowed, one-way)
packages/<lib>     ──╳   packages/<lib>       (discouraged)
```

- `app → packages`: always one-way, never the reverse.
- `packages/<domain> → packages/<lib>`: one-way; domains lean on foundations.
- `packages/<lib> → packages/<lib>`: **discouraged.** If a lib genuinely needs
  another lib, that shared dependency belongs in `node_modules` — i.e. publish
  it as an external dependency rather than letting one in-repo foundation
  package reach sideways into another. This keeps the foundation layer flat and
  prevents a hidden second layer of internal coupling.

The aim throughout is to keep every edge pointing in one direction and to stop
volatile, domain-aware code from being dragged in by stable foundation code.

## Subdividing a domain package

Inside a single domain package, split by the part of the system that owns each
rule:

```
packages/<domain>/
  common/    protocol, type, rule, invariant   (shared across front and server)
  front/     front-side domain rule, invariant
  server/    server-side domain rule, invariant
```

- `common` is the contract: protocols, shared types, and the invariants both
  sides must honor.
- `front` and `server` each hold the rules and invariants specific to that
  runtime, layered on top of `common`.

This lets the front-end app and the server app each import only the slice they
need, while the shared protocol/types stay in exactly one place.

## A concrete tree

Two domains (`ndx`, `pdf`) each shipped as a front app and a server app, with
domain packages and a set of foundation libs:

```
apps/
  ndxFront    presentation + domain + orchestrator
  ndxServer   application + orchestrator
  pdfFront
  pdfServer

packages/
  ndx/
    common    protocol, type, rule, invariant
    front     front domain rule, invariant
    server    server domain rule, invariant
  pdf/
    common
    front
    server
  pdfUtil     foundation
  llmApi      foundation
  socket      foundation
  fetch       foundation
```

Reading the tree against the viewpoints: `ndxFront` is simultaneously a
presentation interface, a domain participant, and an orchestrator; `ndxServer`
is an application-lifecycle orchestrator. Both pull domain invariants and state
from `packages/ndx`, which in turn rests on foundation libs like `fetch` and
`socket`. No package points back up at an app, and no foundation lib points
sideways at another.

## How this shows up in code

The judgment above is enforced as concrete rules in this skill:

- **Dependency Direction & Structure** — the one-way, acyclic, change-rate-
  isolated edges (`app → packages → lib`) that this layout encodes.
- **Abstraction & Module Boundary** — the consistent-abstraction criterion that
  makes a module boundary worth drawing in the first place.
- **Layered & Monorepo Architecture** — the `apps/*` vs `packages/*` placement
  rules and the `common`/`front`/`server` domain split.

For repository/data-access boundaries inside a domain package, see the
`backend-patterns` skill.
