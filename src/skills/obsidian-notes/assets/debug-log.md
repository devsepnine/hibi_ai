---
type: debug
status: open | resolved | archived
severity: low | medium | high | critical
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [type/debug, project/<slug>, topic/<area>]
project: <slug>
resolved_in: null      # "[[ADR-NNNN …]]" or "PR #123" once fixed
related: []
aliases: []
---

# Debug — <one-line symptom> (YYYY-MM-DD)

> [!bug] Symptom
> What the user (or monitoring) sees. Include the exact error message,
> stack trace snippet, or behavioral description. Avoid paraphrasing —
> the literal signal matters when searching later.

## Environment

- Version / commit: `<sha>` or `v1.9.3`
- OS / runtime: `<macOS 15.4, Rust 1.85>`
- Reproduction rate: `<always | flaky 20% | once>`
- First seen: <where / who reported>

## Timeline

| Time | What happened |
|------|---------------|
| HH:MM | Report arrived / alert fired |
| HH:MM | First hypothesis: <X> |
| HH:MM | Ruled out <X> via <check> |
| HH:MM | Hypothesis <Y> confirmed by <evidence> |
| HH:MM | Fix landed in <PR #>; verified |

## Hypotheses explored

### ✗ Hypothesis 1 — <name>

- Why suspected: …
- How tested: …
- Result: ruled out because … [evidence](<link>)

### ✓ Hypothesis 2 — <name>

- Why suspected: …
- Evidence: `<log snippet / repro command>`
- Root cause: <one-paragraph explanation>

## Root cause

> [!info] One-paragraph RC
> Plain-English explanation of the defect. If there's an ADR coming out
> of this, link it: [[ADR-NNNN …]].

## Fix

- What changed: `<file:line>` — before / after description.
- PR: <url> — merged <date>.
- Tests added: [[<test file or description>]] — regression coverage for
  this specific path.

## Prevention

- What would have caught this earlier? (lint rule, test, alert, review
  checklist item). Convert to actionable [ ] tasks and link the
  follow-up:
  - [ ] <preventive action> — [[<tracking note>]]

## Related

- [[ADR-NNNN …]] if a decision came out of this
- [[<release note that ships the fix>]]
- [[<similar past debug>]] for pattern recognition
