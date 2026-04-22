---
type: adr
number: NNNN
status: proposed | accepted | superseded | deprecated
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [type/adr, project/<slug>, topic/<area>, stage/<rfc|implementation|rollout>]
project: <slug>
deciders: ["<handle>"]
supersedes: null        # "[[ADR-0007 ...]]" if this replaces an older decision
supersededBy: null      # Fill in when this ADR is itself retired
related: []
aliases: ["<Alt title>"]
---

# ADR-NNNN: <Concise decision title>

> [!info] Status — <status>
> <status flips from `proposed` → `accepted` once deciders sign off. If
> it's later replaced, update to `superseded` and set `supersededBy`.
> Never delete superseded ADRs; they're part of the audit trail.>

## Context

Why is a decision needed right now? Describe the forcing function (new
requirement, scaling pain, regulatory change, postmortem outcome). Keep
this about **why we're deciding**, not the decision itself.

Link adjacent material:
- Prior art: [[ADR-MMMM …]]
- Triggering incident / debug note: [[Debug YYYY-MM-DD …]]

## Constraints

- Must-haves (hard requirements, non-negotiable)
- Nice-to-haves (tiebreakers)
- Out-of-scope (what we are explicitly not deciding here)

## Options considered

### Option A — <name>

- Shape: one-paragraph summary.
- Pros: ...
- Cons: ...

### Option B — <name>

- Shape: ...
- Pros: ...
- Cons: ...

### Option C — <name>

- Shape: ...
- Pros: ...
- Cons: ...

## Decision

> [!success] Decided: <option>
> One sentence restating the choice. Then 2–5 sentences on **why** this
> option beat the others, citing the constraints. This is the paragraph
> future-you will re-read.

## Consequences

### Positive

- What gets easier / faster / safer because of this.

### Negative

- What we're now accepting as a cost.

### Neutral / to watch

- Side effects whose impact we can't size yet.

## Implementation notes

- Milestone plan (small bullet list; defer deep plans to tickets)
- Rollout / rollback strategy
- Metrics that tell us if this was the right call

## Related

- [[<release note that ships this>]]
- [[ADR-XXXX …]] for decisions this enables or depends on
