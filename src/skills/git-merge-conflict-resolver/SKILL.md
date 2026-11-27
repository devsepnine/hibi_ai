---
name: git-merge-conflict-resolver
description: "Resolve Git merge or rebase conflicts. Git 머지 충돌, 리베이스 충돌 해결."
---

# Resolve Git conflicts

Use this skill to inspect or resolve conflicts during a merge or rebase. Start in the affected checkout and run the commands below from its root. For an inspection request, report the operation and both changes, then stop before editing. Preserve existing worktree and index changes outside the resolution.

## Identify the operation and both changes

1. Read `git status` and list the unmerged paths with `git diff --name-only --diff-filter=U`. Check which paths are already staged, edited, or untracked. If there is no merge or rebase in progress, report what Git shows and establish the requested operation before starting one.
2. Inspect the unmerged index entries for each exact path relative to the checkout root. Read the existing stages with `git show`. For a rename, check both names and the corresponding entries. A stage can be absent when that version has no file at the path.

   ```bash
   git --literal-pathspecs ls-files -u -- "$conflict_path"
   git show ":1:$conflict_path"
   git show ":2:$conflict_path"
   git show ":3:$conflict_path"
   ```

   | Operation | Stage 1 | Stage 2 | Stage 3 |
   |---|---|---|---|
   | Merge | Base version | `HEAD` | Incoming version from `MERGE_HEAD` |
   | Rebase | Base version for the current replay | `HEAD` with any work already replayed | Version from the change being replayed |

3. Read the relevant diffs and commit messages to understand why each side changed. During a rebase, `git rebase --show-current-patch` identifies the patch in progress. Use the stage entries and commit history to distinguish Git's two inputs from any edits the user has already made to the conflicted file.

## Resolve and stage the affected paths

1. Preserve every compatible behavior from both sides. When one side moved code and the other fixed it, apply the fix in the new location. For a deletion or binary conflict, use the history and callers to decide which version belongs in the result. Ask when the evidence leaves an incompatible behavior choice unresolved.

   In Markdown, keep independent sections and list items in the intended hierarchy. Keep frontmatter keys unique and combine list values only where the schema allows it. Resolve competing prose, statuses, and dates using the document's intent and repository conventions. Ask when those sources leave the meaning unclear.

2. Review the edited file against both versions before staging. Stage only the resolved paths, including any intended deletion or rename. Pass each exact path to `git --literal-pathspecs add --` so file names are treated literally.

## Verify before continuing

1. List unmerged paths across the repository with `git diff --name-only --diff-filter=U`. The list must be empty before the operation can continue. Review the staged result for each resolved path. Check each exact path with `git --literal-pathspecs diff --cached --check -- "$resolved_path"`. For a rename, include both paths. Inspect the staged content for leftover conflict markers and compare it with the behavior each change was meant to preserve. For Markdown, also check frontmatter validity, section and list structure, and affected links.
2. Run the project's checks that exercise the affected behavior. Report any failing or unavailable check and distinguish it from a verified result.
3. Report what was resolved, any intentional choice between competing changes, and whether the operation is ready to continue. A request to resolve conflicts ends with the verified resolution staged. Continue, abort, or skip only when the user authorizes that action. Continuing a merge can create a commit, and continuing a rebase can rewrite commits. If a rebase stops again, inspect and verify the next conflict before continuing.
