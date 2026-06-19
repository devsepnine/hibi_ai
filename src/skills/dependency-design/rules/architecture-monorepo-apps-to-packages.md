---
title: "Turbo Monorepo: apps -> packages One-Way, No lib -> lib"
impact: HIGH
impactDescription: a clean apps/packages/lib gradient keeps deploy units independent and stops dependency cycles from forming across the workspace
tags: architecture, monorepo, turborepo
---

## Turbo Monorepo: apps -> packages One-Way, No lib -> lib

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

Reference: [Layered and Turbo Monorepo Architecture](../references/monorepo.md)
