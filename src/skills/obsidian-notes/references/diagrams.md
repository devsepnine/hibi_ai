# Diagrams in Obsidian Notes

Obsidian renders Mermaid natively (no plugin needed) and integrates with
Excalidraw and JSON Canvas as sibling files in the vault. Pick the right
tool for the shape you're describing — inline text diagrams for flows
and hierarchies, separate canvas files for spatial layouts.

## Decision tree

| Shape of the thought | Reach for |
|----------------------|-----------|
| Sequence of steps, state machine, system boxes-and-arrows | **Mermaid** inline |
| Timeline / milestones | **Mermaid** `timeline` or `gantt` |
| Math, equations | **MathJax** (`$...$` / `$$...$$`) |
| Hand-drawn feel, arbitrary 2D layout, sticky-note brainstorm | **Excalidraw** file, embed as `![[file.excalidraw]]` |
| Node-and-link graph with different node kinds | **JSON Canvas** (`.canvas` file) |
| Free-form spatial MOC (notes arranged on a plane) | **JSON Canvas** |
| UML with stronger spec (class/component) than Mermaid | **PlantUML** (requires community plugin) |

Rule of thumb: **text-based diagrams live inside the note**;
**spatial/visual ones live as a sibling file** and are embedded.

## Mermaid — recipe selector

Mermaid supports many diagram types. Use the simplest that conveys the
information; don't reach for `graph TB` when a list would do.

### Flowchart — processes, system diagrams, call flows

````markdown
```mermaid
flowchart LR
  User -->|submit| API
  API --> Auth{Authed?}
  Auth -- yes --> DB[(Postgres)]
  Auth -- no  --> Login[Login page]
```
````

- `LR` / `TB` / `RL` / `BT` for direction.
- `[]` box, `(())` circle, `{}` diamond (decision), `[()]` database.
- Edge labels with `-- text -->` or `|text|`.

### Sequence — interactions between participants

````markdown
```mermaid
sequenceDiagram
  participant U as User
  participant API
  participant DB
  U->>API: POST /login
  API->>DB: SELECT user WHERE email=?
  DB-->>API: row
  API-->>U: 200 { token }
```
````

Use when you care about ordering and who talks to whom.
`->>` sync, `-->>` async/response, `-x` lost message.

### State diagram — lifecycle, status transitions

````markdown
```mermaid
stateDiagram-v2
  [*] --> Draft
  Draft --> Proposed: submit
  Proposed --> Accepted: approve
  Proposed --> Rejected: reject
  Accepted --> Superseded: replace
  Superseded --> [*]
```
````

Perfect for ADR status, bug lifecycle, feature flag rollout.

### ER diagram — data model

````markdown
```mermaid
erDiagram
  USER ||--o{ SESSION : "has"
  USER {
    uuid  id PK
    string email UK
    string role
  }
  SESSION {
    uuid  id PK
    uuid  user_id FK
    date  expires_at
  }
```
````

Use in learning notes for data-model introductions; in ADRs when a
schema change is the decision.

### Class diagram — type/trait relations

````markdown
```mermaid
classDiagram
  class Repository~T~ {
    +find(id): T
    +save(item: T)
  }
  class UserRepo
  Repository <|-- UserRepo
```
````

### Timeline — non-quantitative time

````markdown
```mermaid
timeline
  title hibi_ai milestones
  2025 Q4 : 1.0 release
  2026 Q1 : Scoop bucket · Homebrew tap
  2026 Q2 : find_git_root fix · obsidian-notes skill
```
````

Prefer `timeline` for "what happened when" narratives. Use `gantt` when
the bars carry duration information.

### Gantt — schedules, sprints

````markdown
```mermaid
gantt
  title Sprint 2026-W17
  dateFormat YYYY-MM-DD
  section Skills
  obsidian-notes draft         :a1, 2026-04-22, 2d
  eval iteration-1             :a2, after a1, 1d
  obsidian-notes iterate       :a3, after a2, 2d
  section Release
  1.9.6 build + package        :b1, 2026-04-25, 1d
```
````

### Mindmap — brainstorm hierarchy

````markdown
```mermaid
mindmap
  root((obsidian-notes))
    templates
      dev
        release-note
        adr
      time
        daily
        weekly
        meeting
      knowledge
        moc
        book
    references
      frontmatter
      syntax
      diagrams
```
````

Fine for fleeting notes. Not a substitute for a MOC when the links
matter.

### Quadrant chart — tradeoffs, prioritization

````markdown
```mermaid
quadrantChart
  title Feature prioritization
  x-axis Low effort --> High effort
  y-axis Low impact --> High impact
  quadrant-1 Schedule
  quadrant-2 Do now
  quadrant-3 Park
  quadrant-4 Consider
  "obsidian-notes polish": [0.3, 0.7]
  "installer GUI": [0.9, 0.5]
  "fix Homebrew sync": [0.2, 0.9]
```
````

### Journey — user flow with sentiment

````markdown
```mermaid
journey
  title First-run experience
  section Install
    brew install hibi: 5: User
    hibi sync: 3: User
    see cache: 4: User
```
````

## MathJax — equations

Obsidian renders MathJax inline (`$...$`) and block (`$$...$$`).

```markdown
Inline: $e^{i\pi} + 1 = 0$

Block:
$$
\lim_{n \to \infty} \left(1 + \frac{1}{n}\right)^n = e
$$
```

Use in learning notes when the math IS the content. Avoid for simple
inequalities — `O(n log n)` in backticks is clearer than `$O(n \log n)$`
for most prose.

## PlantUML

Obsidian does not render PlantUML natively; the **PlantUML** community
plugin is required. Syntax lives in a fenced `plantuml` block. Prefer
Mermaid when the diagram fits — it renders without extra plugins, so
your note works on any vault. Fall back to PlantUML only for diagrams
Mermaid can't express (activity diagrams with detailed control flow,
deployment diagrams, certain UML variants).

## Excalidraw integration

For hand-drawn, spatial, or collaborative diagrams:

1. Install the community **Excalidraw** plugin.
2. New file → "Create new Excalidraw drawing" → a `.excalidraw.md`
   file is created alongside regular notes.
3. Embed in any note with `![[My Drawing.excalidraw]]`.

Use cases: architecture sketches in ADRs, whiteboard-style
brainstorms in fleeting notes, hand-drawn flowcharts too messy for
Mermaid.

Downside: the drawing is a JSON file — diffs aren't great in git.
Prefer Mermaid for anything you want to review in pull requests.

## JSON Canvas (`.canvas`)

Obsidian's built-in spatial-layout format. Create with the ribbon or
`New canvas`. Canvas files live alongside notes and can **embed
existing notes as cards**, plus freeform text/file nodes.

Best for:

- **Spatial MOCs** — lay out `[[note cards]]` in zones ("doing",
  "done", "blocked")
- **System diagrams where each node IS a linked note** — click
  through to the ADR behind a box
- **Workshop boards** — sprint planning, retro Start/Stop/Continue

Embed a canvas in a note with `![[Project Board.canvas]]` (recent
Obsidian versions render the preview inline).

See [vault-organization.md](vault-organization.md) for when a Canvas
MOC beats a note-based MOC.

## Anti-patterns

- **Huge flowchart in a learning note** — if the diagram takes 50+
  lines, make it its own `.excalidraw` or `.canvas` file and embed.
- **Mermaid for tables** — use a real Markdown table.
- **ASCII art** — Obsidian preserves whitespace in fenced code blocks,
  but Mermaid / Canvas are more maintainable. Reserve ASCII for short
  directory trees or pipeline bar charts.
- **PlantUML without the plugin** — fallback HTML won't render; viewer
  sees raw text. Check the target vault has the plugin before using.
