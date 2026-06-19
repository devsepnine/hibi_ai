# CLAUDE.md — Orchestration Flow

Defines always-on workflow and decision-making procedures. Detailed, situational policies live in **Skills** (loaded on demand) — see the policy-routing table at the bottom.

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
- For complex/unclear problems, draft a **Problem 1-Pager** first: Background / Problem / Goal / Non-goals / Constraints — request an interview if any item is ambiguous

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
- **Mandatory post-work review**: after any code/content change and before reporting completion, review the diff — run the `code-reviewer` agent (or `/code-review`) on the changed files; for dependency/coupling/module/monorepo changes also apply the `dependency-design` skill. Apply or explicitly defer each finding. Skip only for pure conversation or trivial non-code edits.

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

Per the Anthropic Opus 4.7 guide.

| Effort | Model | Use cases |
|---|---|---|
| `low` | `claude-haiku-4-5` | Single-tool checklist, narrow scope (subagents, classification, quick lookups) |
| `medium` | `claude-sonnet-4-6` | Balanced — tool calls with some reasoning |
| `high` | `claude-sonnet-4-6` | Complex reasoning, careful judgment |
| `xhigh` | `claude-opus-4-7` | Coding, exploration, multi-step (repeated tool calls, deep search) |
| `max` | — | True frontier only (not recommended for typical workloads) |

**Core principle**: *"Don't prompt around — raise the effort."* Opus 4.7 strictly respects effort. At lower effort it scopes to what was asked and nothing more.

**Model selection**: Haiku 4.5 for frequent lightweight workers; Sonnet 4.6 for main dev/tool-heavy work; Opus 4.7 for deep reasoning and long (30min+) agent tasks.

**Tool usage at low effort**: combine calls, use fewer of them, act directly → terse confirmation.
**Tool usage at high effort**: explain the plan before acting, more calls, detailed summaries.

## Agent routing

Agents are isolated workers (own context window, scoped tools) — use them to keep the main context clean and to parallelize. Subagents default to **`low`/`medium` effort with explicit checklists**; raise to `xhigh` only for agentic exploration (multi-step search, repeated tool calls).

| Trigger | Agent | Effort / Model |
|---|---|---|
| Complex feature / refactor planning | the built-in `Plan` agent | high / sonnet-4-6 |
| Architectural decision | `architect` | xhigh / opus-4-7 |
| New feature or bug fix (tests first) | `tdd-guide` | medium / sonnet-4-6 |
| Right after writing code; also pre-commit security checks (secrets, injection, auth) via the `security-review` skill | `code-reviewer` | medium / sonnet-4-6 |
| Build / type failure | `build-error-resolver` | medium / sonnet-4-6 |
| Critical user flows | `e2e-runner` | xhigh / sonnet-4-6 |
| Dead code cleanup | `refactor-cleaner` | xhigh / sonnet-4-6 |
| Documentation | `doc-updater` | xhigh / sonnet-4-6 |

**Parallel execution**: launch independent agents in a single message (multiple Task calls). Never run unrelated analyses sequentially.
**Multi-perspective analysis**: for complex problems, split into focused subagents (factual / senior-engineer / security / consistency / redundancy), one scope each.

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
- **Keep it small**: tasks, commits, PRs small. Record assumptions in Issues/PRs/ADRs.
- **Validate inputs, encode outputs**: never trust unvalidated input.
- **Names over abstraction**: intention-revealing names; avoid premature abstraction.

## Absolute commit rules (always apply — skill carries the rest)

Even before the `commit-rules` skill loads, these are non-negotiable on any commit:
- **NO emojis, NO generation markers** (`Co-Authored-By`, "Generated with Claude Code", etc.)
- **Only commit when the user explicitly asks.** Never auto-commit after finishing work.
- Format: `<type>: [<ticket>] <title>` — full convention in the `commit-rules` skill.

## Policy routing (DRY — each policy has ONE source of truth)

Detailed policies are **Skills**: their content loads only when triggered, keeping always-on context lean. Invoke the skill (or its `/command`) when the situation matches.

| Policy | Source of truth (SSOT) | How to load |
|---|---|---|
| Commit convention | `commit-rules` skill | `/commit` or trigger on git commit |
| PR guidelines | `pull-request` skill | `/pull-request` or trigger on PR work |
| Security rules / OWASP | `security-review` skill | `/security-review` or trigger on auth/input/secrets |
| Testing & TDD | `tdd-workflow` skill | `/tdd` or trigger on new feature/bugfix |
| Coding style / clean code | `coding-standards` skill | trigger on code review/writing |
| Dependency / coupling design | `dependency-design` skill | `/deps` or trigger on module/coupling/dependency/monorepo design |
| Build & type errors | `verification-loop` skill | `/verify`, `/build-fix` |

On-demand references (in the `coding-standards` skill, read when relevant):
- **Code thresholds (LOC, complexity)**: `references/code-thresholds.md`
- **Review checklist (SOLID, severity, concurrency, cross-platform)**: `references/review-checklist.md`
- **Common TS patterns**: `references/patterns.md`
