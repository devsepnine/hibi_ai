# React Native Guidelines

React Native performance rules, optimized for agents and LLMs.

A vendored rule set. `SKILL.md` is the entry point; the rules themselves live
in `rules/`, one file per rule. The upstream compiled output `AGENTS.md` is not
vendored — `rules/` is the source and `SKILL.md` the index.

## Structure

- `SKILL.md` / `SKILL-ko.md` — entry point: when to apply, rule index by category
- `rules/` — one file per rule
  - `_sections.md` — section order, filename prefix, impact level, description
  - `_template.md` — rule file template (frontmatter shape, Incorrect/Correct format)
  - `<prefix>-<topic>.md` — the rules
- `metadata.json` — upstream document metadata (version, abstract, references)

## Impact levels

- `CRITICAL` — highest priority; causes crashes or broken UI
- `HIGH` — significant performance improvements
- `MEDIUM` — moderate performance improvements
- `LOW` — incremental improvements

## Adding a rule

1. Copy `rules/_template.md` to `rules/<prefix>-<topic>.md`.
2. Use a prefix declared in `rules/_sections.md`; add the section there first if none fits.
3. Add it to the rule index in `SKILL.md` and `SKILL-ko.md` — there is no build step, so this is manual.
