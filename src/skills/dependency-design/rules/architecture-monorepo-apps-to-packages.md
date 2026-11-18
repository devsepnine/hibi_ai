---
title: "Workspace Package Boundaries"
impact: HIGH
impactDescription: a clean apps/packages/lib gradient keeps deploy units independent and stops dependency cycles from forming across the workspace
tags: architecture, monorepo, turborepo
---

## Workspace Package Boundaries

Check the repository’s architecture and package boundaries first. The following layout assumes apps own deployments and packages supply reusable capabilities.

1. **`apps/*`**: orchestrators that wire `packages` features together, and each
   app is an independently buildable/deployable unit, an application. The rule:
   `app -> packages` is always one-way. An app never imports from another app.
2. **`packages/<domain>*`**: the domain layer, holding business-logic for the domain's
   invariants and data-access for the domain's long-term state.
3. **`packages/lib*`**: the foundation layer, pure capability that
   business-logic and data-access depend on. `packages/domain -> packages/lib` is
   one-way.

A foundation package may depend on another foundation package when project rules allow that edge, it uses the public API through a declared workspace dependency, and the graph stays acyclic. If the project defines foundation packages as leaves, follow that rule. Publishing is a separate release decision.

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

A valid workspace dependency from a client package to a transport package:

```json
{
  "name": "@acme/api-client",
  "dependencies": {
    "@acme/transport": "workspace:*"
  }
}
```

If `@acme/transport` imports `@acme/api-client` in turn, the graph contains a cycle. Move client-specific behavior back to the client or extract the shared contract below both packages.

For how to structure the components *inside* an app or package, including context,
compound components, and prop boundaries, see the `composition-patterns` skill.

Reference: [Layered and Turbo Monorepo Architecture](../references/monorepo.md)
