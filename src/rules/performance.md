# Performance Optimization

## Effort × Model Matrix (Anthropic Opus 4.7 guide)

| Effort | Model | Use Cases |
|---|---|---|
| `low` | `claude-haiku-4-5` | Single-tool checklist, narrow scope. Subagents, classification, quick lookups, bulk ops |
| `medium` | `claude-sonnet-4-6` | Balanced — tool calls + some reasoning. Drop-in for "good results, lower cost" |
| `high` | `claude-sonnet-4-6` | Complex reasoning, careful judgment. Often the optimal quality/efficiency sweet spot |
| `xhigh` | `claude-opus-4-7` | Coding + agentic exploration: repeated tool calls, deep web/KB search |
| `max` | — | True frontier problems only — adds significant cost for marginal quality gain |

**Core principle**: *"Don't prompt around — raise the effort."* Opus 4.7 strictly respects effort. At lower effort, the model scopes to what was asked and nothing more. If you observe shallow reasoning on a complex problem, raise effort instead of expanding the prompt.

**API default**: `high`. Set `effort` explicitly per workload.

## When to Use Each Effort

- **`low`**: Speed/cost-optimized. Pair with explicit checklists for multi-section tasks. Subagents prefer this.
- **`medium`**: Average workflows that want good results at lower cost.
- **`high`**: Default for nuanced analysis, hard coding problems, multi-file edits.
- **`xhigh`**: Coding agents, multi-step search, long-running explorations. Set `max_tokens` to 64k+.
- **`max`**: Reserved. Most workloads see diminishing returns vs `xhigh`.

## Model Selection Strategy

**Haiku 4.5** (90% of Sonnet capability, 3× cost savings):
- Lightweight agents with frequent invocation
- Worker agents in multi-agent systems
- Short scoped tasks

**Sonnet 4.6** (Best coding model, balanced):
- Main development work
- Tool-heavy workflows
- Code generation

**Opus 4.7** (Deepest reasoning):
- Complex architectural decisions
- Long-running coding/agent tasks (30min+)
- Research and analysis

## Tool Usage Patterns by Effort

**Lower effort** tends to:
- Combine multiple actions into fewer tool calls
- Skip preamble, proceed directly to action
- Use concise confirmation messages

**Higher effort** tends to:
- Make more tool calls
- Explain plan before acting
- Provide detailed change summaries
- Include comprehensive code comments

## Context Window Management

Avoid the last 20% of context window for:
- Large-scale refactoring
- Multi-file feature implementation
- Debugging complex interactions

Lower context-sensitivity tasks:
- Single-file edits
- Independent utility creation
- Documentation updates
- Simple bug fixes

## Build Troubleshooting

If build fails:
1. Use **build-error-resolver** agent
2. Analyze error messages
3. Fix incrementally
4. Verify after each fix
