---
description: Sequential multi-agent workflow for complex tasks. Coordinates the built-in Plan agent, tdd-guide, and code-reviewer in handoff chain.
argument-hint: "[workflow-type] [task-description]"
allowed-tools: Task, Read, Bash, Grep
model: opus
effort: xhigh
---

# Orchestrate Command

Sequential agent workflow. Usage: `/orchestrate [workflow-type] [task-description]`

## Workflow Types

| Type | Agent Chain | Use Case |
|------|-------------|----------|
| `feature` | the built-in `Plan` agent -> tdd-guide -> code-reviewer | Full feature build |
| `bugfix` | the built-in `Explore` agent -> tdd-guide -> code-reviewer | Bug investigation + fix |
| `refactor` | architect -> code-reviewer -> tdd-guide | Safe refactoring |
| `security` | code-reviewer -> architect | Security audit |
| `custom` | user-defined CSV list | Ad-hoc sequence |

## Execution Loop

For each agent in chain:
1. Invoke with prior handoff as context
2. Collect output as structured handoff
3. Pass to next agent
4. Aggregate into final report

## Handoff Format

```markdown
## HANDOFF: [prev-agent] -> [next-agent]
### Context        — what was done
### Findings       — discoveries / decisions
### Files Modified — list of touched files
### Open Questions — unresolved items
### Recommendations — suggested next steps
```

## Agent Responsibilities

| Agent | Reads | Produces |
|-------|-------|----------|
| the built-in `Plan` agent | requirements | implementation plan, deps, risks |
| architect | requirements / plan | design decisions, structure |
| the built-in `Explore` agent | bug report | repro steps, root cause |
| tdd-guide | plan / handoff | tests-first, then minimal impl |
| code-reviewer | impl | quality issues, suggestions, security vuln scan (defers deep OWASP/CWE analysis to the `security-review` skill) |

## Final Report

```
ORCHESTRATION REPORT
Workflow: <type> | Task: <desc>
Chain: <agent -> agent -> ...>

SUMMARY        — one paragraph
AGENT OUTPUTS  — per-agent summary
FILES CHANGED  — list
TEST RESULTS   — pass/fail
SECURITY       — findings
RECOMMENDATION — SHIP / NEEDS WORK / BLOCKED
```

## Parallel Phase

For independent checks, fan out simultaneously then merge:
- code-reviewer (quality + security) + architect (design) -> single merged report

## Examples

```
/orchestrate feature "Add user authentication"
/orchestrate bugfix "Login fails on empty email"
/orchestrate custom "architect,tdd-guide,code-reviewer" "Redesign caching layer"
```

## Tips

- Start with the built-in `Plan` agent for complex features; `architect` for design-heavy work
- Always include `code-reviewer` before merge
- Lean on `code-reviewer` for auth, payment, PII paths (it covers security review via the `security-review` skill)
- Keep handoffs concise — only what the next agent needs
- Run verification (build/tests) between agents on risky transitions
