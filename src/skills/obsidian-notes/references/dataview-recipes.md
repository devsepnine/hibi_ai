# Dataview Recipes

Dataview is the community plugin that turns frontmatter + tags into a
queryable database. Most MOC notes, weekly reviews, and project
overviews benefit from embedding a dataview block rather than
hand-maintaining a list.

These recipes assume the skill's frontmatter schema (see
[frontmatter-conventions.md](frontmatter-conventions.md)).

## Query types

- `TABLE` — rows with chosen columns
- `LIST` — flat bulleted list
- `TASK` — checkboxes (`- [ ]`) across the vault
- `CALENDAR` — heat-map by date field (needs the DV Calendar plugin)

Wrap any query in a fenced block tagged `dataview` (regular DQL) or
`dataviewjs` (JavaScript for more control).

## Per-type recipes

### All accepted ADRs for a project, newest first

````markdown
```dataview
TABLE
  number AS "ID",
  status,
  file.link AS "Note",
  created
FROM #type/adr AND #project/hibi-ai
WHERE status = "accepted"
SORT created DESC
```
````

### Superseded / deprecated ADRs (audit trail)

````markdown
```dataview
TABLE
  file.link AS "ADR",
  status,
  supersededBy AS "Superseded by"
FROM #type/adr
WHERE status = "superseded" OR status = "deprecated"
SORT file.name ASC
```
````

### Open debug logs with severity

````markdown
```dataview
TABLE severity, created, file.link AS "Note"
FROM #type/debug
WHERE status = "open"
SORT severity DESC, created DESC
```
````

### Open action items across all retros

````markdown
```dataview
TASK
FROM #type/retro
WHERE !completed
GROUP BY file.link
```
````

Same shape works for meetings:

````markdown
```dataview
TASK
FROM #type/meeting
WHERE !completed
SORT file.cday DESC
```
````

### This week's daily notes

````markdown
```dataview
LIST
FROM #type/daily
WHERE week = "2026-W17"
SORT file.name ASC
```
````

Dynamic "current week" (dataviewjs):

````markdown
```dataviewjs
const now = new Date()
const year = now.getUTCFullYear()
// simple ISO week calc; for production use luxon via dv.luxon
const firstJan = new Date(Date.UTC(year, 0, 1))
const days = Math.floor((now - firstJan) / 86400000)
const week = Math.ceil((days + firstJan.getUTCDay() + 1) / 7)
const iso = `${year}-W${String(week).padStart(2, "0")}`

dv.list(
  dv.pages('#type/daily')
    .where(p => p.week === iso)
    .sort(p => p.file.name)
    .map(p => p.file.link)
)
```
````

### Weekly review auto-feed: what I shipped this week

````markdown
```dataview
LIST file.link
FROM (#type/release OR #type/adr)
WHERE file.cday >= date("2026-04-20") AND file.cday <= date("2026-04-26")
SORT file.cday ASC
```
````

Bake this into your `weekly-review` template so it self-populates.

### Fleeting notes due for review

````markdown
```dataview
TABLE captured_at, review_on, source
FROM #type/fleeting
WHERE status = "new" AND review_on <= date(today)
SORT review_on ASC
```
````

### Books in progress

````markdown
```dataview
TABLE author, rating, finished_on
FROM #type/book
WHERE status = "reading"
SORT file.cday DESC
```
````

### All meetings with a specific attendee

````markdown
```dataview
TABLE date, meeting_type, file.link AS "Note"
FROM #type/meeting
WHERE contains(attendees, "alice")
SORT date DESC
```
````

### MOC feed: latest notes added to a topic

````markdown
```dataview
LIST file.link
FROM #project/hibi-ai AND -#type/moc
WHERE file.cday >= date(today) - dur(14 days)
SORT file.cday DESC
LIMIT 20
```
````

## Common pitfalls

- **Tag vs field queries** — `FROM #type/adr` and `WHERE type = "adr"`
  both work, but only the tag query can use Obsidian's cached tag
  index. Prefer tags in `FROM`.
- **String fields as dates** — if `created` is a string
  `"2026-04-22"`, compare with `date("2026-04-22")` (not `<="..."`
  which is lexicographic).
- **Array contains** — `contains(array, value)` for `attendees:
  [alice, bob]`. `"alice" in attendees` also works in newer versions.
- **LIMIT before SORT is wrong** — DQL applies them in source order;
  write `SORT ... DESC LIMIT 10`, not `LIMIT 10 ... SORT ...`.

## Performance

Dataview re-queries on every vault change in the affected scope.
Noticeably slow queries are almost always from:

- `FROM ""` (entire vault) with no tag filter — add tags
- `file.content` regex searches — avoid, use frontmatter fields
- Unbounded `SORT` over hundreds of notes — add `LIMIT`

## When to reach for `dataviewjs`

Use the JavaScript mode only when DQL can't express the shape:

- Aggregations beyond `GROUP BY` (bucketing by custom predicate)
- Cross-note math (totals, rolling averages)
- Dynamic date ranges ("this month", relative to today)
- Mutations (rare — prefer Templater)

`dataview` DQL is readable and survives plugin updates better. Reach
for JS only when you need it.
