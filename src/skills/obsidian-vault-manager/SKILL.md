---
name: obsidian-vault-manager
description: "Manage Obsidian vault Git backups, ignores and links. 옵시디언 볼트 백업·링크 점검."
---

# Obsidian vault management

## Decide what Git should track

1. Locate the vault root, Git root, and active Obsidian configuration directory. Read existing ignore rules and use `git status --short` and `git ls-files -- <path>` to inspect current state.
2. Derive the tracking policy from what the user wants to back up, share, or synchronize across devices. Evaluate notes, attachments, settings, workspace state, and plugin data separately. Inspect settings for paths, credentials, or private data and report findings without exposing their values.
3. Change only the ignore rules required by the requested policy. Account for the directory that contains the `.gitignore` when anchoring a pattern. Write comments on their own lines. Test representative included and ignored paths with `git check-ignore -v --no-index -- <path>` and confirm their tracked state with `git ls-files -- <path>`.
4. If Git already tracks a file that matches an ignore rule, report it separately. When the user requests untracking, enumerate the affected files, check the index and working copies, and preserve local files. Recheck the staged diff and the working copies after the change.

See the Git documentation for [ignore patterns](https://git-scm.com/docs/gitignore) and [rule diagnostics](https://git-scm.com/docs/git-check-ignore).

## Audit internal links

1. Read links in the requested source notes. Look for their targets across the permitted areas of the vault, including notes and attachments outside the source folders. Inspect wikilinks, embeds, and internal Markdown links. Skip external links and text in comments or code. When available, use Obsidian's own link resolution to confirm context-dependent targets.
2. Resolve paths according to the link format, using the source note's directory for relative paths and the vault root for vault-root paths. Parse display aliases and fragments separately from the file path. For local Markdown links, separate the fragment before decoding percent escapes in the path and fragment once. Treat a fragment-only link such as `[[#Heading]]` as a reference to the source note. When the file resolves, check heading or block references that can be matched reliably.
3. Mark results unverified when access or coverage limits could affect them. With adequate coverage, record a link as broken when the target search finds no match. For a short link with multiple plausible files, report the candidates as ambiguous when the source location or Obsidian resolution cannot establish the target. Identify missing headings and blocks separately when verified. Include the file, line, link text, and candidates or reason. Keep the audit read-only.

See Obsidian's [internal links guide](https://help.obsidian.md/links) for supported link formats.
