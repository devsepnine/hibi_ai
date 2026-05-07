# CLAUDE.md — Orchestration Flow

Defines workflow and decision-making procedures. For policy details, see `rules/*.md`.

## Pre-work checklist

- **Identify current state**: run `git status` to surface existing changes
- **Understand file structure**: read related code and dependencies
- **Stay in scope**: do not change anything outside the requested area
- **Simple requests**: execute immediately. Only complex requests warrant a planning step.

## Thinking and response language policy (CRITICAL)

- **Thinking step**: reason in English — more precise reasoning
- **Output**: respond in Korean — user readability first
- **Code, commands, technical terms, error messages**: keep in original (English)

## Workflow orchestration

### 1. Plan-mode default
- Tasks with 3+ steps or architectural decisions → enter plan mode
- If things drift from intent, STOP immediately and re-plan
- Reduce ambiguity by writing a detailed spec up front (Q&A first)

### 2. Subagent strategy
- Use subagents aggressively to keep the main context clean
- Delegate research, exploration, and parallel analysis to subagents
- One task per subagent (Anthropic guide: low effort + explicit checklist)

### 3. Self-improvement loop
- On every user correction, record the pattern in `MEMORY.md`
- Write a rule that prevents the same mistake — apply it immediately
- Review relevant lessons at the start of each session

### 4. Verify before completion
- Never mark work complete without proof it works
- Ask: "Would a senior engineer approve this?"
- Run tests, check logs, prove correctness

### 5. Pursue elegance (with balance)
- For non-obvious changes ask: "Is there a more elegant approach?"
- If the fix feels temporary: "Implement the obvious, clear solution given everything I now know"
- Skip this step for simple, clear changes — no over-engineering
- **Judgment criterion**: "Will I understand this code three months from now?"
- **Refactor signal**: same pattern repeated 3 times (Rule of Three)

### 6. Autonomous bug fixes
- When you receive a bug report, just fix it — do not ask for step-by-step instructions
- Track logs, errors, and failing tests yourself
- **Root-cause first**: do not patch symptoms
- **Prevent recurrence**: check whether the same class of bug exists elsewhere

### 7. Parallel execution principle
- Independent work always runs in parallel (multiple Task calls in a single message)
- Never run 3 unrelated analyses sequentially when there are no dependencies

### 8. Git and change safety (CRITICAL)
- **Forbidden**: `commit`, `push`, or branch-strategy changes the user did not explicitly request
- **Protect existing changes**: never silently undo user changes
- **Detect unexpected changes**: stop and confirm if you find changes you did not make
- **Destructive commands**: `reset --hard`, `rm -rf`, `push --force` require explicit approval

## Effort × model policy

Per the Anthropic Opus 4.7 guide. See `rules/performance.md` for details.

| Effort | Model | Use cases |
|---|---|---|
| `low` | `claude-haiku-4-5` | Single-tool checklist, narrow scope (subagents, classification, quick lookups) |
| `medium` | `claude-sonnet-4-6` | Balanced — tool calls with some reasoning |
| `high` | `claude-sonnet-4-6` | Complex reasoning, careful judgment |
| `xhigh` | `claude-opus-4-7` | Coding, exploration, multi-step (repeated tool calls, deep search) |
| `max` | — | True frontier only (not recommended for typical workloads) |

**Core principle**: *"Don't prompt around — raise the effort."* Opus 4.7 strictly respects effort. At lower effort it scopes to what was asked and nothing more.

**Tool usage at low effort**: combine calls, use fewer of them, act directly → terse confirmation.
**Tool usage at high effort**: explain the plan before acting, more calls, detailed summaries.

## Automatic agent invocation

- Right after writing code → `code-reviewer`
- New feature or bug fix → `tdd-guide` (tests first)
- Build failure → `build-error-resolver`
- Pre-commit → `security-reviewer`

## Completion report format

1. **What changed**: file paths with line numbers
2. **Why it changed**: rationale
3. **Verification**: how you proved it works (test / build results)
4. **Next steps**: numbered follow-ups when natural

**Example:**
```
Changed: src/auth/login.ts:42-58 — login validation logic
Why: empty email caused a server error → added client-side validation
Verified:
  - unit tests pass (`npm test auth.test.ts`)
  - E2E pass (happy path + empty input)
Next:
  1. Apply the same pattern to password validation
  2. Add multi-language error messages
```

## Core code-quality principles

- **Simple first**: minimal change. YAGNI / KISS.
- **No laziness**: root-cause analysis. No temporary fixes. Senior-level standard.
- **Minimal blast radius**: change only what's needed. Avoid side effects.
- **Compare at least two alternatives** → state trade-offs → confirm reversibility.

## References (DRY — define each policy in one place)

- **Review four-criteria (SOLID, Clean Code, Functionality, Consistency)**: `agents/review-checklist.md`
- **Code thresholds (LOC, complexity)**: `rules/code-thresholds.md`
- **Commit convention**: `rules/commit-convention.md`
- **PR guidelines**: `rules/pull-request-rules.md`
- **Security rules**: `rules/security.md`
- **Testing requirements**: `rules/testing.md`
- **Agent mapping**: `rules/agents.md`
- **Effort × model details**: `rules/performance.md`
