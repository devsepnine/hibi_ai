---
type: fleeting
status: new | processed | discarded
captured_at: YYYY-MM-DDTHH:MM
review_on: YYYY-MM-DD        # default ~1 week from captured_at
source: shower | walk | reading | conversation | meeting | dream | ...
created: YYYY-MM-DD
tags: [type/fleeting, project/<slug-or-none>, topic/<area>]
project: <slug or none>
related: []
---

# Thought — <one-line summary>

> [!tldr] The idea in one sentence
> Write the spark before it cools. Edit later if you must, but don't
> wait to start. A messy three-line fleeting is better than a clean
> idea you forgot.

## Raw capture

Free-form. Bullets, fragments, half-sentences — whatever the brain
actually produced. Don't polish yet; that's what the review is for.

- <point>
- <point>
- <point>

## Why this might matter

One or two sentences on what problem or context this connects to.
Future-you reading this next week needs to understand *why you
thought it was worth writing down*.

## Links (maybe)

Quick guesses at related notes — none of these have to be real yet.

- [[?? Adjacent idea]]
- [[?? Existing ADR this contradicts]]

Mark uncertain links with `??`; the review step resolves them.

## Review

Filled in during weekly review (or whenever `review_on` hits).

**Decision**: promote | defer | discard

- If **promote**: link to the new note and change `status: processed`
  with `promoted_to: [[<new note>]]` in frontmatter.
- If **defer**: push `review_on` to a new date and record why.
- If **discard**: one-line rationale — "turned out to be obvious",
  "already captured in [[other note]]", "not worth pursuing".

Review on: YYYY-MM-DD

### Upgrade path cheatsheet

| If the idea is… | Promote to |
|-----------------|------------|
| A reusable concept or pattern | `learning` note |
| A decision that needs auditability | `adr` |
| A project-specific TODO | ticket or `daily` task |
| A collection of related ideas | `moc` |
| A reference to a book / article | `book` note (with this as seed link) |

Don't let fleeting notes rot by default — either they become
something more, or they get an honest discard. The audit trail of
"idea → discarded because X" is more valuable than deleting it.
