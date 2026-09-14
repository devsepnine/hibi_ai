---
name: pull-request
description: PR body from the diff, review that questions whether each change is needed, comment triage. Use when opening, reviewing, or answering review comments on a GitHub PR. PR 생성, 풀리퀘스트, PR 리뷰, 리뷰 코멘트 반영, 업스트림 PR, 설정 개선 환류.
---

# Pull Request

A PR is a review request, not a delivery mechanism. Its job is to let someone
else judge the change: what it does, why, what could break, and what evidence
says it works. Everything below serves that, which is why the conventions come
from the repository being worked in rather than from this document — a reviewer
looking for a field their team defined must find it where they expect it.

**Ask instead of guessing.** Most bad PRs come from one of two guesses: writing a
body from what the branch name implies rather than from the diff, or applying a
review comment without checking whether it belongs to this PR. When the diff, the
thread, or the ticket does not answer a question, ask it — one message costs less
than a rationale a reviewer then trusts and repeats. Where the answer would not
change what you write, state the assumption in the body instead of blocking on
it.

## Two cases

| Need | Where |
|---|---|
| Open, update, review, or answer comments on a PR in any repository | §1–§7 below |
| Promote a session-derived improvement into the distributed config | `references/upstream-config.md` — adds source resolution, four gates, and a privacy review on top of §1–§7 |
| Default body template and review checklists (fallback only) | `references/pr-body.md` |

## 1. Read the conventions from the repo, not from memory

Every value below is discoverable. Assuming one is how a PR ends up targeting a
branch that does not exist or carrying a ticket prefix the project never used.

**Base branch** — never assume `main` or `develop`:

```bash
gh repo view --json defaultBranchRef -q .defaultBranchRef.name
gh pr list --state merged --limit 10 --json baseRefName -q '.[].baseRefName' | sort | uniq -c
```

The default branch is the starting guess; what recent merged PRs actually
targeted is the stronger signal, because plenty of projects merge into an
integration branch instead. When the two disagree, say so and ask.

**Ticket ID** — extract, do not invent:

```bash
git branch --show-current          # feature/ABC-123-thing → ABC-123
git log --oneline -20              # the prefix existing commits use, not a number to reuse
```

If the branch and history yield no ticket, **omit the prefix entirely** — the
title is the summary alone. A config PR, a personal project, and a repo with no
tracker all legitimately have no ticket, and inventing a placeholder like
`[TICKET-1]` is worse than having none.

The branch decides. When history carries a prefix the branch does not, say so
and ask instead of borrowing the number — someone else's real ticket reads as
valid in the title, which makes it worse than an omitted one.

Title format, therefore:

```
[ABC-123] Add user authentication system     # ticket exists
Add user authentication system               # no tracker, or none applies
```

**Body** — the repository's own template wins unconditionally:

```bash
ls .github/pull_request_template.md .github/PULL_REQUEST_TEMPLATE.md \
   .github/PULL_REQUEST_TEMPLATE/ docs/pull_request_template.md 2>/dev/null
```

Fill that template's sections with the content §2 derives. Only when none
exists, fall back to `references/pr-body.md`. Do not merge the two — a reviewer
reading their own template with extra sections spliced in cannot tell what the
project requires.

**Tracker links** — follow the convention the repo documents (existing PR bodies
show it). Never hardcode a tracker host.

## 2. Write the body from the diff

Read the diff before writing a word. The branch name, the ticket, and the commit
messages say what someone intended; only the diff says what the PR actually does.

```bash
git diff --stat "$BASE"...HEAD     # shape first: how many files, how large
git diff "$BASE"...HEAD            # then the content, hunk by hunk
git log --oneline "$BASE"..HEAD    # what the author said along the way
```

Three dots, not two: `"$BASE"...HEAD` is what this branch added since it
diverged, so movement on the base branch does not leak into the diff you
describe.

**Match the body to the diff both ways.** Every non-trivial hunk appears
somewhere in the body; every claim in the body maps to a hunk. Each unmatched
item is a defect you get to fix before a reviewer finds it — an unlisted hunk is
scope that crept in, and an unmatched claim is fiction.

**Be concise by writing what the diff cannot show.** One line per change a
reviewer must know about. They can read the diff, so do not narrate it; spend the
space on why this approach, what you rejected, where to start reading, and what
could break. A body that restates the diff in prose is longer and less useful
than three lines that say what is not visible in it.

**Ask when you cannot recover a why.** A magic number, a skipped test, a
dependency bump, a behavior change the ticket does not imply — if the diff does
not explain it and neither does the history, ask the author rather than composing
a plausible reason. A body that states a wrong reason is worse than one that
omits it, because the wrong reason gets quoted in the next design discussion.

If the Changes section grows past what a reviewer will hold in their head, the
problem is the PR, not the body — split it (§3, Size).

## 3. Pass the pre-PR gates

1. **Verification** — lint, type-check, and tests green (`verification-loop`).
2. **Branch** — on a feature branch, not the base branch.
3. **Up to date** — rebased on the target resolved in §1.
4. **Commits** — squash noise; each commit independently buildable (`commit-rules`).
5. **Conflicts** — resolved.
6. **Docs** — updated if behavior or an API changed.
7. **Secrets** — no credentials, PII, debug code, or stray logging in the diff.

Depth for each lives elsewhere — this table is the routing, not the policy:

| Check | Source of truth |
|-------|-----------------|
| File/function size, complexity | `coding-standards` → `references/code-thresholds.md` |
| Secrets, injection, XSS, authn | `security-review` |
| Coverage, regression, E2E paths | `tdd-workflow` |
| Build, type, lint | `verification-loop` |
| Commit message format | `commit-rules` |
| Coupling and module boundaries | `dependency-design` |
| Change summary for the body | `qa-handoff` |
| How much rigor this change warrants | `do-178c` (A–E tier) |

**Size** — keep PRs small and split by logical unit. A reviewer's attention is
the scarce resource; two reviewable PRs beat one that gets rubber-stamped.

## 4. Open it — only when asked

**Preparing a branch and drafting a description is not permission to open a PR.**
Opening one is outward-facing: it notifies people and publishes the branch. Do it
only on an explicit request, the same rule that governs commits. Cases that reach
a public repository raise this bar further — see `references/upstream-config.md`.

Show the rendered title and body first, then:

```bash
gh pr create --base "$BASE" --title "$TITLE" --body-file pr-body.md --draft
gh pr ready <PR-number>      # when it is no longer a draft
```

- **`--body-file`, not `--body`** — a multi-line body with backticks and quotes is easy to mangle through shell quoting.
- **`--draft` when anything is still moving** — CI unverified, a question open, or the branch likely to be rebased.
- **`--fill` only for a single clean commit** — it builds the body from commit messages, which skips §2 entirely; on a branch with noise commits it produces a body nobody wrote.

## 5. Review an existing PR

Gather before judging — the diff alone hides whether CI passed or someone already
raised the point:

```bash
gh pr view <n> --json title,body,baseRefName,isDraft,reviewDecision,statusCheckRollup
gh pr diff <n> --name-only     # shape first
gh pr diff <n>                 # then the content
gh pr view <n> --comments      # what has already been said
gh pr checks <n>
```

Then judge in this order, cheapest question first:

**a. Does the diff match the body?** Trace it both ways, as in §2. Hunks the body
never mentions are the fastest thing to find and the most likely to be scope the
author did not intend to ship.

**b. Is each change needed?** Ask this per hunk, not of the PR as a whole. The
body states a purpose; a hunk that does not serve it is either unmentioned scope
or code nobody asked for.

- **Delete-test** — if this hunk were removed, what breaks? "Nothing" means it is dead on arrival: an unused export, a parameter no caller passes, a branch no input can reach.
- **Caller count** — a new abstraction, option, flag, or generic parameter with exactly one caller and no second use in the diff is speculative. Ask what the second use will be; absent one, the concrete version is the smaller change.
- **Already exists** — grep the repo before accepting a new helper. A reimplementation is the most expensive kind of addition, because both copies now need maintaining.
- **Guards that cannot fire** — a null check on a non-nullable type, a `try` around code that does not throw, validation the caller already performed. Each one tells the next reader that condition is reachable.
- **Unrelated to the purpose** — a rename, a formatting sweep, or a drive-by fix folded into a feature PR. Not wrong to want; wrong to hide here, because it makes the diff unreviewable and the revert unusable.

Necessity is a question, not a verdict. When you cannot tell what a hunk is for,
ask the author — the answer is usually one sentence, and it belonged in the body
anyway.

**c. Is it correct?** Edge cases, error paths, existing callers. Depth in
`/code-review`, and `security-review` when the diff touches auth, input
handling, or secrets.

**d. Is it tested?** Would a test fail if this change were reverted? If not, the
claim is unverified regardless of coverage numbers (`tdd-workflow`).

Posting a review is outward-facing too — draft it, show it, and post only when
asked:

```bash
gh pr review <n> --comment --body-file review.md
gh pr review <n> --approve | --request-changes --body-file review.md
```

Distinguish blocking findings from suggestions explicitly. A review that lists
eight equal-weight comments makes the author guess which two actually block.

## 6. Work through review comments

A review comment is a claim, not an instruction. Applying every comment without
triage is how a focused PR becomes unreviewable, and how a mistaken comment turns
into shipped code.

Read the whole thread first, resolved rounds included. Every listing here
truncates silently at its default, so paginate: a REST page stops at 30, and
`gh pr view --json commits` stops at the oldest 100 — on a longer PR it drops
exactly the recent commits an n-th round turns on.

```bash
gh pr view <n> --comments                     # skim; renders only the recent ones
gh api --paginate "repos/{owner}/{repo}/pulls/<n>/comments" \
  --jq '.[] | "\(.created_at) \(.path):\(.line // .original_line) \(.user.login): \(.body)"'
gh api --paginate "repos/{owner}/{repo}/pulls/<n>/commits" \
  --jq '.[] | "\(.commit.committer.date) \(.sha[0:7]) \(.commit.message | split("\n")[0])"'
```

The line-level comments carry `path` and `line`, which is what tells a comment on
a line this PR changed from one on code it merely sits next to. Both listings
lead with a timestamp, which is what makes a second or third round tractable: a
comment written before a commit may already be handled, and answering it again
spends the reviewer's next round on nothing.

Classify each comment before touching code:

| The comment | What it is | What to do |
|---|---|---|
| Names a defect in a line this PR changed | In scope, blocking | Fix here |
| Style or naming inside the changed lines | In scope | Fix here if cheap; say so if deferred |
| Points at code this PR only moved or touched incidentally | Pre-existing | Propose a follow-up; do not grow this diff |
| Asks for behavior nobody has agreed to | New requirement | Needs author and reviewer to agree before it enters this PR |
| Repeats a point an earlier round settled | Already handled | Link the commit or the earlier reply; do not re-fix |
| Rests on a premise the code contradicts | Mistaken | Reply with the evidence; do not comply silently |

**The scope test**: would this change still be needed if this PR had never
existed? If yes, it is a follow-up, not this PR's work. Growing scope is a
decision the author and reviewer make together and record in the thread — never a
silent extra commit.

**When the class is unclear, ask the commenter.** "Do you want this in this PR or
as a follow-up?" is one line and settles it. Guessing fails in one of two
directions: a PR that grew past reviewability, or a reviewer whose point was
quietly dropped.

Reply to every comment, including the ones you do not act on — name the class it
fell into and where it went. A comment with no reply reads as ignored, and the
next round re-raises it.

## 7. Branch naming

```
<type>/<ticket>-<short-topic>     # feat/ABC-123-oauth-login
<type>/<short-topic>              # fix/token-refresh-race — no ticket
improve/<topic>                   # config contribution (see references/upstream-config.md)
```

`<type>` matches the commit types in `commit-rules`. Never commit straight to the
base branch.

## Related

| Need | Where |
|---|---|
| Config contribution method (gates, privacy, source resolution) | `references/upstream-config.md`, or `/upstream-pr` |
| Extracting a session pattern into a skill before contributing it | `/learn` |
| Default body template, reviewer and author checklists | `references/pr-body.md` |
| Commit message format and splitting | `commit-rules` |
| Change summary to paste into the body | `qa-handoff` |
| Code review depth | `/code-review`, or the `code-reviewer` agent |
| Security sign-off | `security-review` |
| Tier definitions (A–E) driving how much rigor to apply | `do-178c` |
