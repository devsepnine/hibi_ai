---
description: 'Audit dependency direction and coupling of a module/path and report violations by threat ranking with fixes'
argument-hint: "[path|module]"
allowed-tools: Read, Grep, Glob
model: sonnet
effort: high
---

# Deps Command

## What This Command Does

Loads the `dependency-design` skill and audits the dependency direction and coupling of the target named in `$ARGUMENTS` (a path or module). It statically scans the target and its imports for coupling violations:

- **Control coupling / flag arguments** — a caller passes a boolean or mode flag that selects which branch the callee runs, so the caller knows the callee's internals.
- **Cyclic or bidirectional imports** — module A imports B while B imports A (directly or through a cycle), so neither can be understood or changed in isolation.
- **Implementation-knowledge leaks** — a consumer reaches through a public surface into private internals, concrete types, or assumed data shapes instead of a stable contract.
- **Shared-resource / singleton coupling** — modules coordinate through a shared mutable global, singleton, or ambient state rather than explicit parameters.
- **lib -> lib monorepo violations** — a library package imports a sibling library it must not depend on, breaking the intended dependency graph and layering.
- **Abstraction-level inconsistencies** — a high-level policy module depends on low-level mechanism details, inverting the intended direction of the dependency.

## When to Use

- Before extracting, splitting, or merging a module, to confirm dependencies point one way.
- When a change in one file keeps forcing edits in unrelated files (a coupling smell).
- During review of a new package boundary or monorepo `lib` to enforce the dependency graph.
- When circular-import or build-order errors hint at a cycle.

## How It Works

1. **Classify each finding** by the module-coupling threat ranking — Control > External > Common > Contents > Stamp > Data (worst to best) — and by Cynefin context (Clear / Complicated / Complex / Chaotic) so the severity reflects both the coupling type and how well-understood the domain is.
2. **Report each violation** with `file:line`, the coupling type, its rank, assigned severity, and a one-line explanation of why it couples the two sides.
3. **Propose minimal fixes** that move toward one-way, contract-based dependencies: invert the dependency, split a flag-driven function into named functions, introduce a stable interface, or pass state explicitly — smallest change that removes the coupling.

## Example Usage

```
User: /deps src/payments

Agent: Loaded `dependency-design`. Scanned src/payments (11 files).

CRITICAL (Control coupling, Complicated)
  src/payments/processor.ts:42 — charge(order, isRefund) branches on the
  isRefund flag; callers steer internal logic.
  Fix: split into charge(order) and refund(order); no shared flag.

HIGH (Cyclic import, Complex)
  src/payments/ledger.ts:8 imports ./processor, and
  src/payments/processor.ts:5 imports ./ledger.
  Fix: extract the shared LedgerEntry contract into ledger/types.ts;
  have both depend on it one-way.

MEDIUM (lib -> lib violation, Clear)
  src/payments/index.ts:3 imports @repo/billing-ui (UI lib).
  Fix: invert — billing-ui depends on payments, not the reverse.

No Stamp/Data downgrades needed elsewhere.
```

## Related Agents

- `architect` — for redesigning a boundary once violations are confirmed.
- `code-reviewer` — to catch coupling regressions on new diffs.

## Related Commands

- `/code-review` — review uncommitted changes for quality and security.
- `/verify` — confirm the codebase still builds and passes after a fix.

Full methodology lives in the `dependency-design` skill — follow that as the source of truth.
