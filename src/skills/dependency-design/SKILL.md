---
name: dependency-design
description: Dependency design — one-directional deps, circular imports, change ripple, responsibility isolation, abstraction boundaries. Use when designing modules or judging coupling. 의존성 설계, 결합도, 순환 참조, 책임 분리, 영향 범위, 모노레포 구조.
---

# Dependency Design

A harness cannot, on its own, produce modification-resistant code. The hard part is never the tooling — it is deciding what the product is and structuring the code so it survives continuous change. Vibe-coded projects succeed at first build (low complexity) and fail on the tenth revision (complexity compounds, incremental edits ripple). The defense is to intervene in structure early: isolate responsibilities, minimize dependencies, and force them one-directional so any module can be loaded and modified within a bounded context — by a human or an AI.

This skill is a decision methodology, not a fixed recipe. Each product has different responsibilities and change rates, so the organizing structure differs every time. What is portable is the reasoning: classify the problem's understandability, choose a coupling strength to match, enforce one-way dependency, and keep abstraction consistent.

## When to Apply

Reach for the matching reference when the situation below appears.

| Situation | Reference / rules |
|---|---|
| Judging how much complexity a problem carries; deciding how safely a change can be made | `references/complexity.md` (Cynefin classification) |
| Naming and grading a coupling (which kind, how strong, what ripples) | `references/coupling-models.md` (module/connascence/domain models) |
| Choosing what a module exposes vs. hides; keeping abstraction levels consistent | `references/abstraction.md` |
| Structuring code so an AI can own and modify a slice without loading everything | `references/ai-ownership.md` |
| Laying out a monorepo with enforceable one-way dependency between layers | `references/monorepo.md` |
| Enforcing the rules during review | `rules/` (one file per rule; sections in `rules/_sections.md`) |

## Core decision flow

1. **Classify the context (Cynefin).** Gauge how well the problem is understood: `clear` → `complicated` → `complex` → `chaotic`. Lower understanding justifies looser, more flexible coupling; higher understanding justifies tighter, safer coupling. This is the relative yardstick for every later choice. See `references/complexity.md`.
2. **Pick a coupling strategy.** Name the coupling you are creating and grade its strength using the module, connascence, and domain models. Push toward weaker, more explicit coupling unless the change rate justifies the cost. See `references/coupling-models.md`.
3. **Enforce one-way dependency.** Requests imply knowing the target, so direction of interaction fixes the direction of dependency. Keep dependencies acyclic and linear (pipelining) so cause and effect stay traceable and partial edits stay safe. See `references/monorepo.md` and `references/ai-ownership.md`.
4. **Keep abstraction consistent.** Expose abstracted knowledge, never concrete internals, and apply one consistent abstraction criterion per module/layer — inconsistent abstraction makes modularization meaningless. See `references/abstraction.md`.

## Quick Reference

### Complexity & Context (HIGH)

- `complexity-test-safety-for-complex` — Secure a test safety net before changing runtime-coupled (complex) code

### Coupling Types & Threat Ranking (CRITICAL)

- `coupling-avoid-control-coupling` — No flag/mode arguments that steer a callee's internal branches (Control coupling)
- `coupling-no-implementation-leak` — Don't leak implementation knowledge through the interface
- `coupling-data-over-stamp` — Pass only the data needed; avoid train-wreck bridges
- `coupling-isolate-shared-resource` — Isolate shared-resource (Common/External) coupling

### Dependency Direction & Structure (CRITICAL)

- `dependency-unidirectional` — Keep dependencies unidirectional and acyclic
- `dependency-isolate-by-responsibility` — Isolate modules by responsibility (change-rate)
- `dependency-stable-direction` — Depend in the direction of stability (DDD subdomains)

### Abstraction & Module Boundary (HIGH)

- `abstraction-consistency` — Keep abstraction criteria and level consistent
- `abstraction-minimize-context` — Publish abstracted, not concrete, knowledge; minimize context
- `abstraction-encapsulate-knowledge` — Classify domain-specific vs general knowledge; share via contract

### Layered & Monorepo Architecture (MEDIUM)

- `architecture-layer-unidirectional` — Layered architecture: one-way dependency, watch N:N mapping
- `architecture-monorepo-apps-to-packages` — Turbo monorepo: apps→packages one-way, no lib→lib
- `architecture-linear-interconnection` — Control interconnection complexity: linear flow + message constraints

### AI-Friendly Ownership (MEDIUM)

- `ai-partial-ownership` — Structure code for AI partial ownership

## Deep references

- [references/complexity.md](references/complexity.md) — complexity, understandability, and the Cynefin framework for choosing coupling strength.
- [references/coupling-models.md](references/coupling-models.md) — module coupling, connascence (compile-time vs. implicit), and domain coupling models.
- [references/abstraction.md](references/abstraction.md) — interface/implementation/context split, knowledge classification, modeling/categorization/grouping, abstraction consistency.
- [references/ai-ownership.md](references/ai-ownership.md) — code ownership before and after AI, and structuring code for partial, context-bounded ownership.
- [references/monorepo.md](references/monorepo.md) — layered abstraction and a Turbo monorepo layout (`apps/` → `packages/`) enforcing one-way dependency.

Each rule listed above is a file under `rules/` — read the ones the change touches.

## Related skills

For clean-code, naming, and code-smell standards see `coding-standards`; for React component-level composition (compound components, lifting state) see `composition-patterns`; for server-side module and API design see `backend-patterns`. Do not duplicate their guidance here — link to them.
