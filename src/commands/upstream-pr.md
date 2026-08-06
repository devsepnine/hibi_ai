---
description: Promote session-derived improvements into the distributed config as a reviewable PR — detect drift between installed config and src/, gate by blast radius, and open the PR with its evidence.
argument-hint: "[topic]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash
model: sonnet
effort: high
---

# Upstream PR

Thin entry point for feeding what a session taught back into `src/`, which ships
to every installer user. The point of the PR is not the diff — it is the evidence
and the blast radius, because a reviewer cannot judge a rule change without
knowing what failed and who it reaches.

## Locate the source first

Most people install this config and never clone the repo, so do not assume one
is present. Resolve where the change has to land before anything else:

1. **`~/.hibi/install.json`** — the manifest the installer writes: `source` (upstream URL), `version` (a release tag, so the exact source tree is recoverable), `target`, and the installed components. Read it first and trust it over any hardcoded URL; it is also what tells an install-only user where their config came from.
2. **Local clone** — a git remote pointing at that source (`git remote -v`). Use it.
3. **No clone (install-only user)** — say so plainly, name the upstream from the manifest, and offer `gh repo fork <owner>/<repo> --clone`. Their `~/.claude` already holds the change; the fork is only the vehicle for review.
4. **They would rather not fork** — write the proposal to a file in their home or scratchpad directory and point them at the repository's issues page so it can be attached there. An idea in an issue beats an idea lost at session end.
5. **No manifest either** (installed before manifests existed) — fall back to `https://github.com/devsepnine/hibi_ai` and mention that reinstalling will record provenance for next time.

## Collect candidates

- **Drift**: compare the installed config against the repo — `diff -r ~/.claude/skills <repo>/src/skills` and the same for `commands`, `agents`, `CLAUDE.md`, `AGENTS.md`. Anything present locally but not in `src/` is a candidate; anything in `src/` but missing locally is an install gap, not a candidate.
- **Session evidence**: corrections the user made, guidance that was missing when it was needed, and defects found in shipped docs (a wrong path, a rule that contradicts another).
- `$ARGUMENTS`, when given, narrows the sweep to that topic.

## Gate each candidate

A candidate ships only if it passes all four. State the verdict per candidate —
rejections are as useful as approvals.

1. **Generalizable** — does it help a user on a different stack or project? Repo-specific guidance belongs in that project's `CLAUDE.md`; personal preference belongs in `MEMORY.md`. Neither belongs in `src/`.
2. **Evidenced** — can you cite what actually failed or was corrected this session? An opinion without a trace is a note, not a PR.
3. **Tier-appropriate** — `CLAUDE.md` / `AGENTS.md` load in every session for every user (B tier): require the evidence, at least one alternative considered, and the smallest wording that works. Skills, commands, and agents load conditionally (C/D tier): lower bar.
4. **Reversible** — wording reverts cleanly; a rule that changes user habits does not fully revert once released. Higher bar the harder it is to take back.

## Enforce house conventions before committing

- Every `src/` file has an English original and a `-ko.md` twin, and only the English one installs — so KO twins must not reference `-ko` asset paths.
- Detailed policy lives in one skill; `CLAUDE.md` / `AGENTS.md` carry a single line that links to it. Duplicating detail across both is the failure mode to avoid.
- No emojis, no generation markers.

## Ask before anything leaves the machine

Show what would be shared — the diff plus the evidence lines — and ask whether to
upstream it. This is a question, not a step, for two reasons:

- **The evidence comes from a real session**, so it can carry an employer's code, internal paths, ticket IDs, or customer names. Describe the general shape of what failed and never the proprietary detail; if the failure cannot be described without it, say the evidence needs rewriting before the PR can go out.
- **The upstream is public.** Once pushed, the content is public even if the branch is deleted afterwards.

## Open the PR

Improvements go on `improve/<topic>`, never straight to `main`. One logical unit
per commit. The PR body states, per change: the evidence, the tier and who it
reaches, the alternative rejected, and how to revert. Note that `src/` is
package-embedded, so merged changes reach other users at the **next release**,
not at their next `hibi --sync` — that is what makes the PR the last review gate.

Confirm with the user before `git push` and before opening the PR; both are
outward-facing and neither is implied by running this command.

**The PR title and body conventions live in the `pull-request` skill — follow that as the source of truth. For the change-summary section, reuse `qa-handoff`; for pattern extraction into a new skill, `/learn`.**
