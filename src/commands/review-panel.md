---
description: Review an A/B-tier diff with a panel of code-reviewer agents, one lens each, then merge their findings into Act on, Consider, and Dismissed with a verdict.
argument-hint: "[file | PR number | base branch]"
allowed-tools: Agent, Read, Grep, Glob, Bash
model: sonnet
effort: high
---

# Review Panel

An independent review for a diff whose tier in `CLAUDE.md` §9 is A or B. `$ARGUMENTS` names a file, a PR number, or a base branch; with none, use the uncommitted diff, then `git diff main...HEAD`. For a C to E tier diff, the single post-work `code-reviewer` gate is enough; say so and stop.

`/code-review`, or the `code-reviewer` agent, stays the default review gate, and this panel is the extra step for A and B tiers. Independence comes from separate agents each holding one lens. The deliverable is a verdict. Do not apply fixes, commit, or push.

## Steps

1. **Fix the scope and the tier.** Package the diff plus the context files a reviewer needs to read it. Confirm the tier with the `do-178c` skill.
2. **State the intent** in one paragraph, drawn from the request, commit messages, the PR body, and the code. If the intent is unclear, ask before going on.
3. **Launch the panel in one message.** Each reviewer is a `code-reviewer` agent and gets the diff, the intent paragraph, and its one lens. Tell each to judge the execution, not the intent, and to return an empty list when it finds nothing.
   - **Correctness and data loss**: edge cases, error paths, idempotency, partial writes, concurrent access. Trace each execution path it claims.
   - **Security** per the `security-review` skill. Trace each input from its source to the sink.
   - **Design and coupling** per the `dependency-design` skill.
   - **Tests** per `tdd-workflow`: does each assertion fail when the behavior breaks, and does each requirement have a test.
   - For tier A only, add `assurance-auditor` for traceability and structural coverage.
4. **Merge as the lead.** You hold the full session context, so filter and decide rather than aggregate.
   - Deduplicate findings that describe the same issue in different words, and record which lenses raised each.
   - Keep a finding that two lenses raised independently.
   - Re-verify a single-lens finding by tracing the code yourself before you keep it.
   - Apply the filters in `code-reviewer` under "Filter Before Reporting": actual, not hypothetical; a concrete problem, not a preference; no premature abstraction; caused or exposed by this change; no nits inflated into warnings.
   - Never drop a security or correctness finding without tracing its path.
5. **For tier A**, end by asking for human review; the panel does not replace it.

## Reply

- **Intent**: the paragraph from step 2.
- **Panel**: each lens and its finding count.
- **Act on**: about 5 at most. Each with location, the lenses that raised it, and why it blocks a merge. A longer list means the filter was too loose.
- **Consider**: real points whose cost may outweigh fixing them now, each with its tradeoff.
- **Dismissed**: one line each on why, so the user can overrule you.
- **Verdict**: `[APPROVE]`, `[WARN]`, or `[BLOCK]` per the `code-reviewer` approval criteria, plus the human review call for tier A.
