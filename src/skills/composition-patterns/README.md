# React Composition Patterns

Composition patterns for React components that stay flexible as they scale.

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

- `CRITICAL` — foundational patterns; prevents unmaintainable code
- `HIGH` — significant maintainability improvements
- `MEDIUM` — good practices for cleaner code

## Adding a rule

1. Copy `rules/_template.md` to `rules/<prefix>-<topic>.md`.
2. Use a prefix declared in `rules/_sections.md`; add the section there first if none fits.
3. Add it to the rule index in `SKILL.md` and `SKILL-ko.md` — there is no build step, so this is manual.

## Core principles

1. **Composition over configuration** — instead of adding props, let consumers compose
2. **Lift your state** — state in providers, not trapped in components
3. **Compose your internals** — subcomponents access context, not props
4. **Explicit variants** — `ThreadComposer` / `EditComposer`, not `Composer` with `isThread`
