---
type: weekly
status: active | closed
week: YYYY-Wnn
week_start: YYYY-MM-DD
week_end:   YYYY-MM-DD
created: YYYY-MM-DD
tags: [type/weekly, project/<slug-or-none>]
project: <slug or none>
related: []
---

# Week YYYY-Wnn Review

> [!info] Week theme
> If there was one, name it. Otherwise say "no theme" and don't
> invent one — noise in the weekly review compounds across months.

## This week's dailies

Embed a live list so the review note stays current:

```dataview
LIST file.link
FROM #type/daily
WHERE week = "YYYY-Wnn"
SORT file.name ASC
```

## Shipped / completed

- <thing> — link to [[release note]] or ticket
- <thing>

Auto-feed from release / ADR frontmatter (replace dates):

```dataview
LIST file.link
FROM (#type/release OR #type/adr)
WHERE file.cday >= date("YYYY-MM-DD") AND file.cday <= date("YYYY-MM-DD")
SORT file.cday ASC
```

## Learned

Curate the week's learnings into atomic notes, not a long paragraph.

- [[Zustand — useShallow and v5 selector equality]]
- <one-line takeaway> — link to the daily/fleeting that captured it

## Metrics (optional)

Track 1-3 things that matter. Examples:

| Metric | This week | Last week | Delta |
|--------|-----------|-----------|-------|
| Commits | 32 | 28 | +4 |
| Meetings (hrs) | 6 | 9 | −3 |
| Deep-work blocks | 7 | 5 | +2 |

Don't track what you won't act on. If a number never changes your
behavior, drop it from the template.

## Fleeting notes review

```dataview
TABLE captured_at, source, review_on
FROM #type/fleeting
WHERE status = "new" AND review_on <= date(today)
SORT review_on ASC
```

For each due fleeting: **promote** (new note), **defer** (push
`review_on`), or **discard** (mark and move on). See
[[vault-organization — Fleeting → Evergreen]] for the pattern.

## Open action items

From meetings and retros:

```dataview
TASK
FROM #type/meeting OR #type/retro
WHERE !completed
GROUP BY file.link
```

## What went well

- <thing> — why it worked; worth continuing

## What didn't

- <thing> — what was the real cause (not "I didn't have time")

Be specific. "Meetings were bad" is noise; "three back-to-back syncs
Wednesday afternoon blocked a release" is actionable.

## Next week

- [ ] Top goal:
- [ ] Second goal:
- [ ] Followups:

Keep to ~3 goals. Write them so their definition-of-done is obvious by
Friday.

## Related

- [[YYYY-W(nn-1)]] — previous week
- [[MOC — <active project>]]
