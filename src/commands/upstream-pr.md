---
description: Promote a session-derived improvement into the distributed config as a reviewable PR — locate the upstream from the install manifest, gate candidates by blast radius, and open the PR with its evidence.
argument-hint: "[topic]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash
model: sonnet
effort: high
---

# Upstream PR

Thin entry point for feeding what a session taught back into the config that
ships to other users. Resolve the upstream from `~/.hibi/install.json` (a user
who only installed has no clone), collect candidates by diffing every directory
the installer manages, gate each one, and open the change on `improve/<topic>`.

How to invoke: run when a session shows that a shipped rule, skill, agent, or
command should change, or when an improvement was applied locally and needs to
reach other users. `$ARGUMENTS`, when given, narrows the sweep to that topic.

Non-negotiable: running this authorizes the local branch and commits, nothing
more. Show what would be shared and get a **second** confirmation before `git
push` and before opening the PR — both are outward-facing, and session evidence
can carry an employer's code or internal paths that must be rewritten first.

**The full method — source resolution, candidate filtering, the four gates, and PR contents — lives in the `upstream-pr` skill. Follow that as the source of truth.**
