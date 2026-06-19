---
title: "Control Interconnection Complexity: Linear Flow + Message Constraints"
impact: MEDIUM
impactDescription: interconnection complexity ripples across modules and cannot be contained the way modular components can
tags: architecture, interconnection, pipeline
---

## Control Interconnection Complexity: Linear Flow + Message Constraints

A system is **components + interconnections + purpose**. A component is modular,
so the blast radius of a change is bounded inside it. Interconnection complexity
is different: it is not contained by a module boundary, so its ripple spreads
across everything it touches. That is why connection design needs its own
discipline — you cannot rely on encapsulation to absorb it.

Three constraints keep interconnection complexity in check:

- **Make connections linear.** Linear interconnection is a temporal idea: the
  connection happens in a definite order (sequential / pipelined). Order encodes
  causality, so you can reason about one stage at a time without holding the
  whole flow in your head. This is what makes partial analysis and partial
  edits possible — the pipelining strategy.
- **Keep connections unidirectional.** A connection is request -> response one
  way, not a two-way chatty conversation. To send a request you must know the
  target, so direction is the same thing as dependency direction; one-way
  direction also fixes the causal order. Watch for large cycles that become
  bidirectional indirectly even when no single edge is.
- **Constrain messages.** Tighten the schema of what crosses the connection so
  invalid input is rejected at the boundary, before it propagates. A permissive
  payload pushes validation downstream where the ripple is already wide.

**Incorrect:**

```typescript
// Bidirectional, chatty, permissive — the caller and worker call back and forth,
// and the message is an open bag of optional fields validated nowhere.
interface Job {
  kind?: string
  payload?: unknown        // anything goes; errors surface deep downstream
  onProgress?: (pct: number) => void
}

class Worker {
  constructor(private caller: Caller) {}            // worker knows the caller
  run(job: Job) {
    this.caller.notifyStarted()                     // back-edge to caller
    const data = this.caller.fetchMore(job.kind)    // pulls more mid-run
    job.onProgress?.(50)                            // calls back in
    this.caller.notifyDone(data)                    // and again -> cycle
  }
}
```

**Correct:**

```typescript
// One-way pipeline stage with a strict, validated message schema.
import { z } from "zod"

const RenderRequest = z.object({
  documentId: z.string().uuid(),
  pages: z.array(z.number().int().positive()).nonempty(),
})
type RenderRequest = z.infer<typeof RenderRequest>

interface RenderResult {
  documentId: string
  url: string
}

// Each stage takes a validated input and RETURNS an output. No call-backs,
// no reference to the caller. The pipeline composes the stages in order.
function renderStage(input: unknown): RenderResult {
  const req = RenderRequest.parse(input)   // invalid message rejected at the edge
  const url = render(req.documentId, req.pages)
  return { documentId: req.documentId, url }
}

// request -> response, one way; the orchestrator owns the ordering.
const result = renderStage(incomingMessage)
```

Reference: [Complexity, Cynefin, and Degrees of Freedom](../references/complexity.md)
