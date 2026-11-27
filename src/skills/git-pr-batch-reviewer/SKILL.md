---
name: git-pr-batch-reviewer
description: "Review a GitHub pull request batch or queue. GitHub PR 여러 건 검토, 리뷰 대기열."
---

# Review a batch of pull requests

Use this skill when the user asks to review multiple PRs or work through a review queue. For a request to list the queue, return the matching PRs and their review status. Use the existing `pull-request` skill for the review of each individual PR.

## Identify the batch and revisions

1. Resolve each PR to its repository and number. Use the supplied URLs, numbers, or filter. For a queue, list the matching PRs with links and the review information the user requested. Distinguish review requests, commits added since a previous review, and unresolved threads when the evidence supports them. Ask about selection only if the requested set is unclear.
   For a complete queue or filtered batch, follow pagination until the source shows there are no more results. The default `gh pr list` limit caps the whole result at 30 PRs. Deduplicate by repository and PR number. Report any access or API limit that leaves parts of the requested queue unverified.
2. For every selected PR, record its URL, state, draft status, base ref and commit, and head commit. Use an available GitHub integration or `gh` to read them. For example, with `repository` set to the target GitHub repository and `pr_number` set to the selected PR number, use the following command.

   ```bash
   gh pr view "$pr_number" --repo "$repository" \
     --json url,number,state,isDraft,baseRefName,baseRefOid,headRefOid
   ```

## Review each PR at its recorded revision

1. Apply the review criteria in `pull-request` when available. Read the PR intent, diff, relevant review discussion, and check results. Review the diff for the recorded revisions. If using a live PR diff, compare the revisions before and after retrieving it and refresh it if they differ.
2. Read additional files at the reviewed revision when needed. For a trusted revision, use a separate checkout if needed and verify its commit. Preserve the user's current checkout. For an untrusted revision, read files directly at the commit and use available CI results tied to it. Check out and run the untrusted revision only in a disposable sandbox without access to credentials, private files, or the user's shared Git data. If that environment is unavailable, report which checks were left unrun.
3. Keep findings with their PR and revision. Include a file and line or comparable evidence, the impact, and the scope of verification. Note existing comments before repeating a finding. If access or evidence is incomplete, report what is missing and continue reviewing the other selected PRs.
4. Summarize the batch with a result per PR. Include the PR link, head commit, findings or review outcome, and any gaps or blockers.

## Publish when requested

1. Show the drafts and publish only the PRs, review actions, and comments the user explicitly requests. Treat an approval and a request for changes as distinct actions.
2. Immediately before posting each review, re-read the PR state, draft status, base ref and commit, and head commit. If any changed, or the revisions are unavailable, hold that draft. Refresh the affected review before publishing it. Bind the review to the recorded head when the posting tool supports `commit_id` or an equivalent field. Check the path, line, and side of inline findings against the GitHub PR diff. Keep approvals, change requests, and inline findings as drafts if they cannot be tied to the reviewed commit.
3. Record the review link or confirmed status for each result. If posting has an uncertain outcome, look for the newly submitted review before retrying. Report uncertainty if you cannot confirm it.
