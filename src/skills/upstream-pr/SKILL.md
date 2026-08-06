---
name: upstream-pr
description: Promote a session-derived improvement into the distributed config as a reviewable PR — find the upstream from the install manifest, separate real candidates from local noise, gate each one on generalizability, evidence, blast radius, and reversibility, then open the PR carrying that evidence. Use when a session reveals that a shipped rule, skill, agent, or command should change; when someone asks how to contribute a config improvement back; or when an improvement was applied locally and has to reach other users. 설정 개선 환류, 업스트림 PR, 배포 설정 기여, 세션 교훈 반영, 로컬 수정 업스트림, 기여 방법.
keywords: [upstream-pr, 업스트림PR, 설정개선, 기여, contribute, config-improvement, drift, 세션교훈, provenance, install-manifest, fork, PR]
---

# Upstream PR

Feed what a session taught back into the config that ships to other users. The
PR's value is not the diff — it is the evidence and the blast radius, because a
reviewer cannot judge a rule change without knowing what failed and who it
reaches.

## 1. Locate the source

Most people install this config and never clone the repo, so do not assume a
checkout exists.

1. **`~/.hibi/install.json`** — the manifest the installer writes: `source` (upstream URL), `version` (a release tag, so the exact source tree is recoverable), `target`, and the installer-managed component list. Read it first and trust it over any hardcoded URL; it is also what tells an install-only user where their config came from.
2. **Local clone** — a git remote pointing at that source (`git remote -v`). Use it.
3. **No clone** — say so plainly, name the upstream from the manifest, and offer `gh repo fork <owner>/<repo> --clone`. The change already exists in the user's installed config; the fork is only the vehicle for review.
4. **They would rather not fork** — write the proposal to a file in their home or scratchpad directory and point them at the repository's issues page. An idea in an issue beats an idea lost at session end.
5. **No manifest either** (installed before manifests existed) — fall back to `https://github.com/devsepnine/hibi_ai` and mention that reinstalling records provenance for next time.

## 2. Collect candidates

Diff **every** directory the installer manages, not a favourite few — a
correction to an agent or an output style is as upstreamable as one to a skill.
For the Claude target that is `agents`, `commands`, `contexts`, `rules`,
`skills`, `output-styles`, `statusline`, `hooks`, plus the config files
`CLAUDE.md` and `settings.json`; for Codex it is `skills` and `AGENTS.md`. Use the manifest's component list as the ground truth for what the
installer actually put there.

Then subtract the noise, or a real invocation drowns in it:

- **`-ko.md` files and `*/workspace/` subtrees never install by design.** They appear as "missing locally" on every diff. They are exclusions, not gaps.
- **Not every local file came from this repo.** `~/.hibi/sources.yaml` can add other git or local sources, and users write their own skills. Anything outside this repo's install surface is not a candidate no matter how good it is. The manifest already separates this: `components` lists only what came from the bundled source, and any other contributing source appears under `other_sources`.
- **Session evidence** counts as a candidate too: corrections the user made, guidance that was missing when it was needed, defects found in shipped docs (a wrong path, a rule contradicting another).

## 3. Gate each candidate

A candidate ships only if it passes all four. State the verdict per candidate —
rejections are as useful as approvals.

1. **Generalizable** — does it help a user on a different stack or project? Repo-specific guidance belongs in that project's `CLAUDE.md`; personal preference belongs in `MEMORY.md`. Neither belongs upstream.
2. **Evidenced** — can you cite what actually failed or was corrected? An opinion without a trace is a note, not a PR.
3. **Tier-appropriate** — assign the tier on the `do-178c` skill's A–E scale. Always-on guides (`CLAUDE.md`, `AGENTS.md`) load in every session for every user, so they sit at B: require the evidence, at least one alternative considered, and the shortest wording that works. Conditionally loaded skills, commands, and agents clear a lower bar.
4. **Reversible** — wording reverts cleanly; a rule that changes user habits does not fully revert once released. The harder it is to take back, the higher the bar.

## 4. Shape it to house conventions

- **Both twins** — every file has an English original and a `-ko.md` twin, and only the English one installs. KO twins therefore must never reference `-ko` asset paths.
- **One source of truth** — detailed policy lives in one skill; `CLAUDE.md` / `AGENTS.md` carry a single pointer line. Duplicating detail across both is the failure mode to avoid.
- **Codex sees only skills and `AGENTS.md`** — a method that Codex users need belongs in a skill, not in a command, because commands install for Claude only. Reference skills by name in `AGENTS.md`, never by slash-command syntax.
- **A `/learn` artifact still needs its twin** — `/learn` already writes a promotable `skills/<name>/SKILL.md` with frontmatter into the installed tree, but the distributed copy lives at `src/skills/<name>/` and requires the `-ko.md` twin.
- Commit hygiene per the `commit-rules` skill.

## 5. Ask before anything leaves the machine

Show what would be shared — the diff plus the evidence lines — and ask whether to
upstream it. This is a question, not a step, for two reasons:

- **The evidence comes from a real session**, so it can carry an employer's code, internal paths, ticket IDs, or customer names. Describe the general shape of what failed, never the proprietary detail; if the failure cannot be described without it, say the evidence needs rewriting first.
- **The upstream is public.** Once pushed, the content is public even if the branch is deleted afterwards.

## 6. Open the PR

Being asked to run this workflow is the explicit request that authorizes the
local branch and commits — `improve/<topic>`, one logical unit per commit, never
straight to `main`. Pushing and opening the PR are outward-facing and need a
**second** confirmation, which running the workflow does not imply.

The PR body states, per change: the evidence, the tier and who it reaches, the
alternative rejected, and how to revert. Note that the config is embedded in the
release package, so merged changes reach other users at the **next release**
rather than at their next `hibi --sync` — which is what makes the PR the last
review gate.

## Related

| Need | Where |
|---|---|
| PR title and body conventions | `pull-request` skill |
| Change summary for the PR body | `qa-handoff` skill |
| Tier definitions (A–E) | `do-178c` skill |
| Extracting a pattern into a skill first | `/learn` (emits a promotable skill; add the `-ko.md` twin when promoting) |
