---
type: moc
status: active | archived
scope: project | topic | area
hub: false
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [type/moc, project/<slug>, topic/<area>]
project: <slug>
related: []
---

# <Project name> — Project Map

> [!info] What this project is
> Two sentences. First: what problem it solves. Second: current
> status — alive, paused, or winding down.

## Quick links

- **Repository**: [<org/repo>](https://github.com/org/repo)
- **Release notes**: [[Release Notes]] folder, latest [[v<latest>]]
- **Active sprint**: [[YYYY-Wnn Retrospective]] (or [[YYYY-Wnn Review]])
- **Onboarding**: [[<Project> — Onboarding]] when one exists

## Current focus

- Top thing: [[note or task]]
- Secondary: …

Keep this to 2–3 items. When the focus changes, rewrite this section;
the version history is in git.

## Architecture Decisions

```dataview
TABLE number AS "ID", status, file.link AS "Note"
FROM #type/adr AND #project/<slug>
SORT number ASC
```

Open / proposed ADRs stand out — don't leave them in `proposed` for
weeks; either accept, reject, or supersede.

## Releases

```dataview
TABLE version, release_date, file.link AS "Notes"
FROM #type/release AND #project/<slug>
SORT release_date DESC
LIMIT 10
```

## Recent activity

Notes added in the last two weeks touching this project:

```dataview
LIST file.link
FROM #project/<slug>
WHERE file.cday >= date(today) - dur(14 days)
SORT file.cday DESC
LIMIT 20
```

## Open debug logs

```dataview
TABLE severity, created, file.link AS "Note"
FROM #type/debug AND #project/<slug>
WHERE status = "open"
SORT severity DESC, created DESC
```

## Known risks / watchlist

Things not yet bugs but worth keeping an eye on. Each bullet should
link to the note tracking the concern.

- <Risk> — see [[…]]
- <Risk>

## Team / stakeholders

- <handle> — role
- <handle> — role

## Related MOCs

- [[MOC — <sibling project>]] — shared dependencies / users
- [[MOC — <area>]] — broader area this belongs to

## Notes

Anything that doesn't fit the above sections but someone arriving here
should know. Keep it small; if a topic grows, give it its own note
and link here.
