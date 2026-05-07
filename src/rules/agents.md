# Agent Orchestration

## Available Agents

Located in `~/.claude/agents/`. Effort and model recommendations follow the Anthropic Opus 4.7 guide (see `performance.md`).

| Agent | Purpose | When to Use | Effort | Model |
|-------|---------|-------------|--------|-------|
| planner | Implementation planning | Complex features, refactoring | high | sonnet-4-6 |
| architect | System design | Architectural decisions | xhigh | opus-4-7 |
| tdd-guide | Test-driven development | New features, bug fixes | medium | sonnet-4-6 |
| code-reviewer | Code review | After writing code | medium | sonnet-4-6 |
| security-reviewer | Security analysis | Before commits | xhigh | opus-4-7 |
| build-error-resolver | Fix build errors | When build fails | medium | sonnet-4-6 |
| e2e-runner | E2E testing | Critical user flows | xhigh | sonnet-4-6 |
| refactor-cleaner | Dead code cleanup | Code maintenance | xhigh | sonnet-4-6 |
| doc-updater | Documentation | Updating docs | xhigh | sonnet-4-6 |

## Effort Policy (Subagents)

Subagents are independent workers — by default they should use **`low` or `medium` effort with explicit checklists** to keep cost and latency low. Raise to `xhigh` only when the subagent must do agentic exploration (multi-step search, repeated tool calls).

The Anthropic guide notes: *"Lower effort is the best fit for subagents."* Pair `low` with an explicit checklist for multi-section tasks.

## Immediate Agent Usage (No User Prompt Needed)

1. Complex feature requests → **planner** agent
2. Code just written/modified → **code-reviewer** agent
3. Bug fix or new feature → **tdd-guide** agent
4. Architectural decision → **architect** agent
5. Build failure → **build-error-resolver** agent
6. Pre-commit security check → **security-reviewer** agent

## Parallel Task Execution

ALWAYS use parallel Task execution for independent operations:

```
GOOD: Launch 3 agents in parallel in a single message:
1. Agent 1: Security analysis of auth.ts
2. Agent 2: Performance review of cache system
3. Agent 3: Type checking of utils.ts

BAD: Sequential execution when there are no dependencies.
```

## Multi-Perspective Analysis

For complex problems, use split-role sub-agents:
- Factual reviewer
- Senior engineer
- Security expert
- Consistency reviewer
- Redundancy checker

Each perspective gets its own subagent with focused scope.
