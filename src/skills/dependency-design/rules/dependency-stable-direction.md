---
title: Depend in the Direction of Stability (DDD Subdomains)
impact: HIGH
impactDescription: pointing dependencies from volatile to stable code keeps churn from rippling into trusted modules
tags: dependency, ddd, stability
---

## Depend in the Direction of Stability (DDD Subdomains)

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

Reference: [Coupling Models: Module, Connascence, and Domain](../references/coupling-models.md)
