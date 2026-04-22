---
type: release
status: draft | published
version: <major.minor.patch>
release_date: YYYY-MM-DD
changelog_link: <https://github.com/org/repo/releases/tag/v...>
created: YYYY-MM-DD
tags: [type/release, project/<slug>, topic/<area>]
project: <slug>
related: []
---

# v<version> Release Notes

> [!info] Release summary
> One-paragraph pitch: what's different, who should care, whether the
> upgrade is mandatory. Aim for 2–3 sentences — users skim.

## Highlights

- Bullet 1 — one user-visible change per line, not a commit dump.
- Bullet 2 — link to the detail section or ADR.

## Breaking changes

> [!warning] Breaking
> List every source-incompatible change. Describe **before → after** and
> the migration path. If there are none, delete this section.

- `<API/flag name>`: before … → after …. Migration: …

## Bug fixes

- Short description — [GitHub PR](https://github.com/org/repo/pull/123).
  Context: [[<ADR or debug note>]] if one exists.

## Improvements

- Description. Link to underlying [[ADR-NNNN]] when the change had a
  design decision behind it.

## Internal / refactor

- Changes that don't affect users but future maintainers care about.
  Keep this short; tuck details into ADRs or dev logs.

## Upgrade guide

1. Step-by-step from the previous minor version.
2. Call out any data migration, config file renames, or CLI flag changes.
3. If users can stay on the old version, state why they might want to
   move anyway.

## Related

- [[<prior release note>]]
- [[ADR-NNNN …]] for decisions that landed in this version
- [[Retro YYYY-WNN]] if the release closed out a sprint

## External references

- [Release on GitHub](<changelog_link>)
- Migration doc / blog post
