---
type: daily
status: active | closed
date: YYYY-MM-DD
day_of_week: Monday | Tuesday | ...
week: YYYY-Wnn
created: YYYY-MM-DD
tags: [type/daily, project/<slug-or-none>]
project: <slug or none>
related: []
---

# YYYY-MM-DD <Day name>

> [!info] Focus for today
> One-sentence intent: the single thing that, if done, makes today a
> good day. If you can't name one, do that first.

## Plan

- [ ] <task 1> — link to ticket / note if applicable
- [ ] <task 2>
- [ ] <task 3>

Keep this short. If it's more than 5 items, it's a week's worth, not a
day's.

## Notes during the day

Timestamped running log. Add entries as things happen — thinking
out-loud notes, observations, small decisions.

- `HH:MM` — <note>
- `HH:MM` — <note>

For longer thoughts, link to a separate note:
- `HH:MM` — spiked approach X (see [[Spike — approach X]])

## Meetings

List of today's meeting notes as wikilinks — cheap to scan later:

- [[YYYY-MM-DD <meeting slug>]]

## What I did

Bullets at end of day. Past tense. Concrete.

- Shipped …
- Reviewed …
- Debugged … (see [[Debug YYYY-MM-DD <slug>]])

## What I learned

Captures the day's aha moments. If any is big enough to stand alone,
link to a learning / fleeting note:

- [[Fleeting YYYY-MM-DD-HHmm idea]] — caching boundary at edge
- Noted that Zustand v5 uses `Object.is` by default (→ [[Zustand — useShallow and v5 selector equality]])

## Tomorrow

Seed the next daily so future-you doesn't start cold:

- [ ] Follow up on …
- [ ] Draft the ADR for …

## Related

- [[YYYY-Wnn]] — this week's review (gets populated on Friday)
- [[{{previous daily}}]]
