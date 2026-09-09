---
description: Restate requirements, assess risks, and create step-by-step implementation plan. WAIT for user CONFIRM before touching any code.
allowed-tools: Agent, Read, Grep, Glob
model: sonnet
effort: high
---

# Plan Command

Invokes the harness built-in `Plan` agent (no custom agent file needed) to produce an implementation plan before any code is written.

## When to Use

A new feature, a significant architectural change, a complex refactor, a change spanning several files, or requirements that are still ambiguous. Skip it for a single-file obvious edit — planning that costs more than it saves.

## What the agent produces

1. **Requirements restated** in unambiguous terms, with the ambiguities named rather than guessed
2. **Phases** — specific, actionable steps in dependency order
3. **Risks and blockers**, each with the severity that justifies the attention
4. **Complexity estimate** — High / Medium / Low
5. **A halt** — the plan is presented and nothing is written until you confirm

## Important Notes

**CRITICAL**: the agent writes **no code** until you reply with an explicit "yes", "proceed", or similar affirmative.

To steer instead of approving:
- `modify: <your changes>`
- `different approach: <alternative>`
- `skip phase 2 and do phase 3 first`

## Integration with Other Commands

After planning: `/tdd` to implement test-first · `/build-fix` when the build breaks · `/code-review` on the finished implementation.
