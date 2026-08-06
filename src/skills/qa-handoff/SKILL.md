---
name: qa-handoff
description: Turn recent git history into a handoff document a non-developer can act on — a plain-language summary of what changed, plus a QA checklist that can actually be run, with regression risks, a glossary, and commit references for traceability. Use whenever finished development work has to be explained to QA, planners, or CS; when someone asks what changed and what should be tested; before a release or a QA request; or when test scope has to be derived from commits. 개발 내역 정리, QA 인수인계, QA 체크리스트, 비개발자 공유, 변경사항 요약, 테스트 범위 정리, 릴리즈 확인 사항.
keywords: [qa-handoff, QA인수인계, 개발내역정리, QA체크리스트, 변경사항요약, 테스트범위, 비개발자공유, 릴리즈확인, handoff, release-summary, test-scope, git-history]
---

# QA Handoff Notes

Turn a range of git history into one document with two halves: a summary
anyone on the team can read, and a QA checklist they can execute. The reader
knows the product, not the codebase.

## Why this shape

QA loses time on two failures: not knowing which changes are observable, and
not knowing how to reproduce them. So the document separates **user-visible
changes** from **internal changes that need no verification**, and every
checklist item carries preconditions, steps, and an expected result. A change
nobody can observe from outside belongs in the no-verification list — putting
it in the checklist burns a tester's afternoon for nothing.

Traceability runs the other way too: each item cites its commits, so a
developer can answer "why is this item here?" without re-reading the range.

## Workflow

### 1. Check the repo state, then fix the range

Range resolution depends on what is checked out, and every failure here is
silent — you get a plausible document describing the wrong commits. Three
commands first:

```bash
git rev-parse --is-shallow-repository                  # true → history is truncated
git symbolic-ref --short -q HEAD || echo "detached"    # current branch, or detached HEAD
git describe --tags --abbrev=0 2>/dev/null || echo "no tags"
```

- **`git describe` returns the newest tag reachable from HEAD**, not the newest tag in the repo. On a branch cut before the last release it returns an older tag, and `<tag>..HEAD` can resolve to zero commits. Compare with `git tag --sort=-v:refname | head -1`; when they differ, say which one you used and why. When both candidates resolve to an empty range, the answer is not a third tag — the branch in hand is probably behind, so ask the user to confirm it is current (`git checkout main && git pull`) instead of hunting for a range that produces output.
- **Shallow clone**: tags and older commits may be absent, so a range can look far shorter than it is. Tell the user before continuing.
- **Exit 128 from `git describe`** ("fatal: No names found") means the repo has no tags — take the no-tags path, not an error path.

Then resolve the range. Ask only when it is genuinely ambiguous; otherwise take
the default and state which one you used.

- An explicit range from the user (`v1.14.0..HEAD`, a branch, "last week") wins. Read relative phrases as rolling days — "last week" is the last 7 days, not the calendar week — and state that interpretation in the document.
- Default: `<newest reachable tag>..HEAD`.
- No tags: take the smaller of the two candidate windows by counting both.

```bash
git log --oneline --since='7 days ago' | wc -l   # candidate A
git log --oneline -20 | wc -l                    # candidate B
```

- **Zero commits in the resolved range**: stop and report it. A document built from an empty range reads as "nothing changed" when the truth is "wrong range".
- **More than ~30 commits**: a one-screen summary and one row per change cannot both hold. Offer to narrow the range, or group rows by feature instead of by commit — and say which you did. For the diffstat pass use `git log --no-merges --stat=80,40`, or sample the largest commits, rather than dumping every stat block into context.

### 2. Read the history, not just the subjects

```bash
git log --no-merges --date=short --pretty='%h %ad %s' <range>
git log --no-merges --stat <range>     # which files moved
git show <sha> -- <path>               # when the subject is not enough
```

Commit subjects lie by omission — `fix: guard empty input` says nothing about
what the user now sees. Read the diff for anything you cannot classify with
confidence. A guessed checklist is worse than a short one, because QA cannot
tell which items to trust.

### 3. Translate each change into what the user experiences

For every commit, answer one question: **what does someone using the product
now experience differently?** One sentence, in product vocabulary.

| Commit | Weak (dev-speak) | Usable |
|---|---|---|
| `fix: resolve CLI program via PATHEXT` | "resolve_cli_program now walks PATH itself" | "On Windows, MCP servers installed through npm now appear in the list. Before, the list came up empty." |

When the honest answer is "nothing visible", it is an internal change: list it
under internal changes with one clause saying why it is invisible.

One exception outweighs that rule. Changes to authentication, authorization, or
data integrity are invisible on the happy path **by design** — nothing looks
different when they work. They belong in the checklist as negative-path items
("with an expired token the request is rejected"), because what must be verified
is what no longer happens.

### 4. Write the checklist so it can be run

Each item needs preconditions (account, data, environment) → numbered steps →
expected result. Mark priority: must-pass (blocks the release) or secondary.
Order items by user flow rather than commit order — testers move through
screens, not through git history.

### 5. Name the regression risks

Changes to shared modules, config or settings files, migrations, and
platform-specific paths put untouched features at risk. Say which features and
why, so the regression budget goes where the change actually reaches.

### 6. Flag what you could not determine

Unknowns go in the open-questions section together with the question to ask the
developer. Never close a gap with a plausible guess — a wrong expected result
sends QA chasing a bug that does not exist, and costs the team more than an
admitted unknown.

Name who to ask: `git log --format='%an' -1 <sha>` gives the author of the
commit the question came from, so QA is not left guessing which developer owns
it in a range several people touched.

### 7. Choose the destination

Read what kind of ask it is first. Signals that the result will be shared or
kept — the whole team should see it, someone will refer back to it, it goes to
another function — make it a document request. A one-off need for an answer now
is a question.

When it is a document request, ask where it goes unless the user already said.
When it is a question — "what should QA look at before tomorrow's release?" —
answer in the conversation and offer to save it; stopping to ask about a file
path is noise when someone wants an answer now.

- Save as Markdown in the repo — default `docs/handoff/<YYYY-MM-DD>-<range-slug>.md`. Slugify the range first (strip slashes and spaces: `feature/foo..HEAD` → `feature-foo-HEAD`), or a branch name quietly creates a subdirectory.
- Print in the conversation only, for pasting into Slack or Confluence
- Publish to Confluence (Atlassian MCP; needs a space and parent page)

Writing the file into the repo is a content change, so the post-work review gate
in `CLAUDE.md` applies to it like any other.

## Document template

Use `assets/handoff-template.md` as the skeleton. Keep the section order — QA
reads top to bottom and stops once they have what they need.

## Writing rules

- **Conclusion first** — every bullet leads with the outcome, detail after (same rule as `coding-standards`).
- **Product vocabulary** — no file, function, or type names in the summary or checklist. If a technical term is unavoidable, define it in the glossary. Exception: when the repo **is** the product (tooling, configuration, skill or template repos), skill / command / setting names **are** the product vocabulary — name them exactly, and replace environment preconditions with the install or sync state the reader needs.
- **One change, one row** — never collapse three commits into "various improvements".
- **No hedging** — "needs confirmation" is honest; "probably fine" is not.
- **Summary fits one screen** — detail belongs in the checklist, not the summary.

## Related

| Need | Where |
|---|---|
| Release note as an Obsidian vault note | `obsidian-notes` skill (`assets/release-note.md`) |
| PR description for the same range | `pull-request` skill |
| Automated test coverage for the change | `tdd-workflow` skill |
