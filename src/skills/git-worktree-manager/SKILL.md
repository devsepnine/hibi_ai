---
name: git-worktree-manager
description: "Create or reuse Git worktrees for isolated branches. Git 워크트리 생성·재사용."
---

# Git Worktree Manager

Use this skill when the user requests a worktree or the task calls for a separate checkout. When the task is to continue or commit changes already in a checkout, use that checkout unless the user requests isolation.

## Identify the repository and branch

1. Use the repository the user specifies, or the current repository. Resolve the checkout root with `git rev-parse --show-toplevel`, or use the Git directory for a bare repository. Read `git worktree list --porcelain` before choosing a path. The first record identifies the main worktree or bare repository. Use `-z` if parsing the output programmatically.
2. Use the requested branch name or choose one that follows the repository's conventions. Validate the literal name with `git check-ref-format --branch` and check that the output matches. Match the full `refs/heads/<branch>` in the worktree list. Check that the recorded checkout is accessible and that its current branch matches before reusing it. Inspect its status and preserve existing edits.
3. If the branch is checked out in the main worktree and isolation is required, show the existing path and ask for a different task branch. If an existing linked worktree conflicts with an explicitly requested location, show both paths and ask which to use. A detached worktree or an unavailable registration needs separate handling. Leave it in place and explain the conflict if it blocks the request.

## Create only when needed

1. If the local branch exists, use its current tip. If it conflicts with a requested starting point, ask before proceeding. If the branch exists only on a remote and the task is to continue it, use that remote branch. Ask if multiple remotes offer different tips and the intended one is unclear.
2. For a new task branch, prefer the user's starting point, then the repository's documented base or the selected remote's default branch. Resolve the exact ref to a commit. If the base is unclear, ask before creating the worktree. If a fresh remote tip is required, fetch it before resolving the commit. If that fetch fails, report the error and ask before using an available local ref.
3. Use the requested path or the repository's worktree location. If neither is set, choose an unused sibling path beside the main worktree or bare repository. For a path within a checkout, use an ignored location. If a proposed path is occupied by a different checkout or existing files, preserve it. Pick an unused path if the original was your choice, or ask if the user specified it.
4. Before creating a worktree, check that the path is absent from the worktree list and that the first check below succeeds. Then run the applicable creation command. `repo` and `worktree_path` are absolute paths. `branch` is the short local branch name. For remote continuation, set `tracking_ref` to the full remote-tracking ref and confirm its tip immediately before creation. For a new task branch, set `start_commit` to the verified starting commit.

   ```bash
   # Check for existing files, directories, and dangling symlinks
   test ! -e "$worktree_path" && test ! -L "$worktree_path"

   # Attach an existing local branch
   git -C "$repo" worktree add -- "$worktree_path" "$branch"

   # Continue a remote branch with upstream tracking
   git -C "$repo" worktree add --track -b "$branch" -- "$worktree_path" "$tracking_ref"

   # Create a new task branch at the chosen commit
   git -C "$repo" worktree add -b "$branch" -- "$worktree_path" "$start_commit"
   ```

## Verify and use the checkout

- Confirm the command succeeded. Verify the path and branch in the repository's worktree list and check the branch from inside that checkout. If creation fails, report the Git error and inspect the worktree list and local branch ref. Git can leave the branch behind after a checkout failure. Report and preserve any leftover branch. If a concurrent run created the intended checkout, verify it before reuse.
- For remote continuation, verify that `HEAD` matches the recorded start and that `git rev-parse --symbolic-full-name '@{upstream}'` matches `tracking_ref`. If either differs or cannot be read, report the mismatch and leave the checkout in place before doing task work.
- Give the caller the absolute path, branch, and starting point for a new worktree. Use that path for subsequent edits, Git commands, builds, and tests. If setup is needed, follow the repository's instructions in this checkout.
- Leave existing worktrees and branches in place. If a stale registration prevents progress, report the specific path so the user can decide how to handle it.
