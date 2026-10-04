---
name: how
description: "Runtime walkthrough: how code works, traced as a real call chain with a file cited for every claim. Use when asked how a subsystem or flow works before changing it. 어떻게 동작해, 구조 설명, 흐름 파악."
---

# How

Explain how a piece of code works at runtime, at the level a senior engineer needs when joining a subsystem: enough to build a working mental model, not so much that it reads like annotated source. Every claim cites a file, and the flow is the real call chain read from the code, not one guessed from names.

This skill answers how. For what forces shaped the code, use `why`. For a single lookup such as where a symbol is defined, the `Explore` agent is enough. For a vague user report such as "the save button sometimes does nothing", look the report up in the existing `docs/FEATURES.md`, built by `feature-map`, to find the code first, then come back here.

## Step 1. Pin the question and its size

Restate the question in one line. If the scope is ambiguous, state your reading so the user can redirect, and proceed.

- **Simple**: one module, one utility, or a narrow question such as how function X works. Read the code yourself and go to Step 4.
- **Complex**: a subsystem across several files or services, a cross-cutting feature, or a full architectural overview. Go to Step 2.

When in doubt, take the simple path.

## Step 2. Find the entry points

Before fanning out, collect what every explorer needs: the entry points that trigger the behavior, such as a route, a command, an event handler, or a scheduled job, and the top directories involved.

```bash
rg -n '<route|command|event name>' --type-add 'src:*.{ts,tsx,js,py,rs,go}' -t src
rg -n 'fn <symbol>|function <symbol>|def <symbol>|class <symbol>'
```

## Step 3. Fan out one explorer per slice

Split the question into 2 to 4 slices, each a distinct part of the subsystem: for example the request path, the persistence layer, and the background worker. Launch every explorer in one message, one `Agent` per slice with `subagent_type: Explore` and a prompt that asks for very thorough breadth, each given the question, the entry points, its slice, and the template in `references/explorer-prompt.md`. The lead pastes that template into each Agent prompt, because a subagent cannot resolve the skill's references path.

An explorer that cannot trace a link says so. "I could not find what calls X" is a finding. A guessed link is a defect.

## Step 4. Synthesize

Merge the findings into one picture. Where explorers overlap, merge them. Where they disagree, read the code yourself and keep the version the code supports. Check every call chain step you report by opening the cited file, because a stale line number misleads more than a missing one.

Keep gaps visible. An open question from an explorer goes into Gotchas or the end of How It Works, not out of the answer.

## Output

Use these sections in this order. Drop Gotchas when there is nothing worth calling out; keep the rest.

```
Overview:           <one or two paragraphs: what this is, what it does, why it exists>
Key Concepts:       <the types, services, or abstractions the rest depends on, each with its file>
How It Works:       <the flow from trigger to result, step by step, each step as file:line or `path` → `symbol`>
Where Things Live:  <a short file and directory map, only what someone needs to start working>
Gotchas:            <surprising behavior, traps, and open questions, each with its file>
```

How It Works is the longest section. Write it as prose that follows the call chain, for example `src/api/orders.ts:42` → `createOrder` → `src/billing/charge.ts` → `chargeCard`, and say what data moves between steps and where the decision points are. When several components talk to each other or data moves through stages, add a mermaid sequence diagram or flowchart. Skip the diagram when prose already makes the flow clear.

## Writing rules

- Name the concrete thing: "`OrderService` calls `PaymentClient.charge`", not "the service delegates to the client".
- When something is complex, explain why it is complex instead of only describing it.
- When something is simple, keep it short.
- Show a code block only when a snippet carries a point the prose cannot.
- A claim without a file citation does not go in the answer.

## Before handing back

- [ ] Every step in How It Works cites a file you opened
- [ ] The call chain matches the code, not the names
- [ ] Open questions are stated, not hidden
- [ ] Any why question was handed to `why`, not answered from guesses
