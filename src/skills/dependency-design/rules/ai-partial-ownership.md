---
title: Structure Code for AI Partial Ownership
impact: HIGH
impactDescription: code an AI can edit by loading only the task-relevant subset stays changeable; code that drags in a web of cyclic files does not
tags: ai, ownership, context
---

## Structure Code for AI Partial Ownership

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

Reference: [Dependency direction](../references/monorepo.md), [Isolation by responsibility](../references/complexity.md)
