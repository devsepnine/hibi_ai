# Assurance Levels (DAL A–E) — Classification Guide

The assurance tier is the master dial of the `do-178c` overlay. Set it once, and it tunes every existing gate — coverage depth, review independence, security sign-off, verification breadth. Everything else in the overlay follows from this one decision.

Classify by the **worst-case blast radius of failure**, never by effort, code size, or how hard the task felt. A three-line change to an auth check is Tier A; a 2,000-line dashboard refactor may be Tier C. Ask what breaks if the code is wrong and ships — not how much you typed.

> Agent and slash-command names below assume Claude Code. Codex installs skills only — there, read `code-reviewer` as a separate reviewing pass or a human reviewer, `assurance-auditor` as optional-where-available with human review either way, and `/verify` as the `verification-loop` skill. The required rigor does not change with the harness; only the mechanism does.

## Canonical Tier Table

| Tier | DO-178C | Failure impact | Examples | Required rigor (dials existing gates) |
|---|---|---|---|---|
| A | Catastrophic | Irreversible; data/money/security loss | auth, payments, crypto, DB migration/deletion | Max: requirements-based tests + decision/branch coverage (MC/DC aspirational); bidirectional traceability; independent `code-reviewer` + `assurance-auditor` + human review; `security-review` sign-off; full `verification-loop` |
| B | Hazardous | Major breakage, hard to reverse | core business logic, public API contracts, persistent state | High: decision/branch coverage; traceability matrix; independent `code-reviewer` (`assurance-auditor` recommended); `verification-loop` |
| C | Major | Degraded UX, recoverable | internal features, dashboards, non-critical endpoints | Standard: baseline coverage (see `tdd-workflow`); `code-reviewer`; `/verify` |
| D | Minor | Cosmetic, easily fixed | logging, copy, styling | Light: lint/type-check; review optional |
| E | No effect | Throwaway/experimental | scratch scripts, spikes | None required |

## How to Classify

A short decision procedure:

1. Ask: **"What is the worst thing that happens if this is wrong and ships?"**
2. Map that severity to a tier:
   - Irreversible loss of data, money, or security posture → **A**
   - Major breakage that is hard to reverse → **B**
   - Degraded but recoverable user experience → **C**
   - Cosmetic, trivially fixable → **D**
   - No user-visible effect; throwaway → **E**
3. **When uncertain, round up one tier.** The cost of over-assuring a C as a B is small; the cost of under-assuring an A as a C is the failure mode the overlay exists to prevent.
4. **Reclassify if scope changes mid-task.** If a "dashboard tweak" grows to touch the auth path, stop and re-tier — the new gates apply to the whole change.

## What Each Tier Requires

Each tier dials the existing gates; it does not introduce new procedures. Read the owning skill for the actual mechanics.

- **Tier A** — Maximum rigor. Requirements-based tests at the highest coverage bar in `tdd-workflow`, with bidirectional traceability linking each requirement to tests and code. Full-depth `verification-loop`. Independent review by both `code-reviewer` and `assurance-auditor`, plus a human gate. Mandatory `security-review` sign-off. Strictest `dependency-design` coupling discipline so a failure cannot ripple.
- **Tier B** — High rigor. Decision/branch coverage per `tdd-workflow`, a traceability matrix, and full `verification-loop`. Independent `code-reviewer`; `assurance-auditor` recommended. Apply `dependency-design` to keep the blast radius contained.
- **Tier C** — Standard rigor. Baseline coverage from `tdd-workflow`, `code-reviewer`, and `/verify` from `verification-loop`. This is the everyday default.
- **Tier D** — Light touch. Lint and type-check only; review is optional. Near-zero overhead — do not manufacture ceremony for logging or copy changes.
- **Tier E** — None required. Throwaway and experimental work carries no assurance obligation. Near-zero overhead by design.

Do not restate coverage numbers, verification steps, or review checklists here — they live in `tdd-workflow`, `verification-loop`, `code-reviewer`, `security-review`, and `dependency-design` respectively. The tier only selects which of those to apply and how strictly.

## Declaring the Tier

State the tier at task start so reviewers know which gate set to check:

- In the **plan** (the `Problem 1-Pager` from `CLAUDE.md` section 1) for tasks that warrant planning.
- In the **PR description** (see `pull-request`) so reviewers verify the right gates ran.
- In the **commit context** (see `commit-rules`) for the baseline record.

Recording the tier is what makes assurance auditable: a reviewer can confirm an A-tier change actually went through `assurance-auditor` and `security-review`, not just `code-reviewer`.

## Independence by Tier

Independence means the reviewer is not the author. This maps onto the existing "Mandatory post-work review" gate in `CLAUDE.md` section 4 — the tier only sets how many independent eyes are required. Do not duplicate that gate; dial it.

| Tier | Independent review required |
|---|---|
| A | Separate `assurance-auditor` **and** human review (on top of `code-reviewer`) |
| B | Independent `code-reviewer` (`assurance-auditor` recommended) |
| C | `code-reviewer` |
| D | Optional |
| E | Optional |
