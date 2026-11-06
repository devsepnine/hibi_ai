---
name: qa-handoff
description: "Turn git history into a non-developer handoff: change summary, QA checklist, regression risks, commits. Use when summarizing merged work for non-developers. QA 인수인계, 비개발자 공유, 변경사항 요약, 확인 체크리스트, 개발 내역 정리."
---

# QA Handoff Notes

Turn a range of git history into one document with two halves: a summary
anyone on the team can read, and a QA checklist they can execute. The reader
knows the product, not the codebase.

## Why this shape

QA loses time on two failures: not knowing which changes are observable, and
not knowing how to reproduce them. Compare the previous and current behavior,
then give QA the setup, reproduction steps, and expected result for each check.
Omit internal changes with no observable effect. A separate list of work QA
does not need to check adds reading without helping them test.

Keep commit evidence in a short reference at the end when needed. Do not repeat
hashes in the summary, change table, or checklist. QA should not need to read code.

## Workflow

### 1. Check the repo state, then fix the range

Range resolution depends on what is checked out, and every failure here is
silent, and you get a plausible document describing the wrong commits. Three
commands first:

```bash
git rev-parse --is-shallow-repository                  # true → history is truncated
git symbolic-ref --short -q HEAD || echo "detached"    # current branch, or detached HEAD
git describe --tags --abbrev=0 2>/dev/null || echo "no tags"
```

- **`git describe` returns the newest tag reachable from HEAD**, not the newest tag in the repo. On a branch cut before the last release it returns an older tag, and `<tag>..HEAD` can resolve to zero commits. Compare with `git tag --sort=-v:refname | head -1`; when they differ, say which one you used and why. When both candidates resolve to an empty range, the answer is not a third tag. The branch in hand is probably behind, so ask the user to confirm it is current, for example with `git checkout main && git pull`, instead of hunting for a range that produces output.
- **Shallow clone**: tags and older commits may be absent, so a range can look far shorter than it is. Tell the user before continuing.
- **Exit 128 from `git describe`**, with the message "fatal: No names found", means the repo has no tags. Take the no-tags path, not an error path.

Then resolve the range. Ask only when it is genuinely ambiguous; otherwise take
the default and state which one you used.

- An explicit range from the user, such as `v1.14.0..HEAD`, a branch, or "last week", wins. Read relative phrases as rolling days: "last week" is the last 7 days, not the calendar week. State that interpretation in the document.
- For the feature or ticket just completed in the conversation, use its known commits rather than expanding to a whole release.
- Default: `<newest reachable tag>..HEAD`.
- No tags: take the smaller of the two candidate windows by counting both.

```bash
git log --oneline --since='7 days ago' | wc -l   # candidate A
git log --oneline -20 | wc -l                    # candidate B
```

- **Zero commits in the resolved range**: stop and report it. A document built from an empty range reads as "nothing changed" when the truth is "wrong range".
- **More than ~30 commits**: a one-screen summary and one row per change cannot both hold. Offer to narrow the range, or group rows by feature instead of by commit, and say which you did. For the diffstat pass use `git log --no-merges --stat=80,40`, or sample the largest commits, rather than dumping every stat block into context.

### 2. Read the history, not just the subjects

```bash
git log --no-merges --date=short --pretty='%h %ad %s' <range>
git log --no-merges --stat <range>     # which files moved
git show <sha> -- <path>               # when the subject is not enough
```

Commit subjects lie by omission. `fix: guard empty input` says nothing about
what the user now sees. Read the diff for anything you cannot classify with
confidence. A guessed checklist is worse than a short one, because QA cannot
tell which items to trust.

### 3. Translate each change into what the user experiences

For every commit, answer one question: **what does someone using the product
now experience differently?** One sentence, in product vocabulary.

| Commit | Weak, dev-speak | Usable |
|---|---|---|
| `fix: resolve CLI program via PATHEXT` | "resolve_cli_program now walks PATH itself" | "On Windows, MCP servers installed through npm now appear in the list. Before, the list came up empty." |

When the honest answer is "nothing visible", omit it from the QA document.
Keep refactoring and test maintenance in the development record.

One exception outweighs that rule. Changes to authentication, authorization, or
data integrity are invisible on the happy path **by design**, since nothing looks
different when they work. They belong in the checklist as negative-path items
such as "with an expired token the request is rejected", because what must be verified
is what no longer happens.

### 4. Connect API changes to the product

When this work changes an API contract or consumes a changed contract, include
**API changes** after the before/after comparison. In Korean use **API 변경사항**.
Name the endpoint when known, then list the changed field, what changed, and
the screen or behavior that uses it. Exact API field names belong in this section;
pair each with a plain description so QA can connect a response to the screen.
For example, a new reward quantity combines with the unit value to display the
bonus amount in a selection list. Link the contract or related backend ticket.
Distinguish a planned API change from one verified in a real response. Do not
invent a previous field or imply that adding a consumer deploys the provider.
Omit the section when there is no relevant API change. Do not paste full schemas
or turn the QA checklist into code inspection.

### 5. Write the checklist so it can be run

Frame the checklist as areas the developer would like QA to check, not release gates.
Use **Setup**, **Reproduction steps**, and **Expected result** for each item.
In Korean, use **전제**, **재현 절차**, and **기대 결과** exactly.
Include only account, data, or environment details needed to reproduce the behavior.
Number the actions. Do not label items required, secondary, must-pass, or release-blocking.
Order items by user flow rather than commit order, because testers move through
screens, not through git history.

### 6. Name the regression risks

Name the existing feature to recheck and the behavior that should remain unchanged.
Explain the user impact, not the dependency graph. For example, ask QA to confirm
that existing bonuses still show the same names and amounts.

### 7. Explain automated verification in product terms

Start with the behavior exercised and the observed result. Add test counts or
coverage briefly afterward. State whether tests used sample responses or a live
service, and distinguish feature coverage from whole-application coverage.
Name the remaining manual or integration checks. Do not turn a local pass into
a claim of deployment, full E2E coverage, or production readiness.

### 8. Flag what you could not determine

Unknowns go in the open-questions section together with the question to ask the
developer. Never close a gap with a plausible guess, because a wrong expected result
sends QA chasing a bug that does not exist, and costs the team more than an
admitted unknown.

Name who to ask: `git log --format='%an' -1 <sha>` gives the author of the
commit the question came from, so QA is not left guessing which developer owns
it in a range several people touched.

### 9. Choose the destination

Read what kind of ask it is first. Signals that the result will be shared or
kept: the whole team should see it, someone will refer back to it, it goes to
another function. Any of these makes it a document request. A one-off need for an answer now
is a question.

When it is a document request, ask where it goes unless the user already said.
When it is a question, such as "what should QA look at before tomorrow's release?",
answer in the conversation and offer to save it; stopping to ask about a file
path is noise when someone wants an answer now.

- Save as Markdown in the repo, default `docs/handoff/<YYYY-MM-DD>-<range-slug>.md`. Slugify the range first by stripping slashes and spaces, for example `feature/foo..HEAD` → `feature-foo-HEAD`, or a branch name quietly creates a subdirectory.
- Print in the conversation only, for pasting into Slack or Confluence
- Publish to Confluence through the Atlassian MCP, which needs a space and parent page
- Update the requested Jira description. Read its title, assignee, and current body first. For "my ticket", locate the user's implementation ticket rather than assuming the parent epic. Ask only if the target remains ambiguous. Preserve unrelated content and read the saved description back.

Writing the file into the repo is a content change, so the post-work review gate
in `CLAUDE.md` applies to it like any other.

## Document template

Use `assets/handoff-template.md` as the skeleton. Keep the section order: QA
reads top to bottom and stops once they have what they need.
Use these Korean section names: 전체 요약, 기존과 달라진 변경사항, API 변경사항 when relevant, QA 체크리스트,
회귀 주의 영역, 자동 검증 결과, 용어 설명, 개발자 확인 사항.
Do not include a default deployment prerequisites section. Put essential setup
in the relevant checklist item and unresolved questions in developer follow-up.

## Writing rules

- **Conclusion first**: every bullet leads with the outcome, detail after. This is the same rule as `coding-standards`.
- **Product vocabulary**: no file, function, or type names in the summary or checklist. If a technical term is unavoidable, define it in the glossary. Exception: when the repo **is** the product, as in tooling, configuration, skill, or template repos, skill / command / setting names **are** the product vocabulary. Name them exactly, and replace environment preconditions with the install or sync state the reader needs.
- **One change, one row**: never collapse three commits into "various improvements".
- **No hedging**: "needs confirmation" is honest; "probably fine" is not.
- **Summary fits one screen**: detail belongs in the checklist, not the summary.
- **Before and after**: describe each observable change with the previous and current behavior. Keep internal implementation details out of the comparison.
- **Glossary stays precise**: include only terms needed to understand this handoff. Explain each in one short sentence using everyday words. Distinguish similar terms without defining one unfamiliar term through another. Omit code symbols and irrelevant implementation terms.
- **Developer follow-up stays actionable**: keep unresolved questions, name who can answer, and say which QA check needs the answer. Do not add a generic deployment checklist.

## Related

| Need | Where |
|---|---|
| Release note as an Obsidian vault note | `obsidian-notes` skill, `assets/release-note.md` |
| PR description for the same range | `pull-request` skill |
| Automated test coverage for the change | `tdd-workflow` skill |
