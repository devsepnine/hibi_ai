---
name: pull-request
description: PR body from the diff, review that questions whether each change is needed, comment triage. Use when opening, reviewing, or answering review comments on a GitHub PR. PR 생성, 풀리퀘스트, PR 리뷰, 리뷰 코멘트 반영, 업스트림 PR, 설정 개선 환류.
---

# Pull Request

A PR is a review request, not a delivery mechanism. Its job is to let someone
else judge the change: what it does, why, what could break, and what evidence
says it works. Everything below serves that, which is why the conventions come
from the repository being worked in rather than from this document. A reviewer
looking for a field their team defined must find it where they expect it.

**Ask instead of guessing.** Most bad PRs come from one of two guesses: writing a
body from what the branch name implies rather than from the diff, or applying a
review comment without checking whether it belongs to this PR. When the diff, the
thread, or the ticket does not answer a question, ask it. One message costs less
than a rationale a reviewer then trusts and repeats. Where the answer would not
change what you write, state the assumption in the body instead of blocking on
it.

## Two cases

Load `technical-writing` before drafting or editing a PR title, description, review, or comment reply. Apply its language and writing rules, then revise the draft before returning or posting it. Preserve the repository's PR template and review conventions.

| Need | Where |
|---|---|
| Open, update, review, or answer comments on a PR in any repository | §1 to §7 below |
| Promote a session-derived improvement into the distributed config | `references/upstream-config.md`, which adds source resolution, four gates, and a privacy review on top of §1 to §7 |
| Default body template and review checklists, fallback only | `references/pr-body.md` |

## 1. Read the conventions from the repo, not from memory

Every value below is discoverable. Assuming one is how a PR ends up targeting a
branch that does not exist or carrying a ticket prefix the project never used.

**Checkout and target**. Work from the absolute path of the checkout that holds the change. Record its head branch and commit. Identify the target repository and base branch separately from the remote and ref that will hold the PR head. Resolve these from the user's request, Git remotes, and repository conventions. Confirm any ambiguity that would change where the PR opens.

**Base branch**. Use the target repository to find its conventions. Set `TARGET_REPO` to `HOST/OWNER/REPO` and `BASE` to the selected base branch name. Use `BASE_REF` for the fetched Git ref that represents that same branch.

```bash
gh repo view "$TARGET_REPO" --json defaultBranchRef -q .defaultBranchRef.name
gh pr list --repo "$TARGET_REPO" --state merged --limit 10 --json baseRefName -q '.[].baseRefName' | sort | uniq -c
```

The default branch is the starting guess; what recent merged PRs actually
targeted is the stronger signal, because plenty of projects merge into an
integration branch instead. When the two disagree, say so and ask.

**Ticket ID**: extract, do not invent:

```bash
git branch --show-current          # feature/ABC-123-thing → ABC-123
git log --oneline -20              # the prefix existing commits use, not a number to reuse
```

If the branch and history yield no ticket, **omit the prefix entirely**. The
title is the summary alone. A config PR, a personal project, and a repo with no
tracker all legitimately have no ticket, and inventing a placeholder like
`[TICKET-1]` is worse than having none.

The branch decides. When history carries a prefix the branch does not, say so
and ask instead of borrowing the number. Someone else's real ticket reads as
valid in the title, which makes it worse than an omitted one.

Title format, therefore:

```
[ABC-123] Add user authentication system     # ticket exists
Add user authentication system               # no tracker, or none applies
```

**Body**: the repository's own template wins unconditionally:

```bash
ls .github/pull_request_template.md .github/PULL_REQUEST_TEMPLATE.md \
   .github/PULL_REQUEST_TEMPLATE/ docs/pull_request_template.md 2>/dev/null
```

Fill that template's sections with the content §2 derives. Only when none
exists, fall back to `references/pr-body.md`. Do not merge the two. A reviewer
reading their own template with extra sections spliced in cannot tell what the
project requires.

**Tracker links**: follow the convention the repo documents, which existing PR bodies
show. Never hardcode a tracker host.

## 2. Write the body from the diff

Read the diff before writing a word. The branch name, the ticket, and the commit
messages say what someone intended; only the diff says what the PR actually does.

```bash
git diff --stat "$BASE_REF"...HEAD     # shape first: how many files, how large
git diff "$BASE_REF"...HEAD            # then the content, hunk by hunk
git log --oneline "$BASE_REF"..HEAD    # what the author said along the way
```

`"$BASE_REF"...HEAD` compares the branch with its merge base, so the diff describes the changes this branch adds. Use the commit recorded in §1 in place of `HEAD` if the checkout may move during drafting.

**Match the body to the diff both ways.** Every non-trivial hunk appears
somewhere in the body; every claim in the body maps to a hunk. Each unmatched
item is a defect you get to fix before a reviewer finds it. An unlisted hunk is
scope that crept in, and an unmatched claim is fiction.

**Be concise by writing what the diff cannot show.** One line per change a
reviewer must know about. They can read the diff, so do not narrate it; spend the
space on why this approach, what you rejected, where to start reading, and what
could break. A body that restates the diff in prose is longer and less useful
than three lines that say what is not visible in it.

**Ask when you cannot recover a why.** A magic number, a skipped test, a
dependency bump, a behavior change the ticket does not imply: if the diff does
not explain it and neither does the history, ask the author rather than composing
a plausible reason. A body that states a wrong reason is worse than one that
omits it, because the wrong reason gets quoted in the next design discussion.

If the Changes section grows past what a reviewer will hold in their head, the
problem is the PR, not the body. Split it, see §3, Size.

## 3. Pass the pre-PR gates

1. **Verification**: lint, type-check, and tests green, see `verification-loop`.
2. **Branch**: on a feature branch, not the base branch.
3. **Base**. Compare with the current target in §1. Report divergence and follow the repository's integration policy. Rewriting history requires a request that covers the rewrite.
4. **Commits**. Check that each commit is independently buildable. See `commit-rules`.
5. **Conflicts**: resolved.
6. **Docs**: updated if behavior or an API changed.
7. **Code comments**: every comment the diff touched, and every comment it
   invalidated elsewhere, still holds; see `coding-standards` → Comments.
8. **Secrets**: no credentials, PII, debug code, or stray logging in the diff.

Depth for each lives elsewhere. This table is the routing, not the policy:

| Check | Source of truth |
|-------|-----------------|
| File/function size, complexity | `coding-standards` → `references/code-thresholds.md` |
| Code comments: stale, orphaned, unfounded | `coding-standards` → Comments and `references/review-checklist.md` |
| Secrets, injection, XSS, authn | `security-review` |
| Coverage, regression, E2E paths | `tdd-workflow` |
| Build, type, lint | `verification-loop` |
| Commit message format | `commit-rules` |
| Coupling and module boundaries | `dependency-design` |
| Change summary for the body | `qa-handoff` |
| How much rigor this change warrants | `do-178c`, tiers A to E |

**Size**: keep PRs small and split by logical unit. A reviewer's attention is
the scarce resource; two reviewable PRs beat one that gets rubber-stamped.

## 4. Open it, only when asked

**Preparing a branch and drafting a description is not permission to open a PR.**
Opening one is outward-facing: it notifies people and publishes the branch. Do it
only on an explicit request for that push or PR, the same rule that governs commits,
and that request never carries over: an earlier "just handle it", an approval that
covered a previous push, an accepted plan, and a permission mode that would
auto-approve the command are each not one. Without it, show the
draft and stop. Cases that reach a public repository raise this bar further; see
`references/upstream-config.md`.

Before an authorized push or PR, recheck the checkout, branch, and local commit from §1. Verify the push remote's push URL and destination ref. Before creating a PR, confirm that the remote head ref holds the commit reviewed in §2 and that the requested base exists in the target repository. If the head needs to be published first, handle that as a separate push request.

Show the target repository, base and head, and the rendered title and body. Use `HEAD_SPEC` for the verified branch name, qualified with its owner for a fork when the tool supports that head. If the tool cannot express the verified head, keep the PR as a draft in the response.

```bash
gh pr create --repo "$TARGET_REPO" --base "$BASE" --head "$HEAD_SPEC" \
  --title "$TITLE" --body-file pr-body.md --draft
```

Use `gh pr ready "$PR_NUMBER" --repo "$TARGET_REPO"` when the user asks to mark that draft ready.

- **`--body-file`, not `--body`**: a multi-line body with backticks and quotes is easy to mangle through shell quoting.
- **`--draft` when anything is still moving**: CI unverified, a question open, or the branch likely to be rebased.
- **`--fill` only for a single clean commit**: it builds the body from commit messages, which skips §2 entirely; on a branch with noise commits it produces a body nobody wrote.

## 5. Review an existing PR

Gather the diff and CI status before judging. Read prior discussion as described in §6.

```bash
gh pr view "$PR_NUMBER" --repo "$TARGET_REPO" \
  --json title,body,baseRefName,baseRefOid,headRefOid,changedFiles,isDraft,reviewDecision,statusCheckRollup
gh pr diff "$PR_NUMBER" --repo "$TARGET_REPO" --name-only
gh pr diff "$PR_NUMBER" --repo "$TARGET_REPO"
gh pr checks "$PR_NUMBER" --repo "$TARGET_REPO"
```

Record the base and head commit IDs. Recheck both after reading live PR files or diffs and fetch them again if either moved. Compare the number of collected paths with `changedFiles` and check for missing patches. If the diff is incomplete, try the paginated `pulls/{n}/files` endpoint in the target repository and check for remaining gaps.

A local diff between the recorded revisions can help inspect missing code. Label it as a local comparison because GitHub may use a different merge base for the PR. Report any part of the PR diff you could not verify. Read direct callers at the recorded head when they bear on a finding, and follow their history when it explains intent. Check any proposed remedy against the same evidence.

Then judge in this order, cheapest question first:

**a. Does the diff match the body?** Trace it both ways, as in §2. Hunks the body
never mentions are the fastest thing to find and the most likely to be scope the
author did not intend to ship.

**b. Is each change needed?** Ask this per hunk, not of the PR as a whole. The
body states a purpose; a hunk that does not serve it is either unmentioned scope
or code nobody asked for.

- **Delete-test**: if this hunk were removed, what breaks? "Nothing" means it is dead on arrival: an unused export, a parameter no caller passes, a branch no input can reach.
- **Caller count**: a new abstraction, option, flag, or generic parameter with exactly one caller and no second use in the diff is speculative. Ask what the second use will be; absent one, the concrete version is the smaller change.
- **Already exists**: grep the repo before accepting a new helper. A reimplementation is the most expensive kind of addition, because both copies now need maintaining.
- **Guards that cannot fire**: a null check on a non-nullable type, a `try` around code that does not throw, validation the caller already performed. Each one tells the next reader that condition is reachable.
- **Unrelated to the purpose**: a rename, a formatting sweep, or a drive-by fix folded into a feature PR. Not wrong to want; wrong to hide here, because it makes the diff unreviewable and the revert unusable.

Necessity is a question, not a verdict. When you cannot tell what a hunk is for,
ask the author. The answer is usually one sentence, and it belonged in the body
anyway.

**c. Is it correct when it fails?** Review success and reachable failure paths using `coding-standards` → `references/review-checklist.md`, Failure paths in self-review and PR review. Trace the trigger through errors, partial effects, cleanup, and the caller's resulting state. Distinguish confirmed defects from test gaps and unverified risks. Depth in `/code-review`, and `security-review` when the diff touches auth, input handling, or secrets.

If the diff can change when shared state is read, written, or cleared, which
value is used as current, or what remains after a partial failure, write down
the expected final state before concluding the review. Trace the affected values
from their producer through storage to the consumer, including code outside the
diff. If the producer can omit a value or return it empty, check what the
consumer reads when older data remains. Check event order and existing data
when they can change the result. In review comments that ask to change or
clear shared state, say which operation may do so and what must survive when
ownership is unknown or has changed. Verify the final state and what the user
or caller sees. Recheck these cases in follow-up reviews of the same PR and
extend them when the patch adds state or changes how it is handled.

When a follow-up review finds another material defect affecting the same
required outcome after a fix, review the affected flow end to end before
posting. On the current head, include other entry points and operations that
can change that outcome, even outside the latest patch. Check the final state
after success and reachable failures, then group confirmed findings in one
review.

**d. Is it tested?** Would a test fail if this change were reverted? If not, the
claim is unverified regardless of coverage numbers. Check meaningful assertions for relevant failure scenarios as well as success, including state after failure rather than only an error being thrown. Use `tdd-workflow` for case design, reuse adequate existing tests, and state any unexecuted checks.

For an untrusted revision, inspect files directly at the commit and use available CI evidence for that revision. Check it out and run its checks only in a disposable sandbox without access to credentials, private files, or the user's shared Git data.

Show the review draft and post only on explicit request. Recheck the recorded base and head before posting. If either moved, refresh the affected findings. Bind approvals, change requests, and reviews with inline comments to the reviewed head via `commit_id` or an equivalent field. Keep those actions as drafts if the tool cannot bind them. Group inline comments in one review, and verify the path, line, and side of each against the GitHub PR diff. If the tool cannot place an inline finding, keep it in the draft or use a permalink in a body-only comment when the user has requested that action instead.

Distinguish blocking findings from suggestions explicitly. A review that lists
eight equal-weight comments makes the author guess which two actually block.

## 6. Work through review comments

A review comment is a claim, not an instruction. Applying every comment without
triage is how a focused PR becomes unreviewable, and how a mistaken comment turns
into shipped code.

Read three sources in the target repository before classifying comments.

1. Inline review threads via GraphQL `reviewThreads`. Keep thread IDs, resolution and outdated flags, diff anchors, timestamps, and replies. Page through threads and each thread's comments using `pageInfo`.
2. Review bodies from `pulls/{n}/reviews`. An approval can include findings that lack an inline thread.
3. PR conversation from `issues/{n}/comments`. A comment here may link a commit that addresses an inline finding.

Paginate the REST listings and report any coverage you could not retrieve. Use the paginated `pulls/{n}/commits` listing to place follow-up commits on the same timeline. Match a finding and any follow-up by thread, location, and evidence. A claim that a commit addressed it remains a claim until you verify the change on the PR head. An empty thread alone leaves the response status unknown.

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
decision the author and reviewer make together and record in the thread, never a
silent extra commit.

**When the class is unclear, draft a question for the commenter.** Ask whether the change belongs in this PR or in a follow-up.

Draft replies for the requested comments. Verify any claimed fix on the remote PR head and link the relevant commit. Show the exact PR, thread or comment, and reply text before sending an explicitly requested reply. Recheck the destination, current discussion, and remote PR head before sending. If the head moved, verify any code claims again. Request another review only when the user requests that action.

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
| Config contribution method: gates, privacy, source resolution | `references/upstream-config.md`, or `/upstream-pr` |
| Evaluating a session pattern before proposing a skill change | `skill-improvement` |
| Default body template, reviewer and author checklists | `references/pr-body.md` |
| Commit message format and splitting | `commit-rules` |
| Change summary to paste into the body | `qa-handoff` |
| Code review depth | `/code-review`, or the `code-reviewer` agent |
| Security sign-off | `security-review` |
| Tier definitions, A to E, driving how much rigor to apply | `do-178c` |
