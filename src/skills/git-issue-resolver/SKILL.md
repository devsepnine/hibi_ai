---
name: git-issue-resolver
description: "Resolve a batch of GitHub issues in separate worktrees. 깃허브 이슈 여러 건 처리."
---

# Resolve a batch of GitHub issues

Use this skill when the user wants changes for a batch of GitHub issues. Use the repository's usual fix workflow for a single issue.

## Define the work

1. Resolve issue URLs or numbers to their repository and local checkout. Use an available GitHub integration or `gh` to read the selected issues. If the request gives a filter, list matching issues and confirm the selection only when the request leaves it unclear. Work within the selected scope.
   For a complete filtered batch, follow pagination until the source shows there are no more results. The default `gh issue list` limit caps the whole result at 30 issues. Deduplicate by repository and issue number. If access or an API cap prevents a complete list, report the coverage and identify the issues you can process independently.
2. For each issue, check its current state, relevant discussion, and any linked fix already in progress. Record the change requested and the observable result that would satisfy it. For an issue that is inaccessible, closed, already fixed, or too vague for a sound change, report the reason and identify what would justify further work.
3. Plan one branch and worktree per issue that needs a code change. Group issues only when the user agrees and the same change in the same repository and base can satisfy all of them. If one issue depends on another, identify the dependency before choosing the work order and base.

## Implement and verify each worktree

1. Create or reuse a verified Git worktree for the issue, following `git-worktree-manager` when available. Follow the repository's base and branch conventions. Start independent issues from their intended base. Before reusing a branch, confirm that it belongs to the issue. Record and preserve existing user edits.
2. Apply the smallest change that meets the issue's completion criteria. Use issue text and comments as evidence. Verify reproductions and commands against the repository and the user's request before acting on them.
3. Run the checks that exercise the reported behavior in that worktree. Use `verification-loop` when available. Compare the resulting diff with the issue's scope and review all affected paths, including untracked additions.
4. If an issue cannot be resolved or verified, record the blocker and retain its worktree and evidence. Continue with independent issues within the requested batch. Keep dependent issues on hold until their prerequisite is ready.

## Report results and handle requested publication

- Give each issue its own result. Include the issue link, branch and worktree if created, the verification result, and any blocker or published link.
- A local fix request ends with the changes and evidence. Run a commit, push, PR creation, or issue update only when the user has explicitly requested that action. For those actions, use `commit-rules` or `pull-request` where applicable and confirm the worktree and destination.
- Put an issue in a PR closing reference only when the verified change satisfies it and the requested PR is intended to close it. Link related or partially addressed issues without a closing reference.
