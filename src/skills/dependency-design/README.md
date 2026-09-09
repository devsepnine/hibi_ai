# Dependency Design

Dependency, coupling, and abstraction decisions that keep software modifiable and AI-ownable.

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
- `references/` — deep-dive methodology, read on demand (complexity, coupling models, abstraction, AI ownership, monorepo)
- `evals/` — eval cases verifying the skill triggers and applies correctly

## Impact levels

- `CRITICAL` — foundational dependency-direction / coupling rules; violation produces unmaintainable, non-isolatable code
- `HIGH` — significant modifiability or AI-ownability gains (abstraction consistency, one-way layering)
- `MEDIUM` — good practices that reduce ripple and clarify boundaries

## Adding a rule

1. Copy `rules/_template.md` to `rules/<prefix>-<topic>.md`.
2. Use a prefix declared in `rules/_sections.md`; add the section there first if none fits.
3. Add it to the rule index in `SKILL.md` and `SKILL-ko.md` — there is no build step, so this is manual.

In the `-ko` files, keep each rule's `title` and `impactDescription`, and the
section titles, in English — translate only the body prose. `SKILL.md` indexes
rules by those English titles, so translating one desynchronizes the index.
