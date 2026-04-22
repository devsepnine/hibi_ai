# Vault Organization

A consistent folder layout, MOC strategy, and capture-to-evergreen flow
make a vault stay navigable as it grows. The schema is intentionally
light — it defers to frontmatter + tags for most filtering and uses
folders for physical colocation only.

## Suggested folder layout

```
<vault>/
├── Daily/              # YYYY-MM-DD.md (one per day)
├── Weekly/             # YYYY-Wnn.md
├── Meetings/           # YYYY-MM-DD <slug>.md
├── Release Notes/      # v<semver>.md
├── ADR/                # ADR-NNNN-<slug>.md
├── Retros/             # YYYY-Wnn.md
├── Debug/              # YYYY-MM-DD <slug>.md
├── Library/            # Books/, Zustand/, React/, ...
├── MOC/                # project and topic maps
├── Fleeting/           # YYYY-MM-DD-HHmm.md (captured ideas)
├── Attachments/        # images, PDFs, canvases
└── + Inbox.md          # default landing page for quick captures
```

Folders here group **by note lifecycle**, not by project. A single
project's work spreads across `Daily/`, `Meetings/`, `ADR/`, etc. —
the connection is made by the `project/<slug>` tag and the project's
MOC note, not by physically colocating files.

Start a new vault with just `Daily/`, `MOC/`, and `Fleeting/`. Add the
rest as they earn their first entry.

## PARA-ish adaptation

If you prefer PARA (Projects / Areas / Resources / Archive), layer it
via MOCs rather than folders:

- A **project** MOC lives in `MOC/<Project Name>.md`.
- An **area** MOC (ongoing responsibility, no end date) lives in
  `MOC/<Area>.md` with `scope: area` in frontmatter.
- **Resources** are evergreen `Library/` notes.
- **Archive** is just `status: archived` on the MOC; the constituent
  notes don't move — dataview filters them by frontmatter.

This avoids the common PARA pain of a note fitting two categories:
notes stay put, and the MOC curates which "bucket" they belong to at
any given time.

## Zettelkasten lite

For personal / research vaults, the learning-note template (atomic,
one idea, one link out, one link in) is the Zettelkasten unit. Layer
it in:

- **Fleeting** → `Fleeting/` (raw capture)
- **Literature** → `Library/Books/...` (one book = one note; key
  highlights as block refs)
- **Permanent / evergreen** → `Library/<Topic>/...` (atomic concepts)

The upgrade path (fleeting → evergreen) is explicit — see the
"Fleeting → Evergreen" section below.

## MOC strategy

A **Map of Content** is a note whose job is to curate other notes.
Three flavors:

### Project MOC — `MOC/<Project>.md`

The front door to everything for a project: status, current focus,
key ADRs, latest releases, open debug logs, dashboard to daily notes
in the current week. Heavy use of dataview.

Template: [`project-moc`](../assets/project-moc.md).

### Topic / subject MOC — `MOC/<Topic>.md`

Curates across projects. Example: `MOC/Performance.md` links every
ADR, learning note, and debug log tagged `topic/perf`.

Use `scope: topic` in frontmatter to distinguish from project MOCs.

### Area MOC — `MOC/<Area>.md`

Ongoing responsibilities without an end date (e.g., `MOC/Health.md`,
`MOC/Reading List.md`). Use `scope: area`.

### When NOT to make a MOC

- Fewer than ~10 related notes → just use a tag and Obsidian's tag
  pane; a MOC would duplicate it.
- Short-lived context → a project note or a pinned weekly review is
  enough.

Resist the urge to MOC everything. The best MOCs are the ones you
actively navigate to — if you never click your MOC, delete it.

## Fleeting → Evergreen upgrade path

Fleeting notes are cheap to create, easy to let rot. Build the upgrade
pattern into your workflow so the good ones graduate.

### Weekly fleeting review (in your weekly-review note)

Embed a dataview query that lists fleeting notes with
`review_on <= today`:

````markdown
```dataview
TABLE captured_at, source
FROM #type/fleeting
WHERE status = "new" AND review_on <= date(today)
SORT review_on ASC
```
````

For each due item, decide one of three fates:

| Decision | Action | Frontmatter change |
|----------|--------|-------------------|
| **Promote** — idea has legs | Create a new learning/ADR/MOC note; move key content; link back via `related: [[original fleeting]]` | `status: processed`, add `promoted_to: [[new note]]` |
| **Defer** — needs more time | Push `review_on` forward | (no other change) |
| **Discard** — didn't pan out | Mark as discarded; optionally add a one-line why | `status: discarded` |

Don't delete discarded fleetings — the "didn't work" record has audit
value, and the note is tiny.

### What makes an evergreen note (Zettelkasten-flavored)

When promoting, aim for:

- **One idea per note** — if you want to split it into two, do.
- **Title that could be a search query** — "Zustand v5 selector
  reference equality" beats "Zustand stuff".
- **At least one outgoing link** — an evergreen note with no links
  out is an orphan; find a related concept.
- **At least one incoming link** — add a `[[This Note]]` reference
  from the MOC or a parent topic. Otherwise no one will find it.

## Attachments & binary assets

Keep them out of the note folders:

- `Attachments/` — default for inline images
- `Attachments/Excalidraw/` — for `.excalidraw.md` files
- `Attachments/Canvas/` — optional; `.canvas` often lives next to the
  note that embeds it

Obsidian setting → **Files & Links → Default location for new
attachments** → `In subfolder under current folder` + folder name
`Attachments/` gives consistent placement.

Never commit large binaries (video, > 10 MB PDFs) to the vault's git
repo — use a link to cloud storage instead.

## Search patterns that rely on this layout

- "This week's work" → `#type/daily` + filter by `week`
- "All ADRs for hibi-ai still open" → `#type/adr AND #project/hibi-ai`
  + `status = proposed`
- "Reading list" → `#type/book + status = "reading"` (or just open
  `MOC/Reading List.md`)
- "Overdue fleeting notes" → `#type/fleeting + review_on <= today`

The layout exists to make these queries effortless, not to organize
the notes themselves — which the tags + frontmatter already do.
