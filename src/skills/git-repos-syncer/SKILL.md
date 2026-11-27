---
name: git-repos-syncer
description: "Check or fast-forward multiple Git repos. 여러 Git 저장소 상태, 일괄 갱신."
---

# Check and update local Git repositories

Use this skill for a batch of local repositories. A status request ends with the report. An update request authorizes fast-forwarding the selected local checkouts within its stated scope.

## Identify and compare checkouts

1. Use the paths or parent directories the user supplies. Discover repositories within those roots and deduplicate identical checkout paths. Identify the branch and checkout to use for each repository. Use the current branch by default and its configured upstream unless the user specifies another target. For a different local branch, locate the checkout that already has it checked out.
2. Inspect each checkout's status, including staged, unstaged, and untracked changes and any Git operation in progress. Report detached checkouts, branches with no usable upstream, and missing checkouts as unavailable for update.
3. For a cached status report, label the comparison as cached. For fresh status or an update, refresh the selected remote branch. Confirm `remote` is the named remote, `remote_ref` is its full `refs/heads/` source, and `tracking_ref` is the matching full `refs/remotes/` destination. Resolve a symbolic remote `HEAD` to its branch before fetching. If fetch fails, report the error and keep that entry out of the update set.

   ```bash
   git -C "$checkout" fetch --no-tags --no-prune --no-prune-tags \
     --no-recurse-submodules --no-write-fetch-head --refmap= -- \
     "$remote" "$remote_ref:$tracking_ref"
   ```

4. Resolve the local tip and upstream tip to commit IDs. Compare them with `git rev-list --left-right --count` using the local tip on the left and the upstream tip on the right. Report equal tips as up to date, remote-only commits as behind, local-only commits as ahead, and commits on both sides as diverged. Mark ancestry that cannot be established as unknown. Report checkout changes separately from the branch comparison.

## Fast-forward the selected checkouts

1. Show which checkouts can advance and why the others are skipped. Apply only within the user's requested scope. An eligible checkout is clean and behind its fetched upstream, with no Git operation in progress. Confirm that its local tip is an ancestor of the target commit.
2. Just before updating each checkout, confirm its path, branch, local tip, upstream and cleanliness still match the preview. If they changed, hold that checkout and refresh the preview before applying it. Use the recorded upstream commit as `target_commit`.

   ```bash
   git -C "$checkout" -c submodule.recurse=false merge \
     --ff-only --no-autostash --no-squash --no-overwrite-ignore -- "$target_commit"
   ```

3. Read back the branch, commit and checkout status. Report an update only after confirming the requested branch reached the planned commit. If Git reports a collision with an ignored file, leave that checkout pending and report the path. If Git fails or the state differs, report what happened and continue with independent checkouts. Summarize each path, branch, upstream and outcome.
