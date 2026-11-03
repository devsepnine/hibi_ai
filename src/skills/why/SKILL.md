---
name: why
description: "Design archaeology and regression identity: trace why code exists or whether a fixed bug truly recurred through git, PRs, tickets, docs, and runtime evidence. 설계 의도, 재현, 회귀 원인, 히스토리 추적."
---

# Why

Find the forces that gave code its shape: the bug it fixed, the constraint it obeys, the trade-off someone chose. Code shows what it does; its motivation lives in commits, PRs, tickets, docs, and conversations, all incomplete. A confident guess misleads more than an honest "we don't know", so every claim carries its confidence tier from `references/epistemics.md`.

This skill answers why. For how code behaves at runtime, read the code or use the `Explore` agent. For what a change could break, use `blast-radius`.

## Step 1. Pin the target and the question

The target is a chunk of code, a pattern, a constant, or a named decision. The question is a rationale, a trade-off, a motivating edge case, an external constraint, or a history sweep. If the target is vague, take the best reading from the conversation, state it in one line so the user can redirect, and proceed.

## Step 2. Prove a reported regression is the same failure

When the user asks why a fixed bug recurred, identify the old and new failures before explaining either one. Compare three facts:

| Fact | What must match |
|---|---|
| Error signature | Status, error code, exception, or stable message |
| Failure stage | The same request, handler, or call chain step |
| Violated behavior | The same expected behavior from a ticket, spec, test, or product decision |

Call the new case a regression only when all three facts match, or when direct evidence explains a changed signature or stage while preserving the same violated behavior. The same visible screen or user symptom does not prove a regression. An unexplained mismatch means the cases are related at most. If the current result satisfies its documented contract, classify the report as misfiled instead of inventing a defect.

Keep a separate anomaly separate. Missing input, a new upstream response, or a suspicious configuration may explain the screen without being a bug. Call it a defect only when a source establishes which expectation it violates.

## Step 3. Build the code anchor

Before fanning out, collect the file paths and line ranges, the key symbols, the commits that touched the target, and the PR and ticket numbers they mention:

```bash
git blame -L <start>,<end> <file>          # last-touch commits per line
git log --follow --oneline -- <file>        # history through renames
git log -S '<exact string>' -- <file>       # commits that added or removed this text
git show <hash>                             # one commit's full diff
gh pr view <n> --json title,body,comments,reviews,closingIssuesReferences
```

Trace back past the latest commit. The current shape is usually several earlier decisions stacked up, and the newest commit is rarely the one that explains it.

## Step 4. Fan out one investigator per evidence source

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

## Step 5. Synthesize

Merge the findings. Keep each claim in its tier and do not upgrade an inference to a fact while summarizing. Where sources disagree, keep both readings as competing hypotheses.

## Output

```
The question:        <restated in one line>
Failure identity:    <confirmed regression, related failure, or misfiled report; omit for non-regression questions>
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
