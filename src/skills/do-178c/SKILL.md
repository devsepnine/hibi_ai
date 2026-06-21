---
name: do-178c
description: Pragmatic DO-178C-derived assurance methodology for general/AI-assisted development - risk-tiered rigor, bidirectional traceability, independent verification, structural coverage, and derived-requirement feedback. Use when working on safety-critical or high-blast-radius changes, assigning an assurance level, building traceability, or auditing verification rigor. 안전필수 개발, 보증수준, 위험도 티어, 양방향 추적성, 독립 검증, 구조 커버리지, 파생요구, DO-178C 개발론.
keywords: [do-178c, assurance-level, traceability, structural-coverage, derived-requirements, 보증수준, 추적성, 안전필수, 위험도티어]
---

# DO-178C Assurance Methodology

DO-178C is an avionics software certification standard, but three of its principles transfer cleanly to any development: rigor should scale to the risk a change carries, every requirement should trace both ways to the code and tests that satisfy it, and the person who verifies work should not be the person who built it. This skill steals those transferable principles and applies them pragmatically — it does not import certification paperwork or avionics procedure.

This harness already owns most of the gates these principles imply: tests and coverage, the verification loop, the independent review gate, coupling rules, and security sign-off all live in existing skills. So this skill is a thin overlay. It adds only four net-new things — the criticality/assurance tier dial, bidirectional traceability, structural-coverage discipline, and derived-requirement feedback — and delegates everything else by cross-reference.

## When to Use

- A change is safety-critical or has a high blast radius (auth, payments, crypto, data migration, persistent state, public API contracts).
- You need to assign an assurance level to a piece of work and dial its rigor accordingly.
- You need traceability between requirements, code, and tests (forward or backward).
- You are auditing whether the verification rigor matches the risk.

D/E-tier work (cosmetic, throwaway, experimental) can skip this skill — the overhead is not justified.

## Delegates To (DRY)

This skill restates nothing the harness already owns. It points at the SSOT and adds only the tier-scaling on top.

| DO-178C concern | Owned by (existing SSOT) | This skill adds |
|---|---|---|
| Requirements baseline | CLAUDE.md "Problem 1-Pager" + `eval-harness` | Tier-gated baseline before A/B work |
| Requirements-based tests | `tdd-workflow` | Trace each test to a requirement |
| Coverage thresholds | `tdd-workflow` | Structural-coverage discipline per tier |
| Verification activity | `verification-loop` | Scale which checks run by tier |
| Independent review | CLAUDE.md "Mandatory post-work review" + `code-reviewer` | `assurance-auditor` + human gate for A-tier |
| Coupling / dependency direction | `dependency-design` | Tighter scrutiny at A/B |
| Security sign-off | `security-review` | Mandatory for A-tier |
| Configuration management / baselines | `commit-rules` + `pull-request` | Trace key on commits/PRs |

The full objective-to-owner map is in `references/objectives-map.md`.

## Core Flow

1. **Classify the assurance level (A-E).** Judge failure impact and blast radius, then assign a tier. The tier is the master dial for every later step. See `references/assurance-levels.md`.
2. **Establish the requirements baseline.** For A/B work, write the Problem 1-Pager (CLAUDE.md section 1) and capture acceptance evals before implementing — cross-ref `eval-harness`. The baseline is what traceability traces to.
3. **Implement; surface derived requirements.** As you build, any behavior introduced below the spec (a retry, a cache, a default, an error code) is a derived requirement — surface it, do not bury it. See Derived-Requirement Feedback below.
4. **Build bidirectional traceability.** Link every requirement forward to its code and tests, and every changed unit backward to a requirement. Flag orphans. See `references/traceability.md`.
5. **Run tier-scaled verification.** Delegate the activity to `verification-loop` and the tests/thresholds to `tdd-workflow`, then add structural coverage at the tier's required level. See Structural Coverage below.
6. **Get independent verification.** Run the CLAUDE.md section-4 post-work `code-reviewer` gate (reviewer != author). For A-tier, also run the `assurance-auditor` agent and require human review.
7. **QA sign-off.** Compose the existing gates rather than inventing a new one: `verification-loop` must report READY and the `pull-request` pre-PR checklist must pass before the change is considered done.

## Assurance Levels

The tier is the master dial — it sets the required rigor for coverage, traceability, review, and security on every step above. Classify once, early; everything downstream reads from it.

| Tier | DO-178C | Failure impact | Examples | Required rigor (dials existing gates) |
|---|---|---|---|---|
| A | Catastrophic | Irreversible; data/money/security loss | auth, payments, crypto, DB migration/deletion | Max: requirements-based tests + decision/branch coverage (MC/DC aspirational); bidirectional traceability; independent `code-reviewer` + `assurance-auditor` + human review; `security-review` sign-off; full `verification-loop` |
| B | Hazardous | Major breakage, hard to reverse | core business logic, public API contracts, persistent state | High: decision/branch coverage; traceability matrix; independent `code-reviewer` (`assurance-auditor` recommended); `verification-loop` |
| C | Major | Degraded UX, recoverable | internal features, dashboards, non-critical endpoints | Standard: baseline coverage (see `tdd-workflow`); `code-reviewer`; `/verify` |
| D | Minor | Cosmetic, easily fixed | logging, copy, styling | Light: lint/type-check; review optional |
| E | No effect | Throwaway/experimental | scratch scripts, spikes | None required |

See `references/assurance-levels.md` for the classification guide.

## Structural Coverage

Structural coverage measures which parts of the code the tests actually exercised — a check on test completeness, not a substitute for requirements-based tests.

- **Statement coverage** — every statement executed at least once.
- **Decision/branch coverage** — every branch of every decision taken both ways (true and false).
- **MC/DC (Modified Condition/Decision Coverage)** — each condition independently shown to affect the decision outcome. This is the A-tier ideal, but most JS/TS/Python toolchains cannot measure true MC/DC, so do not gate on it; treat it as aspirational.

Gate at statement coverage, then decision/branch coverage, per the tier table above. Defer actual percentages to `tdd-workflow` — it owns the numbers.

When coverage reveals unexercised code, triage it: **dead code** (unreachable, no requirement) should be removed; **deactivated code** (intentionally inactive — a feature flag, a guarded fallback) should be justified and documented, not deleted.

## Derived-Requirement Feedback

A derived requirement is behavior introduced below the spec — something the implementation needs that the requirements never stated. Common forms: a retry policy, a cache, a default value, an internal error code, a timeout.

The rule: surface every derived requirement to the spec owner so it can be reviewed and, if accepted, folded into the baseline. Record the pattern via the CLAUDE.md section-3 self-improvement loop and `MEMORY.md`. Never embed it silently. This is the direct antidote to an AI quietly adding unrequested behavior — by forcing each below-spec decision to surface, derived-requirement feedback keeps the implementation honest against the baseline. If a derived requirement changes the blast radius (e.g. it newly touches money, secrets, or persistent state), re-classify the assurance tier accordingly.

## Bidirectional Traceability

Traceability runs both ways. **Forward**: every requirement must have implementing code and a test that proves it — a gap means an unbuilt or untested requirement. **Backward**: every changed unit must trace to a requirement — an untraceable change is an orphan (scope creep or a hidden derived requirement) and must be flagged.

Reuse the IDs the harness already produces as the trace key: ticket IDs (`commit-rules` / `pull-request`), eval names (`eval-harness`), and test names (`tdd-workflow`). Do not invent a parallel ID scheme. See `references/traceability.md`.

## Deep References

- `references/assurance-levels.md` — tier classification guide and worked examples.
- `references/traceability.md` — building and auditing the forward/backward trace.
- `references/objectives-map.md` — full DO-178C-objective-to-harness-owner map.

## Related Skills

Link to these; do not duplicate their content.

- `tdd-workflow` — requirements-based tests and coverage thresholds (owns the numbers).
- `verification-loop` — build/type/lint/test/security verification activity and the READY gate.
- `eval-harness` — acceptance evals that form the requirements baseline.
- `dependency-design` — coupling and dependency-direction scrutiny.
- `security-review` — security sign-off, mandatory at A-tier.
- `commit-rules` — commit conventions; carries the trace key on commits.
- `pull-request` — pre-PR checklist and PR conventions.
- `assurance-auditor` agent / `/do-178c` command — independent assurance audit for A-tier work.
