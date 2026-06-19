---
title: "Classify Domain-Specific vs General Knowledge, Share via Contract"
impact: HIGH
impactDescription: reusing data shaped for one purpose as another module's input creates model coupling that turns every internal change into a cross-module break
tags: abstraction, encapsulation, contract
---

## Classify Domain-Specific vs General Knowledge, Share via Contract

Every module owns a slice of domain knowledge that blends two kinds: **general**
knowledge (broadly applicable, stable) and **domain-specific** knowledge (peculiar
to this problem). The two pull in opposite directions when published. Publishing
general knowledge raises reuse and compatibility; publishing domain-specific
knowledge raises clarity and reliability. Decide deliberately which kind a given
boundary should expose, then abstract it before publishing.

How you share that knowledge decides the coupling. **Model coupling** reuses data
that was published for another purpose — most often a persistence row or DTO — as
a second module's input. It looks convenient but binds every consumer to the
shape of someone else's internals, so a storage change ripples outward. **Contract
coupling** publishes data made only for the interaction. Prefer a purpose-built
interaction contract: it is the most stable form of sharing because each side can
change its internals freely as long as the contract holds.

**Incorrect:**

```ts
// A persistence row is published; other modules couple to its shape (model coupling).
interface UserRow {
  id: number
  pw_hash: string // storage detail
  created_ts: number // storage column name + epoch units
  pref_json: string // serialized blob, schema implicit
}

// Notifications now depends on the database's internal shape.
function sendWelcome(user: UserRow) {
  const prefs = JSON.parse(user.pref_json) // Notifications is bound to the storage row shape, not a contract
  mailer.send(prefs.locale, user.id)
}
```

**Correct:**

```ts
// A purpose-built interaction contract, abstracted from storage (contract coupling).
interface WelcomeRequested {
  userId: string
  locale: string
}

// The User module maps its internal row to the contract; consumers see only that.
function toWelcomeRequested(row: UserRow): WelcomeRequested {
  return { userId: String(row.id), locale: JSON.parse(row.pref_json).locale }
}

// Notifications depends on the stable contract, not on storage internals.
function sendWelcome(evt: WelcomeRequested) {
  mailer.send(evt.locale, evt.userId)
}
```

For broader coupling levels (intrusive, functional, model, contract) and how to
move toward looser ones, see the `backend-patterns` skill.

Reference: [Abstraction and Module Boundaries](../references/abstraction.md)
