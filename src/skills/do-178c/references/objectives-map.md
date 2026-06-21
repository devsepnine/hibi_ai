# DO-178C to Harness Objectives Map

This is THE anti-duplication map. For each DO-178C process area it names the existing harness component that already owns the work, so the `do-178c` skill never restates it. The `do-178c` skill is a thin overlay: it cross-references the owners below and adds only four net-new items.

DO-178C groups work into processes: planning, development, verification, configuration management, and quality assurance, plus tool qualification (DO-330). The harness already implements most of these through existing skills, agents, and `CLAUDE.md` policy. The table below maps each DO-178C process area to its existing owner and states what, if anything, `do-178c` adds.

Action legend:
- `CROSS-REF` — the owner is sufficient; `do-178c` only points to it and may demand it explicitly for higher tiers.
- `EXTEND` — the owner exists but `do-178c` stretches its scope.
- `NEW` — no current owner; `do-178c` introduces this.

## Mapping Table

| DO-178C process / objective | Existing harness owner | `do-178c` action (CROSS-REF / EXTEND / NEW) |
|---|---|---|
| Planning (the 5 plans: PSAC / SDP / SVP / SCMP / SQAP) | `CLAUDE.md` section 1 Problem 1-Pager + plan mode | CROSS-REF — one up-front spec per non-trivial task replaces the five-plan set; no separate plan documents |
| Requirements (HLR / LLR) | Problem 1-Pager + `eval-harness` skill | CROSS-REF — the 1-Pager captures intent; `eval-harness` encodes acceptance criteria as evals |
| Coding standards | `coding-standards` skill | CROSS-REF — that skill is the single source for naming, style, and code smells |
| Requirements-based testing (normal + robustness) | `tdd-workflow` skill | CROSS-REF — additionally demand robustness and boundary cases explicitly, not just happy-path tests |
| Test coverage thresholds | `tdd-workflow` skill | CROSS-REF — defer to its percentages; never restate the numbers here |
| Structural coverage (statement / decision; MC/DC) | none today | NEW — gate at statement then decision/branch; MC/DC aspirational only (see policy below) |
| Verification (review / analysis / test; verification of verification) | `verification-loop` skill + `code-reviewer` agent | CROSS-REF — plus a meta-check that tests are not tautological (verification of the verification) |
| Bidirectional traceability | none today | NEW — link requirement to test to code, navigable in both directions |
| Independence of verification (reviewer != author) | `CLAUDE.md` section 4 post-work review + `code-reviewer` agent | CROSS-REF — for A-tier add the `assurance-auditor` agent on top of `code-reviewer` |
| Configuration management / baselines | `commit-rules` skill + `pull-request` skill | CROSS-REF — commits and PRs are the baseline and change-control mechanism |
| Problem reporting | issue tracker / ticket scheme | CROSS-REF — tickets are the problem-report record; reference them in commits per `commit-rules` |
| SQA gate | `verification-loop` READY state + `pull-request` pre-PR checklist | CROSS-REF — the existing gates are the quality sign-off |
| Assurance levels (DAL) | none today | NEW — the criticality/assurance tier dial (A through E) |
| Derived requirements feedback | none today | NEW — reuse the `CLAUDE.md` section 3 self-improvement / `MEMORY.md` pattern to feed discovered requirements back |
| Tool qualification (DO-330) for AI tools / checkers | `eval-harness` skill | CROSS-REF / EXTEND — eval the AI tools and checkers you rely on; treat a passing eval suite as the qualification evidence |

## The Four Net-New Items

Everything above is a pointer to an existing owner except these four, which the `do-178c` skill owns outright:

1. **Criticality / assurance tier dial (DAL A through E).** A single dial that sets how hard the existing gates push. It does not add new gates; it dials the rigor of `tdd-workflow`, `verification-loop`, `code-reviewer`, `assurance-auditor`, and `security-review` up or down by tier.

2. **Bidirectional traceability.** A navigable link from each requirement to its tests to its code, and back. No existing skill maintains this mapping.

3. **Structural-coverage discipline.** Gate at statement coverage, then decision/branch coverage. MC/DC is the A-tier ideal only — most JS/TS/Python stacks cannot measure true MC/DC, so do not gate on it. Actual coverage percentages live in `tdd-workflow`, not here.

4. **Derived-requirement feedback.** When implementation or testing surfaces a requirement that was not in the original spec (a derived requirement), feed it back into the spec and re-evaluate its criticality tier, reusing the section 3 self-improvement / `MEMORY.md` loop.
