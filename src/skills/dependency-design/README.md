# Dependency Design

A structured repository for dependency, coupling, and abstraction decisions that keep vibe-coded software modifiable and AI-ownable. The methodology classifies a problem's understandability, matches a coupling strength to it, enforces one-way dependency, and keeps abstraction consistent.

## Structure

- `SKILL.md` / `SKILL-ko.md` - Skill entry point: when-to-apply table, core decision flow, and links into references. The `-ko` file mirrors it in Korean with byte-identical frontmatter.
- `references/` - Deep-dive methodology, read on demand:
  - `complexity.md` - complexity, understandability, Cynefin framework
  - `coupling-models.md` - module / connascence / domain coupling models
  - `abstraction.md` - interface/implementation/context, abstraction consistency
  - `ai-ownership.md` - structuring code for partial, context-bounded AI ownership
  - `monorepo.md` - layered abstraction and Turbo monorepo one-way dependency
- `rules/` - Individual rule files (one per rule):
  - `_sections.md` - Section metadata (section prefixes, titles, impact levels)
  - `_template.md` - Template for creating new rules
  - `<section>-<topic>.md` - Individual rule files
- `metadata.json` - Document metadata (version, organization, abstract, references)
- **`AGENTS.md`** - Compiled output (generated from `rules/`)
- `evals/` - Eval cases verifying the skill triggers and applies correctly

## Creating a New Rule

1. Copy `rules/_template.md` to `rules/<section>-<topic>.md`.
2. Choose a section prefix that is declared in `rules/_sections.md` (each prefix maps to a section title and impact level). If no existing prefix fits, add the new section to `rules/_sections.md` first, then use its prefix.
3. Fill in the frontmatter and content. Write a clear **Incorrect** example (with explanation of what ripples) and a **Correct** example (with explanation of why the coupling is now acceptable or one-directional).
4. Regenerate `AGENTS.md` so the compiled rule set picks up the new file.

> In the `-ko` files, keep each rule's `title`, `impactDescription`, and the section titles in English (translate only the body prose). The `AGENTS.md` / `AGENTS-ko.md` table-of-contents anchors are generated from these headings, so translating a heading would break the anchor links.

## Impact Levels

- `CRITICAL` - Foundational dependency direction / coupling rules; violation produces unmaintainable, non-isolatable code.
- `HIGH` - Significant modifiability or AI-ownability improvements (abstraction consistency, one-way layering).
- `MEDIUM` - Good practices that reduce ripple and clarify boundaries.
