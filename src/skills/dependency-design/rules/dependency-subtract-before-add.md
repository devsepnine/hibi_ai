---
title: Subtract Before You Add
impact: MEDIUM
impactDescription: building on dead code and pass-through layers ties new code to dependencies nobody needs, and every later change must carry them
tags: dependency, dead-code, refactor
---

## Subtract Before You Add

Before adding a module, a parameter, or a layer, remove what the change makes dead or redundant: unused exports, one-caller wrappers, pass-through adapters, validators that re-check what the type already guarantees, and orphan references. New code that is built on top of these takes a dependency on them, and every later change must carry the extra edges.

The order matters. Subtracting first shrinks the graph you are about to change, so the addition lands on the real dependencies only. A subtraction is also easy to review on its own, and it keeps the addition small.

This rule is about order, not about how to delete. For the deletion itself, use `/refactor-clean` and its `refactor-cleaner` agent.

**Incorrect:**

```ts
// The old adapter has one caller left, and the new feature is wired through it.
export function legacyFetchUser(id: string) {
  return userClient.get(id) // pass-through, adds nothing
}

export async function loadProfile(id: string) {
  const user = await legacyFetchUser(id) // new code now depends on the dead layer
  return { ...user, badges: await badgeClient.list(id) }
}
```

**Correct:**

```ts
// Step 1, its own change: delete the pass-through and point its caller at the client.
// Step 2: add the feature against the real dependency.
export async function loadProfile(id: string) {
  const [user, badges] = await Promise.all([userClient.get(id), badgeClient.list(id)])
  return { ...user, badges }
}
```

Reference: [AI-friendly ownership](../references/ai-ownership.md)
