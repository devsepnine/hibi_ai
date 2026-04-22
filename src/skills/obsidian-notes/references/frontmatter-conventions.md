# Frontmatter Conventions

Every note emitted by this skill starts with a YAML frontmatter block. The
schema is stable so Obsidian dataview queries, graph filters, and folder
index notes can rely on the field names.

## Canonical shape

```yaml
---
type: release | adr | retro | debug | learning | daily | weekly | meeting | moc | book | fleeting
status: <per-type, see below>
created: YYYY-MM-DD
updated: YYYY-MM-DD      # optional; add when editing an older note
tags: [type/<cat>, project/<name>, topic/<area>, ...]
project: <project slug>
related: ["[[Note A]]", "[[Note B]]"]
aliases: ["Alt Title"]   # optional
---
```

## Field reference

### `type` (required)
Controls which template applies and which `status` values are legal.

| Value | Template |
|-------|----------|
| `release`  | Release notes |
| `adr`      | Architecture Decision Record |
| `retro`    | Sprint/iteration retrospective |
| `debug`    | Debugging session log |
| `learning` | Library/tool/pattern learning note |
| `daily`    | Daily log / dev journal |
| `weekly`   | Weekly review |
| `meeting`  | Meeting minutes |
| `moc`      | Map of Content / project index |
| `book`     | Book / article capture |
| `fleeting` | Quick idea, captured fast, reviewed later |

Don't invent new types here — add a new template first.

### `status` (required)
Allowed values depend on `type`.

| type | status values | Transitions |
|------|---------------|-------------|
| `release`  | `draft` → `published` | `draft` until version is cut |
| `adr`      | `proposed` → `accepted` → `superseded` \| `deprecated` | Never delete a superseded ADR — mark it and link `supersededBy` in frontmatter |
| `retro`    | `active` → `closed` | `active` while action items are open |
| `debug`    | `open` → `resolved` → `archived` | Flip to `resolved` once fix lands; add `archived` months later to quiet search |
| `learning` | `draft` → `stable` → `stale` | `stale` when API changes enough to invalidate content |
| `daily`    | `active` → `closed` | `closed` at end-of-day or when next daily starts |
| `weekly`   | `active` → `closed` | `active` while filling during the week; `closed` after review |
| `meeting`  | `scheduled` → `captured` → `actioned` \| `archived` | `actioned` once every action item links a follow-up; `archived` after 90+ days |
| `moc`      | `active` → `archived` | `archived` when the project/topic winds down; keep for history |
| `book`     | `reading` → `completed` → `revisited` | `revisited` after re-read; add a new revisit log beneath |
| `fleeting` | `new` → `processed` \| `discarded` | `processed` once content is promoted into a learning/ADR/MOC; `discarded` if it didn't pan out |

### `created` (required), `updated` (optional)
ISO `YYYY-MM-DD`. Use creation date, not "today" if back-dating from notes.

### `tags` (required)
Three axes are required; more are fine. Stick to the taxonomy in the next
section.

### `project` (required)
Lowercase slug, kebab-case if multi-word. Match the `project/<slug>` tag.
Example: `project: hibi-ai` with tag `project/hibi-ai`.

### `related` (optional)
Array of wikilinks to notes that share context. Bidirectional linking is
Obsidian's superpower — prefer two light `related` entries to one big
embedded block.

### `aliases` (optional)
Alternate titles for search. Useful for ADRs (e.g., aliases include the
underlying topic: `"Switch DB to Postgres"`).

### Type-specific extras
Some types carry extra frontmatter. Keep these in the same block.

#### `adr`
```yaml
number: 12                        # zero-padded in filename, plain here
supersedes: "[[ADR-0007 ...]]"    # if applicable
supersededBy: "[[ADR-0019 ...]]"  # when this one is retired
deciders: ["alice", "bob"]
```

#### `release`
```yaml
version: 1.9.4
release_date: 2026-04-21
changelog_link: https://github.com/org/repo/releases/tag/v1.9.4
```

#### `retro`
```yaml
sprint: 2026-W16
team: platform
```

#### `debug`
```yaml
severity: low | medium | high | critical
resolved_in: "[[ADR-0012 ...]]" | "<PR #>" | null
```

#### `learning`
```yaml
library: zustand
version: 5.0.12
```

#### `daily`
```yaml
date: 2026-04-22          # same as created, kept so dataview can WHERE date = ...
day_of_week: Tuesday      # optional; templater often injects this
week: 2026-W17            # ISO week; lets weekly reviews glob by prefix
```

#### `weekly`
```yaml
week: 2026-W17
week_start: 2026-04-20    # Monday of the week
week_end:   2026-04-26
```

#### `meeting`
```yaml
date: 2026-04-22
start_time: "14:00"
duration_min: 30
attendees: ["alice", "bob"]
meeting_type: sync | review | 1on1 | planning | retro | adhoc
agenda_link: "https://..."   # or "[[Agenda ...]]"
```

#### `moc`
```yaml
scope: project | topic | area    # what the MOC curates
hub: true                        # set on the single "front door" MOC for a vault section
```

#### `book`
```yaml
author: "John Ousterhout"
title: "A Philosophy of Software Design"
isbn: "9781732102200"
year: 2018
rating: 4                  # optional 1..5
finished_on: 2026-04-15
```

#### `fleeting`
```yaml
captured_at: 2026-04-22T14:37
review_on: 2026-04-29      # default ~1 week out; dataview surfaces when due
source: "shower" | "reading" | "walk" | "conversation" | ...
```

## Tag taxonomy

Three required axes, each prefixed so the root tag tree stays organized:

### `type/<category>`
Mirrors the `type` field.
- `type/release`, `type/adr`, `type/retro`, `type/debug`, `type/learning`
- `type/daily`, `type/weekly`, `type/meeting`
- `type/moc`, `type/book`, `type/fleeting`

### `project/<slug>`
Same slug as `project` field. Examples:
- `project/hibi-ai`, `project/installer`, `project/dashboard-frontend`

### `topic/<area>`
Domain area — pick existing vault conventions where possible. Common:
- `topic/auth`, `topic/perf`, `topic/build`, `topic/ci`, `topic/db`
- `topic/ui`, `topic/api`, `topic/devx`, `topic/ops`, `topic/security`
- `topic/testing`, `topic/docs`

Add new topic tags sparingly. Prefer reusing an existing one over coining
a one-note-only tag.

### Optional axes

- `stage/<phase>` — `stage/rfc`, `stage/implementation`, `stage/rollout`
- `tech/<stack>` — `tech/rust`, `tech/typescript`, `tech/react`
- `owner/<person>` — if your vault tracks ownership via tag

Avoid `status/...` tags — `status` has its own frontmatter field.

## Worked example

```yaml
---
type: adr
number: 12
status: accepted
created: 2026-04-21
updated: 2026-04-22
tags: [type/adr, project/hibi-ai, topic/build, tech/rust, stage/implementation]
project: hibi-ai
deciders: ["owenkim"]
supersedes: "[[ADR-0007 Local-only installer sync]]"
related: ["[[v1.9.4 Release Notes]]", "[[Sync bundled cache design]]"]
aliases: ["Homebrew sync fix", "find_git_root marker"]
---

# ADR-0012: Scope `find_git_root` to the hibi_ai repo

...
```

Dataview can now slice this: "all accepted ADRs for project/hibi-ai in
2026", "all learning notes tagged topic/rust", etc. without parsing bodies.
