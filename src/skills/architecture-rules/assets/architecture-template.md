# <Project> Architecture Rules

Verified at `<short-sha>` · `<YYYY-MM-DD>` · Structural view: `<docs/CODEMAPS/INDEX.md or "none">` · Feature map: `<docs/FEATURES.md or "none">`
Premises: <what holds for the whole codebase, such as "single Next.js app, no workspaces", or delete this line>

## How to use these rules

Before adding a file in a new place, a new module, or an import that crosses units:

1. Find the unit you are in and the unit you would import from in **Units**.
2. Check every rule that names either unit. A `MUST` violation is a defect. A `SHOULD` violation needs a reason in the PR.
3. A file listed under **Known violations** is not a precedent. Do not copy its pattern.

## Units

| Unit | Path | Responsibility | May depend on |
|---|---|---|---|
| `<unit>` | `<path/>` | <one sentence> | `<unit>`, `<unit>` |

## Rules

### `arch-<kebab-id>`

- **Level**: MUST
- **Rule**: <one sentence an agent can check against a diff>
- **Why**: <the user's answer from Decisions, or the evidence>
- **Evidence**: <N of M files> · `<command that measured it>`
- **Exceptions**: `<path>`, <reason> · or none
- **Enforced by**: `<tool rule id>` · draft: `<tool>` · review

## Known violations

| Path | Rule | Decision |
|---|---|---|
| `<path>` | `arch-<id>` | fix · accepted, <reason> |

## Decisions

| Date | Question | Answer |
|---|---|---|
| `<YYYY-MM-DD>` | <the gray area as asked> | <the answer and its consequence> |
