---
name: architecture-rules
description: "Write docs/ARCHITECTURE.md for an existing codebase: rules mined from its imports and layout, gray areas settled by Q&A, lint drafts. Use when defining or refreshing architecture rules. 아키텍처 규칙, 아키텍처 문서화, 레이어 규칙."
---

# Architecture Rules

Write `docs/ARCHITECTURE.md`: the rules an AI must follow before it adds a file, a module, or an import in this project. Most of them are already in the code, as patterns nearly every file follows. The rest are decisions nobody wrote down, and the code alone cannot tell a rule from an accident.

So the method has two halves. Mine every rule the code can prove, with its evidence. Then ask the user only about the gray areas, where the code contradicts itself or cannot reveal intent.

## Why this shape

A rule document written from memory describes the architecture someone intended. One written only from code freezes every accident as a rule. Neither tells the next agent what to do when it meets a file that breaks the pattern. This document records, per rule, how strongly it holds, the proof, and the known exceptions, so a violation in new code is distinguishable from one that is already accepted.

This is not `dependency-design`, which judges whether a coupling is healthy in general. It is not a codemap from `/update-codemaps`, which describes structure without saying what is allowed. It is not `feature-map`, which maps reports to code. Link them; never copy one into another.

## Workflow

### 1. Preflight: new document or update

```bash
test -f docs/ARCHITECTURE.md && grep -m1 -E '^(Verified at|검증 시점)' docs/ARCHITECTURE.md
git rev-parse --short HEAD
ls .dependency-cruiser.* .eslintrc* eslint.config.* importlinter* .importlinter go-arch-lint.* 2>/dev/null
```

- **Document exists**: update mode, see [Update mode](#update-mode).
- **No document**: full build, steps 2 to 7.
- Note the stack from the manifests: `package.json` and workspaces, `Cargo.toml` workspace, `go.mod`, `pyproject.toml`. Note any boundary lint already configured, because its rules are decided rules and need no question.
- Write the document's prose in the language the repository's other docs use. Keep the template's headings and field labels, rule IDs, `MUST`, and `SHOULD` in English, because the verify block below greps for them.
- Write every path relative to the repository root, including inside a monorepo package.

### 2. Find the units

A unit is a part of the code that other parts depend on as a whole: a package, a crate, a top-level module, or a layer such as `routes`, `services`, and `repositories`. Take them from the manifests and the import graph, not from folder names alone. A folder that nothing imports as a unit is not one.

For each unit, record its path and one sentence of responsibility, written from what its files do.

### 3. Measure the candidate rules

Every candidate rule is a pattern with a count. Measure it with a command you can rerun, and keep the command, because update mode reruns it.

| Kind | What to measure | How |
|---|---|---|
| Dependency direction | Which unit imports which, and cycles | the stack's graph tool when installed, such as `madge`, `dependency-cruiser`, `cargo modules`, or `go list -deps`; otherwise grep the import statements |
| Forbidden edges | Imports that exist in only a few files | the same graph, filtered to edges with low counts |
| Placement | Where each kind of file lives: routes, components, tests, migrations | glob counts per kind |
| Naming | File and symbol naming per unit | glob and grep counts |
| Cross-cutting access | Where env and config, the database, logging, and HTTP clients are touched from | grep for the client or accessor, grouped by unit |

Record each candidate as: the rule in one sentence, the count, such as `47 of 50 files`, the command, and every counter-example as a path.

### 4. Sort the candidates

- **Proven**: every instance follows it, or the counter-examples are generated code, vendored code, or tests. Draft it as a rule without asking.
- **Gray**: the code contradicts itself, such as 3 services importing a route module, a cycle, a folder doing two jobs, or two naming styles of similar size. Only the user can say which side is the rule.
- **Intent the code cannot show**: a planned migration, a layer that is supposed to exist but is empty, a deprecated module still in use. Ask.

A boundary lint that already exists turns its rules into proven rules.

### 5. Settle the gray areas by Q&A

Ask in batches of up to four questions with `AskUserQuestion`. Each question carries its evidence: the count, the counter-example paths, and what each answer would mean for those files. Put the recommended answer first, with the reason. The answers are usually:

- **Make it a rule**: the counter-examples become known violations to fix.
- **Allow an exception**: scope it to named paths and say why.
- **Not a rule**: drop the candidate.
- **Migrating**: the rule is the target state, and the old pattern is allowed until a named condition.

Never ask what the code already answers. Record every answer with the date in the Decisions section, because the next person will ask the same question.

### 6. Write the document

Use `assets/architecture-template.md`. For each rule:

- **ID**: stable kebab-case with an `arch-` prefix, such as `arch-routes-no-repository-import`.
- **Level**: `MUST` when a violation is a defect, `SHOULD` when a violation needs a reason in review.
- **Rule**: one sentence an agent can check against a diff.
- **Why**: the user's answer or the evidence; never an invented reason.
- **Evidence**: the count and the command that measured it.
- **Exceptions**: paths with their reason, or `none`.
- **Enforced by**: the lint rule that checks it, `draft: <tool>` when a draft is proposed, or `review` when only a reviewer can check it.

Put known violations in their own section, each with the rule ID and the decision: fix or accepted. Keep the document under about 300 lines. Paths, not descriptions; no line numbers, because they drift.

### 7. Draft the enforcement, then wire it in

For every `MUST` that a tool can check, such as import direction, a forbidden import path, or placement by glob, draft the config in the tool the project already uses. When it has none, use the stack's usual one: `dependency-cruiser` or `eslint-plugin-boundaries` for TypeScript, `import-linter` for Python, `go-arch-lint` or `depguard` for Go, and module visibility or workspace crate boundaries for Rust.

Show the draft in the reply. Do not install a package, add a config file, or change the build without the user's agreement. If the tool is already installed, run the draft once in check mode and report what it flags; the known violations from step 6 should be exactly that list.

Then make sure an agent reads the rules before it writes code. Propose a one-line pointer in the project's `CLAUDE.md` or `AGENTS.md`, and add it when the user agrees:

```markdown
- Before adding a module, a file in a new place, or an import across units, check `docs/ARCHITECTURE.md`.
```

Writing the document is a content change, so the post-work review gate in `CLAUDE.md` applies.

### Verify before handing back

```bash
DOC=docs/ARCHITECTURE.md
# every repo-relative path exists; globs and <placeholders> are skipped
grep -ohE '`[^`<> /][^`<> ]*/[^`<> ]*`' $DOC | tr -d '`' | grep -v '[*?{[]' | sort -u \
  | while read -r p; do test -e "$p" || echo "MISSING $p"; done
# rule IDs are unique as headings; the same ID may recur in Known violations
grep -oE '^### `arch-[a-z0-9-]+`' $DOC | sort | uniq -d | sed 's/^/DUPLICATE /'
# every rule carries every field
rules=$(grep -c '^### `arch-' $DOC)
for f in Level Rule Why Evidence Exceptions 'Enforced by'; do
  n=$(grep -c "^- \*\*$f\*\*" $DOC); [ "$n" = "$rules" ] || echo "FIELD $f: $n of $rules rules"
done
```

The block prints nothing when the document is sound. Then rerun each evidence command and confirm the counts in the document still match.

## Update mode

1. Diff from the recorded SHA: `git diff --name-status -M <sha>..HEAD`. When the SHA is unreachable, say so and remeasure every rule.
2. Repoint paths for renamed files, marked `R`, in Units, Exceptions, and Known violations. Drop an exception or a violation whose file was deleted, marked `D`.
3. Rerun each rule's evidence command. A rule whose count did not move stays as written. When most rules are suspect, run the verify block over the whole document first, and reopen a Decision only when the files it named appear in the diff.
4. A new counter-example is a question, not an edit: fix it, allow it as an exception, or relax the rule. Ask in the step-5 format.
5. A new unit, or a unit that disappeared, goes through steps 2 to 5 for its rules only.
6. Never rewrite a decided rule silently. Report the rules added, changed, and removed, then move the header SHA to HEAD.

## Related

| Need | Where |
|---|---|
| Whether a coupling is healthy in general | `dependency-design` |
| A design decision for new work | `architect` agent |
| How the code is organized today | `/update-codemaps` |
| Which code a vague report points at | `feature-map` |
