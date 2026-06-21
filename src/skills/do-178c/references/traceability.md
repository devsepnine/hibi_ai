# Bidirectional Traceability

Traceability is the connective tissue that proves a change is complete and contains nothing extra. It links three things that already exist in this harness — a requirement, the code that implements it, and the test that exercises it — so that you can answer two questions with evidence rather than memory: *did we build everything we were asked to?* and *did we build only that?*

This reference owns only the trace discipline. It does not define requirements (see CLAUDE.md section 1 "Problem 1-Pager" and the `eval-harness` skill), the tests themselves (see the `tdd-workflow` skill), or the review gate that audits the trace (see CLAUDE.md section 4 and the `code-reviewer` agent).

## Why Bidirectional

A single direction catches only one class of failure. You need both.

- **Forward traceability** (requirement → code → test) proves nothing was *dropped*: every requirement has implementing code and a covering test. It catches the unimplemented feature and the untested requirement.
- **Backward traceability** (code → requirement) proves nothing was *smuggled in*: every changed function and branch traces back to a requirement. It catches orphan code, speculative generality, and gold-plating — work that nobody asked for and that now has to be maintained forever.

Run only the forward check and you ship gold-plated code that passes every test. Run only the backward check and you ship a clean, minimal codebase that quietly omits a requirement. The two checks are not redundant; each is blind to the other's failure mode.

## The Trace Key

Do not invent a parallel identifier scheme. Traceability reuses the identifiers the harness already produces; the matrix is just a join over them.

| Identifier | Origin | SSOT |
|---|---|---|
| `[TICKET]` | The ticket id in the commit/PR title | `commit-rules`, `pull-request` skills |
| Eval id | The id of an eval case | `eval-harness` skill |
| 1-Pager item | A Goal/requirement line from the Problem 1-Pager | CLAUDE.md section 1 |
| Test id/name | The test function name or case id | `tdd-workflow` skill |

A requirement is keyed by whichever of these fits its origin (a ticket for feature work, an eval id for an AI-behavior requirement, a 1-Pager line for a spec item). Code is keyed by `file:symbol`. Tests are keyed by their id or name. The matrix connects these keys — it never replaces them.

## Lightweight Trace Matrix

The matrix is a small Markdown table, not a tool. Keep it where the change is reviewed: inline in the PR description, or in an optional `TRACE.md` for a larger feature. One row per requirement.

| Req (1-Pager item / ticket / eval) | Code (`file:symbol`) | Test (id/name) | Status |
|---|---|---|---|

Worked example for a small auth change:

| Req (1-Pager item / ticket / eval) | Code (`file:symbol`) | Test (id/name) | Status |
|---|---|---|---|
| `[AUTH-42]` reject empty email | `src/auth/login.ts:validateEmail` | `login.test.ts > rejects empty email` | Done |
| `[AUTH-42]` lock account after 5 failures | `src/auth/login.ts:recordFailure` | `login.test.ts > locks after 5 failures` | Done |
| eval `auth-refusal-01` no credential echo in error | `src/auth/login.ts:errorResponse` | `evals/auth-refusal-01` | Done |

The matrix earns its keep precisely because it is cheap. The moment it needs a database or a dedicated tool, it has outgrown its purpose for this harness.

## Forward Check

Read down the requirement column. Every requirement row must name at least one covering test.

- A requirement with code but no test is **untested** — flag it and route to the `tdd-workflow` skill to add the test before the change is considered complete.
- A requirement with neither code nor test is **unimplemented** — flag it; the change does not satisfy its own spec.

The forward check is the test-completeness gate seen from the requirement side. It does not define how much coverage a test must give — coverage thresholds live in the `tdd-workflow` skill.

## Backward Check

Read up from the code. Every changed function and every new branch must trace back to a requirement row.

- Code that traces to **no requirement and no test** is orphan code. Resolve it one of two ways:
  - **Delete it** if it is gold-plating or speculative generality — the cheapest code is the code you do not keep.
  - **Capture it as a derived requirement** if it is genuinely needed but implicit (an error path, a guard, a performance constraint the spec assumed but never stated). Add a requirement row for it so it becomes traceable and testable like any other. Feeding these derived requirements back into the spec is itself a deliverable — see the derived-requirements discipline in the `do-178c` skill.

The backward check is where over-engineering gets caught structurally rather than by taste.

## When to Maintain It

Tie the rigor to the criticality tier (see the tier table in the `do-178c` skill).

- **Tier A / B**: the trace matrix is **required**. The `assurance-auditor` agent audits both the forward and backward checks as part of the independent review gate.
- **Tier C and below**: optional. A reviewer may still ask for it on a confusing change, but it is not gated.

The `assurance-auditor` agent owns the audit of these two checks; the `tdd-workflow` skill owns the tests they point to; this reference owns only the linkage between them.
