# Default PR Body and Review Checklists

## When this template applies

Only as a fallback. If the repository ships its own PR template, that one wins
without exception — the maintainers chose those sections, and a reviewer looking
for a field they defined should find it. Use what follows only when the search in
`SKILL.md` §1 finds nothing.

Trim any section the change does not touch. A template padded with "N/A" rows
buries the two sections a reviewer actually needs.

## Template

```markdown
#### Type
> One, matching the commit type: feat / fix / refactor / style / docs / test / chore

#### Background
> What this PR does and why. Link the issue or ticket if one exists.

#### Changes
> Major modifications. Add reviewer notes for non-obvious parts.

**Major files:** `path/file.ext` — summary

**Compatibility:** none / breaking API (what breaks, who migrates) / schema migration / data migration

#### Testing
**Automated:** which suites cover this — new tests for new code, a regression test for a bug fix
**Manual:** what was exercised by hand, and where
**Performance:** no impact / improvement / degradation (reason)

#### Screenshots
> UI changes only. Before / after.

#### Links
> Issue or ticket, design doc, related PR. Omit the row if there is none.

#### Checklist
- [ ] Self-review done
- [ ] Commits follow `commit-rules`
- [ ] No debug code or stray logging
- [ ] No secrets or sensitive data
- [ ] Docs updated · lockfile if deps changed · CHANGELOG if breaking
```

## Reviewer checklist

Order matters — the first two are cheap and catch the most, and their method is
`SKILL.md` §5.

- [ ] **Body matches the diff** — every claim traced to a hunk, every non-trivial hunk covered by the body
- [ ] **Necessity** — each hunk serves the stated purpose; nothing dead, speculative, already present elsewhere, or unrelated
- [ ] **Functionality** — meets the stated requirement, and the PR says what that requirement is
- [ ] **Correctness** — edge cases, error paths, and concurrent access, not just the happy path
- [ ] **Code quality** — readable, maintainable, consistent with surrounding code
- [ ] **Design** — appropriate architecture; coupling justified (depth via `dependency-design`)
- [ ] **Security** — no injection, no leaked secrets, authn/authz intact (depth via `security-review`)
- [ ] **Performance** — no regression on a path that matters
- [ ] **Testing** — coverage adequate for the tier; a bug fix carries a regression test
- [ ] **Documentation** — updated wherever behavior or an API changed

## Author checklist

- [ ] Self-review the diff before asking anyone else to
- [ ] Body written from the diff, not from the branch name or the ticket (`SKILL.md` §2)
- [ ] Provide context — why, not just what; ask rather than inventing a rationale you cannot recover
- [ ] Reply to every comment, naming the class it fell into and where it went (`SKILL.md` §6)
- [ ] CI passing before requesting review

## Language

- Technical terms stay in English (API, database, migration, refactoring).
- Match the repository's existing PR language; default to English when nothing indicates otherwise.
- Example: "Add caching logic to improve API response speed"
