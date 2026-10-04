---
name: why
description: "Design archaeology: why code is shaped this way, traced through git, PRs, tickets, and docs, with confidence tiers. Use when asked why something was built so or what a change must keep. 왜 이렇게 만들었지, 설계 의도, 히스토리 추적."
---

# Why

Find the forces that gave code its shape: the bug it fixed, the constraint it obeys, the trade-off someone chose. Code shows what it does; its motivation lives in commits, PRs, tickets, docs, and conversations, all incomplete. A confident guess misleads more than an honest "we don't know", so every claim carries its confidence tier from `references/epistemics.md`.

This skill answers why. For how code behaves at runtime, read the code or use the `Explore` agent. For what a change could break, use `blast-radius`.

## Step 1. Pin the target and the question

The target is a chunk of code, a pattern, a constant, or a named decision. The question is a rationale, a trade-off, a motivating edge case, an external constraint, or a history sweep. If the target is vague, take the best reading from the conversation, state it in one line so the user can redirect, and proceed.

## Step 2. Build the code anchor

Before fanning out, collect the file paths and line ranges, the key symbols, the commits that touched the target, and the PR and ticket numbers they mention:

```bash
git blame -L <start>,<end> <file>          # last-touch commits per line
git log --follow --oneline -- <file>        # history through renames
git log -S '<exact string>' -- <file>       # commits that added or removed this text
git show <hash>                             # one commit's full diff
gh pr view <n> --json title,body,comments,reviews,closingIssuesReferences
```

Trace back past the latest commit. The current shape is usually several earlier decisions stacked up, and the newest commit is rarely the one that explains it.

## Step 3. Fan out one investigator per evidence source

Source control is always available. For the rest, check which MCP servers this session actually has, and map each to one category:

| Category | Typical source | Best at surfacing |
|---|---|---|
| Source control | git, `gh` | rationale written during review |
| Issue tracker | Jira through the Atlassian MCP, GitHub Issues | the product or business forcing function |
| Long-form docs | Confluence, `docs/adr/`, design docs in the repo | rationale written before the code |
| Error tracking | Sentry MCP | the exceptions behind a guard, retry, or null check |
| Analytics | ClickHouse MCP | where a threshold or flag value came from |
| Observability | `cloudflare-observability` or another metrics MCP, when present | the runtime signal a timeout or rate limit reacts to |

Launch every matching investigator in one message, one `Agent` per source, each given the code anchor, the user's question, and the tiers from `references/epistemics.md`. An investigator that finds nothing still reports what it searched. Skip a category only for a written reason: no source is available, or the source is provably irrelevant, such as error tracking for a build script.

When the PR body already answers the question completely, say so and answer inline after checking the other sources would add nothing.

## Step 4. Synthesize

Merge the findings. Keep each claim in its tier and do not upgrade an inference to a fact while summarizing. Where sources disagree, keep both readings as competing hypotheses.

## Output

```
The question:        <restated in one line>
The code:            <file:line ranges and symbols>
What we found:       <Direct and Supported claims, each with its source>
What we can infer:   <Inferred claims, with the chain of reasoning>
Competing readings:  <Speculative hypotheses side by side>
What we don't know:  <what was searched, with which terms, and found nothing>
Sources consulted:   <one line per source, including empty and skipped ones with the reason>
```

If the question is a step toward changing this code, end with a constraint set for planning the change:

- **Preserve**: behavior a past decision depends on
- **Change**: what the original reason no longer requires
- **Avoid**: an approach the history shows already failed
- **Risk**: what could break if the forgotten reason still holds
