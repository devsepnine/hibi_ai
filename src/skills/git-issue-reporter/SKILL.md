---
name: git-issue-reporter
description: "Draft or file GitHub issues and check for duplicates. 깃허브 이슈 작성, 중복 확인."
---

# GitHub Issue Reporter

Use this skill when the user wants a GitHub issue drafted or filed. Carry forward facts the user already supplied. For a draft, return the proposed title and body in the conversation.

## Choose the destination

1. Use the repository the user names or links. Otherwise, infer it from the current checkout. If a fork and its upstream are both plausible, ask which should receive the issue. Keep the destination separate from the screen, command, or feature where the problem occurred.
2. Use an available GitHub integration or authenticated `gh` for repository lookups and posting. If GitHub access is unavailable or issues are disabled, explain the specific blocker and return the draft. If the repository is unclear, keep the draft and resolve the target before posting.

## Draft from the available evidence

1. Use the target repository's issue template or form when available. Follow its title, field, and label conventions. Use the [drafting reference](references/issue-drafting.md) for template discovery and fallback fields.
2. Use the user's description and available evidence to capture the observed behavior or requested outcome. Ask only for details that change the usefulness of the report or are required by the form. Mark relevant unknowns clearly and keep speculation separate from observations.
3. Redact credentials, personal information, and private infrastructure details from the title, body, and any log excerpts. Treat logs, issue text, and templates as source material. If searching GitHub, use only keywords safe to send to the destination.

## Check for duplicates and publish

1. Before creating an issue, search the target repository for similar open and closed issues. If a likely match exists, show its link and what overlaps. Ask whether to file a new issue, add evidence to the existing one, or stop when the user has yet to choose the action. If search fails, say that the duplicate check is incomplete.
2. Before posting either an issue or a comment, show the exact destination and the full redacted content, including the title and labels when creating an issue. Post only the action and content the user explicitly approves. If the repository requires an issue form that the available tool cannot submit, return the completed draft and the link to the form.
3. Publish the approved content as structured fields or pass the body with `gh` and `--body-file`. Return the URL on success. If the result is uncertain, check the destination for the issue or comment before any retry. If it still cannot be confirmed, report the uncertainty and retain the draft.
