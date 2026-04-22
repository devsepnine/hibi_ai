---
type: meeting
status: scheduled | captured | actioned | archived
date: YYYY-MM-DD
start_time: "HH:MM"
duration_min: 30
meeting_type: sync | review | 1on1 | planning | retro | adhoc
attendees: ["<handle>", "<handle>"]
agenda_link: null          # "[[Agenda ...]]" or "https://..."
created: YYYY-MM-DD
tags: [type/meeting, project/<slug>]
project: <slug>
related: []
---

# YYYY-MM-DD <Meeting slug / topic>

> [!info] Purpose
> One sentence: what this meeting is trying to resolve. If you can't
> name it, consider whether the meeting should be an email/async
> thread instead.

## Attendees

- <handle> — role if not obvious
- <handle>

## Agenda

1. …
2. …
3. …

If the agenda came from `agenda_link`, you can just embed:
`![[Agenda YYYY-MM-DD ...]]`

## Discussion

Structured by agenda item. Each item gets a mini-section so later
readers can jump without re-reading the whole transcript.

### 1. <Topic>

- Key points raised
- Data shared (link to [[supporting note]] or paste numbers)
- Disagreements and their resolution (or that they're unresolved)

### 2. <Topic>

…

## Decisions

> [!success] Decided
> Numbered decisions with owner when applicable. If a decision is
> weighty enough to deserve an ADR, flag it and create one:
> [[ADR-NNNN …]] to be drafted by <owner>.

1. **<Decision>** — context in one line.
2. **<Decision>** — …

If nothing was decided, say so explicitly: `> [!question] Deferred —
no decision.` It keeps the reader from wondering if they missed it.

## Action items

Each task becomes an Obsidian task (`- [ ]`), with an owner and due
date, and a link to the follow-up note when one exists.

- [ ] <Action> — owner: <handle> — due: YYYY-MM-DD — tracking:
  [[Follow-up note]] or `<ticket url>`
- [ ] <Action> — owner: <handle> — due: YYYY-MM-DD

Dataview will roll these up across meetings into the weekly review.

## Open questions

- What we don't know yet. These often seed the next meeting's agenda.

## Parking lot

Topics raised that weren't on the agenda and didn't fit in time.
Promote to the next meeting or a proper ticket.

## Related

- [[Previous meeting in this series]]
- [[MOC — <project>]]
- Adjacent decisions: [[ADR-NNNN …]]
